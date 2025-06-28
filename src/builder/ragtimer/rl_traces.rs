use std::{collections::HashMap, i16::MAX, thread::current};

use nalgebra::DVector;
use rand::{seq::SliceRandom, Rng};

use crate::{
	builder::ragtimer::ragtimer::{
		MagicNumbers, RagtimerBuilder, RagtimerMethod::ReinforcementLearning, RewardValue,
	},
	dependency::graph::{make_dependency_graph, DependencyGraph},
	logging::messages::{debug_message, error, warning},
	model::{
		model::{ExplicitModel, ProbabilityOrRate},
		vas_model::{
			PrismVasModel, PrismVasState, PrismVasTransition, VasStateVector, VasTransition,
			VasValue,
		},
		vas_trie::VasTrieNode,
	},
	trace::{
		self,
		trace_trie::{self, TraceTrieNode},
	},
};

const MAX_TRACE_LENGTH: usize = 10000;

/// This is the builder for the Ragtimer tool, specifically for the RL Traces method.
/// It implements the `Builder` trait and provides methods to build the explicit state space
/// using reinforcement learning traces.
impl<'a> RagtimerBuilder<'a> {
	/// Function to set default magic numbers for the RL traces method.
	pub fn default_magic_numbers(&mut self) -> MagicNumbers {
		MagicNumbers {
			num_traces: 100,
			dependency_reward: 1.0,
			base_reward: 0.1,
			trace_reward: 0.01,
			smallest_history_window: 50,
			clamp: 10.0,
		}
	}

	/// Initializes the rewards for each transition in the model based on the dependency graph.
	/// For now, it initializes all rewards to zero, then adds DEPENDENCY_REWARD to the reward of any transition
	/// that appears in the dependency graph.
	fn initialize_rewards(
		&self,
		dependency_graph: &DependencyGraph,
	) -> HashMap<usize, RewardValue> {
		let mut rewards = HashMap::new();
		let magic_numbers = match &self.method {
			ReinforcementLearning(magic_numbers) => magic_numbers,
			_ => panic!("RagtimerBuilder::add_rl_traces called with non-RL method"),
		};
		let model = self.abstract_model;
		let all_transitions = model.transitions.clone();
		let dep_transitions = dependency_graph.get_transitions();
		for transition in all_transitions {
			rewards.insert(transition.transition_id, magic_numbers.base_reward);
		}
		for transition in dep_transitions {
			if let Some(reward) = rewards.get_mut(&transition.transition_id) {
				*reward += magic_numbers.dependency_reward;
			}
		}
		rewards
	}

	/// Updates the rewards based on the trace and its probability.
	/// This function will be called multiple times to update the rewards for the RL traces method.
	fn update_rewards(
		&mut self,
		rewards: &mut HashMap<usize, RewardValue>,
		trace: &Vec<usize>,
		trace_probability_history: &Vec<ProbabilityOrRate>,
	) {
		let magic_numbers = match &self.method {
			ReinforcementLearning(magic_numbers) => magic_numbers,
			_ => panic!("RagtimerBuilder::add_rl_traces called with non-RL method"),
		};
		let latest_probability = trace_probability_history.last().cloned().unwrap_or(0.0);
		if trace.len() == 0 || latest_probability <= 0.0 {
			debug_message!(
				"Skipping reward update for trace {:?} with probability {:.3e}",
				trace,
				latest_probability
			);
			return;
		}
		// Use the last 10% of entries to compute the average probability
		let history_len = trace_probability_history.len();
		let window_size = if history_len < magic_numbers.smallest_history_window {
			history_len
		} else {
			((history_len as f64) * 0.2).ceil() as usize
		};
		let window_size = window_size.max(1); // Ensure at least 1
		let start_idx = history_len.saturating_sub(window_size);
		let recent_probs = &trace_probability_history[start_idx..];
		let avg_recent_prob = if !recent_probs.is_empty() {
			recent_probs.iter().copied().sum::<f64>() / recent_probs.len() as f64
		} else {
			0.0
		};

		// Only give reward if this trace's probability is higher than the recent average
		// Reward is proportional to the log-ratio of latest to average probability.
		// This gives positive reward for increased probability, negative for decreased.
		// Clamp the log-ratio to avoid extreme values.
		let log_ratio = if avg_recent_prob > 0.0 && latest_probability > 0.0 {
			(latest_probability / avg_recent_prob).ln()
		} else {
			0.0
		};
		// Scale to a reasonable range
		let trace_reward = (log_ratio).clamp(-magic_numbers.clamp, magic_numbers.clamp)
			/ trace.len() as f64
			* magic_numbers.trace_reward;

		// Update the rewards for each transition in the trace
		for &transition_id in trace {
			if let Some(reward) = rewards.get_mut(&transition_id) {
				*reward += trace_reward;
				// debug_message!("Updated reward for transition {}: {:.3e}", transition_id, *reward));
			} else {
				error!("Transition ID {} not found in rewards map.", transition_id);
			}
		}
	}

