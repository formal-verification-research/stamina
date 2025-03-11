use std::collections::BTreeSet;

use super::model::*;

use nalgebra::SVector;

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
		unimplemented!();
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
	// An override function to find the rate probability
	// (when this is not provided defaults to the implemenation in
	// rate_probability_at). The override must be stored in static
	// memory for now (may change this later).
	custom_rate_fn: Option<&'static dyn Fn(&VasState<M>) -> f64>,
}

impl<const M: usize> VasTransition<M> {
	fn set_vectors(&mut self, increment: Box<[u64]>, decrement: Box<[u64]>) {
		self.update_vector = increment - decrement;
		self.enabled_bounds = decrement;
	}
	fn set_rate(&mut self, rate: f64) {
		self.rate_const = rate;
	}
	fn set_custom_rate_fn(&mut self, rate_fn: &'static dyn Fn(&VasState<M>) -> f64) {
		self.custom_rate_fn = Some(rate_fn);
	}
}

impl<const M: usize> Transition for VasTransition<M> {
	type StateType = VasState<M>;
	type RateOrProbabilityType = f64;

	/// Check to see if our state is above every bound in the enabled
	/// bound. We use try-fold to short circuit and return false if we
	/// encounter at least one value that does not satisfy.
	fn enabled(&self, state: &VasState<M>) -> bool {
		self.enabled_bounds.
			iter()
			.zip(state)
			.try_fold(true,
				|(bound, state_val)|
				if state_val >= bound { Some(true) } else { None }
			).is_some()

	}

	fn rate_probability_at(&self, state: &VasState<M>) -> Option<f64> {

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
pub(crate) struct AbstractVas<const M: usize> {
	variable_names: [String; M],
	init_states: Vec<VasState<M>>,
	trans: Vec<VasTransition<M>>,
	m_type: ModelType,
}

impl<const M: usize> AbstractModel for AbstractVas<M> {
	type TransitionType = VasTransition<M>;
	type StateType = VasState<M>;

	fn transitions(&self) -> impl Iterator<Item=VasTransition<M>> {
	    self.trans.iter()
	}

	fn initial_states(&self) -> impl Iterator<Item=VasState<M>> {
	    self.init_states.iter()
	}

	fn model_type(&self) -> ModelType {
		self.m_type
	}
}
