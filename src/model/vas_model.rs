use std::{collections::BTreeSet, iter::Map};

use super::model::*;

use nalgebra::{DVector, Matrix, SVector};

struct StateLabel {
	// TODO
}

pub(crate) struct VasState<const M: usize> {
	// The state values
	vector: SVector<i64, M>,
	// The labelset for this state
	labels: Option<BTreeSet<StateLabel>>
}

impl<const M: usize> State for VasState<M> {
	type VariableValueType = i64;
	// type StateLabelType = StateLabel;

	fn valuate(&self, var_name: &str) -> i64 {
	    // TODO
	}
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct VasTransition<const M: usize> {
	// The update vector
	update_vector: SVector<i64, M>,
	// The minimum elementwise count for a transition to be enabled
	enabled_bounds: SVector<i64, M>,
	// The rate constant used in CRNs
	rate_const: f64,
}

impl<const M: usize> Transition for VasTransition<M> {
	type StateType = VasState<M>;
	type RateOrProbabilityType = f64;

	fn rate_probability_at(&self, state: VasState) -> Option<f64> {
		self.rate_const * self.update_vector
			.zip_fold(state.vector, 1.0, |(state_i, update_i), acc| {
				if update_i <= 0.0 {
					Some(acc * state_i.powi(-update_i))
				} else {
					Some(acc)
				}
			})
	}

	fn next_state(&self, state: VasState) -> Option<StateType> {
		// TODO
	}
}

/// The data for an abstract Vector Addition System
pub(crate) struct AbstractVas<const M: usize> {
	init_states: Vec<VasState<M>>,
	trans: Vec<VasTransition<M>>,
	m_type: ModelType,
}

impl<const M: usize> AbstractModel for AbstractVas<M> {
	type TransitionType = VasTransition<M>;
	type StateType = VasState<M>;

	fn transitions(&self) -> Iterator<VasTransition<M>> {
	    self.trans.iter()
	}

	fn initial_states(&self) -> Iterator<VasState<M>> {
	    self.init_states.iter()
	}

	fn model_type(&self) -> ModelType {
		self.m_type
	}
}