	/// Maintains rewards at a reasonable level with the following rules:
	/// 1. If a reaction is in the dependency graph, it should have a reward of at least DEPENDENCY_REWARD.
	// TODO: Adjust this more as time goes on (run many tests to see what works best)
	fn maintain_rewards(
		&mut self,
		rewards: &mut HashMap<usize, RewardValue>,
		dependency_graph: &DependencyGraph,
	) {
		let magic_numbers = match &self.method {
			ReinforcementLearning(magic_numbers) => magic_numbers,
			_ => panic!("RagtimerBuilder::add_rl_traces called with non-RL method"),
		};
		for transition in dependency_graph.get_transitions() {
			if let Some(reward) = rewards.get_mut(&transition.transition_id) {
				if *reward < magic_numbers.dependency_reward {
					*reward = magic_numbers.dependency_reward;
				}
			}
		}
	}

	/// Returns a list of transition IDs that are enabled in the current state.
	fn get_available_transitions(&self, current_state: &VasStateVector) -> Vec<usize> {
		let x = self
			.abstract_model
			.transitions
			.iter()
			.filter(|t| {
				t.enabled_bounds
					.iter()
					.zip(current_state.iter())
					.all(|(bound, &val)| val >= (*bound).try_into().unwrap())
			})
			.map(|t| t.transition_id)
			.collect();
		// debug_message!("Available transitions: {:?}", x));
		x
	}

	/// Calculates the transition rate for a given transition in the context
	/// of the current state under the SCK assumption for CRN models.
	fn crn_transition_rate(
		&self,
		current_state: &VasStateVector,
		transition: &VasTransition,
	) -> ProbabilityOrRate {
		let mut transition_rate = 0.0;
		for (_, &current_value) in current_state.iter().enumerate() {
			transition_rate += transition.rate_const * (current_value as ProbabilityOrRate);
		}
		transition_rate
	}

	/// Calculates the transition probability for a given transition in the context
	/// of the current state under the SCK assumption for CRN models.
	fn crn_transition_probability(
		&self,
		current_state: &VasStateVector,
		transition: &VasTransition,
	) -> ProbabilityOrRate {
		let total_outgoing_rate = self.crn_total_outgoing_rate(current_state);
		// debug_message!(
		//     "Transition probability {:.3e} for transition {:?} in state {:?} with total outgoing rate {:.3e}",
		//     self.crn_transition_rate(current_state, transition) / total_outgoing_rate, transition, current_state, total_outgoing_rate
		// ));
		self.crn_transition_rate(current_state, transition) / total_outgoing_rate
	}

