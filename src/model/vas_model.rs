use std::{collections::BTreeSet, iter::Map};

use super::model::*;

use nalgebra::{DVector, Matrix};

struct StateLabel {
	// TODO
}

struct VasState {
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
	}
}

struct VasTransition {
	// The update vector
	update_vector: DVector<i64>,
	// The minimum elementwise count for a transition to be enabled
	enabled_bounds: DVector<i64>,
	// The rate constant used in CRNs
	rate_const: f64,
}

impl Transition for VasTransition {
	type StateType = VasState;
	type RateOrProbabilityType = f64;

	fn rate_probability_at(&self, state: VasState) -> Option<f64> {
		self.rate_const * self.update_vector
			.zip_fold(state.vector, 1.0, |(state_i, update_i), acc| {
				if update_i <= 0.0 {
					acc * state_i.powi(-update_i)
				} else {
					acc
				}
			})
	}

	fn next_state(&self, state: VasState) -> Option<StateType> {
		// TODO
	}
}

/// The data for an abstract Vector Addition System
struct AbstractVas {
	init_states: Vec<VasState>,
	trans: Vec<VasTransition>,
	m_type: ModelType,
}

impl AbstractModel for AbstractVas {
	type TransitionType = VasTransition;
	type StateType = VasState;

	fn transitions(&self) -> Iterator<VasTransition> {
	    self.trans.iter()
	}

	fn initial_states(&self) -> Iterator<VasState> {
	    self.init_states.iter()
	}

	fn model_type(&self) -> ModelType {
		self.m_type
	}
}
