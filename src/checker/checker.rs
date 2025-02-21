use crate::*;

use model::*;
use builder::*;

pub(crate) trait Checker {
	type AbstractModelType: AbstractModel;
	type ExplicitModelType: ExplicitModel;
	type BuilderType: Builder;

	// TODO
}
