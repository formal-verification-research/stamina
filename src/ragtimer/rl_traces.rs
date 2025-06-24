use nalgebra::DVector;
use rand::{seq::SliceRandom, Rng};
use std::collections::HashMap;

use crate::{
	dependency::graph::DependencyGraph, logging::messages::*, model::vas_model::VasTransition,
	ragtimer::trace_trie::TraceTrie, AbstractVas,
};

const DEPENDENCY_REWARD: f64 = 10.0; // Reward for transitions in the dependency graph
const BASE_REWARD: f64 = 1.0; // Reward for transitions in the dependency graph
const BASE_TRACE_REWARD: f64 = 0.1; // Base reward for an appearance of a transition in a successful trace, scaled by probability
									// TODO: Find a better way to constrain the trace length
const MAX_TRACE_LENGTH: usize = 300; // Maximum length of a trace
const MAX_DEPTH: usize = 4; // Maximum recursion depth for generating unique traces
const SMALLEST_HISTORY_WINDOW: usize = 100; // Minimum size of the history window for average probability calculation

/// Prints the rewards for each transition in the model to the debug log.
fn debug_print_rewards(model: &AbstractVas, rewards: &HashMap<usize, f64>) {
	for (transition_id, reward) in rewards {
		debug_message!(
			"{}.{}: {}",
			model
				.get_transition_from_id(*transition_id)
				.unwrap()
				.transition_name,
			transition_id,
			reward
		);
	}
}

/// Initializes the rewards for each transition in the model based on the dependency graph.
/// For now, it initializes all rewards to zero, then adds DEPENDENCY_REWARD to the reward of any transition
/// that appears in the dependency graph.
fn initialize_rewards(
	model: &AbstractVas,
	dependency_graph: &DependencyGraph,
) -> HashMap<usize, f64> {
	let mut rewards = HashMap::new();
	let all_transitions = model.transitions.clone();
	let dep_transitions = dependency_graph.get_transitions();
	for transition in all_transitions {
		rewards.insert(transition.transition_id, BASE_REWARD);
	}
	for transition in dep_transitions {
		if let Some(reward) = rewards.get_mut(&transition.transition_id) {
			*reward += DEPENDENCY_REWARD;
		}
	}
	rewards
}

/// Initializes the rewards for each transition in the model based on the dependency graph.
/// For now, it initializes all rewards to zero, then adds DEPENDENCY_REWARD to the reward of any transition
/// that appears in the dependency graph.
fn maintain_dg_rewards(dependency_graph: &DependencyGraph, rewards: &mut HashMap<usize, f64>) {
	let dep_transitions = dependency_graph.get_transitions();
	for transition in dep_transitions {
		if let Some(reward) = rewards.get_mut(&transition.transition_id) {
			*reward += (DEPENDENCY_REWARD * BASE_TRACE_REWARD).max(DEPENDENCY_REWARD);
		}
	}
}

/// Top-level function to generate a specified number of unique traces from the model.
/// It initializes the rewards, generates traces, updates rewards based on the traces,
/// and maintains high rewards to transitions in the dependency graph.
pub fn generate_traces(
	model: &AbstractVas,
	dependency_graph: &DependencyGraph,
	num_traces: usize,
) -> Vec<Vec<VasTransition>> {
	let mut traces = Vec::with_capacity(num_traces);
	let mut trace_probability_history: Vec<f64> = Vec::new();
	let mut rewards = initialize_rewards(model, dependency_graph);

	debug_print_rewards(model, &rewards);
	let mut trace_trie = TraceTrie::new();

	for i in 0..num_traces {
		let (trace, _) = generate_unique_trace(
			model,
			&mut trace_trie,
			&mut rewards,
			&mut trace_probability_history,
			1,
		);
		update_rewards(&trace, &trace_probability_history, &mut rewards);
		maintain_dg_rewards(&dependency_graph, &mut rewards);
		traces.push(
			trace
				.iter()
				.filter_map(|&t| model.get_transition_from_id(t))
				.cloned()
				.collect(),
		);
		if i % (num_traces / 10).max(1).min(500) == 0 {
			let width = num_traces.to_string().len();
			debug_message!(
				"Generated trace {:>width$}/{} with cumulative probability {:.6e}",
				i,
				num_traces,
				trace_probability_history.iter().sum::<f64>(),
				width = width,
			);
			let mut sorted_rewards: Vec<_> = rewards.iter().collect();
			sorted_rewards.sort_by_key(|(id, _)| *id);
			for (transition_id, reward) in sorted_rewards {
				debug_message!("\tTransition {}: reward = {:.6e}", transition_id, reward);
			}
			// debug_message!("Running probability: {:.6e}", trace_probability_history.iter().sum::<f64>()));
		}
	}

	let total_probability: f64 = trace_probability_history.iter().sum();
	debug_message!("Total trace probability sum: {:.6e}", total_probability);

	traces
}

