use metaverify::*;

use crate::*;

use builder::*;
use model::*;

use std::collections::VecDeque;
#[trusted]
#[derive(Debug, Clone, Copy, PartialEq)]
enum RangeResult {
	NoResult,
	Range(f64, f64),
}
#[trusted]
impl Default for RangeResult {
	#[trusted]
	fn default() -> Self {
		Self::NoResult
	}
}
#[trusted]
struct StateProbability {
	state_id: usize,
	// reachability probability
	probability: f64,
	terminal: bool,
	new: bool,
}
#[trusted]
impl Default for StateProbability {
	#[trusted]
	fn default() -> Self {
		Self {
			state_id: 0,
			probability: 0.0,
			terminal: true,
			new: true,
		}
	}
}

#[trusted]
pub(crate) struct StaminaBuilder<AbstractModelType, ExplicitModelType>
	where
		AbstractModelType: AbstractModel,
		ExplicitModelType: ExplicitModel,
{

	abstract_model: Arc<AbstractModelType>,
	window: f64,
	kappa: f64, // probability threshold
	max_iters: u16,
	cur_iters: u16,
	// If this is provided then we make any state satisfying the right, or not satisfying
	// the left formula, absorbing
	state_formulae: Option<(StateFormula, StateFormula)>,
}

#[trusted]
impl StaminaBuilder<AbstractModelType, ExplicitModelType> {
	type StateType = AbstractModelType::StateType;
	/// Maps a state ID to that state's valuation and probabilistic information
	/// about it (i.e., what we currently think its reachability is).
	#[trusted]
	fn id_to_state(&self, id: usize) -> (StateType, Arc<StateProbability>) {
		// TODO
		unimplemented!();
	}

	/// This 
	#[trusted]
	fn reserve_state_index(&mut self, state: &StateType, index: usize) {
		// TODO
		unimplemented!();
	}

	#[trusted]
	fn find_or_create_sp(&mut self, state: &StateType) -> Arc<StateProbability> {
		unimplemented!();
	}

	#[trusted]
	fn can_preterminate(&self, state: &StateType) -> bool {
		if let Some(left_formula, right_formula) = self.state_formulae {
			!left_formula.satisfied(state) || right_formula.satisfied(state)
		} else {
			false
		}
	}
}
#[trusted]
impl<AbstractModelType, ExplicitModelType> Builder for StaminaBuilder<AbstractModelType, ExplicitModelType> {
	type AbstractModelType = StaminaBuilder::AbstractModelType;
	type ExplicitModelType = StaminaBuilder::ExplicitModelType;
	type ResultType = RangeResult;

	/// Because we have an absorbing state, this is an abstracted model
	#[trusted]
	fn is_abstracted(&self) -> bool {
		true
	}

	#[trusted]
	fn creates_pmin(&self) -> bool {
		true
	}
	#[trusted]
	fn creates_pmax(&self) -> bool {
		true
	}

	#[trusted]
	fn finished(&mut self, result: &ResultType) -> bool {
		match result {
			RangeResult::NoResult => {
				// TODO: other processing
				// We're not done
				false
			},
			Range(p_min, p_max) => {
				if p_min > p_max {
					panic!("Got invalid Pmin/Pmax pair! ({}/{})", p_min, p_max);
				}
				self.cur_iters += 1;
				self.cur_iters >= self.max_iters || p_max - p_min <= self.window
			}
		}
	}

	#[trusted]
	fn get_abstract_model(&self) -> Arc<AbstractModelType> {
		self.abstract_model.clone()
	}

	#[trusted]
	fn build(&mut self, explicit_model: &mut ExplicitModelType) {
		let model = self.abstract_model;
		if self.cur_iters == 0 {
			// For the artificial absorbing state
			// TODO: should this be in the ExplicitModel trait?
			explicit_model.reserve_index(0);
		}
		// Get the initial states and put them into a queue
		let mut queue = model.initial_states()
			.map(|(state, id)| {
				self.reserve_state_index(state, id);
				(state, self.find_or_create_sp(state))
			})
			.collect::<VecDeque<_>>();
		// Explore until the queue is empty
		while queue.len() > 0 {
			let (cur_state, sp) = queue.pop_front();
			// Optimization. If we can preterminate this state, then we don't need
			// to explore its successors.
			if self.can_preterminate(&cur_state) {
				sp.terminal = true;
				explicit_model.add_entry(sp.state_id, sp.state_id, 1.0);
				continue;
			}
			// Terminate if our threshold is low enough
			if sp.terminal && sp.probability < self.kappa {
				explicit_model.add_entry(sp.state_id, 0, model.exit_rate(cur_state));
				continue;
			}
			// Enqueue successors to this state and keep exploring
			let enqueue_all = sp.probability == 0;
			let exit_rate = if enqueue_all { 1.0 } else { model.exit_rate(cur_state) };
			for (rate, next_state) in model.next_states(cur_state) {
				let mut next_sp = self.find_or_create_sp(next_state);
				// Handle enqueuing for exploration
				if enqueue_all {
					// If the current reachability is zero then we can enqueue without
					// updating the successors' reachability
					queue.push_back((next_state, next_sp));
				} else {
					next_sp.probability += (rate / exit_rate) * sp.probability;
				}
				// Handle adding to the sparse matrix
				if sp.new {
					// TODO
				}
			}
		}
	}
}
