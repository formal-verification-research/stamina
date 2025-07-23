/// This module implements the cycle commute algorithm for VAS models.
/// It generates a PRISM-compatible state space from a given trace file.
/// It then uses the trace to build a highly-concurrent and cyclical state space of the VAS model
use std::{
	fs::File,
	io::{BufRead, BufReader},
};

use nalgebra::DVector;

use crate::{
	model::{
		model::ProbabilityOrRate,
		vas_model::{AbstractVas, VasStateVector, VasTransition, VasValue},
		vas_trie,
	},
	*,
};
use itertools::Itertools;
use std::io::Write;

/// Temporary constant max depth for the cycle commute algorithm.
const MAX_DEPTH: usize = 2;
const MAX_CYCLE_LENGTH: usize = 2;

/// PrismStyleExplicitState represents a state in the PRISM-style explicit state space as described at
/// <https://www.prismmodelchecker.org/manual/RunningPRISM/ExplicitModelImport>
#[derive(Debug, Clone)]
struct PrismStyleExplicitState {
	/// The VAS state vector
	state_vector: VasStateVector,
	/// The total outgoing rate of the state, used to calculate the absorbing rate and mean residence time
	total_rate: ProbabilityOrRate,
	/// Label for the state, currently unused
	label: String,
	/// Vector of next states, here only for convenience in lookup while building the state space.
	next_states: Vec<usize>,
}

impl PrismStyleExplicitState {
	/// Creates a new PrismStyleExplicitState from the given parameters.
	fn from_state(
		state_vector: VasStateVector,
		total_rate: ProbabilityOrRate,
		label: String,
		next_states: Vec<usize>,
	) -> Self {
		PrismStyleExplicitState {
			state_vector,
			total_rate,
			label,
			next_states,
		}
	}
}

/// This struct represents a transition in the PRISM-style explicit state space
/// as described at https://www.prismmodelchecker.org/manual/RunningPRISM/ExplicitModelImport
#[derive(Debug, Clone)]
struct PrismStyleExplicitTransition {
	/// The ID (in Prism) of the state from which the transition originates
	from_state: usize,
	/// The ID (in Prism) of the state to which the transition goes
	to_state: usize,
	/// The CTMC rate (for Prism) of the transition
	rate: ProbabilityOrRate,
}

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

/// This function prints the PRISM-style explicit state space to .sta and .tra files.
/// The .sta file contains the state vectors and their IDs,
/// while the .tra file contains the transitions between states with their rates.
fn print_prism_files(
	model: &AbstractVas,
	prism_states: &[PrismStyleExplicitState],
	prism_transitions: &[PrismStyleExplicitTransition],
	output_file: &str,
) {
	// Write .sta file
	let mut sta_file = match File::create(format!("{}.sta", output_file)) {
		Ok(f) => f,
		Err(e) => {
			error!("Error creating .sta file: {}", e);
			return;
		}
	};
	// header
	let var_names = model.variable_names.join(" ");
	writeln!(sta_file, "({})", var_names).unwrap();
	// states
	for i in 0..prism_states.len() {
		let state_str = prism_states[i]
			.state_vector
			.iter()
			.map(|x| x.to_string())
			.collect::<Vec<_>>()
			.join(",");
		writeln!(sta_file, "{}: ({})", i, state_str).unwrap();
	}
	// Write .tra file
	let mut tra_file = match File::create(format!("{}.tra", output_file)) {
		Ok(f) => f,
		Err(e) => {
			error!("Error creating .tra file: {}", e);
			return;
		}
	};
	// header
	let num_states = prism_states.len();
	let num_transitions = prism_transitions.len();
	writeln!(tra_file, "{} {}", num_states, num_transitions).unwrap();
	// transitions
	for t in prism_transitions.iter() {
		writeln!(tra_file, "{} {} {}", t.from_state, t.to_state, t.rate).unwrap();
	}
	// Output results to the specified output file
	message!(
		"Resulting explicit state space written to: {}.sta, .tra",
		output_file
	);
	message!(
		"Check this with the following command:\n
		prism -importtrans {}.tra -importstates {}.sta -ctmc",
		output_file,
		output_file
	);
}