/// Generates a single unique trace from the model.
/// It recursively generates a trace of arbitrary length, ensuring that it is unique
/// and that it reaches a target state defined in the model.
fn generate_unique_trace(
	model: &AbstractVas,
	trace_trie: &mut TraceTrie,
	rewards: &HashMap<usize, f64>,
	trace_probability_history: &mut Vec<f64>,
	depth: usize,
) -> (Vec<usize>, f64) {
	let mut trace = Vec::new();
	let mut trace_probability = 1.0;
	let mut current_state = model.initial_states.clone().get(0).unwrap().vector.clone();
	let mut trace_length = 0;

	// Safety check to prevent infinite recursion
	if depth > MAX_DEPTH {
		warning!("Maximum recursion depth exceeded while generating unique trace. Aborting.");
		return (trace, trace_probability);
	}

	// Starting at the initial state, generate a trace of arbitrary length
	loop {
		// Safety check to prevent infinite loops
		if trace_length > MAX_TRACE_LENGTH {
			warning!(
				"Trace length exceeded maximum of {}. Stopping generation.",
				MAX_TRACE_LENGTH
			);
			break; // Prevent infinite loops
		}
		trace_length += 1;

		// Check if we have reached a target state
		// TODO: Accept more kinds of target states, not just equality to a specific value
		let target = model.target.clone();
		// debug_message!("Checking target state: {:?}", target));
		if let Some(&val) = current_state.get(target.variable_index) {
			if val == target.target_value.try_into().unwrap() {
				// debug_message!("Reached target state: {:?}. Ending trace generation.", current_state));
				// debug_message!("Reached target state\t[\t{}\t]", current_state.iter().map(|x| x.to_string()).collect::<Vec<String>>().join("\t")));
				break;
			}
		} else {
			error!("Current state {:?} does not have a value for target variable index {}. Cannot check for target state.", current_state, target.variable_index);
		}

		// Get all the available transitions from the current state
		let mut rng = rand::rng();
		let available_transitions = get_available_transitions(&model, &current_state);
		if available_transitions.is_empty() {
			warning!("No available transitions from state: {:?}. Cannot continue trace generation from this state.", current_state);
			break; // No more transitions available, end the trace
		}
		let mut shuffled_transitions: Vec<usize> = available_transitions.clone();
		shuffled_transitions.shuffle(&mut rng);
		let total_reward: f64 = available_transitions
			.iter()
			.map(|t| rewards.get(t).unwrap_or(&0.0))
			.sum();

		// debug_message!("Shuffled transitions: {:?}", shuffled_transitions));

		// debug_message!("Current state: {:?}, Available transitions: {:?}, Total reward: {}", current_state, shuffled_transitions, total_reward));

		// Pick the next transition based on the rewards
		for (_, &transition) in shuffled_transitions.iter().enumerate() {
			// debug_message!("Considering transition {} ({}/{})", transition, index + 1, shuffled_transitions.len()));
			let transition_reward = rewards.get(&transition).unwrap_or(&0.0);
			// debug_message!("Considering transition {} with reward {}", transition, transition_reward));
			let selection_probability = if total_reward > 0.0 {
				transition_reward / total_reward
			} else {
				*transition_reward
			};
			if rng.random::<f64>() < selection_probability {
				if let Some(vas_transition) = model.get_transition_from_id(transition) {
					current_state =
						current_state + vas_transition.update_vector.clone().map(|x| x as i64);
					trace.push(transition);
					// debug_message!("Transition {} selected with reward {:.3e}. Current state updated to: {:?}", transition, transition_reward, current_state));
					trace_probability *=
						crn_transition_probability(model, vas_transition, &current_state);
				} else {
					error!("Transition ID {} not found in model.", transition);
				}
				break;
			}
		}
	}
	if trace_trie.contains_or_insert(&trace) {
		// If the trace is unique, we can continue
		debug_message!("Found duplicate trace. Trying again.",);
		return generate_unique_trace(
			model,
			trace_trie,
			rewards,
			trace_probability_history,
			depth + 1,
		);
	}

	if trace_probability.is_finite() && trace_probability < 1.0 {
		trace_probability_history.push(trace_probability);
		// debug_message!("Probability: {:.3e}", trace_probability));
	}

	(trace, trace_probability)
}

