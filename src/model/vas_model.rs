use std::{
	collections::{BTreeSet, HashMap},
	fmt,
	fs::File,
	io::stdout,
};

use crate::{
	logging::messages::*,
	model::{model::ExplicitModel, vas_trie::VasTrieNode},
	parser::vas_file_reader,
	property::property,
	trace::trace_trie::TraceTrieNode,
	validator::vas_validator::validate_vas,
	warning,
};

use nalgebra::DVector;
use std::io::Write;

use super::model::{AbstractModel, ModelType, ProbabilityOrRate, State, Transition};

const ROUNDING_ERROR: f64 = 1e-6;

/// Type alias for a VAS variable valuation
pub type VasValue = i128;
pub type VasStateVector = DVector<VasValue>;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct StateLabel {
	// Add fields as needed
}

/// A state in a Vector Addition System (VAS)
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct VasState {
	// The state values
	pub(crate) vector: VasStateVector,
	// The labelset for this state
	labels: Option<BTreeSet<property::StateFormula>>,
}

impl VasState {
	// TODO: Maybe this shouldn't be none labels, or have an init label?

	/// Creates a new VasState with the given vector
	pub fn new(vector: VasStateVector) -> Self {
		Self {
			vector,
			labels: None,
		}
	}
}

impl property::Labeled for VasState {
	type LabelType = property::StateFormula;

	fn labels(&self) -> impl Iterator<Item = &property::StateFormula> {
		self.labels
			.as_ref()
			.map(|labels| labels.iter())
			.into_iter()
			.flatten()
	}

	fn has_label(&self, label: &Self::LabelType) -> bool {
		self.labels
			.as_ref()
			.map_or(false, |labels| labels.contains(label))
	}
}

impl evalexpr::Context for VasState {
	type NumericTypes = evalexpr::DefaultNumericTypes; // Use the default numeric types provided by evalexpr

	fn get_value(&self, _identifier: &str) -> Option<&evalexpr::Value<Self::NumericTypes>> {
		todo!()
	}

	fn call_function(
		&self,
		_identifier: &str,
		_argument: &evalexpr::Value<Self::NumericTypes>,
	) -> evalexpr::error::EvalexprResultValue<Self::NumericTypes> {
		todo!()
	}

	fn are_builtin_functions_disabled(&self) -> bool {
		todo!()
	}

	fn set_builtin_functions_disabled(
		&mut self,
		_disabled: bool,
	) -> evalexpr::EvalexprResult<(), Self::NumericTypes> {
		todo!()
	}
	// Implement required methods for evalexpr::Context
}

impl State for VasState {
	type VariableValueType = u64;

	fn valuate(&self, _var_name: &str) -> Self::VariableValueType {
		todo!()
	}
}

/// A transition in a Vector Addition System (VAS)
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct VasTransition {
	pub(crate) transition_id: usize,
	pub(crate) transition_name: String,
	// The update vector
	pub(crate) update_vector: VasStateVector,
	// The minimum elementwise count for a transition to be enabled
	pub(crate) enabled_bounds: VasStateVector,
	// The rate constant used in CRNs
	pub(crate) rate_const: ProbabilityOrRate,
	// An override function to find the rate probability
	// (when this is not provided defaults to the implemenation in
	// rate_probability_at). The override must be stored in static
	// memory for now (may change this later).
	pub(crate) custom_rate_fn: Option<CustomRateFn>,
}

#[derive(Clone)]
pub(crate) struct CustomRateFn(
	std::sync::Arc<dyn Fn(&VasState) -> ProbabilityOrRate + Send + Sync + 'static>,
);

impl PartialEq for CustomRateFn {
	fn eq(&self, _: &Self) -> bool {
		false // Custom equality logic can be implemented if needed
	}
}

impl std::fmt::Debug for CustomRateFn {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str("CustomRateFn")
	}
}

