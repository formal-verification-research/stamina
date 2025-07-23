use metaverify::*;

use crate::*;

use builder::*;
use model::*;

type LowerBound = Option<ProbabilityOrRate>;

struct AffineSpace {
	// TODO
}

pub(crate) struct WayfarerBuilder {
	create_pmin: bool,
	model_built: bool,
	solution_space: AffineSpace,
	nested_spaces: Option<Vec<AffineSpace>>,
	abstract_model: Arc<AbstractModelType>,
}

impl Builder for WayfarerBuilder {
	type AbstractModelType = /* TODO */;
	type ExplicitModelType = /* TODO */;
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
		self.create_pmin
	}

	/// Whether or not we are finished or should continue. We only build once so this returns
	/// `false` if `build()` has not yet been called, and `true` if `build()` has been called.
	fn finished(&mut self, result: &ResultType) -> bool {
		self.model_built
	}

	/// Gets the abstract model that we're working with
	fn get_abstract_model(&self) -> Arc<AbstractModelType> {
		self.abstract_model.clone()
	}

	/// Performs the next iteration of building the model
	fn build(&mut self, explicit_model: &mut ExplicitModelType) {
		// Forcibly do not try to rebuild the model
		if self.model_built {
			return;
		}
		unimplemented!();
		self.model_built = true;
	}

}