/// Updates the reward function. This is the function that can use the most fine-tuning.
fn update_rewards(
	trace: &[usize],
	trace_probability_history: &Vec<f64>,
	rewards: &mut HashMap<usize, f64>,
) {
	let latest_probability = trace_probability_history.last().cloned().unwrap_or(0.0);
	if trace.len() == 0 || latest_probability <= 0.0 {
		// debug_message("Trace is empty or has zero probability. No rewards to update.");
		return;
	}
	// Use the last 10% of entries to compute the average probability
	let history_len = trace_probability_history.len();
	let window_size = if history_len < SMALLEST_HISTORY_WINDOW {
		history_len
	} else {
		((history_len as f64) * 0.2).ceil() as usize
	};
	let window_size = window_size.max(1); // Ensure at least 1
	let start_idx = history_len.saturating_sub(window_size);
	let recent_probs = &trace_probability_history[start_idx..];
	let avg_recent_prob = if !recent_probs.is_empty() {
		recent_probs.iter().copied().sum::<f64>() / recent_probs.len() as f64
	} else {
		0.0
	};

	// Only give reward if this trace's probability is higher than the recent average
	// Reward is proportional to the log-ratio of latest to average probability.
	// This gives positive reward for increased probability, negative for decreased.
	// Clamp the log-ratio to avoid extreme values.
	let log_ratio = if avg_recent_prob > 0.0 && latest_probability > 0.0 {
		(latest_probability / avg_recent_prob).ln()
	} else {
		0.0
	};
	// Scale to roughly [-20, 20] range
	let trace_reward = (log_ratio).clamp(-10.0, 10.0) / trace.len() as f64 * BASE_TRACE_REWARD;

	// Update the rewards for each transition in the trace
	for &transition_id in trace {
		if let Some(reward) = rewards.get_mut(&transition_id) {
			*reward += trace_reward;
			// debug_message!("Updated reward for transition {}: {:.3e}", transition_id, *reward));
		} else {
			error!("Transition ID {} not found in rewards map.", transition_id);
		}
	}

	// for (transition_id, reward) in rewards.iter().sorted_by(|a, b| a.0.cmp(b.0)) {
	//     let old_reward = *reward - trace_reward;
	//     if old_reward == *reward {
	//         continue;
	//     }
	//     debug_message!(
	//         "Transition {}: old reward = {:.3e}, new reward = {:.3e}",
	//         transition_id, old_reward, reward
	//     ));
	// }

	// debug_message!("Change reward by\t{:.3e}", trace_reward));
	// debug_message!("Latest probability: {:.3e}", latest_probability));
	// debug_message!("Recent probability: {:.3e}", avg_recent_prob));
	// debug_message("Not yet implemented: update_rewards function");
}

/// Returns a list of transition IDs that are enabled in the current state.
fn get_available_transitions(model: &AbstractVas, current_state: &DVector<i64>) -> Vec<usize> {
	// Debug print: show enabled_bounds vs current_state for each transition
	// for t in &model.transitions {
	//     let enabled = t.enabled_bounds.iter().zip(current_state.iter())
	//         .map(|(bound, &val)| format!("({} >= {})", val, bound))
	//         .collect::<Vec<_>>()
	//         .join("\t");
	//     debug_message!(
	//         "Transition {}: enabled_bounds vs current_state: [{}]",
	//         t.transition_id, enabled
	//     ));
	// }
	let x = model
		.transitions
		.iter()
		.filter(|t| {
			t.enabled_bounds
				.iter()
				.zip(current_state.iter())
				.all(|(bound, &val)| val >= (*bound).try_into().unwrap())
		})
		.map(|t| t.transition_id)
		.collect();
	// debug_message!("Available transitions: {:?}", x));
	x
}

/// Prints the traces to a file, with each trace on a new line and transitions separated by tabs.
pub fn print_traces_to_file(traces: &Vec<Vec<VasTransition>>, file_path: &str) {
	use std::fs::File;
	use std::io::Write;

	let mut file = File::create(file_path).expect("Failed to create file");
	for trace in traces {
		let trace_str = trace
			.iter()
			.map(|t| t.transition_name.clone())
			.collect::<Vec<String>>()
			.join("\t");
		writeln!(file, "{}", trace_str).expect("Failed to write to file");
	}
}

/// Calculates the transition probability for a given transition in the context
/// of the current state under the SCK assumption for CRN models.
fn crn_transition_probability(
	model: &AbstractVas,
	transition: &VasTransition,
	current_state: &DVector<i64>,
) -> f64 {
	let mut total_outgoing_rate = 0.0;
	let available_transitions = get_available_transitions(model, current_state);
	for t in available_transitions {
		if let Some(vas_transition) = model.get_transition_from_id(t) {
			let mut this_transition_rate = 0.0;
			for (_, &current_value) in current_state.iter().enumerate() {
				this_transition_rate += vas_transition.rate_const * current_value as f64;
			}

			total_outgoing_rate += this_transition_rate;
		} else {
			error!("Transition ID {} not found in model.", t);
			return 0.0; // If the transition is not found, return 0 probability
		}
	}
	let mut transition_rate = 0.0;
	for (_, &current_value) in current_state.iter().enumerate() {
		transition_rate += transition.rate_const * current_value as f64;
	}

	transition_rate / total_outgoing_rate
}