impl CustomRateFn {
	fn set_custom_rate_fn(
		&mut self,
		rate_fn: std::sync::Arc<dyn Fn(&VasState) -> ProbabilityOrRate + Send + Sync + 'static>,
	) {
		self.0 = rate_fn;
	}
}

impl VasTransition {
	pub fn set_custom_rate_fn(
		&mut self,
		rate_fn: std::sync::Arc<dyn Fn(&VasState) -> ProbabilityOrRate + Send + Sync + 'static>,
	) {
		self.custom_rate_fn = Some(CustomRateFn(rate_fn));
	}

	pub fn new(
		transition_id: usize,
		transition_name: String,
		increment: Box<[VasValue]>,
		decrement: Box<[VasValue]>,
		rate_const: ProbabilityOrRate,
	) -> Self {
		Self {
			transition_id,
			transition_name,
			// update_vector: DVector::from_data(increment) - DVector::from_data(decrement),
			update_vector: DVector::from_iterator(
				increment.len(),
				increment
					.iter()
					.zip(decrement.iter())
					.map(|(inc, dec)| *inc - *dec),
			),
			enabled_bounds: DVector::from_iterator(decrement.len(), decrement),
			rate_const,
			custom_rate_fn: None,
		}
	}

	/// Calculates the SCK rate of the transition.
	/// This function is temporary and intended only for quick C&C result generation ---
	/// it will eventually be replaced by a system-wide more-powerful rate calculation
	/// that allows for more complex rate calculations.
	pub fn get_sck_rate(&self, state: &VasStateVector) -> ProbabilityOrRate {
		self.rate_const
			* self
				.enabled_bounds
				.iter()
				.zip(state.iter())
				.filter(|(bound, _)| **bound != 0)
				.map(|(_, &s)| s as ProbabilityOrRate)
				.product::<ProbabilityOrRate>()
	}

	/// Check to see if our state is above every bound in the enabled
	/// bound. We use try-fold to short circuit and return false if we
	/// encounter at least one value that does not satisfy.
	/// This function is used with a plain state vector rather than object.
	pub fn enabled_vector(&self, state: &VasStateVector) -> bool {
		self.enabled_bounds
			.iter()
			.zip(state.iter())
			.try_fold(true, |_, (bound, state_val)| {
				if *state_val >= *bound {
					Some(true)
				} else {
					None
				}
			})
			.is_some()
	}
}

impl Transition for VasTransition {
	type StateType = VasState;
	type RateOrProbabilityType = ProbabilityOrRate;

	/// Check to see if our state is above every bound in the enabled
	/// bound. We use try-fold to short circuit and return false if we
	/// encounter at least one value that does not satisfy.

	fn enabled(&self, state: &VasState) -> bool {
		self.enabled_bounds
			.iter()
			.zip(state.vector.iter())
			.try_fold(true, |_, (bound, state_val)| {
				if *state_val >= *bound {
					Some(true)
				} else {
					None
				}
			})
			.is_some()
	}

	fn rate_probability_at(&self, state: &VasState) -> Option<ProbabilityOrRate> {
		let enabled = self.enabled(state);
		if enabled {
			let rate = if let Some(rate_fn) = &self.custom_rate_fn {
				(rate_fn.0)(state)
			} else {
				// Compute the transition rate using the same equation that
				// is used for the chemical kinetics equation
				self.rate_const
					* self
						.update_vector
						.zip_fold(&state.vector, 1.0, |acc, state_i, update_i| {
							if (update_i as ProbabilityOrRate) <= 0.0 {
								acc * (state_i as ProbabilityOrRate)
									.powf(-(update_i as ProbabilityOrRate))
							} else {
								acc
							}
						})
			};
			Some(rate)
		} else {
			None
		}
	}

	fn next_state(&self, state: &VasState) -> Option<Self::StateType> {
		let enabled = self.enabled(state);
		if enabled {
			Some(VasState {
				vector: &state.vector + &self.update_vector.map(|val| val),
				labels: state.labels.clone(),
			})
		} else {
			None
		}
	}