	/// Calculates the transition probability for a given transition in the context
	/// of the current state under the SCK assumption for CRN models.
	fn crn_total_outgoing_rate(&self, current_state: &VasStateVector) -> ProbabilityOrRate {
		let mut total_outgoing_rate = 0.0;
		let available_transitions = self.get_available_transitions(current_state);
		for t in available_transitions {
			if let Some(vas_transition) = self.abstract_model.get_transition_from_id(t) {
				total_outgoing_rate += self.crn_transition_rate(current_state, vas_transition);
			} else {
				error!("Transition ID {} not found in model.", t);
				return 0.0; // If the transition is not found, return 0 probability
			}
		}
		// debug_message!(
		//     "Total outgoing rate for state {:?} is {:.3e}",
		//     current_state, total_outgoing_rate
		// ));
		total_outgoing_rate
	}

	/// Stores the explicit trace in the explicit model.
	fn store_explicit_trace(&self, explicit_model: &mut PrismVasModel, trace: &Vec<usize>) {
		// Start with the initial state
		let mut current_state = self.abstract_model.initial_states[0].vector.clone();
		let mut next_state = current_state.clone();
		let mut current_state_id: usize = 0; // Start with the initial state ID
		let mut next_state_id: usize = 0;
		for &transition_id in trace {
			if let Some(vas_transition) = self.abstract_model.get_transition_from_id(transition_id)
			{
				// Store the current state with correct absorbing rate
				let available_state_id = explicit_model.states.len();
				if let Some(existing_id) = explicit_model
					.state_trie
					.insert_if_not_exists(&current_state, available_state_id)
				{
					current_state_id = existing_id;
				} else {
					warning!("During exploration, current state {:?} does not already exist in the model, but it should. Adding it under ID {}", current_state, available_state_id);
					current_state_id = available_state_id;
					let current_outgoing_rate = self.crn_total_outgoing_rate(&current_state);
					explicit_model.add_state(PrismVasState {
						state_id: current_state_id,
						vector: current_state.clone(),
						label: if current_state.len() > self.abstract_model.target.variable_index
							&& current_state[self.abstract_model.target.variable_index]
								== self.abstract_model.target.target_value
						{
							Some("target".to_string())
						} else {
							None
						},
						total_outgoing_rate: current_outgoing_rate,
					});
					explicit_model.add_transition(PrismVasTransition {
						transition_id: usize::MAX,
						from_state: current_state_id,
						to_state: 0,                 // Absorbing state
						rate: current_outgoing_rate, // Start out by assuming every outgoing transition goes to absorbing state
					});
					explicit_model
						.transition_map
						.entry(current_state_id)
						.or_insert_with(Vec::new)
						.push((0, explicit_model.transitions.len() - 1));
				}
				// Find the next state after applying the transition
				next_state = current_state.clone() + vas_transition.update_vector.clone();
				let available_state_id = explicit_model.states.len();
				if let Some(existing_id) = explicit_model
					.state_trie
					.insert_if_not_exists(&next_state, available_state_id)
				{
					next_state_id = existing_id;
				} else {
					next_state_id = available_state_id;
					let next_outgoing_rate = self.crn_total_outgoing_rate(&next_state);
					explicit_model.add_state(PrismVasState {
						state_id: next_state_id,
						vector: next_state.clone(),
						label: if next_state.len() > self.abstract_model.target.variable_index
							&& next_state[self.abstract_model.target.variable_index]
								== self.abstract_model.target.target_value
						{
							Some("target".to_string())
						} else {
							None
						},
						total_outgoing_rate: next_outgoing_rate,
					});
					explicit_model.add_transition(PrismVasTransition {
						transition_id: usize::MAX,
						from_state: next_state_id,
						to_state: 0,              // Absorbing state
						rate: next_outgoing_rate, // Start out by assuming every outgoing transition goes to absorbing state
					});
					explicit_model
						.transition_map
						.entry(next_state_id)
						.or_insert_with(Vec::new)
						.push((0, explicit_model.transitions.len() - 1));
				}
			} else {
				error!("Transition ID {} not found in model.", transition_id);
			}
			// Add the transition to the explicit model
			let transition_exists = explicit_model.transition_map.get(&current_state_id).map_or(
				false,
				|to_state_map| {
					to_state_map
						.iter()
						.any(|(to_state, _)| *to_state == next_state_id)
				},
			);
			if !transition_exists {
				let transition_rate = if let Some(vas_transition) =
					self.abstract_model.get_transition_from_id(transition_id)
				{
					self.crn_transition_rate(&current_state, vas_transition)
				} else {
					error!("Transition ID {} not found in model.", transition_id);
					0.0
				};
				explicit_model.add_transition(PrismVasTransition {
					transition_id,
					from_state: current_state_id,
					to_state: next_state_id,
					rate: transition_rate,
				});
				// Update the transition map
				explicit_model
					.transition_map
					.entry(current_state_id)
					.or_insert_with(Vec::new)
					.push((next_state_id, explicit_model.transitions.len() - 1));

				// Update the absorbing state transition of the current state to account for the new transition
				if let Some(outgoing_transitions) =
					explicit_model.transition_map.get_mut(&current_state_id)
				{
					// Find the index of the absorbing transition (to_state == 0)
					if let Some((absorbing_index, _)) = outgoing_transitions
						.iter()
						.find(|(to_state, _)| *to_state == 0)
					{
						// Update the rate of the absorbing transition
						let absorbing_transition =
							&mut explicit_model.transitions[*absorbing_index];
						absorbing_transition.rate -= transition_rate;
					}
				} else {
					error!("No outgoing transitions found for state ID {}. Something probably went wrong with its absorbing state.", current_state_id);
				}
			}
			current_state = next_state.clone();
		}
	}

