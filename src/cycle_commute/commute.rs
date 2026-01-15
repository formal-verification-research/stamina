use itertools::Itertools;

use crate::{
	model::vas_model::{
		AbstractVas, PrismVasModel, PrismVasState, PrismVasTransition, VasTransition,
	},
	trace::trace_trie::TraceTrieNode,
	*,
};

/// This is the main function that implements the cycle & commute algorithm.
/// It reads an explicit prism-style model (assuming EVERY trace leads to a target),
/// builds the state space from the trace,
/// builds the user-specified set of concurrent and cyclical transitions,
/// and generates the PRISM-style explicit state space files (.sta and .tra).
pub fn cycle_commute(
	abstract_model: &mut AbstractVas,
	explicit_model: &mut PrismVasModel,
	max_commute_depth: usize,
	max_cycle_length: usize,
) {
	if max_commute_depth == 0 && max_cycle_length == 0 {
		return;
	}
	message!(
		"Starting Cycle & Commute with max commute depth {} and max cycle length {}.",
		max_commute_depth,
		max_cycle_length
	);
	// Do a depth first search of the trace trie to gather traces
	let mut traces = Vec::new();
	// Stack is a vector of (transition id, current trace, current node)
	let mut stack: Vec<(Vec<usize>, &TraceTrieNode)> = Vec::new();
	if let TraceTrieNode::Node(children) = &explicit_model.trace_trie {
		for (child_id, child_node) in children {
			let mut initial_trace = Vec::new();
			initial_trace.push(child_id.clone());
			stack.push((initial_trace, child_node));
		}
	} else if let TraceTrieNode::LeafNode = &explicit_model.trace_trie {
		// No transitions at all
		error!("Error: Trace trie is empty");
		return;
	}

	while let Some((current_trace, current_node)) = stack.pop() {
		match current_node {
			TraceTrieNode::LeafNode => {
				let mut prism_trace: Vec<PrismVasTransition> = Vec::new();
				let mut current_state = abstract_model.initial_states[0].vector.clone();
				let mut current_state_id = 1;
				for &transition_id in current_trace.iter() {
					if let Some(transition) = abstract_model.get_transition_from_id(transition_id) {
						let next_state =
							(current_state.clone() + transition.update_vector.clone()).clone();
						let mut next_state_id = explicit_model.states.len();
						if let Some(existing_id) = explicit_model
							.state_trie
							.insert_if_not_exists(&next_state, next_state_id)
						{
							next_state_id = existing_id;
						} else {
							error!(
								"Error: New state with vector {:?} (next_state_id: {}) should have already been added to the explicit model at this phase. Current state_id: {}, current_state: {:?}, transition: {} ({})",
								next_state,
								next_state_id,
								current_state_id,
								current_state,
								transition_id,
								abstract_model.get_transition_from_id(transition_id).unwrap().transition_name,
							);
						}
						let prism_transition = explicit_model.transitions.iter().find(|t| {
							t.from_state == current_state_id && t.to_state == next_state_id
						});
						if let Some(prism_transition) = prism_transition {
							prism_trace.push(prism_transition.clone());
						} else {
							error!("Error: Transition from state {} to state {} not found in explicit model.", current_state_id, next_state_id);
						}
						current_state = next_state.clone();
						current_state_id = next_state_id;
					}
				}
				traces.push(prism_trace);
			}
			TraceTrieNode::Node(children) => {
				// Continue traversing the trie
				for (child_id, child_node) in children {
					let mut new_trace = current_trace.clone();
					new_trace.push(child_id.clone());
					stack.push((new_trace, child_node));
				}
			}
		}
	}

	let mut num_states_added = 0;

	if max_commute_depth > 0 {
		// Add commuted/parallel traces
		for trace in traces {
			commute(
				abstract_model,
				explicit_model,
				&trace,
				0,
				max_commute_depth,
				&mut num_states_added,
			);
		}
		message!(
			"Commuting complete. Method added {} states. Explicit model now has {} states and {} transitions. Now adding cycles...",
			num_states_added,
			explicit_model.states.len(),
			explicit_model.transitions.len()
		);
	} else {
		message!("Skipping commuting phase.");
	}

	if max_cycle_length > 0 {
		add_cycles(
			abstract_model,
			explicit_model,
			max_cycle_length,
			&mut num_states_added,
		);
	} else {
		message!("Skipping cycle addition phase.");
	}

	message!(
		"Cycle & Commute complete. Method added {} states in total. Explicit model now has {} states and {} transitions.",
		num_states_added,
		explicit_model.states.len(),
		explicit_model.transitions.len()
	);
}

