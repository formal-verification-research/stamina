use metaverify::trusted;

use crate::property::property::Labeled;

pub type ProbabilityOrRate = f64;

// TODO: should we include the skeleton code for nondeterministic actions?

/// A trait representing a state object. Generally these will need
/// to have some global context so implementing structs are recommended
/// to use lifetime parameters and contain a reference to the state
/// space's metadata (i.e., a variable ordering in the case of a VAS)

pub(crate) trait State: evalexpr::Context + Labeled + Clone + PartialEq {
	type VariableValueType: num::Integer;
	// type StateLabelType: Label;

	// Functions for which no default implementation is provided
	// and must be provided by derived types

	/// Valuates the state by a certain variable name

	fn valuate(&self, var_name: &str) -> Self::VariableValueType;
}

/// A trait representing a transition in a model

pub(crate) trait Transition: Clone + PartialEq {
	type StateType: State;
	type RateOrProbabilityType: num::Float;
	// type TransitionLabelType: Label;

	// Functions for which no default implementation is provided
	// and must be provided by derived types

	/// The rate or probability at the state `state`, if it's enabled
	fn rate_probability_at(&self, state: &Self::StateType) -> Option<Self::RateOrProbabilityType>;

	/// If this transition is enabled at state `state`, returns a `Some(StateType)` with the
	/// next state in it, otherwise returns `None`. Does not return rates.
	fn next_state(&self, state: &Self::StateType) -> Option<Self::StateType>;

	// Functions for which we can provide a default implementation

	/// Whether or not the transition is enabled to occur at `state`. It is recommended
	/// that implementing structs do NOT use the default implementation which just checks
	/// if `next_state(state)` returns a valid value.
	fn enabled(&self, state: &Self::StateType) -> bool {
		self.next_state(state).is_some()
	}

	/// Gets the next state and its rate probability if it exist given a
	/// current state. Provides a default implementation if not.
	fn next(
		&self,
		state: &Self::StateType,
	) -> Option<(Self::RateOrProbabilityType, Self::StateType)> {
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
	/// The model is in continuous time and transitions are exponentially distributed
	ContinuousTime,
	/// There are discrete time steps and the transitions are probabilities, rather than rates.
	DiscreteTime,
}

pub(crate) trait AbstractModel {
	type StateType: State;
	type TransitionType: Transition;

	// Functions for which no default implementation is provided
	// and must be provided by derived types

	fn transitions(&self) -> impl Iterator<Item = Self::TransitionType>;

	fn initial_states(&self) -> impl Iterator<Item = (Self::StateType, usize)>;
	/// The type of this model

	fn model_type(&self) -> ModelType;

	// Functions for which we can provide a default implementation

	// /// Finds all next states for a certain state.
	// fn next_states(&self, state: &Self::StateType)
	// 	-> impl Iterator<Item=(<Self::TransitionType as Transition>::RateOrProbabilityType, <<Self as AbstractModel>::TransitionType as Transition>::StateType)> {
	// 	self.transitions().filter_map(|t| t.next(state))
	// }

	// /// Finds the exit rate (or exit probability) for a state. If a discrete time model,
	// /// this will always return `1.0` and can be used to check if implementations are correct.
	// fn exit_rate(&self, state: &Self::StateType) -> <Self::TransitionType as Transition>::RateOrProbabilityType {
	// 	self.next_states(state).map(|(rate, _state)| rate).sum()
	// }

	// /// Only finds successors for transitions that pass a certain filter predicate `filter`.
	// /// This is useful in Wayfarer/ISR, as well as pancake abstraction.
	// fn next_filtered(&self, state: &Self::StateType, filter: &dyn Fn(Self::TransitionType) -> bool)
	// -> impl Iterator<Item=(<Self::TransitionType as Transition>::RateOrProbabilityType, Self::StateType)> {

	// self.transitions()
	// 	.filter(filter) // This filter call applies our filter function
	// 	.filter_map(|t| t.next(state)) // and this one filters enabledness
	// unimplemented!();
	// }
}

pub(crate) trait ExplicitModel: Default {
	type StateType: State;
	type TransitionType: Transition;
	type MatrixType; // TODO: derive shit for this nonsense

	/// Maps the state to a state index (in our case just a usize)
	fn state_to_index(&self, state: &Self::StateType) -> Option<usize>;

	/// Like `state_to_index` but if the state is not present adds it and
	/// assigns it a new index
	fn find_or_add_index(&mut self, state: &Self::StateType) -> usize;

	/// Reserve an index in the explicit model (useful for artificially introduced absorbing
	/// states). Returns whether or not the index was able to be reserved.
	fn reserve_index(&mut self, index: usize) -> bool;

	/// The number of states added to our model so far
	fn state_count(&self) -> usize;

	/// The type of this model
	fn model_type(&self) -> ModelType;

	/// Adds an entry to the sparse matrix
	fn add_entry(
		&mut self,
		from_idx: usize,
		to_idx: usize,
		entry: <Self::TransitionType as Transition>::RateOrProbabilityType,
	);

	/// Converts this model into a sparse matrix
	fn to_matrix(&self) -> Self::MatrixType;

	/// Whether or not this model has not been expanded yet/is empty
	fn is_empty(&self) -> bool;

	/// Whether or not `state` is present in the model
	fn has_state(&self, state: &Self::StateType) -> bool {
		self.state_to_index(state).is_some()
	}
}
