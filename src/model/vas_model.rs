use std::{collections::BTreeSet, fmt};

use crate::{parser::vas_file_reader, property::property, validator::vas_validator::validate_vas};

use creusot_contracts::trusted;
use nalgebra::DVector;

use super::model::{AbstractModel, ModelType, State, Transition};

#[trusted]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct StateLabel {
	// Add fields as needed
}
#[trusted]
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct VasState {
	// The state values
	pub vector: DVector<i64>,
	// The labelset for this state
	labels: Option<BTreeSet<property::StateFormula>>,
}
#[trusted]
impl VasState {
	// TODO: Maybe this shouldn't be none labels, or have an init label?
	#[trusted]
	pub fn new(vector: DVector<i64>) -> Self {
		Self { 
			vector, 
			labels: None,
		}
	}
}
#[trusted]
impl property::Labeled for VasState {
	
	type LabelType = property::StateFormula;

	#[trusted]
	fn labels(&self) -> impl Iterator<Item = &property::StateFormula> {
		self.labels
			.as_ref()
			.map(|labels| labels.iter())
			.into_iter()
			.flatten()
	}
		
	#[trusted]
	fn has_label(&self, label: &Self::LabelType) -> bool {
		self.labels
			.as_ref()
			.map_or(false, |labels| labels.contains(label))
	}
}

#[trusted]
impl evalexpr::Context for VasState {
	type NumericTypes = evalexpr::DefaultNumericTypes; // Use the default numeric types provided by evalexpr

	#[trusted]
	fn get_value(&self, identifier: &str) -> Option<&evalexpr::Value<Self::NumericTypes>> {
		todo!()
	}

	#[trusted]
	fn call_function(
		&self,
		identifier: &str,
		argument: &evalexpr::Value<Self::NumericTypes>,
	) -> evalexpr::error::EvalexprResultValue<Self::NumericTypes> {
		todo!()
	}

	#[trusted]
	fn are_builtin_functions_disabled(&self) -> bool {
		todo!()
	}

	#[trusted]
	fn set_builtin_functions_disabled(
		&mut self,
		disabled: bool,
	) -> evalexpr::EvalexprResult<(), Self::NumericTypes> {
		todo!()
	}
	// Implement required methods for evalexpr::Context
}

#[trusted]
impl State for VasState {
	type VariableValueType = u64; 

	#[trusted]
	fn valuate(&self, var_name: &str) -> Self::VariableValueType {
		todo!()
	}
}

#[trusted]
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct VasTransition {
	pub transition_id: usize,
	pub transition_name: String,
	// The update vector
	pub update_vector: DVector<i128>,
	// The minimum elementwise count for a transition to be enabled
	pub enabled_bounds: DVector<u64>,
	// The rate constant used in CRNs
	pub rate_const: f64,
	// An override function to find the rate probability
	// (when this is not provided defaults to the implemenation in
	// rate_probability_at). The override must be stored in static
	// memory for now (may change this later).
	pub custom_rate_fn: Option<CustomRateFn>,
}

#[trusted]
#[derive(Clone)]
pub(crate) struct CustomRateFn(std::sync::Arc<dyn Fn(&VasState) -> f64 + Send + Sync + 'static>);
#[trusted]
impl PartialEq for CustomRateFn {
	#[trusted]
	fn eq(&self, _: &Self) -> bool {
		false // Custom equality logic can be implemented if needed
	}
}
#[trusted]
impl std::fmt::Debug for CustomRateFn {
	#[trusted]
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str("CustomRateFn")
	}
}
#[trusted]
impl CustomRateFn {
	#[trusted]
	fn set_custom_rate_fn(&mut self, rate_fn: std::sync::Arc<dyn Fn(&VasState) -> f64 + Send + Sync + 'static>) {
		self.0 = rate_fn;
	}
}
#[trusted]
impl VasTransition {
	// pub fn set_vectors(&mut self, increment: Box<[u64]>, decrement: Box<[u64]>) {
	// 	self.update_vector = increment - decrement;
	// 	self.enabled_bounds = decrement;
	// }
	// pub fn set_rate(&mut self, rate: f64) {
	// 	self.rate_const = rate;
	// }
	#[trusted]
	pub fn set_custom_rate_fn(&mut self, rate_fn: std::sync::Arc<dyn Fn(&VasState) -> f64 + Send + Sync + 'static>) {
		self.custom_rate_fn = Some(CustomRateFn(rate_fn));
	}
	#[trusted]
	pub fn new(transition_id: usize, transition_name: String, increment: Box<[u64]>, decrement: Box<[u64]>, rate_const: f64) -> Self {
		Self { 
			transition_id,
			transition_name,
			// update_vector: DVector::from_data(increment) - DVector::from_data(decrement), 
			update_vector: DVector::from_iterator(
				increment.len(),
				increment.iter().zip(decrement.iter()).map(|(inc, dec)| *inc as i128 - *dec as i128),
			),
			enabled_bounds: DVector::from_iterator(
				decrement.len(),
				decrement.iter().map(|(dec)| *dec as u64),
			),
			rate_const, 
			custom_rate_fn: None }
	}
}
#[trusted]
impl Transition for VasTransition {
	type StateType = VasState;
	type RateOrProbabilityType = f64;

	/// Check to see if our state is above every bound in the enabled
	/// bound. We use try-fold to short circuit and return false if we
	/// encounter at least one value that does not satisfy.
	#[trusted]
	fn enabled(&self, state: &VasState) -> bool {
		self.enabled_bounds
			.iter()
			.zip(state.vector.iter())
			.try_fold(true, |_, (bound, state_val)| {
				if *state_val >= *bound as i64 { Some(true) } else { None }
			})
			.is_some()

	}