/// Recursively takes the model and existing state space and generates
/// many concurrent traces, expanding the state space with parallel traces.
fn commute(
	abstract_model: &AbstractVas,
	explicit_model: &mut PrismVasModel,
	trace: &Vec<PrismVasTransition>,
	depth: usize,
	max_depth: usize,
	num_states_added: &mut usize,
) {
	// Base case: if the depth is greater than the max depth, return
	if depth >= max_depth {
		return;
	}
	// Get universally enabled transitions
	// Clone the state vector to avoid holding an immutable borrow during mutation
	let initial_state_vector = abstract_model.initial_states[0].vector.clone();
	let mut current_state = initial_state_vector.clone(); // Start from the initial state
													   // To do: maybe make this a hash set instead for faster lookups?
	let mut enabled_transitions: Vec<&VasTransition> = abstract_model
		.transitions
		.iter()
		.filter(|t| t.enabled_vector(&current_state))
		.collect();
	let mut universally_enabled_transitions: Vec<&VasTransition> = enabled_transitions.clone();
	for _transition in trace {
		current_state = initial_state_vector.clone(); // Start from the initial state
		enabled_transitions = abstract_model
			.transitions
			.iter()
			.filter(|t| t.enabled_vector(&current_state))
			.collect();
		universally_enabled_transitions.retain(|t| enabled_transitions.contains(t));
	}
	// debug_message!(
	// 	"At depth {}, found {} universally enabled transitions: {:?}",
	// 	depth,
	// 	universally_enabled_transitions.len(),
	// 	universally_enabled_transitions.iter().map(|t| t.transition_name.clone()).collect::<Vec<_>>()
	// );
	// Fire all universally enabled transitions from the initial state to create parallel traces
	// From each state in the trace, fire all universally enabled transitions
	// debug_message!(
	// 	"Trace: {:?}",
	// 	trace
	// 		.iter()
	// 		.map(|tt| abstract_model.get_transition_from_id(tt.transition_id).unwrap().transition_name.clone())
	// 		.collect::<Vec<_>>()
	// );
	let mut parallel_traces: Vec<Vec<PrismVasTransition>> = Vec::new();
	parallel_traces.extend(universally_enabled_transitions.iter().map(|_| Vec::new()));
	for (_, trace_transition) in trace.iter().enumerate() {
		let state_id = trace_transition.from_state;
		let state_vector = explicit_model.states[state_id].vector.clone();
		let abstract_trace_transition = abstract_model
			.get_transition_from_id(trace_transition.transition_id)
			.unwrap();
		for (commutable_index, commutable_transition) in
			universally_enabled_transitions.iter().enumerate()
		{
			// Compute the next state
			if !(commutable_transition.enabled_vector(&state_vector)) {
				continue;
			}
			let vertical_state =
				(state_vector.clone() + commutable_transition.update_vector.clone()).clone();
			// debug_message!(
			// 	"Firing transition {}\n\t{} +\n\t{} =\n\t{}",
			// 	commutable_transition.transition_name,
			// 	format!("{:?}", state_vector.iter().collect::<Vec<_>>()),
			// 	format!("{:?}", commutable_transition.update_vector.iter().collect::<Vec<_>>()),
			// 	format!("{:?}", vertical_state.iter().collect::<Vec<_>>())
			// );
			// Check that the next state contains only non-negative entries
			if vertical_state.iter().any(|&x| x < 0) {
				continue;
			}
			// Fire the commutable transition first
			let mut vertical_state_id = explicit_model.states.len();
			if let Some(existing_id) = explicit_model
				.state_trie
				.insert_if_not_exists(&vertical_state, vertical_state_id)
			{
				vertical_state_id = existing_id;
				// debug_message!(
				// 	"State {:?} already exists with ID {}",
				// 	vertical_state.iter().collect::<Vec<_>>(),
				// 	vertical_state_id
				// );
			} else {
				// Create a new state
				explicit_model.add_state(PrismVasState {
					state_id: vertical_state_id,
					vector: vertical_state.clone(),
					label: None,
					used_rate: 0.0,
					total_outgoing_rate: abstract_model.crn_total_outgoing_rate(&vertical_state),
				});
				// debug_message!(
				// 	"Added new state ID {} with vector {:?}",
				// 	vertical_state_id,
				// 	vertical_state.iter().collect::<Vec<_>>()
				// );
				*num_states_added += 1;
			}
			// Check if the transition already exists
			let transition_exists =
				explicit_model
					.transition_map
					.get(&state_id)
					.map_or(false, |to_state_map| {
						to_state_map
							.iter()
							.any(|(to_state, _)| *to_state == vertical_state_id)
					});
			if !transition_exists {
				// Create the new transition
				let new_transition = PrismVasTransition {
					transition_id: commutable_transition.transition_id,
					from_state: state_id,
					to_state: vertical_state_id,
					rate: commutable_transition.get_sck_rate(&state_vector),
				};
				explicit_model.add_transition(new_transition);
			}

			// Fire the trace transition from the new state
			if !(commutable_transition.enabled_vector(&vertical_state)) {
				continue;
			}
			let horizontal_state =
				(vertical_state.clone() + abstract_trace_transition.update_vector.clone()).clone();

			// Check that the next state contains only non-negative entries
			if horizontal_state.iter().any(|&x| x < 0) {
				continue;
			}
			let mut horizontal_state_id = explicit_model.states.len();
			if let Some(existing_id) = explicit_model
				.state_trie
				.insert_if_not_exists(&horizontal_state, horizontal_state_id)
			{
				horizontal_state_id = existing_id;
			} else {
				// Create a new state
				explicit_model.add_state(PrismVasState {
					state_id: horizontal_state_id,
					vector: horizontal_state.clone(),
					label: None,
					used_rate: 0.0,
					total_outgoing_rate: abstract_model.crn_total_outgoing_rate(&horizontal_state),
				});
				*num_states_added += 1;
				if *num_states_added % 1000 == 0 {
					debug_message!(
						"C&C added {} states so far\t(total {} states)",
						num_states_added,
						explicit_model.states.len()
					);
				}
			}
			// Check if the transition already exists
			let transition_exists = explicit_model
				.transition_map
				.get(&vertical_state_id)
				.map_or(false, |to_state_map| {
					to_state_map
						.iter()
						.any(|(to_state, _)| *to_state == horizontal_state_id)
				});
			let horizontal_new_transition = PrismVasTransition {
				transition_id: abstract_trace_transition.transition_id,
				from_state: vertical_state_id,
				to_state: horizontal_state_id,
				rate: abstract_trace_transition.get_sck_rate(&vertical_state),
			};
			if !transition_exists {
				// Create the new transition
				explicit_model.add_transition(horizontal_new_transition.clone());
			}
			// Update the parallel traces
			parallel_traces[commutable_index].push(horizontal_new_transition.clone());
		}
	}
	// Recurse on each parallel trace
	for parallel_trace in parallel_traces {
		commute(
			abstract_model,
			explicit_model,
			&parallel_trace,
			depth + 1,
			max_depth,
			num_states_added,
		);
	}
}

