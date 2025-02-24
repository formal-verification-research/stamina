use evalexpr::*;

// TODO: should we include the skeleton code for nondeterministic actions?

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

/// A trait representing a state object. Generally these will need
/// to have some global context so implementing structs are recommended
/// to use lifetime parameters and contain a reference to the state
/// space's metadata (i.e., a variable ordering in the case of a VAS)
pub(crate) trait State: evalexpr::Context + Labeled + Clone + PartialEq {
	type VariableValueType: evalexpr::EvalexprInt;
	type StateLabelType: Label;
	
	// Functions for which no default implementation is provided
	// and must be provided by derived types

	/// Valuates the state by a certain variable name
	fn valuate(&self, var_name: &str) -> VariableValueType;
	
}

/// A trait representing a transition in a model
pub(crate) trait Transition: Labeled + Clone + PartialEq {
	type StateType: State;
	type RateOrProbabilityType: EvalexprFloat;
	type TransitionLabelType: Label;
	
	// Functions for which no default implementation is provided
	// and must be provided by derived types

	/// The rate or probability at the state `state`, if it's enabled
	fn rate_probability_at(&self, state: &dyn StateType) -> Option<RateOrProbabilityType>;
	/// If this transition is enabled at state `state`, returns a `Some(StateType)` with the
	/// next state in it, otherwise returns `None`. Does not return rates.
	fn next_state(&self, state: &dyn StateType) -> Option<StateType>;

	// Functions for which we can provide a default implementation

	fn enabled(&self, state: &dyn StateType) -> bool {
		self.next_state(state).is_some()
	}

	fn next(&self, state: &dyn StateType) -> Option<(RateOrProbabilityType, StateType)> {
		if let Some(rate) = self.rate_probability_at(state) {
			// If we can't unwrap the next_state the implementation of this
			// trait is wrong (only should be none if this trait is not enabled
			Some((rate, self.next_state(state).unwrap()))
		} else {
			None
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ModelType {
	ContinuousTime,
	DiscreteTime,
}

pub(crate) trait AbstractModel {
	type StateType: State;
	type TransitionType: Transition;

	// Functions for which no default implementation is provided
	// and must be provided by derived types
	
	fn transitions(&self) -> Iterator<TransitionType>;
	fn initial_states(&self) -> Iterator<StateType>;
	/// The type of this model
	fn model_type(&self) -> ModelType;
	
	// Functions for which we can provide a default implementation

	/// Finds all next states for a certain state.
	fn next_states(&self, state: &dyn StateType) 
		-> Iterator<(TransitionType::RateOrProbabilityType, StateType)> {
		self.transitions.filter_map(|t| t.next(state))
	}

	/// Only finds successors for transitions that pass a certain filter predicate `filter`.
	/// This is useful in Wayfarer/ISR, as well as pancake abstraction.
	fn next_filtered(&self, state: &dyn StateType, filter: &dyn Fn(TransitionType) -> bool)
		-> Iterator<(TransitionType::RateOrProbabilityType, StateType)> {

		self.transitions
			.filter(filter) // This filter call applies our filter function
			.filter_map(|t| t.next(state)) // and this one filters enabledness
	}
}

pub(crate) trait ExplicitModel: Default {
	type StateType: State;
	type TransitionType: Transition;
	type MatrixType; // TODO: derive shit for this nonsense

	/// Maps the state to a state index (in our case just a usize)
	fn state_to_index(&self, state: &dyn StateType) -> Option<usize>;
	/// Like `state_to_index` but if the state is not present adds it and
	/// assigns it a new index
	fn find_or_add_index(&mut self, state: &dyn StateType) -> usize;
	/// The number of states added to our model so far
	fn state_count(&self) -> usize;
	/// The type of this model
	fn model_type(&self) -> ModelType;
	/// Adds an entry to the sparse matrix
	fn add_entry(&mut self, from_idx: usize, to_idx: usize, entry: TransitionType::RateOrProbabilityType);
	/// Converts this model into a sparse matrix
	fn to_matrix(&self) -> MatrixType;
	/// Whether or not this model has not been expanded yet/is empty
	fn empty(&self) -> bool;

	/// Whether or not `state` is present in the model
	fn has_state(&self, state: &dyn StateType) -> bool {
		self.state_to_index(state).is_some()
	}
}