	fn next(
		&self,
		state: &Self::StateType,
	) -> Option<(Self::RateOrProbabilityType, Self::StateType)> {
		if let Some(rate) = self.rate_probability_at(state) {
			// If we can't unwrap the next_state the implementation of this
			// trait is wrong (only should be none if this trait is not enabled
			Some((rate, self.next_state(state).unwrap()))
		} else {
			None
		}
	}
}

#[derive(Clone, Debug)]
pub struct VasProperty {
	pub(crate) variable_index: usize,
	pub(crate) target_value: VasValue,
}

/// The data for an abstract Vector Addition System
pub(crate) struct AbstractVas {
	pub(crate) variable_names: Box<[String]>,
	pub(crate) initial_states: Vec<VasState>,
	pub(crate) transitions: Vec<VasTransition>,
	pub(crate) m_type: ModelType,
	pub(crate) target: VasProperty,
	// pub(crate) z3_context: Option<z3::Context>, // Removed because z3::Context and z3::Config do not implement Clone
}

impl AbstractModel for AbstractVas {
	type TransitionType = VasTransition;
	type StateType = VasState;

	fn transitions(&self) -> impl Iterator<Item = VasTransition> {
		self.transitions.iter().cloned()
	}

	fn initial_states(&self) -> impl Iterator<Item = (VasState, usize)> {
		self.initial_states
			.iter()
			.cloned()
			.enumerate()
			.map(|(i, state)| (state, i))
	}

	fn model_type(&self) -> ModelType {
		self.m_type
	}
}

pub enum AllowedRelation {
	Equal,
	LessThan,
	GreaterThan,
}

impl fmt::Display for AllowedRelation {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let relation_str = match self {
			AllowedRelation::Equal => "=",
			AllowedRelation::LessThan => "<",
			AllowedRelation::GreaterThan => ">",
		};
		write!(f, "{}", relation_str)
	}
}

// TODO: May need to allow discrete/continuous time models
// for now we will just use continuous time models

impl AbstractVas {
	pub fn new(
		variable_names: Box<[String]>,
		initial_states: Vec<VasState>,
		transitions: Vec<VasTransition>,
		target: VasProperty,
	) -> Self {
		Self {
			variable_names,
			initial_states,
			transitions,
			m_type: ModelType::ContinuousTime,
			target,
			// z3_context: None, // z3_context is not initialized here
		}
	}

	/// Calls a parser to get a VAS model from a file

	pub fn from_file(filename: &str) -> Result<Self, String> {
		match vas_file_reader::build_model(filename) {
			Ok(model) => {
				debug_message!("Parsing gave OK result");
				Ok(model)
			}
			Err(err) => {
				error!("ERROR DURING PARSING: {}", err);
				Err(err.to_string())
			}
		}
	}

	/// Runs the validator on the model and its property

	pub fn validate_model(&self, property: VasProperty) -> Result<String, String> {
		let result = validate_vas(self, &property);
		result
	}

	/// Look up the index/ID of a transition by its name

	pub fn get_transition_from_name(&self, transition_name: &str) -> Option<&VasTransition> {
		self.transitions
			.iter()
			.find(|t| t.transition_name == transition_name)
	}

	/// Look up the name of a transition by its index

	pub fn get_transition_from_id(&self, transition_id: usize) -> Option<&VasTransition> {
		self.transitions
			.iter()
			.find(|t| t.transition_id == transition_id)
	}

	/// Outputs a model in a debuggable string format

	pub fn debug_print(&self) -> String {
		let mut output = String::new();
		output.push_str(&format!("VasModel:"));
		output.push_str(&format!("Variables: {:?}", self.variable_names));
		output.push_str(&format!("Initial States: {:?}", self.initial_states));
		output.push_str(&format!("Transitions: {:?}", self.transitions));
		output
	}

	/// Outputs a model in a human-readable string format

