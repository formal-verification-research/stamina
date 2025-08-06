/// This module implements the cycle commute algorithm for VAS models.
/// It generates a PRISM-compatible state space from a given trace file.
/// It then uses the trace to build a highly-concurrent and cyclical state space of the VAS model
use std::{
	fs::File,
	io::{BufRead, BufReader},
	collections::HashMap,
};

use nalgebra::DVector;

use crate::{
	model::{
		model::ProbabilityOrRate,
		vas_model::{AbstractVas, PrismVasModel, PrismVasState, PrismVasTransition, VasTransition, VasValue},
		vas_trie,
	}, trace::trace_trie::TraceTrieNode, *
};
use itertools::Itertools;
use std::io::Write;

/// Temporary constant max depth for the cycle commute algorithm.
const MAX_DEPTH: usize = 2;
const MAX_CYCLE_LENGTH: usize = 2;

// /// PrismStyleExplicitState represents a state in the PRISM-style explicit state space as described at
// /// <https://www.prismmodelchecker.org/manual/RunningPRISM/ExplicitModelImport>
// #[derive(Debug, Clone)]
// struct PrismStyleExplicitState {
// 	/// The VAS state vector
// 	state_vector: VasStateVector,
// 	/// The total outgoing rate of the state, used to calculate the absorbing rate and mean residence time
// 	total_rate: ProbabilityOrRate,
// 	/// Label for the state, currently unused
// 	label: String,
// 	/// Vector of next states, here only for convenience in lookup while building the state space.
// 	next_states: Vec<usize>,
// }

// impl PrismStyleExplicitState {
// 	/// Creates a new PrismStyleExplicitState from the given parameters.
// 	fn from_state(
// 		state_vector: VasStateVector,
// 		total_rate: ProbabilityOrRate,
// 		label: String,
// 		next_states: Vec<usize>,
// 	) -> Self {
// 		PrismStyleExplicitState {
// 			state_vector,
// 			total_rate,
// 			label,
// 			next_states,
// 		}
// 	}
// }

// /// This struct represents a transition in the PRISM-style explicit state space
// /// as described at https://www.prismmodelchecker.org/manual/RunningPRISM/ExplicitModelImport
// #[derive(Debug, Clone)]
// struct PrismStyleExplicitTransition {
// 	/// The ID (in Prism) of the state from which the transition originates
// 	from_state: usize,
// 	/// The ID (in Prism) of the state to which the transition goes
// 	to_state: usize,
// 	/// The CTMC rate (for Prism) of the transition
// 	rate: ProbabilityOrRate,
// }

/// This function calculates the outgoing rate of a transition.
/// It currently assumes the SCK assumption that the rate
/// depends on the product of the enabled bounds.
impl VasTransition {
	/// Calculates the SCK rate of the transition.
	/// This function is temporary and intended only for quick C&C result generation ---
	/// it will eventually be replaced by a system-wide more-powerful rate calculation
	/// that allows for more complex rate calculations.
	fn get_sck_rate(&self) -> ProbabilityOrRate {
		self.rate_const
			* self
				.enabled_bounds
				.iter()
				.filter(|&&r| r != 0)
				.map(|&r| (r as ProbabilityOrRate))
				.product::<ProbabilityOrRate>()
	}
}

