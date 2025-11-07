use crate::{
	builder::{builder::Builder, ragtimer::rl_traces::default_magic_numbers},
	cycle_commute::commute::cycle_commute,
	debug_message,
	logging::messages::error,
	message,
	model::{
		model::ProbabilityOrRate,
		vas_model::{
			AbstractVas, PrismVasModel, PrismVasState, PrismVasTransition, VasStateVector,
			VasTransition,
		},
	},
	warning,
};

pub type RewardValue = f64;
type LowerBound = Option<ProbabilityOrRate>;

pub(super) const MAX_TRACE_LENGTH: usize = 1000000;

/// Magic numbers used for RL traces in Ragtimer.
#[derive(Debug)]
pub struct MagicNumbers {
	pub num_traces: usize,
	pub dependency_reward: RewardValue,
	pub base_reward: RewardValue,
	pub trace_reward: RewardValue,
	pub smallest_history_window: usize,
	pub clamp: f64,
}

/// Enum representing the method used by Ragtimer to build the model.
pub enum RagtimerApproach {
	ReinforcementLearning(MagicNumbers),
	RandomPathExploration,
	RandomDependencyGraph(usize),
}

/// Builder for Ragtimer, which builds an abstracted model using the specified method.
/// It implements the Builder trait and is used to create a probability lower bound P_min
pub(crate) struct RagtimerBuilder<'a> {
	pub abstract_model: &'a AbstractVas,
	pub model_built: bool,
	pub approach: RagtimerApproach,
	pub traces_complete: usize,
}

impl<'a> Builder for RagtimerBuilder<'a> {
	type AbstractModelType = AbstractVas;
	type ExplicitModelType = PrismVasModel;
	type ResultType = LowerBound;

	/// Whether or not this model builder builds an abstracted model. In our case, yes.
	fn is_abstracted(&self) -> bool {
		true
	}

	/// Whether this model builder creates a model that should be used to create a
	/// probability lower bound ($P_{min}$). Wayfarer always creates a $P_{min}$ so this always
	/// returns true.
	fn creates_pmin(&self) -> bool {
		true
	}

	/// Whether this model builder creates a model that should be used to create a
	/// probability upper bound ($P_{max}$). Wayfarer can optionally also check upper bound but by
	/// default does not.
	fn creates_pmax(&self) -> bool {
		false
	}

	/// Whether or not we are finished or should continue. We only build once so this returns
	/// `false` if `build()` has not yet been called, and `true` if `build()` has been called.
	fn finished(&mut self, _result: &Self::ResultType) -> bool {
		self.model_built
	}

	/// Gets the abstract model that we're working with
	fn get_abstract_model(&self) -> &AbstractVas {
		self.abstract_model
	}

	/// Builds the explicit state space using the specified method.
	fn build(&mut self, explicit_model: &mut Self::ExplicitModelType) {
		// Do not try to rebuild the model
		if self.model_built {
			return;
		}
		let method = &self.approach;
		match method {
			RagtimerApproach::ReinforcementLearning(_) => {
				// self.method = RagtimerMethod::ReinforcementLearning(self.default_magic_numbers());
				self.add_rl_traces(explicit_model, None);
			}
			RagtimerApproach::RandomPathExploration => {
				todo!()
			}
			RagtimerApproach::RandomDependencyGraph(_) => {
				self.add_dep_traces(explicit_model, None);
			}
		}
		self.model_built = true;
	}
}

impl<'a> RagtimerBuilder<'a> {
	/// Creates a new RagtimerBuilder with the given abstract model and approach.
	pub fn new(abstract_model: &'a AbstractVas, approach: Option<RagtimerApproach>) -> Self {
		let mut builder = RagtimerBuilder {
			abstract_model,
			model_built: false,
			approach: RagtimerApproach::RandomPathExploration, // Placeholder will be set properly below
			traces_complete: 0,
		};
		if let Some(m) = approach {
			builder.approach = m;
		} else {
			builder.approach = RagtimerApproach::ReinforcementLearning(default_magic_numbers());
		}
		builder
	}

	/// Returns a list of transition IDs that are enabled in the current state.
	pub(super) fn get_available_transitions(&self, current_state: &VasStateVector) -> Vec<usize> {
		let available_transitions = self
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
		available_transitions
	}

	/// Returns a list of transition IDs that are enabled in the current state in a subset.
	pub(super) fn get_available_transition_subset(
		&self,
		current_state: &VasStateVector,
		subset: &Vec<VasTransition>,
	) -> Vec<usize> {
		let available_transitions = subset
			.iter()
			.filter(|t| {
				t.enabled_bounds
					.iter()
					.zip(current_state.iter())
					.all(|(bound, &val)| val >= (*bound).try_into().unwrap())
			})
			.map(|t| t.transition_id)
			.collect();
		available_transitions
	}

	/// Calculates the transition rate for a given transition in the context
	/// of the current state under the SCK assumption for CRN models.
	pub(super) fn crn_transition_rate(
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
	pub(super) fn crn_transition_probability(
		&self,
		current_state: &VasStateVector,
		transition: &VasTransition,
	) -> ProbabilityOrRate {
		let total_outgoing_rate = self.crn_total_outgoing_rate(current_state);
		self.crn_transition_rate(current_state, transition) / total_outgoing_rate
	}

	/// Calculates the transition probability for a given transition in the context
	/// of the current state under the SCK assumption for CRN models.
	pub(super) fn crn_total_outgoing_rate(
		&self,
		current_state: &VasStateVector,
	) -> ProbabilityOrRate {
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
		total_outgoing_rate
	}

	/// Stores the explicit trace in the explicit model.
	pub(super) fn store_explicit_trace(
		&mut self,
		explicit_model: &mut PrismVasModel,
		trace: &Vec<usize>,
	) {
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
						transition_id: transition_id,
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
						transition_id: transition_id,
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
		self.traces_complete += 1;
	}
}

pub fn ragtimer(
	model_file: &str,
	approach: RagtimerApproach,
	max_cycle_length: usize,
	max_commute_depth: usize,
	output: &str,
) {
	// Attempt to parse the model file
	if let Ok(mut abstract_model) = AbstractVas::from_file(model_file) {
		let mut explicit_model = PrismVasModel::from_abstract_model(&abstract_model);
		let approach = match approach {
			RagtimerApproach::ReinforcementLearning(magic_numbers) => {
				message!("Using Reinforcement Learning approach for Ragtimer.");
				let magic_numbers = magic_numbers;
				RagtimerApproach::ReinforcementLearning(magic_numbers)
			}
			RagtimerApproach::RandomPathExploration => {
				message!("Using Random Path Exploration approach for Ragtimer.");
				RagtimerApproach::RandomPathExploration
			}
			RagtimerApproach::RandomDependencyGraph(num_traces) => {
				message!("Using Deterministic Dependency Graph approach for Ragtimer.");
				RagtimerApproach::RandomDependencyGraph(num_traces)
			}
		};
		// Run trace generation
		let mut ragtimer_builder = RagtimerBuilder::new(&abstract_model, Some(approach));
		ragtimer_builder.build(&mut explicit_model);
		debug_message!("Traces added to explicit model with Ragtimer");
		// Run cycle and commute
		cycle_commute(
			&mut abstract_model,
			&mut explicit_model,
			max_commute_depth,
			max_cycle_length,
		);
		// Output the explicit model to PRISM files
		explicit_model.print_explicit_prism_files(output);
		message!("Ragtimer complete. Output written to {}", output);
	} else {
		error!("Failed to parse model file: {}", model_file);
	}
}