	pub fn nice_print(&self) -> String {
		let mut output = String::new();
		output.push_str("==========================================\n");
		output.push_str("              BEGIN VAS MODEL             \n");
		output.push_str("==========================================\n");
		output.push_str("Variables:\n");
		self.variable_names
			.iter()
			.for_each(|name| output.push_str(&format!("\t{}", name)));
		output.push_str("\n");
		output.push_str("Initial States:\n");
		for state in self.initial_states.clone() {
			state
				.vector
				.iter()
				.for_each(|name| output.push_str(&format!("\t{}", name)));
		}
		output.push_str("\n");
		output.push_str("Transitions:\n");
		for transition in self.transitions.clone() {
			output.push_str(&format!(
				"\t{}\t{}\n",
				transition.transition_id, transition.transition_name
			));
			output.push_str("\t\tUpdate:\t[");
			transition
				.update_vector
				.iter()
				.for_each(|name| output.push_str(&format!("\t{}", name)));
			output.push_str("\t]\n\t\tEnable:\t[");
			transition
				.enabled_bounds
				.iter()
				.for_each(|name| output.push_str(&format!("\t{}", name)));
			output.push_str(&format!("\t]\n\t\tRate:\t{}\n", transition.rate_const));
		}
		output.push_str("Target:\n");
		output.push_str(&format!(
			"\tVariable: {}\n",
			self.variable_names
				.get(self.target.variable_index)
				.map(|s| s.as_str())
				.unwrap_or("Unknown")
		));
		output.push_str(&format!("\tTarget Value: {}\n", self.target.target_value));
		output.push_str("==========================================\n");
		output.push_str("               END VAS MODEL              \n");
		output.push_str("==========================================\n");
		output
	}

	/// Returns a list of transition IDs that are enabled in the current state.
	pub fn get_available_transitions(&self, current_state: &VasStateVector) -> Vec<usize> {
		let available_transitions = self
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
		// debug_message!("Available transitions for {:?}: {:?}", current_state, available_transitions);
		available_transitions
	}