/// This function combinatorially finds cycles of transitions (i.e., update vectors add to 0)
/// and adds them to every where they are enabled.
fn add_cycles(
	abstract_model: &AbstractVas,
	explicit_model: &mut PrismVasModel,
	max_cycle_length: usize,
	num_states_added: &mut usize,
) {
	// Collect all transition indices for easier cycle enumeration
	let transition_indices: Vec<usize> = (0..abstract_model.transitions.len()).collect();
	// For all cycle lengths from 2 up to max_cycle_length
	// let mut seen_cycle_counts: Vec<Vec<usize>> = Vec::new();

	for cycle_len in (2..=max_cycle_length).rev() {
		// Generate all possible multisets (with repetition) of transitions
		for cycle in Itertools::combinations_with_replacement(transition_indices.iter(), cycle_len)
		{
			// For each multiset, check if the sum of update vectors is zero
			if (0..abstract_model.transitions[*cycle[0]].update_vector.len()).all(|j| {
				cycle
					.iter()
					.map(|&&i| abstract_model.transitions[i].update_vector[j])
					.sum::<i128>() == 0
			}) {
				// This is a cycle
				debug_message!("Applying cycle: {:?}.", cycle,);
				// Get every permutation of the cycle
				let mut cycle_permutations = Vec::new();
				let mut cycle_indices = cycle.clone();
				cycle_indices.sort(); // Ensure canonical order for deduplication
				for perm in cycle_indices
					.iter()
					.permutations(cycle_indices.len())
					.unique()
				{
					cycle_permutations.push(perm.into_iter().copied().collect::<Vec<_>>());
				}
				// Add the cycle to all states where it is enabled (i.e., where the current state + min_vector is non-negative)
				// Right now, 1 is the index of the first real initial state. 0 is the absorbing state.
				for state_id in 1..explicit_model.states.len() {
					let state_vector = explicit_model.states[state_id].vector.clone();
					// Check if the cycle is enabled at this state (state_vector + min_vector >= 0)
					// For each permutation of the cycle, try to fire the transitions in order
					for perm in &cycle_permutations {
						// For each permutation, find the min possible value for each values
						let mut min_vector =
							abstract_model.transitions[*perm[0]].update_vector.clone();
						let mut running_sum = min_vector.clone();
						for &idx in &perm[1..] {
							running_sum += abstract_model.transitions[*idx].update_vector.clone();
							for i in 0..min_vector.len() {
								if running_sum[i] < min_vector[i] {
									min_vector[i] = running_sum[i];
								}
							}
						}
						let enabled = state_vector
							.iter()
							.zip(min_vector.iter())
							.all(|(&s, &m)| s + m >= 0);
						if !enabled {
							continue;
						}
						let mut current_state = state_vector.clone();
						let mut current_state_id = state_id;
						// Try to apply each transition in the permutation
						for &&idx in perm {
							let transition = &abstract_model.transitions[idx];
							// Compute next state
							let next_state =
								current_state.clone() + transition.update_vector.clone();
							// Insert or get the state ID
							let mut next_state_id = explicit_model.states.len();
							if let Some(existing_id) = explicit_model
								.state_trie
								.insert_if_not_exists(&next_state, next_state_id)
							{
								next_state_id = existing_id;
							} else {
								// Compute total outgoing rate for the new state
								explicit_model.add_state(PrismVasState {
									state_id: next_state_id,
									vector: next_state.clone(),
									label: None,
									used_rate: 0.0,
									total_outgoing_rate: abstract_model
										.crn_total_outgoing_rate(&next_state),
								});
								*num_states_added += 1;
							}
							// Add transition if not already present
							let transition_exists = explicit_model
								.transition_map
								.get(&current_state_id)
								.map_or(false, |to_state_map| {
									to_state_map
										.iter()
										.any(|(to_state, _)| *to_state == next_state_id)
								});
							if !transition_exists {
								// Create the new transition
								let new_transition = PrismVasTransition {
									transition_id: explicit_model.transitions.len(),
									from_state: current_state_id,
									to_state: next_state_id,
									rate: transition.get_sck_rate(&current_state),
								};
								explicit_model.add_transition(new_transition);
							}
							current_state = next_state;
							current_state_id = next_state_id;
						}
					}
				}
				debug_message!(
					"C&C added {} states so far\t(total {} states)",
					num_states_added,
					explicit_model.states.len()
				);
			}
		}
	}
}
