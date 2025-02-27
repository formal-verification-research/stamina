use evalexpr::*;

use std::fmt::Display;

/// A trait representing a label on a labeled type
pub(crate) trait Label: ToString + Clone {
	type LabeledType;

	/// Whether or not a label represents a subset of this label.
	/// E.g., a label containing `"A > 5 & B < 3"` would be a subset
	/// of the label `"A > 5"`.
	fn contains(&self, label: &Self);
	/// Composes two labels to create a label that represents both
	fn compose(&self, label: &Self) -> Self;
}

/// A trait representing a labeled type
pub(crate) trait Labeled {
	type LabelType: Label;

	// Functions for which no default implementation is provided
	// and must be provided by derived types

	/// Whether or note this object has label `label`
	fn has_label(&self, label: &dyn StateLabelType) -> bool;
	/// The labels associated with this object
	fn labels(&self) -> Iterator<LabelType>;
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum PropertyClass {
	ContinuousStochasticLogic, // CSL (for CTMCs and CMDPs)
	ProbabilisticComputationTreeLogic, // PCTL (for DTMCs and MDPs)
	ProbabilisticComputationTreeLogicStar, // PCTL* (extended PCTL for DTMCs and MDPs)
	LinearTemporalLogic, // Nonprobabilistic properties
}

#[derive(Debug)]
pub(crate) enum Property {
	/// Where the state formula holds for all
    Globally(StateFormula),
    Finally(StateFormula, Option<f64>), // Optional bound
	Until(StateFormula, StateFormula, Option<f64>), // Optional bound
}

#[derive(Debug)]
pub(crate) enum PropertyQuery {
	/// We are computing the probability of something.
	Probability(Property), // TODO: should have Option<(evalexpr::Operator, f64)> for specific
						   // bounds? Or just leave this as is?
	MaxProbability(Property),
	SteadyState(Property),

}

/// A trait representing any type of CSL, PCTL, or LTL property
// pub(crate) trait PropertyQuery {
	// fn property_class(&self) -> PropertyClass;
// }

pub(crate) enum StateFormula {
	StateLabel(String),
	Expression(evalexpr::Expression)
}

impl Label for StateFormula {

}

impl Display for StateFormula {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
		// TODO
	}
}

