use crate::*;

use model::*;

pub(crate) trait Builder {
	type AbstractModelType: AbstractModel;
	type ExplicitModelType: ExplicitModel;
	type ResultType: Clone + Copy + PartialEq;

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
	fn finished(&mut self, result: ResultType) -> bool;

	// TODO
}
