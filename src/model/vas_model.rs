use std::{
	collections::{BTreeSet, HashMap},
	fmt,
};

use crate::{
	logging::messages::*,
	model::{model::ExplicitModel, vas_trie::VasTrieNode},
	parser::vas_file_reader,
	property::property,
	validator::vas_validator::validate_vas,
};

use metaverify::trusted;
use nalgebra::DVector;

use super::model::{AbstractModel, ModelType, ProbabilityOrRate, State, Transition};

/// Type alias for a VAS variable valuation
pub type VasValue = i128;
pub type VasStateVector = DVector<VasValue>;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct StateLabel {
	// Add fields as needed
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct VasState {
	// The state values
	pub(crate) vector: VasStateVector,
	// The labelset for this state
	labels: Option<BTreeSet<property::StateFormula>>,
}

impl VasState {
	// TODO: Maybe this shouldn't be none labels, or have an init label?

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

	fn get_value(&self, identifier: &str) -> Option<&evalexpr::Value<Self::NumericTypes>> {
		todo!()
	}

	fn call_function(
		&self,
		identifier: &str,
		argument: &evalexpr::Value<Self::NumericTypes>,
	) -> evalexpr::error::EvalexprResultValue<Self::NumericTypes> {
		todo!()
	}

	fn are_builtin_functions_disabled(&self) -> bool {
		todo!()
	}

	fn set_builtin_functions_disabled(
		&mut self,
		disabled: bool,
	) -> evalexpr::EvalexprResult<(), Self::NumericTypes> {
		todo!()
	}
	// Implement required methods for evalexpr::Context
}

impl State for VasState {
	type VariableValueType = u64;

	fn valuate(&self, var_name: &str) -> Self::VariableValueType {
		todo!()
	}
}

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
	// pub fn set_vectors(&mut self, increment: Box<[u64]>, decrement: Box<[u64]>) {
	// 	self.update_vector = increment - decrement;
	// 	self.enabled_bounds = decrement;
	// }
	// pub fn set_rate(&mut self, rate: ProbabilityOrRate) {
	// 	self.rate_const = rate;
	// }

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
}

impl VasTransition {
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
	pub(crate) z3_context: Option<z3::Context>, // Removed because z3::Context and z3::Config do not implement Clone
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
			z3_context: None, // z3_context is not initialized here
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
}

/// Transition data for Prism export of a VAS
pub(crate) struct PrismVasTransition {
	pub(crate) transition_id: usize,
	pub(crate) from_state: usize,
	pub(crate) to_state: usize,
	pub(crate) rate: ProbabilityOrRate,
}

/// Transition data for Prism export of a VAS
pub(crate) struct PrismVasState {
	pub(crate) state_id: usize,
	pub(crate) vector: DVector<i128>,
	pub(crate) label: Option<String>, // Optional label for the state, useful for sink states
	pub(crate) total_outgoing_rate: ProbabilityOrRate, // Optional total outgoing rate for the state
}

/// The data for an explicit Prism export of a VAS
// TODO: Do we want to have a target stored here?
pub(crate) struct PrismVasModel {
	pub(crate) variable_names: Vec<String>,
	pub(crate) states: Vec<PrismVasState>,
	pub(crate) transitions: Vec<PrismVasTransition>,
	pub(crate) m_type: ModelType,
	pub(crate) state_trie: VasTrieNode, // Optional trie for storing traces, if needed
	pub(crate) transition_map: HashMap<usize, Vec<(usize, usize)>>, // Quick transition from-(to, transitions list index) lookup
}

/// Default implementation for PrismVasModel
impl Default for PrismVasModel {
	fn default() -> Self {
		PrismVasModel {
			variable_names: Vec::new(),
			states: Vec::new(),
			transitions: Vec::new(),
			m_type: ModelType::ContinuousTime,
			state_trie: VasTrieNode::new(), // No trie by default
			transition_map: HashMap::new(), // No transitions by default
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
				total_outgoing_rate: 0.0, // No outgoing rate by default
			});
			return new_index; // Return the newly added index
		}
	}

	/// Reserve an index in the explicit model (useful for artificially introduced absorbing
	/// states). Returns whether or not the index was able to be reserved.
	fn reserve_index(&mut self, index: usize) -> bool {
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
		from_idx: usize,
		to_idx: usize,
		entry: <Self::TransitionType as Transition>::RateOrProbabilityType,
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
		model
	}

	/// Adds a transition to the model
	pub fn add_transition(&mut self, transition: PrismVasTransition) {
		self.transitions.push(transition);
	}

	/// Adds a state to the model
	pub fn add_state(&mut self, state: PrismVasState) {
		self.states.push(state);
	}
}
