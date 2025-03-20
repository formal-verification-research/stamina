use crate::*;

use model::*;
use builder::*;

pub(crate) trait Checker {
	type AbstractModelType: AbstractModel;
	type ExplicitModelType: ExplicitModel;
	type BuilderType: Builder;
	type ResultType: Clone + Copy + PartialEq + Default;

	// TODO
	
	fn builder(&self) -> &BuilderType;
	fn builder_mut(&self) -> &mut BuilderType;
	/// Checks the explicit model and returns a result
	fn check(&mut self, model: &ExplicitModelType) -> ResultType;
	
	/// Builds the model and checks it
	fn build_and_check(&mut self) -> ResultType {
		let mut bldr = self.builder_mut();
		
		let mut explicit_model: ExplicitModelType::default();
		let mut finished = false;
		while !finished {
			// TODO	
		}
	}

}