/// This is the main function that implements the cycle & commute algorithm.
/// It reads a trace file, builds the state space from the trace,
/// builds the user-specified set of concurrent and cyclical transitions,
/// and generates the PRISM-style explicit state space files (.sta and .tra).
pub fn cycle_commute(model: &AbstractVas, trace_file: &str, output_file: &str) {
	// Read the trace list
	let trace_file = match File::open(trace_file) {
		Ok(f) => f,
		Err(e) => {
			error!("Error opening trace file: {}", e);
			return;
		}
	};
	// Inititalize the bookkeeping things
	let mut current_state = model.initial_states[0].vector.clone();
	let mut current_state_id = 1;
	let mut prism_states: Vec<PrismStyleExplicitState> = Vec::new();
	let mut prism_transitions: Vec<PrismStyleExplicitTransition> = Vec::new();
	let mut seed_trace: Vec<PrismStyleExplicitTransition> = Vec::new();
	// State trie for super quick lookups
	let mut state_trie = vas_trie::VasTrieNode::new();
	state_trie.insert_if_not_exists(&current_state, current_state_id);
	// Create the absorbing state
	let absorbing_state = DVector::from_element(current_state.len(), -1);
	let absorbing_state_id = 0;
	// Add the absorbing state to the prism states
	prism_states.insert(
		absorbing_state_id,
		PrismStyleExplicitState {
			state_vector: absorbing_state,
			total_rate: 0.0,
			label: "SINK".to_string(),
			next_states: Vec::new(),
		},
	);
	// Read the trace file line by line (traces are line-separated)
	let trace_reader = BufReader::new(trace_file);
	for trace in trace_reader.lines() {
		let trace = match trace {
			Ok(t) => t,
			Err(e) => {
				error!("Error reading trace line: {}", e);
				continue;
			}
		};
		// Reset current state for each trace
		current_state = model.initial_states[0].vector.clone();
		current_state_id = 1;
		// Build the state space from the original trace
		let transitions: Vec<&str> = trace.split_whitespace().collect();
		for transition_name in transitions {
			// Apply the transition to the current state
			let transition = model.get_transition_from_name(transition_name);
			if let Some(t) = transition {
				// Update the current state based on the transition
				let next_state =
					(current_state.clone().cast::<VasValue>() + t.update_vector.clone()).clone();
				let mut next_state_id = current_state_id + 1;
				if next_state.iter().any(|&x| x < 0) {
					error!(
						"ERROR: Next state contains non-positive values: {:?}",
						next_state
					);
					return;
				}
				// Add the new state to the trie if it doesn't already exist
				let potential_id = state_trie.insert_if_not_exists(&next_state, next_state_id);
				if potential_id.is_some() {
					next_state_id = potential_id.unwrap();
				} else {
					// TODO: This only works for CRNs right now. Need to generalize for VAS with custom formulas.
					let rate_sum = model
						.transitions
						.iter()
						.map(|trans| trans.get_sck_rate())
						.sum();
					prism_states.push(PrismStyleExplicitState::from_state(
						next_state.clone(),
						rate_sum,
						format!("State {}", current_state_id),
						Vec::new(),
					));
				}
				// Check if the transition is already in the current state's outgoing transitions
				if prism_states.get(current_state_id).map_or(true, |s| {
					!s.next_states.iter().any(|tr| *tr == next_state_id)
				}) {
					// Add the transition to the current state's outgoing transitions
					let this_transition = PrismStyleExplicitTransition {
						from_state: current_state_id,
						to_state: next_state_id,
						rate: t.get_sck_rate(),
					};
					prism_states[current_state_id]
						.next_states
						.push(next_state_id);
					prism_transitions.push(this_transition.clone());
					seed_trace.push(this_transition.clone());
				}
				// Move along the state space
				current_state = next_state.clone();
				current_state_id = next_state_id;
			} else {
				error!("ERROR: Transition {} not found in model", transition_name);
				return;
			}
		}
	}
	// Add commuted/parallel traces
	commute(
		&model,
		&mut prism_states,
		&mut state_trie,
		&mut prism_transitions,
		&seed_trace,
		0,
		MAX_DEPTH,
	);
	// Add cycles to the state space
	add_cycles(
		&model,
		&mut prism_states,
		&mut state_trie,
		&mut prism_transitions,
		MAX_CYCLE_LENGTH,
	);
	// Add transitions to the absorbing state
	for i in 1..prism_states.len() {
		let transition_to_absorbing = PrismStyleExplicitTransition {
			from_state: i,
			to_state: absorbing_state_id,
			rate: prism_states[i].total_rate
				- prism_transitions
					.iter()
					.filter(|tr| {
						tr.to_state != absorbing_state_id
							&& prism_states[i].next_states.contains(&tr.to_state)
					})
					.map(|tr| tr.rate)
					.sum::<ProbabilityOrRate>(),
		};
		prism_transitions.push(transition_to_absorbing);
	}
	print_prism_files(model, &prism_states, &prism_transitions, output_file);
	visualize_prism_state_space(&prism_states, &prism_transitions, output_file);
}

