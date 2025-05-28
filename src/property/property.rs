use evalexpr::*;
use metaverify::trusted;

use std::fmt::{Display, Error, Formatter};

/// A trait representing a label on a labeled type
#[trusted]
pub(crate) trait Label: ToString + Clone {
	type LabeledType;

	/// Whether or not a label represents a subset of this label.
	/// E.g., a label containing `"A > 5 & B < 3"` would be a subset
	/// of the label `"A > 5"`.
	#[trusted]
	fn contains(&self, label: &Self);
	/// Composes two labels to create a label that represents both
	#[trusted]
	fn compose(&self, label: &Self) -> Self;
}

/// A trait representing a labeled type
#[trusted]
pub(crate) trait Labeled {
	type LabelType: Label;

	// Functions for which no default implementation is provided
	// and must be provided by derived types

	/// Whether or note this object has label `label`
	#[trusted]
	fn has_label(&self, label: &Self::LabelType) -> bool;
	/// The labels associated with this object
	#[trusted]
	fn labels(&self) -> impl Iterator<Item = &Self::LabelType>;
}

#[trusted]
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum PropertyClass {
	ContinuousStochasticLogic,             // CSL (for CTMCs and CMDPs)
	ProbabilisticComputationTreeLogic,     // PCTL (for DTMCs and MDPs)
	ProbabilisticComputationTreeLogicStar, // PCTL* (extended PCTL for DTMCs and MDPs)
	LinearTemporalLogic,                   // Nonprobabilistic properties
}

#[trusted]
#[derive(Debug)]
pub(crate) enum Property {
	/// Where the state formula holds for all
	Globally(StateFormula),
	Finally(StateFormula, Option<f64>),             // Optional bound
	Until(StateFormula, StateFormula, Option<f64>), // Optional bound
}

#[trusted]
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

#[trusted]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub(crate) enum StateFormula {
	StateLabel(String),
	Expression(Box<crate::property::property::StateFormula>),
}

#[trusted]
impl Label for StateFormula {
	type LabeledType = StateFormula;
	#[trusted]
	fn contains(&self, label: &Self) {
		todo!()
	}

	#[trusted]
	fn compose(&self, label: &Self) -> Self {
		todo!()
	}
}

impl Display for StateFormula {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
		todo!()
	}
}
