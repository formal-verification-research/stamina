use std::default;

use crate::{
	builder::builder::Builder,
	model::{
		model::ProbabilityOrRate,
		vas_model::{AbstractVas, PrismVasModel},
	},
};

pub type RewardValue = f64;
type LowerBound = Option<ProbabilityOrRate>;

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

pub enum RagtimerMethod {
	ReinforcementLearning(MagicNumbers),
	DeterministicDependencyGraph,
}

pub(crate) struct RagtimerBuilder<'a> {
	pub abstract_model: &'a AbstractVas,
	pub model_built: bool,
	pub method: RagtimerMethod,
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
	fn finished(&mut self, result: &Self::ResultType) -> bool {
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

		let method = &self.method;
		match method {
			RagtimerMethod::ReinforcementLearning(_) => {
				// self.method = RagtimerMethod::ReinforcementLearning(self.default_magic_numbers());
				self.add_rl_traces(explicit_model, None);
			}
			RagtimerMethod::DeterministicDependencyGraph => {
				todo!()
			}
		}

		self.model_built = true;
	}
}

impl<'a> RagtimerBuilder<'a> {
	/// Creates a new RagtimerBuilder with the given abstract model and method.
	pub fn new(abstract_model: &'a AbstractVas, method: Option<RagtimerMethod>) -> Self {
		let mut builder = RagtimerBuilder {
			abstract_model,
			model_built: false,
			method: RagtimerMethod::DeterministicDependencyGraph,
		};

		if let Some(m) = method {
			builder.method = m;
		} else {
			builder.method = RagtimerMethod::ReinforcementLearning(builder.default_magic_numbers());
		}

		builder
	}
}