	/// Generates a single trace based on the rewards and magic numbers.
	/// This function will be called multiple times to generate traces for the RL traces method.
	fn generate_single_trace(
		&mut self,
		rewards: &HashMap<usize, RewardValue>,
	) -> (Vec<usize>, ProbabilityOrRate) {
		let mut trace = Vec::new();
		let mut trace_probability = 1.0;
		let vas_target = &self.abstract_model.target;

		// Starting in the initial state, generate a trace
		let mut current_state = self.abstract_model.initial_states[0].vector.clone();
		while trace.len() < MAX_TRACE_LENGTH {
			// Check if we have reached the target state
			if current_state.len() > vas_target.variable_index {
				if current_state[vas_target.variable_index] == vas_target.target_value {
					break;
				}
			} else {
				error!(
					"Current state length {} is less than target variable index {}",
					current_state.len(),
					vas_target.variable_index
				);
			}
			// Get available transitions
			let available_transitions = self.get_available_transitions(&current_state);
			if available_transitions.is_empty() {
				// No available transitions, warn the user and break out of the loop
				warning!(
					"No available transitions found in state {:?}. Ending trace generation.",
					current_state
				);
				break;
			}
			// Shuffle the available transitions to add randomness
			let mut shuffled_transitions = available_transitions.clone();
			shuffled_transitions.shuffle(&mut rand::rng());
			// Find the total reward for the available transitions
			let total_reward: RewardValue = shuffled_transitions
				.iter()
				.filter_map(|&t_id| rewards.get(&t_id))
				.sum();
			// Pick a transition based on the rewards and magic numbers
			for (_, &transition) in shuffled_transitions.iter().enumerate() {
				// debug_message!("Considering transition {} ({}/{})", transition, index + 1, shuffled_transitions.len()));
				let transition_reward = rewards.get(&transition).unwrap_or(&0.0);
				// debug_message!("Considering transition {} with reward {}", transition, transition_reward));
				let selection_probability: RewardValue = if total_reward > 0.0 {
					transition_reward / total_reward
				} else {
					*transition_reward
				};
				if rand::rng().random::<RewardValue>() < selection_probability {
					if let Some(vas_transition) =
						self.abstract_model.get_transition_from_id(transition)
					{
						current_state = current_state + vas_transition.update_vector.clone();
						trace.push(transition);
						// debug_message!("Transition {} selected with reward {:.3e}. Current state updated to: {:?}", transition, transition_reward, current_state));
						trace_probability *=
							self.crn_transition_probability(&current_state, &vas_transition);
						// debug_message!(
						//     "Transition {} selected with reward {:.3e}. Current state updated to: {:?}, trace probability: {:.3e}",
						//     transition, transition_reward, current_state, trace_probability
						// ));
					} else {
						error!("Transition ID {} not found in model.", transition);
					}
					break;
				}
			}
		}

		(trace, trace_probability)
	}