	/// Returns a list of transition IDs that are enabled in the current state in a subset.
	pub fn get_available_transition_subset(
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

	/// Calculates the transition probability for a given transition in the context
	/// of the current state under the SCK assumption for CRN models.
	pub fn crn_total_outgoing_rate(&self, current_state: &VasStateVector) -> ProbabilityOrRate {
		let mut total_outgoing_rate = 0.0;
		let available_transitions = self.get_available_transitions(current_state);
		for t in available_transitions {
			if let Some(vas_transition) = self.get_transition_from_id(t) {
				total_outgoing_rate += vas_transition.get_sck_rate(current_state);
				// debug_message!(
				// 	"TOTALING transition ID {} from state {:?} with rate {} to total outgoing rate, new total: {}",
				// 	t,
				// 	current_state.iter().collect::<Vec<_>>(),
				// 	vas_transition.get_sck_rate(),
				// 	total_outgoing_rate
				// );
			} else {
				error!("Transition ID {} not found in model.", t);
				return 0.0; // If the transition is not found, return 0 probability
			}
		}
		total_outgoing_rate
	}

	/// Calculates the probability for a transition given the current state
	pub fn transition_probability(
		&self,
		current_state: &VasStateVector,
		transition: &VasTransition,
	) -> ProbabilityOrRate {
		let total_outgoing_rate = self.crn_total_outgoing_rate(current_state);
		if total_outgoing_rate == 0.0 {
			warning!(
				"No outgoing transitions from state {}, returning 0 probability.",
				format!(
					"[{}]",
					current_state
						.iter()
						.map(|v| v.to_string())
						.collect::<Vec<_>>()
						.join(",")
				)
			);
			return 0.0; // No outgoing transitions, return 0 probability
		}
		if let Some(transition_rate) = transition.rate_probability_at(&VasState {
			vector: current_state.clone(),
			labels: None,
		}) {
			transition_rate / total_outgoing_rate
		} else {
			0.0 // Transition not enabled, return 0 probability
		}
	}
}

#[derive(Clone)]
/// Transition data for Prism export of a VAS
pub(crate) struct PrismVasTransition {
	pub(crate) transition_id: usize,
	pub(crate) from_state: usize,
	pub(crate) to_state: usize,
	pub(crate) rate: ProbabilityOrRate,
}

/// Transition data for Prism export of a VAS
#[derive(Clone)]
pub(crate) struct PrismVasState {
	pub(crate) state_id: usize,
	pub(crate) vector: DVector<i128>,
	pub(crate) label: Option<String>, // Optional label for the state, useful for sink states
	pub(crate) used_rate: ProbabilityOrRate, // Optional total outgoing rate for the state
	pub(crate) total_outgoing_rate: ProbabilityOrRate, // Optional total outgoing rate for the state
}

/// The data for an explicit Prism export of a VAS
// TODO: Do we want to have a target stored here?
#[derive(Clone)]
pub(crate) struct PrismVasModel {
	pub(crate) variable_names: Vec<String>,
	pub(crate) states: Vec<PrismVasState>,
	pub(crate) transitions: Vec<PrismVasTransition>,
	pub(crate) m_type: ModelType,
	pub(crate) state_trie: VasTrieNode, // Optional trie for storing states, if needed
	pub(crate) trace_trie: TraceTrieNode, // Optional trie for storing traces, if needed
	pub(crate) transition_map: HashMap<usize, Vec<(usize, usize)>>, // Quick transition from-(to, transition id) lookup
}

/// Default implementation for PrismVasModel
impl Default for PrismVasModel {
	fn default() -> Self {
		// Create the absorbing state
		let absorbing_state = DVector::from_element(0, -1);
		let absorbing_state_id = 0;
		// Add the absorbing state to the prism states
		let mut states = Vec::new();
		states.push(PrismVasState {
			state_id: absorbing_state_id,
			vector: absorbing_state,
			label: Some("absorbing".to_string()), // Label for the absorbing state
			used_rate: 0.0,                       // No used rate for the absorbing state
			total_outgoing_rate: 0.0,             // No outgoing rate for the absorbing state
		});
		PrismVasModel {
			variable_names: Vec::new(),
			states: states,
			transitions: Vec::new(),
			m_type: ModelType::ContinuousTime,
			state_trie: VasTrieNode::new(),   // No trie by default
			transition_map: HashMap::new(),   // No transitions by default
			trace_trie: TraceTrieNode::new(), // No trace trie by default
		}
	}
}

impl ExplicitModel for PrismVasModel {
	type StateType = VasState;
	type TransitionType = VasTransition;
	type MatrixType = (); // TODO: There is no matrix type for PrismVasModel, using this placeholder

	/// Maps the state to a state index (in our case just a usize)
	fn state_to_index(&self, state: &Self::StateType) -> Option<usize> {
		for my_state in self.states.iter() {
			if my_state.vector == state.vector.map(|v| v as i128) {
				return Some(my_state.state_id);
			}
		}
		None
	}

	/// Like `state_to_index` but if the state is not present adds it and
	/// assigns it a new index
	fn find_or_add_index(&mut self, state: &Self::StateType) -> usize {
		let index = self.state_to_index(state);
		if let Some(idx) = index {
			return idx; // State already exists, return its index
		} else {
			let new_index = self.states.len();
			self.states.push(PrismVasState {
				state_id: new_index,
				vector: state.vector.map(|v| v as i128),
				label: None,              // No label by default
				used_rate: 0.0,           // No used rate by default
				total_outgoing_rate: 0.0, // No outgoing rate by default
			});
			return new_index; // Return the newly added index
		}
	}

	/// Reserve an index in the explicit model (useful for artificially introduced absorbing
	/// states). Returns whether or not the index was able to be reserved.
	fn reserve_index(&mut self, _index: usize) -> bool {
		todo!()
	}

	/// The number of states added to our model so far
	fn state_count(&self) -> usize {
		todo!()
	}

