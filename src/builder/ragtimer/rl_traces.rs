use std::collections::HashMap;

use itertools::Itertools;
use rand::{seq::SliceRandom, Rng};
use std::io::{stdout, Write};

use crate::{
	builder::ragtimer::ragtimer::{
		MagicNumbers, RagtimerApproach::ReinforcementLearning, RagtimerBuilder, RewardValue,
		MAX_TRACE_LENGTH,
	},
	dependency::graph::{make_dependency_graph, DependencyGraph},
	logging::messages::{debug_message, error, message},
	model::{
		model::ProbabilityOrRate,
		vas_model::{PrismVasModel, PrismVasState},
		vas_trie::VasTrieNode,
	},
	trace::trace_trie::TraceTrieNode,
};

const DEFAULT_NUM_TRACES: usize = 1000;
const DEFAULT_DEPENDENCY_REWARD: f64 = 100.0;
const DEFAULT_BASE_REWARD: f64 = 0.1;
const DEFAULT_TRACE_REWARD: f64 = 0.01;
const DEFAULT_SMALLEST_HISTORY_WINDOW: usize = 50;
const DEFAULT_CLAMP: f64 = 10.0;

/// Function to set default magic numbers for the RL traces method.
pub fn default_magic_numbers() -> MagicNumbers {
	MagicNumbers {
		num_traces: DEFAULT_NUM_TRACES,
		dependency_reward: DEFAULT_DEPENDENCY_REWARD,
		base_reward: DEFAULT_BASE_REWARD,
		trace_reward: DEFAULT_TRACE_REWARD,
		smallest_history_window: DEFAULT_SMALLEST_HISTORY_WINDOW,
		clamp: DEFAULT_CLAMP,
	}
}

/// This is the builder for the Ragtimer tool, specifically for the RL Traces method.
/// It implements the `Builder` trait and provides methods to build the explicit state space
/// using reinforcement learning traces.
impl<'a> RagtimerBuilder<'a> {
	/// Initializes the rewards for each transition in the model based on the dependency graph.
	/// For now, it initializes all rewards to zero, then adds DEPENDENCY_REWARD to the reward of any transition
	/// that appears in the dependency graph.
	fn initialize_rewards(
		&self,
		dependency_graph: &DependencyGraph,
	) -> HashMap<usize, RewardValue> {
		let mut rewards = HashMap::new();
		let magic_numbers = match &self.approach {
			ReinforcementLearning(magic_numbers) => magic_numbers,
			_ => panic!("RagtimerBuilder::add_rl_traces called with non-RL method"),
		};
		let num_dependencies = dependency_graph.get_transitions().len() as f64;
		debug_message!("Number of dependencies in graph: {}", num_dependencies);
		let effective_dependency_reward = magic_numbers.dependency_reward / num_dependencies;
		debug_message!(
			"Effective dependency reward per transition: {:.3e}",
			effective_dependency_reward
		);
		let model = self.abstract_model;
		let all_transitions = model.transitions.clone();
		for transition in all_transitions {
			rewards.insert(transition.transition_id, magic_numbers.base_reward);
			if let Some(distance) = dependency_graph.distance_to_root(&transition.transition_name) {
				let additional_reward = effective_dependency_reward / (distance as f64 + 1.0);
				if let Some(reward) = rewards.get_mut(&transition.transition_id) {
					*reward += additional_reward;
					debug_message!(
						"Transition {} is in dependency graph at distance {}: adding reward {:.3e}, total reward now {:.3e}",
						transition.transition_id,
						distance,
						additional_reward,
						*reward
					);
				}
			}
		}
		// debug_message!("Rewards for each transition:",);
		// for &transition_id in rewards.keys().sorted() {
		// 	if let Some(reward) = rewards.get(&transition_id) {
		// 		println!("  Transition {}: {:.3e}", transition_id, *reward);
		// 	}
		// }
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
		let magic_numbers = match &self.approach {
			ReinforcementLearning(magic_numbers) => magic_numbers,
			_ => panic!("RagtimerBuilder::add_rl_traces called with non-RL method"),
		};
		let latest_probability = trace_probability_history.last().cloned().unwrap_or(0.0);
		if trace.is_empty() {
			debug_message!("Skipping reward update for empty trace.");
			return;
		}
		if latest_probability <= 0.0 {
			// Apply a negative reward to each transition in the trace
			let penalty = -magic_numbers.trace_reward;
			for &transition_id in trace {
				if let Some(reward) = rewards.get_mut(&transition_id) {
					*reward += penalty;
				} else {
					error!("Transition ID {} not found in rewards map.", transition_id);
				}
			}
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
		let magic_numbers = match &self.approach {
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

	/// Generates a single trace based on the rewards and magic numbers.
	/// This function will be called multiple times to generate traces for the RL traces method.
	fn generate_rl_trace(
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
			let available_transitions = self
				.abstract_model
				.get_available_transitions(&current_state);
			if available_transitions.is_empty() {
				trace_probability *= 0.01;
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
				let transition_reward = rewards.get(&transition).unwrap_or(&0.0);
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
						trace_probability *= self
							.abstract_model
							.crn_transition_probability(&current_state, &vas_transition);
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
		message!("Beginning Ragtimer RL Trace Generation");
		let magic_numbers = match &self.approach {
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
			total_outgoing_rate: self.abstract_model.crn_total_outgoing_rate(&current_state),
		});

		// If the dependency graph is not provided, we try to construct it from the abstract model.
		let owned_dep_graph;
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
		// Print the dependency graph
		debug_message!("Dependency Graph:");
		dependency_graph_ref.nice_print(self.abstract_model);
		let mut rewards = self.initialize_rewards(dependency_graph_ref);
		// Generate the traces one-by-one, repeating if the trace is not unique
		let num_traces = magic_numbers.num_traces;
		// print a line to give whitespace for the progress bar
		println!("\nTRACE GENERATION PROGRESS:");
		for i in 0..num_traces {
			let mut trace;
			let mut trace_probability;
			let mut trace_attempts = 0;
			loop {
				// Generate a single trace
				(trace, trace_probability) = self.generate_rl_trace(&rewards);
				// If the trace already exists or is empty, we try to generate a new one.
				if !trace_trie.exists_or_insert(&trace) && !trace.is_empty() {
					break;
				}
				trace_attempts += 1;
				if trace_attempts > 20 {
					break;
				}
			}
			trace_probability_history.push(trace_probability);
			// Store explicit prism states and transitions for this trace
			self.store_explicit_trace(explicit_model, &trace);
			// Update the rewards based on the trace
			self.update_rewards(&mut rewards, &trace, &trace_probability_history);
			self.maintain_rewards(&mut rewards, dependency_graph_ref);
			// Print the trace generation progress every 100 traces
			let percent_step = (num_traces as f64 / 100.0).ceil().max(1.0) as usize;
			if i % percent_step == 0 || i == num_traces - 1 {
				let bar_width = 40;
				let progress = (i + 1) as f64 / num_traces as f64;
				let filled = (progress * bar_width as f64).round() as usize;
				let bar = format!(
					"\r|{}{}| {}/{} traces ({:.1}%)",
					"█".repeat(filled),
					" ".repeat(bar_width - filled),
					i + 1,
					num_traces,
					progress * 100.0
				);
				print!("{}", bar);
				stdout().flush().unwrap();
			}
		}
		println!("\n");
		message!("All RL traces generated.");
		explicit_model.trace_trie = trace_trie;

		message!(
			"Ragtimer RL Traces complete. Explicit model now has {} states and {} transitions.",
			explicit_model.states.len(),
			explicit_model.transitions.len()
		);
	}
}
