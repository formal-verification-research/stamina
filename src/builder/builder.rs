use crate::model::model::{AbstractModel, ExplicitModel};

/// A trait which must be implemented by any struct that
/// builds a model (i.e., converts it from `AbstractModel` to
/// `ExplicitModel`. The philosophy behind this trait is that
/// a `Checker` will iteratively
pub(crate) trait Builder {
	type AbstractModelType: AbstractModel;
	type ExplicitModelType: ExplicitModel;
	type ResultType: Clone + Copy + PartialEq + Default;

	/// Whether or not this model builder builds an abstracted model
	fn is_abstracted(&self) -> bool;

	/// Whether this model builder creates a model that should be used to create a
	/// probability lower bound ($P_{min}$)
	fn creates_pmin(&self) -> bool;

	/// Whether this model builder creates a model that should be used to create a
	/// probability upper bound ($P_{max}$)
	fn creates_pmax(&self) -> bool;

	/// Whether or not we are finished or should continue. The reason that this takes
	/// a `&mut self` is many implementations may want to only have exactly one
	/// iteration and keep an internal flag tripped after this function is called.
	fn finished(&mut self, result: &Self::ResultType) -> bool;

	/// Performs the next iteration of building the model
	fn build(&mut self, explicit_model: &mut Self::ExplicitModelType);

	/// Gets the abstract model that we're working with
	fn get_abstract_model(&self) -> &Self::AbstractModelType;

	// TODO
}