	/// The type of this model
	fn model_type(&self) -> ModelType {
		todo!()
	}

	/// Adds an entry to the sparse matrix
	fn add_entry(
		&mut self,
		_from_idx: usize,
		_to_idx: usize,
		_entry: <Self::TransitionType as Transition>::RateOrProbabilityType,
	) {
		todo!()
	}

	/// Converts this model into a sparse matrix
	fn to_matrix(&self) -> Self::MatrixType {
		todo!()
	}

	/// Whether or not this model has not been expanded yet/is empty
	fn is_empty(&self) -> bool {
		self.states.is_empty() && self.transitions.is_empty()
	}

	/// Whether or not `state` is present in the model
	fn has_state(&self, state: &Self::StateType) -> bool {
		self.state_to_index(state).is_some()
	}
}

impl PrismVasModel {
	/// Creates a new empty PrismVasModel
	pub fn new() -> Self {
		Self::default()
	}

	pub fn from_abstract_model(abstract_model: &AbstractVas) -> Self {
		let mut model = Self::new();
		model.variable_names = abstract_model.variable_names.clone().into_vec();
		model.m_type = abstract_model.m_type;
		// Create the absorbing state
		let absorbing_state = DVector::from_element(model.variable_names.len(), -1);
		let absorbing_state_id = 0;
		model.states = Vec::new();
		// Add the absorbing state to the prism states
		model.add_state(PrismVasState {
			state_id: absorbing_state_id,
			vector: absorbing_state,
			label: Some("absorbing".to_string()), // Label
			used_rate: 0.0,                       // No used rate
			total_outgoing_rate: 0.0,             // No outgoing rate
		});
		model
	}

	/// Adds a transition to the model
	pub fn add_transition(&mut self, transition: PrismVasTransition) {
		let transition_id = transition.transition_id;
		let from_state = transition.from_state;
		let to_state = transition.to_state;
		// let transition_rate = transition.rate;
		if let Some(from_state) = self.states.iter_mut().find(|s| s.state_id == from_state) {
			from_state.used_rate += transition.rate;
			if from_state.used_rate > from_state.total_outgoing_rate + ROUNDING_ERROR {
				error!(
					"State {:?} has used rate {} greater than total outgoing rate {} after adding transition {}.",
					from_state.vector.iter().collect::<Vec<_>>(),
					from_state.used_rate,
					from_state.total_outgoing_rate,
					transition_id
				);
			}
		}
		self.transitions.push(transition);
		self.transition_map
			.entry(from_state)
			.or_insert_with(Vec::new)
			.push((to_state, self.transitions.len() - 1));
		// debug_message!(
		// 	"Added transition {} from state {} to state {} with rate {}.",
		// 	transition_id,
		// 	from_state,
		// 	to_state,
		// 	transition_rate,
		// );
	}

	/// Adds a state to the model
	pub fn add_state(&mut self, state: PrismVasState) {
		self.states.push(state);
	}