/// Recursively takes the model and existing state space and generates
/// many concurrent traces, expanding the state space with parallel traces.
fn commute(
	model: &AbstractVas,
	prism_states: &mut Vec<PrismStyleExplicitState>,
	state_trie: &mut vas_trie::VasTrieNode,
	prism_transitions: &mut Vec<PrismStyleExplicitTransition>,
	trace: &Vec<PrismStyleExplicitTransition>,
	depth: usize,
	max_depth: usize,
) {
	// Base case: if the depth is greater than the max depth, return
	if depth >= max_depth {
		return;
	}
	// Get universally enabled transitions
	// Clone the state vector to avoid holding an immutable borrow during mutation
	let initial_state_vector = prism_states[trace[0].from_state].state_vector.clone();
	let mut current_state = initial_state_vector.clone(); // Start from the initial state
													   // To do: maybe make this a hash set instead for faster lookups?
	let mut enabled_transitions: Vec<&VasTransition> = model
		.transitions
		.iter()
		.filter(|t| t.enabled_vector(&current_state))
		.collect();
	let mut universally_enabled_transitions: Vec<&VasTransition> = enabled_transitions.clone();
	for _transition in trace {
		current_state = initial_state_vector.clone(); // Start from the initial state
		enabled_transitions = model
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
		let state_vector = prism_states[state_id].state_vector.clone();
		for transition in &universally_enabled_transitions {
			// Compute the next state
			let next_state = (state_vector.clone() + transition.update_vector.clone()).clone();
			// Skip if next state has negative entries
			if next_state.iter().any(|&x| x < 0) {
				continue;
			}
			// Insert or get the state ID
			let mut next_state_id = prism_states.len();
			if let Some(existing_id) = state_trie.insert_if_not_exists(&next_state, next_state_id) {
				next_state_id = existing_id;
			} else {
				// Compute total outgoing rate for the new state
				let rate_sum = model
					.transitions
					.iter()
					.map(|trans| trans.get_sck_rate())
					.sum();
				prism_states.push(PrismStyleExplicitState::from_state(
					next_state.clone(),
					rate_sum,
					format!("State {}", next_state_id),
					Vec::new(),
				));
			}
			// Check if this transition already exists
			if !prism_states[state_id].next_states.contains(&next_state_id) {
				let new_transition = PrismStyleExplicitTransition {
					from_state: state_id,
					to_state: next_state_id,
					rate: transition.get_sck_rate(),
				};
				prism_states[state_id].next_states.push(next_state_id);
				prism_transitions.push(new_transition.clone());
				// Step 2. For each new state, create a new trace with the transition added
				let mut new_trace = trace[..i + 1].to_vec();
				new_trace.push(new_transition);
				commute(
					&model,
					prism_states,
					state_trie,
					prism_transitions,
					&new_trace,
					depth + 1,
					max_depth,
				);
			}
		}
	}
}

