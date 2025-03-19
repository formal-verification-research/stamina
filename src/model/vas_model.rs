use std::collections::BTreeSet;

use super::model::*;

use nalgebra::DVector;

struct StateLabel {
	// TODO
}

pub(crate) struct VasState {
	// The state values
	vector: DVector<i64>,
	// The labelset for this state
	labels: Option<BTreeSet<StateLabel>>
}

impl State for VasState {
	type VariableValueType = i64;
	// type StateLabelType = StateLabel;

	fn valuate(&self, var_name: &str) -> i64 {
	    // TODO
		unimplemented!();
	}
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct VasTransition {
	// The update vector
	update_vector: DVector<i64>,
	// The minimum elementwise count for a transition to be enabled
	enabled_bounds: DVector<i64>,
	// The rate constant used in CRNs
	rate_const: f64,
	// An override function to find the rate probability
	// (when this is not provided defaults to the implemenation in
	// rate_probability_at). The override must be stored in static
	// memory for now (may change this later).
	custom_rate_fn: Option<&'static dyn Fn(&VasState) -> f64>,
}

impl VasTransition {
	// pub fn set_vectors(&mut self, increment: Box<[u64]>, decrement: Box<[u64]>) {
	// 	self.update_vector = increment - decrement;
	// 	self.enabled_bounds = decrement;
	// }
	// pub fn set_rate(&mut self, rate: f64) {
	// 	self.rate_const = rate;
	// }
	pub fn set_custom_rate_fn(&mut self, rate_fn: &'static dyn Fn(&VasState) -> f64) {
		self.custom_rate_fn = Some(rate_fn);
	}
	pub fn new(increment: Box<[u64]>, decrement: Box<[u64]>, rate_const: f64) -> Self {
		Self { 
			update_vector: DVector::from(increment) - DVector::from(decrement), 
			enabled_bounds: DVector::from(decrement), 
			rate_const, 
			custom_rate_fn: None }
	}
}

impl Transition for VasTransition {
	type StateType = VasState;
	type RateOrProbabilityType = f64;

	/// Check to see if our state is above every bound in the enabled
	/// bound. We use try-fold to short circuit and return false if we
	/// encounter at least one value that does not satisfy.
	fn enabled(&self, state: &VasState) -> bool {
		self.enabled_bounds.
			iter()
			.zip(state)
			.try_fold(true,
				|(bound, state_val)|
				if state_val >= bound { Some(true) } else { None }
			).is_some()

	}

	fn rate_probability_at(&self, state: &VasState) -> Option<f64> {

		let enabled = self.enabled(state);
		if enabled {
			let rate = if let Some(rate_fn) = self.custom_rate_fn {
				rate_fn(state)
			} else {
				// Compute the transition rate using the same equation that
				// is used for the chemical kinetics equation
				self.rate_const * self.update_vector
				.zip_fold(&state.vector, 1.0, |(state_i, update_i), acc| {
					if update_i <= 0.0 {
						acc * state_i.powi(-update_i)
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
			state + self.update_vector
		} else {
			None
		}
	}
}

/// The data for an abstract Vector Addition System
pub(crate) struct AbstractVas {
	variable_names: Box<[String]>,
	init_states: Vec<VasState>,
	trans: Vec<VasTransition>,
	m_type: ModelType,
}

impl AbstractModel for AbstractVas {
	type TransitionType = VasTransition;
	type StateType = VasState;

	fn transitions(&self) -> impl Iterator<Item=VasTransition> {
	    self.trans.iter()
	}

	fn initial_states(&self) -> impl Iterator<Item=VasState> {
	    self.init_states.iter()
	}

	fn model_type(&self) -> ModelType {
		self.m_type
	}
}
