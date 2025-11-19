use rand::seq::IndexedRandom;
use std::io::{stdout, Write};

use crate::{
	builder::ragtimer::ragtimer::{
		RagtimerApproach::RandomDependencyGraph, RagtimerBuilder, MAX_TRACE_LENGTH,
	},
	dependency::graph::{make_dependency_graph, DependencyGraph},
	logging::messages::{debug_message, error, message},
	model::{
		vas_model::{PrismVasModel, PrismVasState, VasTransition},
		vas_trie::VasTrieNode,
	},
	trace::trace_trie::TraceTrieNode,
};

/// This is the builder for the Ragtimer tool, specifically for the RL Traces method.
/// It implements the `Builder` trait and provides methods to build the explicit state space
/// using reinforcement learning traces.
impl<'a> RagtimerBuilder<'a> {
	/// Recursively builds dependency graph traces and adds them to the explicit model.
	/// Effectively a depth-first search through the dependency graph.
	fn generate_dep_trace(&self, allowed_transitions: &Vec<VasTransition>) -> Vec<usize> {
		let mut trace = Vec::new();
		let vas_target = &self.abstract_model.target;

		// Starting in the initial state, generate a trace
		let mut current_state = self.abstract_model.initial_states[0].vector.clone();
		while trace.len() < MAX_TRACE_LENGTH {
			// Check if we have reached the target state
			if current_state.len() > vas_target.variable_index {
				if current_state[vas_target.variable_index] == vas_target.target_value {
					break;
				}
			} else {
				error!(
					"Current state length {} is less than target variable index {}",
					current_state.len(),
					vas_target.variable_index
				);
			}
			// Get available transitions
			let available_transitions = self
				.abstract_model
				.get_available_transition_subset(&current_state, allowed_transitions);
			if available_transitions.is_empty() {
				break;
			}
			// Pick a random transition
			let selected_transition = available_transitions.choose(&mut rand::rng());
			if let Some(vas_transition) =
				selected_transition.and_then(|&id| self.abstract_model.get_transition_from_id(id))
			{
				current_state = current_state + vas_transition.update_vector.clone();
				trace.push(vas_transition.transition_id);
			}
		}

		trace
	}

	/// High-level function that builds the explicit state space with dependency traces.
	pub fn add_dep_traces(
		&mut self,
		explicit_model: &mut PrismVasModel,
		dependency_graph: Option<&DependencyGraph>,
	) {
		message!("Beginning Ragtimer Dependency Trace Generation");
		// Determine the number of traces to generate
		let num_traces = match &self.approach {
			RandomDependencyGraph(n) => *n,
			_ => panic!("RagtimerBuilder:add_dep_traces called with non-DeterministicDependencyGraph method or invalid number of traces."),
		};

		// Set up trace generation structures
		let mut trace_trie = TraceTrieNode::new();

		// Set up state space storage structures
		explicit_model.state_trie = VasTrieNode::new();
		let current_state_id = 1;
		let current_state = self.abstract_model.initial_states[0].vector.clone();
		explicit_model
			.state_trie
			.insert_if_not_exists(&current_state, current_state_id);
		explicit_model.add_state(PrismVasState {
			state_id: current_state_id,
			vector: current_state.clone(),
			label: Some("init".to_string()),
			total_outgoing_rate: self.abstract_model.crn_total_outgoing_rate(&current_state),
		});

		// If the dependency graph is not provided, we try to construct it from the abstract model.
		let owned_dep_graph;
		let dependency_graph_ref: &DependencyGraph = match dependency_graph {
			Some(dep_graph) => dep_graph,
			None => {
				let dep_graph_result = make_dependency_graph(&self.abstract_model);
				match dep_graph_result {
					Ok(Some(dep_graph)) => {
						owned_dep_graph = Some(dep_graph);
						owned_dep_graph.as_ref().unwrap()
					}
					Ok(None) => {
						error!("No dependency graph could be constructed.");
						return;
					}
					Err(e) => {
						error!("Error constructing dependency graph: {}", e);
						return;
					}
				}
			}
		};
		// Print the dependency graph
		debug_message!("Dependency Graph:");
		dependency_graph_ref.nice_print(self.abstract_model);
		let allowed_transitions = dependency_graph_ref.get_transitions();
		// Generate the traces one-by-one, repeating if the trace is not unique
		// print a line to give whitespace for the progress bar
		println!("\nTRACE GENERATION PROGRESS:");
		for i in 0..num_traces {
			let mut trace;
			let mut trace_attempts = 0;
			loop {
				// Generate a single trace
				trace = self.generate_dep_trace(&allowed_transitions);
				// If the trace already exists or is empty, we try to generate a new one.
				if !trace_trie.exists_or_insert(&trace) && !trace.is_empty() {
					break;
				}
				trace_attempts += 1;
				if trace_attempts > 20 {
					break;
				}
			}
			// Store explicit prism states and transitions for this trace
			self.store_explicit_trace(explicit_model, &trace);

			// Print the trace generation progress every 100 traces
			let percent_step = (num_traces as f64 / 100.0).ceil().max(1.0) as usize;
			if i % percent_step == 0 || i == num_traces - 1 {
				let bar_width = 40;
				let progress = (i + 1) as f64 / num_traces as f64;
				let filled = (progress * bar_width as f64).round() as usize;
				let bar = format!(
					"\r|{}{}| {}/{} traces ({:.1}%)",
					"█".repeat(filled),
					" ".repeat(bar_width - filled),
					i + 1,
					num_traces,
					progress * 100.0
				);
				print!("{}", bar);
				stdout().flush().unwrap();
			}
		}
		println!("\n");
		message!("All RDG traces generated.");
		explicit_model.trace_trie = trace_trie;

		message!(
			"Ragtimer RDG Traces complete. Explicit model now has {} states and {} transitions.",
			explicit_model.states.len(),
			explicit_model.transitions.len()
		);
	}
}