	#[trusted]
	fn rate_probability_at(&self, state: &VasState) -> Option<f64> {

		let enabled = self.enabled(state);
		if enabled {
			let rate = if let Some(rate_fn) = &self.custom_rate_fn {
				(rate_fn.0)(state)
			} else {
				// Compute the transition rate using the same equation that
				// is used for the chemical kinetics equation
				self.rate_const * self.update_vector
				.zip_fold(&state.vector, 1.0, |acc, state_i, update_i| {
					if (update_i as f64) <= 0.0 {
						acc * (state_i as f64).powf(-update_i as f64)
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

	#[trusted]
	fn next_state(&self, state: &VasState) -> Option<Self::StateType> {
		let enabled = self.enabled(state);
		if enabled {
			Some(VasState {
				vector: &state.vector + &self.update_vector.map(|val| val as i64),
				labels: state.labels.clone(),
			})
		} else {
			None
		}
	}
	
	#[trusted]
	fn next(&self, state: &Self::StateType) -> Option<(Self::RateOrProbabilityType, Self::StateType)> {
				if let Some(rate) = self.rate_probability_at(state) {
					// If we can't unwrap the next_state the implementation of this
					// trait is wrong (only should be none if this trait is not enabled
					Some((rate, self.next_state(state).unwrap()))
				} else {
					None
				}
			}
}

/// The data for an abstract Vector Addition System
#[trusted]
pub struct AbstractVas {
	pub variable_names: Box<[String]>,
	pub initial_states: Vec<VasState>,
	pub transitions: Vec<VasTransition>,
	pub m_type: ModelType,
}

#[trusted]
impl AbstractModel for AbstractVas {
	type TransitionType = VasTransition;
	type StateType = VasState;

	#[trusted]
	fn transitions(&self) -> impl Iterator<Item=VasTransition> {
		self.transitions.iter().cloned()
	}

	#[trusted]
	fn initial_states(&self) -> impl Iterator<Item=(VasState, usize)> {
		self.initial_states.iter().cloned().enumerate().map(|(i, state)| (state, i))
	}

	#[trusted]
	fn model_type(&self) -> ModelType {
		self.m_type
	}
}
#[trusted]
pub enum AllowedRelation {
	Equal,
	LessThan,
	GreaterThan,
}
#[trusted]
impl fmt::Display for AllowedRelation {
	#[trusted]
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let relation_str = match self {
			AllowedRelation::Equal => "=",
			AllowedRelation::LessThan => "<",
			AllowedRelation::GreaterThan => ">",
		};
		write!(f, "{}", relation_str)
	}
}
#[trusted]
// TODO: Temporarily created a property for VAS only. Eventually need to integrate with property.
pub struct VasProperty {
	pub variable_name: String,
	pub variable_id: usize,
	pub relation: AllowedRelation,
	pub value: u64,
}
#[trusted]
impl VasProperty {
	#[trusted]
	pub fn new(variable_name: String, variable_id: usize, relation: AllowedRelation, value: u64) -> Self {
		Self { variable_name, variable_id, relation, value }
	}
	#[trusted]
	pub fn nice_print(&self) -> String {
		format!("Property: {}({}) {} {}", self.variable_name, self.variable_id, self.relation, self.value)
	}
}

// TODO: May need to allow discrete/continuous time models
// for now we will just use continuous time models
#[trusted]
impl AbstractVas {
	#[trusted]
	pub fn new(variable_names: Box<[String]>, initial_states: Vec<VasState>, transitions: Vec<VasTransition>) -> Self {
		Self { 
			variable_names,
			initial_states, 
			transitions, 
			m_type: ModelType::ContinuousTime,
		}
	}
	#[trusted]
	pub fn from_file(filename: &str) -> Result<(Self,VasProperty), String> {
		match vas_file_reader::build_model(filename) {
			Ok((model, property)) => {
				println!("Parsing gave OK result");
				Ok((model, property))
			}
			Err(err) => {
				println!("ERROR DURING PARSING:");
				println!("{}", err.to_string());
				Err(err.to_string())
			},
		}
	}
	#[trusted]
	pub fn validate_model(&self, property: VasProperty) -> Result<String, String> {
		let result = validate_vas(self, &property);
		result
	}
	#[trusted]
	pub fn debug_print(&self) {
		println!("VasModel:");
		println!("Variables: {:?}", self.variable_names);
		println!("Initial States: {:?}", self.initial_states);
		println!("Transitions: {:?}", self.transitions);
	}
	#[trusted]
	pub fn nice_print(&self) -> String {
		let mut output = String::new();
		
		output.push_str("==========================================\n");
		output.push_str("              BEGIN VAS MODEL             \n");
		output.push_str("==========================================\n");
		
		output.push_str("Variables:\n");
		self.variable_names.iter().for_each(|name| output.push_str(&format!("\t{}", name)));
		output.push_str("\n");

		output.push_str("Initial States:\n");
		for state in self.initial_states.clone() {
			state.vector.iter().for_each(|name| output.push_str(&format!("\t{}", name)));
		}
		output.push_str("\n");

		output.push_str("Transitions:\n");
		for transition in self.transitions.clone() {
			output.push_str(&format!("\t{}\t{}\n", transition.transition_id, transition.transition_name));
			output.push_str("\t\tUpdate:\t[");
			transition.update_vector.iter().for_each(|name| output.push_str(&format!("\t{}", name)));
			output.push_str("\t]\n\t\tEnable:\t[");
			transition.enabled_bounds.iter().for_each(|name| output.push_str(&format!("\t{}", name)));
			output.push_str(&format!("\t]\n\t\tRate:\t{}\n", transition.rate_const));
		}

		output.push_str("==========================================\n");
		output.push_str("               END VAS MODEL              \n");
		output.push_str("==========================================\n");

		output
	}
}