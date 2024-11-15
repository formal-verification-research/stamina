use prusti_contracts::*;

struct Transition<'a> {
    increment_vector: &'a [u64],
	decrement_vector: &'a [u64],
	transition_name: String,

}

struct DependencyGraph {

}

struct GraphNode<'a> {
	transition: Box<Transition<'a>>,
	children: Vec<Box<GraphNode<'a>>>,
}

impl<'a> GraphNode<'a> {
	// #[pure]
	#[requires(state.len() == self.transition.increment_vector.len())]
	#[requires(state.len() == self.transition.decrement_vector.len())]
	// #[ensures(result <==>
	// 		  //forall(|i: usize| {
	// 	//i < state.len() ==> (state[i] >= self.transition.decrement_vector[i])
	// }))]
	fn is_enabled(&self, state: &'a [u64]) -> bool {
		(0..state.len()).try_fold(true, |_acc, i| {
			body_invariant!(i < state.len());
			if state[i] >= self.transition.decrement_vector[i] {
				Some(true)
			}
			else {
				None
			}
		}).is_some()
	}

}