	/// High-level function that builds the explicit state space with RL traces.
	pub fn add_rl_traces(
		&mut self,
		explicit_model: &mut PrismVasModel,
		dependency_graph: Option<&DependencyGraph>,
	) {
		let magic_numbers = match &self.method {
			ReinforcementLearning(magic_numbers) => magic_numbers,
			_ => panic!("RagtimerBuilder::add_rl_traces called with non-RL method"),
		};
		// Set up trace generation structures
		let mut trace_trie = TraceTrieNode::new();
		let mut trace_probability_history: Vec<ProbabilityOrRate> = Vec::new();

		// Set up state space storage structures
		explicit_model.state_trie = VasTrieNode::new();
		let current_state_id = 1;
		let current_state = self.abstract_model.initial_states[0].vector.clone();
		explicit_model
			.state_trie
			.insert_if_not_exists(&current_state, current_state_id);
		explicit_model.add_state(PrismVasState {
			state_id: current_state_id,
			vector: current_state.clone(),
			label: Some("init".to_string()),
			total_outgoing_rate: self.crn_total_outgoing_rate(&current_state),
		});
		let absorbing_state = DVector::from_element(current_state.len(), -1);
		let absorbing_state_id = 0;
		explicit_model
			.state_trie
			.insert_if_not_exists(&absorbing_state, absorbing_state_id);
		explicit_model.add_state(PrismVasState {
			state_id: absorbing_state_id,
			vector: absorbing_state,
			label: Some("sink".to_string()),
			total_outgoing_rate: 0.0,
		});

		// If the dependency graph is not provided, we try to construct it from the abstract model.
		let mut owned_dep_graph = None;
		let dependency_graph_ref: &DependencyGraph = match dependency_graph {
			Some(dep_graph) => dep_graph,
			None => {
				let dep_graph_result = make_dependency_graph(&self.abstract_model);
				match dep_graph_result {
					Ok(Some(dep_graph)) => {
						owned_dep_graph = Some(dep_graph);
						owned_dep_graph.as_ref().unwrap()
					}
					Ok(None) => {
						error!("No dependency graph could be constructed.");
						return;
					}
					Err(e) => {
						error!("Error constructing dependency graph: {}", e);
						return;
					}
				}
			}
		};
		let mut rewards = self.initialize_rewards(dependency_graph_ref);
		// Generate the traces one-by-one, repeating if the trace is not unique
		for i in 0..magic_numbers.num_traces {
			let mut trace;
			let mut trace_probability;
			loop {
				// Generate a single trace
				(trace, trace_probability) = self.generate_single_trace(&rewards);
				// If the trace already exists or is empty, we try to generate a new one.
				if !trace_trie.exists_or_insert(&trace) && !trace.is_empty() {
					break;
				}
				debug_message!("Trace {} already exists, generating a new one.", i);
			}
			debug_message!(
				"Generated trace {}: {:?} with probability {:.3e}",
				i,
				trace,
				trace_probability
			);
			trace_probability_history.push(trace_probability);
			// Store explicit prism states and transitions for this trace
			self.store_explicit_trace(explicit_model, &trace);
			// Update the rewards based on the trace
			self.update_rewards(&mut rewards, &trace, &trace_probability_history);
			self.maintain_rewards(&mut rewards, dependency_graph_ref);
		}
	}
}