	/// Adds absorbing transitions to all states
	pub fn add_absorbing_transitions(&mut self) {
		// Clone states to avoid holding an immutable borrow on self while we mutably
		// add transitions to self.transitions.
		println!("\nABSORBING TRANSITION ADDITION PROGRESS:");
		let num_states = self.states.len();
		let mut num_added = 0;
		let percent_step = (num_states as f64 / 100.0).ceil().max(1.0) as usize;
		for state in self.states.clone() {
			if num_added % percent_step == 0 || num_added == num_states - 1 {
				let bar_width = 40;
				let progress = (num_added + 1) as f64 / num_states as f64;
				let filled = (progress * bar_width as f64).round() as usize;
				let bar = format!(
					"\r|{}{}| {}/{} states ({:.1}%)",
					"█".repeat(filled),
					" ".repeat(bar_width - filled),
					num_added + 1,
					num_states,
					progress * 100.0
				);
				print!("{}", bar);
				stdout().flush().unwrap();
			}
			num_added += 1;
			if state.state_id == 0 {
				// Skip the absorbing state itself
				continue;
			}
			let total_outgoing_rate = state.total_outgoing_rate;
			if total_outgoing_rate == 0.0 {
				// No need to add an absorbing transition if there are no outgoing transitions
				continue;
			}
			let used_rate = state.used_rate;
			if used_rate >= total_outgoing_rate {
				if used_rate > total_outgoing_rate + ROUNDING_ERROR {
					println!("");
					error!(
						"State {} {:?} has used rate {} greater than total outgoing rate {}",
						state.state_id,
						state.vector.iter().collect::<Vec<_>>(),
						used_rate,
						total_outgoing_rate
					);
					self.transitions.iter().for_each(|tr| {
						if tr.from_state == state.state_id {
							debug_message!(
								"    Transition {} from state {} ({:?}) to state {} ({:?}) with rate {}",
								tr.transition_id,
								tr.from_state,
								self.states[tr.from_state].vector.iter().collect::<Vec<_>>(),
								tr.to_state,
								self.states[tr.to_state].vector.iter().collect::<Vec<_>>(),
								tr.rate
							);
						}
					});
					panic!("Inconsistent rates for state {}", state.state_id);
				}
				// No need to add an absorbing transition if all rate is already used
				continue;
			}
			// Add the absorbing transition
			let transition = PrismVasTransition {
				transition_id: usize::MAX, // Use a special ID for absorbing transitions
				from_state: state.state_id,
				to_state: 0,
				rate: total_outgoing_rate - used_rate, // Absorbing transitions have zero rate
			};
			// debug_message!("vas_model.rs adding absorbing transition()");
			self.add_transition(transition);
		}
		println!("\n");
		message!("All absorbing transitions added.");
	}

	/// This function prints the PRISM-style explicit state space to .sta and .tra files.
	/// The .sta file contains the state vectors and their IDs,
	/// while the .tra file contains the transitions between states with their rates.
	pub fn print_explicit_prism_files(&mut self, output_file: &str) {
		// Write .lab file
		let mut lab_file = match File::create(format!("{}.lab", output_file)) {
			Ok(f) => f,
			Err(e) => {
				error!("Error creating .lab file: {}", e);
				return;
			}
		};

		// Write labels and state associations
		writeln!(lab_file, "0=\"init\" 1=\"deadlock\"").unwrap();
		writeln!(lab_file, "0: 1").unwrap();
		writeln!(lab_file, "1: 0").unwrap();
		// Write .sta file
		let mut sta_file = match File::create(format!("{}.sta", output_file)) {
			Ok(f) => f,
			Err(e) => {
				error!("Error creating .sta file: {}", e);
				return;
			}
		};

		// header info
		let num_states = self.states.len();
		let num_transitions = self.transitions.len();
		let var_names = self.variable_names.join(",");
		writeln!(sta_file, "({})", var_names).unwrap();
		// states
		for i in 0..num_states {
			let state_str = self.states[i]
				.vector
				.iter()
				.map(|x| x.to_string())
				.collect::<Vec<_>>()
				.join(",");
			writeln!(sta_file, "{}: ({})", i, state_str).unwrap();
		}
		// Write .tra file
		let mut tra_file = match File::create(format!("{}.tra", output_file)) {
			Ok(f) => f,
			Err(e) => {
				error!("Error creating .tra file: {}", e);
				return;
			}
		};
		writeln!(tra_file, "{} {}", num_states, num_transitions).unwrap();
		// transitions
		for t in self.transitions.iter() {
			writeln!(tra_file, "{} {} {}", t.from_state, t.to_state, t.rate).unwrap();
		}
		// Output results to the specified output file
		message!(
			"Resulting explicit state space written to: {}.tra,sta,lab",
			output_file
		);
		message!(
			"Check this with the following command:\n\n\tprism -importmodel {}.tra,sta,lab <property file>.csl -ctmc\n",
			output_file
		);
	}
}