/// This is the main function that implements the cycle & commute algorithm.
/// It reads an explicit prism-style model (assuming EVERY trace leads to a target), 
/// builds the state space from the trace,
/// builds the user-specified set of concurrent and cyclical transitions,
/// and generates the PRISM-style explicit state space files (.sta and .tra).
pub fn cycle_commute(abstract_model: &mut AbstractVas, explicit_model: &mut PrismVasModel, output_file: &str) {
	// Inititalize the bookkeeping things
	let mut current_state = abstract_model.initial_states[0].vector.clone();
	let mut current_state_id = 1;
	// State trie for super quick lookups
	let mut state_trie = vas_trie::VasTrieNode::new();
	state_trie.insert_if_not_exists(&current_state, current_state_id);
	
	// Do a depth first search of the trace trie to gather traces
	let mut traces = Vec::new();
	// Stack is a vector of (transition id, current trace, current node)
	let mut stack: Vec<(Vec<usize>, &TraceTrieNode)> = Vec::new();
	if let TraceTrieNode::Node(children) = &explicit_model.trace_trie {
		for (child_id, child_node) in children {
			let mut initial_trace = Vec::new();
			initial_trace.push(child_id.clone());
			stack.push((
				initial_trace,
				child_node,
			));
		}
	} else if let TraceTrieNode::LeafNode = &explicit_model.trace_trie {
		// No transitions at all
		error!("Error: Trace trie is empty");
		return;
	}

	while let Some((current_trace, current_node)) = stack.pop() {
		match current_node {
			TraceTrieNode::LeafNode => {
				// Reached the end of a trace
				let mut prism_trace: Vec<PrismVasTransition> = Vec::new();
				let mut current_state_id = 1;
				for &transition_id in current_trace.iter() {
					if let Some(transition) = abstract_model.get_transition_from_id(transition_id) {
						let next_state = (current_state.clone() + transition.update_vector.clone()).clone();
						let mut next_state_id = explicit_model.states.len();
						if let Some(existing_id) = state_trie.insert_if_not_exists(&next_state, next_state_id) {
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
						let prism_transition = explicit_model.transitions.iter()
							.find(|t| t.from_state == current_state_id && t.to_state == next_state_id);
						if let Some(prism_transition) = prism_transition {
							prism_trace.push(prism_transition.clone());
						} else {
							error!("Error: Transition from state {} to state {} not found in explicit model.", current_state_id, next_state_id);
						}						
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
					stack.push((
						new_trace,
						child_node,
					));
				}
			}
		}
	}

	// Add commuted/parallel traces
	for trace in traces {
		commute(
			abstract_model,
			explicit_model,
			&trace,
			0,
			MAX_DEPTH,
		);
	}





	
	
	// debug_message!("Found traces: {}", traces.len());

	// // for debugging, print all traces found so far
	// for trace in &traces {
	// 	debug_message!(
	// 		"{}",
	// 		trace
	// 			.iter()
	// 			.map(|t| t.to_string())
	// 			.collect::<Vec<_>>()
	// 			.join(" ")
	// 	);
	// }

	// Now we have all the traces plus the explicit model.



// 	for trace in traces {
// 		let trace = match trace {
// 			Ok(t) => t,
// 			Err(e) => {
// 				error!("Error reading trace line: {}", e);
// 				continue;
// 			}
// 		};
// 		// Reset current state for each trace
// 		current_state = abstract_model.initial_states[0].vector.clone();
// 		current_state_id = 1;
// 		// Build the state space from the original trace
// 		let transitions: Vec<&str> = trace.split_whitespace().collect();
// 		for transition_name in transitions {
// 			// Apply the transition to the current state
// 			let transition = abstract_model.get_transition_from_name(transition_name);
// 			if let Some(t) = transition {
// 				// Update the current state based on the transition
// 				let next_state =
// 					(current_state.clone().cast::<VasValue>() + t.update_vector.clone()).clone();
// 				let mut next_state_id = current_state_id + 1;
// 				if next_state.iter().any(|&x| x < 0) {
// 					error!(
// 						"ERROR: Next state contains non-positive values: {:?}",
// 						next_state
// 					);
// 					return;
// 				}
// 				// Add the new state to the trie if it doesn't already exist
// 				let potential_id = state_trie.insert_if_not_exists(&next_state, next_state_id);
// 				if potential_id.is_some() {
// 					next_state_id = potential_id.unwrap();
// 				} else {
// 					// TODO: This only works for CRNs right now. Need to generalize for VAS with custom formulas.
// 					let rate_sum = abstract_model
// 						.transitions
// 						.iter()
// 						.map(|trans| trans.get_sck_rate())
// 						.sum();
// 					prism_states.push(PrismStyleExplicitState::from_state(
// 						next_state.clone(),
// 						rate_sum,
// 						format!("State {}", current_state_id),
// 						Vec::new(),
// 					));
// 				}
// 				// Check if the transition is already in the current state's outgoing transitions
// 				if prism_states.get(current_state_id).map_or(true, |s| {
// 					!s.next_states.iter().any(|tr| *tr == next_state_id)
// 				}) {
// 					// Add the transition to the current state's outgoing transitions
// 					let this_transition = PrismStyleExplicitTransition {
// 						from_state: current_state_id,
// 						to_state: next_state_id,
// 						rate: t.get_sck_rate(),
// 					};
// 					prism_states[current_state_id]
// 						.next_states
// 						.push(next_state_id);
// 					prism_transitions.push(this_transition.clone());
// 					seed_trace.push(this_transition.clone());
// 				}
// 				// Move along the state space
// 				current_state = next_state.clone();
// 				current_state_id = next_state_id;
// 			} else {
// 				error!("ERROR: Transition {} not found in model", transition_name);
// 				return;
// 			}
// 		}
// 	}
// 	// Add commuted/parallel traces
// 	commute(
// 		&abstract_model,
// 		&mut prism_states,
// 		&mut state_trie,
// 		&mut prism_transitions,
// 		&seed_trace,
// 		0,
// 		MAX_DEPTH,
// 	);
// 	// Add cycles to the state space
// 	add_cycles(
// 		&abstract_model,
// 		&mut prism_states,
// 		&mut state_trie,
// 		&mut prism_transitions,
// 		MAX_CYCLE_LENGTH,
// 	);
// 	// Add transitions to the absorbing state
// 	for i in 1..prism_states.len() {
// 		let transition_to_absorbing = PrismStyleExplicitTransition {
// 			from_state: i,
// 			to_state: absorbing_state_id,
// 			rate: prism_states[i].total_rate
// 				- prism_transitions
// 					.iter()
// 					.filter(|tr| {
// 						tr.to_state != absorbing_state_id
// 							&& prism_states[i].next_states.contains(&tr.to_state)
// 					})
// 					.map(|tr| tr.rate)
// 					.sum::<ProbabilityOrRate>(),
// 		};
// 		prism_transitions.push(transition_to_absorbing);
// 	}
// 	PrismVasModel::print_explicit_prism_files(abstract_model, &prism_states, &prism_transitions, output_file);
// 	visualize_prism_state_space(&prism_states, &prism_transitions, output_file);
}

/// Recursively takes the model and existing state space and generates
/// many concurrent traces, expanding the state space with parallel traces.
fn commute(
	abstract_model: &AbstractVas,
	explicit_model: &mut PrismVasModel,
	trace: &Vec<PrismVasTransition>,
	depth: usize,
	max_depth: usize,
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
	debug_message!(
		"{} universally enabled transitions: {}",
		universally_enabled_transitions.len(),
		&universally_enabled_transitions
			.iter()
			.map(|t| t.transition_name.as_str())
			.collect::<Vec<_>>()
			.join(" ")
	);
	// Fire all universally enabled transitions from the initial state to create parallel traces
	// Do this in 2 steps:
	// Step 1. From each state in the trace, fire all universally enabled transitions
	for (i, trace_transition) in trace.iter().enumerate() {
		let state_id = trace_transition.from_state;
		let state_vector = explicit_model.states[state_id].vector.clone();
		for transition in &universally_enabled_transitions {
			// Compute the next state
			let next_state = (state_vector.clone() + transition.update_vector.clone()).clone();
			// Skip if next state has negative entries
			if next_state.iter().any(|&x| x < 0) {
				continue;
			}
			// Insert or get the state ID
			let mut next_state_id = explicit_model.states.len();
			if let Some(existing_id) = explicit_model.state_trie.insert_if_not_exists(&next_state, next_state_id) {
				next_state_id = existing_id;
			} else {
				// Compute total outgoing rate for the new state
				let rate_sum = abstract_model
					.transitions
					.iter()
					.map(|trans| trans.get_sck_rate())
					.sum();
				explicit_model.states.push(
					PrismVasState {
						state_id: next_state_id,
						vector: next_state.clone(),
						label: None,
						total_outgoing_rate: rate_sum,
					}
				);
			}
			// Check if this transition already exists
			let transition_exists = explicit_model.transition_map.get(&state_id).map_or(
				false,
				|to_state_map| {
					to_state_map
						.iter()
						.any(|(to_state, _)| *to_state == next_state_id)
				},
			);
			if !transition_exists {
				// Create the new transition
				let new_transition = PrismVasTransition { 
					transition_id: explicit_model.transitions.len(), 
					from_state: state_id, 
					to_state: next_state_id,
					rate: transition.get_sck_rate(),
				};
				// explicit_model[state_id].next_states.push(next_state_id);
				explicit_model.transitions.push(new_transition);
				// Step 2. For each new state, create a new trace with the transition added
				let mut new_trace = trace[..i + 1].to_vec();
				// Get the last transition index before mutably borrowing explicit_model
				let last_transition_index = explicit_model.transitions.len() - 1;
				// Now push the reference after the mutable borrow
				let last_transition = explicit_model.transitions[last_transition_index].clone();
				new_trace.push(last_transition);
				commute(
					abstract_model,
					explicit_model,
					&new_trace,
					depth + 1,
					max_depth,
				);
			}
		}
	}
}

// /// This function combinatorially finds cycles of transitions (i.e., update vectors add to 0)
// /// and adds them to every where they are enabled.
// fn add_cycles(
// 	model: &AbstractVas,
// 	prism_states: &mut Vec<PrismStyleExplicitState>,
// 	state_trie: &mut vas_trie::VasTrieNode,
// 	prism_transitions: &mut Vec<PrismStyleExplicitTransition>,
// 	max_cycle_length: usize,
// ) {
// 	// Collect all transition indices for easier cycle enumeration
// 	let transition_indices: Vec<usize> = (0..model.transitions.len()).collect();
// 	// For all cycle lengths from 2 up to max_cycle_length
// 	for cycle_len in 2..=max_cycle_length {
// 		// Generate all possible multisets (with repetition) of transitions
// 		for cycle in Itertools::combinations_with_replacement(transition_indices.iter(), cycle_len)
// 		{
// 			// For each multiset, check if the sum of update vectors is zero
// 			let mut sum_update = model.transitions[*cycle[0]].update_vector.clone();
// 			for &idx in &cycle[1..] {
// 				sum_update += model.transitions[*idx].update_vector.clone();
// 			}
// 			if sum_update.iter().all(|&x| x == 0) {
// 				// This is a cycle
// 				debug_message!("Found cycle: {:?}", cycle);
// 				// Get every permutation of the cycle
// 				let mut cycle_permutations = Vec::new();
// 				let mut cycle_indices = cycle.clone();
// 				cycle_indices.sort(); // Ensure canonical order for deduplication
// 				for perm in cycle_indices
// 					.iter()
// 					.permutations(cycle_indices.len())
// 					.unique()
// 				{
// 					cycle_permutations.push(perm.into_iter().copied().collect::<Vec<_>>());
// 				}
// 				// Add the cycle to all states where it is enabled (i.e., where the current state + min_vector is non-negative)
// 				// Right now, 1 is the index of the first real initial state. Eventually, maybe we make this safer by including
// 				// the absorbing state ID in the calculation
// 				for state_id in 1..prism_states.len() {
// 					let state_vector = prism_states[state_id].state_vector.clone();
// 					// Check if the cycle is enabled at this state (state_vector + min_vector >= 0)
// 					// For each permutation of the cycle, try to fire the transitions in order
// 					for perm in &cycle_permutations {
// 						// For each permutation, find the min possible value for each values
// 						let mut min_vector = model.transitions[*cycle[0]].update_vector.clone();
// 						let mut running_sum = min_vector.clone();
// 						for &idx in &cycle[1..] {
// 							running_sum += model.transitions[*idx].update_vector.clone();
// 							for i in 0..min_vector.len() {
// 								if running_sum[i] < min_vector[i] {
// 									min_vector[i] = running_sum[i];
// 								}
// 							}
// 						}
// 						let enabled = state_vector
// 							.iter()
// 							.zip(min_vector.iter())
// 							.all(|(&s, &m)| (s) + m >= 0);
// 						if !enabled {
// 							continue;
// 						}
// 						let mut current_state = state_vector.clone();
// 						let mut prev_state_id = state_id;
// 						// Try to apply each transition in the permutation
// 						for &&idx in perm {
// 							let transition = &model.transitions[idx];
// 							// Check if enabled: min_vector + update_vector must be non-negative
// 							if (current_state.clone() + transition.update_vector.clone())
// 								.iter()
// 								.any(|&x| x < 0)
// 							{
// 								break;
// 							}
// 							// Compute next state
// 							let next_state =
// 								current_state.clone() + transition.update_vector.clone();
// 							// Insert or get the state ID
// 							let mut next_state_id = prism_states.len();
// 							if let Some(existing_id) =
// 								state_trie.insert_if_not_exists(&next_state, next_state_id)
// 							{
// 								next_state_id = existing_id;
// 							} else {
// 								// Compute total outgoing rate for the new state
// 								let rate_sum = model
// 									.transitions
// 									.iter()
// 									.map(|trans| trans.get_sck_rate())
// 									.sum();
// 								prism_states.push(PrismStyleExplicitState::from_state(
// 									next_state.clone(),
// 									rate_sum,
// 									format!("State {}", next_state_id),
// 									Vec::new(),
// 								));
// 							}
// 							// Add transition if not already present
// 							if !prism_states[prev_state_id]
// 								.next_states
// 								.contains(&next_state_id)
// 							{
// 								let new_transition = PrismStyleExplicitTransition {
// 									from_state: prev_state_id,
// 									to_state: next_state_id,
// 									rate: transition.get_sck_rate(),
// 								};
// 								prism_states[prev_state_id].next_states.push(next_state_id);
// 								prism_transitions.push(new_transition);
// 							}
// 							current_state = next_state;
// 							prev_state_id = next_state_id;
// 						}
// 					}
// 				}
// 			}
// 		}
// 	}
// }

// /// This function takes the explicit state space and generates a visualization using Graphviz.
// fn visualize_prism_state_space(
// 	prism_states: &[PrismStyleExplicitState],
// 	prism_transitions: &[PrismStyleExplicitTransition],
// 	output_file: &str,
// ) {
// 	let mut dot_file = match File::create(format!("{}.dot", output_file)) {
// 		Ok(f) => f,
// 		Err(e) => {
// 			error!("Error creating .dot file: {}", e);
// 			return;
// 		}
// 	};
// 	writeln!(dot_file, "digraph StateSpace {{").unwrap();
// 	// Write nodes
// 	for (i, state) in prism_states.iter().enumerate() {
// 		let label = format!(
// 			"{}\\n({})",
// 			state.label,
// 			state
// 				.state_vector
// 				.iter()
// 				.map(|x| x.to_string())
// 				.collect::<Vec<_>>()
// 				.join(",")
// 		);
// 		writeln!(dot_file, "    {} [label=\"{}\"];", i, label).unwrap();
// 	}
// 	// Write edges
// 	for t in prism_transitions {
// 		writeln!(
// 			dot_file,
// 			"    {} -> {} [label=\"{:.2}\"];",
// 			t.from_state, t.to_state, t.rate
// 		)
// 		.unwrap();
// 	}
// 	writeln!(dot_file, "}}").unwrap();
// 	message!("Graphviz .dot file written to: {}.dot", output_file);
// 	message!("You can visualize it with: dot -Tpng -O <file>.dot");
// }