/// This function combinatorially finds cycles of transitions (i.e., update vectors add to 0)
/// and adds them to every where they are enabled.
fn add_cycles(
	model: &AbstractVas,
	prism_states: &mut Vec<PrismStyleExplicitState>,
	state_trie: &mut vas_trie::VasTrieNode,
	prism_transitions: &mut Vec<PrismStyleExplicitTransition>,
	max_cycle_length: usize,
) {
	// Collect all transition indices for easier cycle enumeration
	let transition_indices: Vec<usize> = (0..model.transitions.len()).collect();
	// For all cycle lengths from 2 up to max_cycle_length
	for cycle_len in 2..=max_cycle_length {
		// Generate all possible multisets (with repetition) of transitions
		for cycle in Itertools::combinations_with_replacement(transition_indices.iter(), cycle_len)
		{
			// For each multiset, check if the sum of update vectors is zero
			let mut sum_update = model.transitions[*cycle[0]].update_vector.clone();
			for &idx in &cycle[1..] {
				sum_update += model.transitions[*idx].update_vector.clone();
			}
			if sum_update.iter().all(|&x| x == 0) {
				// This is a cycle
				debug_message!("Found cycle: {:?}", cycle);
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
				// Right now, 1 is the index of the first real initial state. Eventually, maybe we make this safer by including
				// the absorbing state ID in the calculation
				for state_id in 1..prism_states.len() {
					let state_vector = prism_states[state_id].state_vector.clone();
					// Check if the cycle is enabled at this state (state_vector + min_vector >= 0)
					// For each permutation of the cycle, try to fire the transitions in order
					for perm in &cycle_permutations {
						// For each permutation, find the min possible value for each values
						let mut min_vector = model.transitions[*cycle[0]].update_vector.clone();
						let mut running_sum = min_vector.clone();
						for &idx in &cycle[1..] {
							running_sum += model.transitions[*idx].update_vector.clone();
							for i in 0..min_vector.len() {
								if running_sum[i] < min_vector[i] {
									min_vector[i] = running_sum[i];
								}
							}
						}
						let enabled = state_vector
							.iter()
							.zip(min_vector.iter())
							.all(|(&s, &m)| (s) + m >= 0);
						if !enabled {
							continue;
						}
						let mut current_state = state_vector.clone();
						let mut prev_state_id = state_id;
						// Try to apply each transition in the permutation
						for &&idx in perm {
							let transition = &model.transitions[idx];
							// Check if enabled: min_vector + update_vector must be non-negative
							if (current_state.clone() + transition.update_vector.clone())
								.iter()
								.any(|&x| x < 0)
							{
								break;
							}
							// Compute next state
							let next_state =
								current_state.clone() + transition.update_vector.clone();
							// Insert or get the state ID
							let mut next_state_id = prism_states.len();
							if let Some(existing_id) =
								state_trie.insert_if_not_exists(&next_state, next_state_id)
							{
								next_state_id = existing_id;
							} else {
								// Compute total outgoing rate for the new state
								let rate_sum = model
									.transitions
									.iter()
									.map(|trans| trans.get_sck_rate())
									.sum();
								prism_states.push(PrismStyleExplicitState::from_state(
									next_state.clone(),
									rate_sum,
									format!("State {}", next_state_id),
									Vec::new(),
								));
							}
							// Add transition if not already present
							if !prism_states[prev_state_id]
								.next_states
								.contains(&next_state_id)
							{
								let new_transition = PrismStyleExplicitTransition {
									from_state: prev_state_id,
									to_state: next_state_id,
									rate: transition.get_sck_rate(),
								};
								prism_states[prev_state_id].next_states.push(next_state_id);
								prism_transitions.push(new_transition);
							}
							current_state = next_state;
							prev_state_id = next_state_id;
						}
					}
				}
			}
		}
	}
}

/// This function takes the explicit state space and generates a visualization using Graphviz.
fn visualize_prism_state_space(
	prism_states: &[PrismStyleExplicitState],
	prism_transitions: &[PrismStyleExplicitTransition],
	output_file: &str,
) {
	let mut dot_file = match File::create(format!("{}.dot", output_file)) {
		Ok(f) => f,
		Err(e) => {
			error!("Error creating .dot file: {}", e);
			return;
		}
	};
	writeln!(dot_file, "digraph StateSpace {{").unwrap();
	// Write nodes
	for (i, state) in prism_states.iter().enumerate() {
		let label = format!(
			"{}\\n({})",
			state.label,
			state
				.state_vector
				.iter()
				.map(|x| x.to_string())
				.collect::<Vec<_>>()
				.join(",")
		);
		writeln!(dot_file, "    {} [label=\"{}\"];", i, label).unwrap();
	}
	// Write edges
	for t in prism_transitions {
		writeln!(
			dot_file,
			"    {} -> {} [label=\"{:.2}\"];",
			t.from_state, t.to_state, t.rate
		)
		.unwrap();
	}
	writeln!(dot_file, "}}").unwrap();
	message!("Graphviz .dot file written to: {}.dot", output_file);
	message!("You can visualize it with: dot -Tpng -O <file>.dot");
}
