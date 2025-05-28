/// This module implements the cycle commute algorithm for VAS models.
/// It generates a PRISM-compatible state space from a given trace file.
/// It then uses the trace to build a highly-concurrent and cyclical state space of the VAS model
use std::{fs::File, io::{BufRead, BufReader}};

use nalgebra::DVector;

use crate::{logging, model::{vas_model::{AbstractVas, VasTransition}, vas_trie}};
use std::io::Write;

/// PrismStyleExplicitState represents a state in the PRISM-style explicit state space as described at
/// https://www.prismmodelchecker.org/manual/RunningPRISM/ExplicitModelImport
struct PrismStyleExplicitState {
    /// The VAS state vector
    state_vector: DVector<i64>,
    /// The total outgoing rate of the state, used to calculate the absorbing rate and mean residence time
    total_rate: f64,
    /// Label for the state, currently unused
    label: String,
    /// Vector of next states, here only for convenience in lookup while building the state space.
    next_states: Vec<usize>,
}

impl PrismStyleExplicitState {
    /// Creates a new PrismStyleExplicitState from the given parameters.
    fn from_state(state_vector: DVector<i64>, total_rate: f64, label: String, next_states: Vec<usize>) -> Self {
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
struct PrismStyleExplicitTransition {
    /// The ID (in Prism) of the state from which the transition originates
    from_state: usize,
    /// The ID (in Prism) of the state to which the transition goes
    to_state: usize,
    /// The CTMC rate (for Prism) of the transition
    rate: f64,
}

/// This function calculates the outgoing rate of a transition.
/// It currently assumes the SCK assumption that the rate
/// depends on the product of the enabled bounds.
impl VasTransition {
    /// Calculates the SCK rate of the transition.
    /// This function is temporary and intended only for quick C&C result generation --- 
    /// it will eventually be replaced by a system-wide more-powerful rate calculation
    /// that allows for more complex rate calculations.
    fn get_sck_rate(&self) -> f64 {
        self.rate_const * self.enabled_bounds.iter()
            .filter(|&&r| r != 0)
            .map(|&r| (r as f64))
            .product::<f64>()
    }
}

/// This function prints the PRISM-style explicit state space to .sta and .tra files.
/// The .sta file contains the state vectors and their IDs,
/// while the .tra file contains the transitions between states with their rates.
fn print_prism_files(model: AbstractVas, prism_states: &[PrismStyleExplicitState], prism_transitions: &[PrismStyleExplicitTransition], output_file: &str) {
    // Write .sta file
    let mut sta_file = match File::create(format!("{}.sta", output_file)) {
        Ok(f) => f,
        Err(e) => {
            logging::messages::error(&format!("Error creating .sta file: {}", e));
            return;
        }
    };
    // header
    let var_names = model.variable_names.join(" ");
    writeln!(sta_file, "({})", var_names).unwrap();
    // states
    for i in 0..prism_states.len() {
        let state_str = prism_states[i].state_vector.iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join(",");
        writeln!(sta_file, "{}: ({})", i, state_str).unwrap();
    }
    // Write .tra file
    let mut tra_file = match File::create(format!("{}.tra", output_file)) {
        Ok(f) => f,
        Err(e) => {
            logging::messages::error(&format!("Error creating .tra file: {}", e));
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
    logging::messages::message(&format!("Resulting explicit state space written to: {}.sta, .tra", output_file));
    logging::messages::message(&format!("Check this with the following command:\n
        prism -importtrans {}.tra -importstates {}.sta -ctmc", output_file, output_file));
}

/// This is the main function that implements the cycle & commute algorithm.
/// It reads a trace file, builds the state space from the trace,
/// builds the user-specified set of concurrent and cyclical transitions,
/// and generates the PRISM-style explicit state space files (.sta and .tra).
pub fn cycle_commute(model: AbstractVas, trace_file: &str, output_file: &str) {
    // Read the trace list
    let trace_file = match File::open(trace_file) {
        Ok(f) => f,
        Err(e) => {
            logging::messages::error(&format!("Error opening trace file: {}", e));
            return;
        }
    };
    // Inititalize the bookkeeping things
    let mut current_state = model.initial_states[0].vector.clone();
    let mut current_state_id = 1;
    let mut prism_states: Vec<PrismStyleExplicitState> = Vec::new();
    let mut prism_transitions: Vec<PrismStyleExplicitTransition> = Vec::new();
    // State trie for super quick lookups
    let mut state_trie = vas_trie::VasTrie::new();
    state_trie.insert(&current_state, 0);
    // Create the absorbing state
    let absorbing_state = DVector::from_element(current_state.len(), 0);
    let absorbing_state_id = 0;
    // Add the absorbing state to the prism states
    prism_states.insert(absorbing_state_id, PrismStyleExplicitState {
        state_vector: absorbing_state,
        total_rate: 0.0,
        label: "SINK".to_string(),
        next_states: Vec::new(),
    });
    // Read the trace file line by line (traces are line-separated)
    let trace_reader = BufReader::new(trace_file);
    for trace in trace_reader.lines() {
        let trace = match trace {
            Ok(t) => t,
            Err(e) => {
                logging::messages::error(&format!("Error reading trace line: {}", e));
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
                let next_state = (current_state.clone().cast::<i128>() + t.update_vector.clone()).clone().cast::<i64>();
                let mut next_state_id = current_state_id + 1;
                if next_state.iter().any(|&x| x < 0) {
                    logging::messages::error(&format!("ERROR: Next state contains non-positive values: {:?}", next_state));
                    return;
                }
                // Add the new state to the trie if it doesn't already exist
                let potential_id = state_trie.id_else_insert(&next_state, next_state_id);
                if potential_id.is_some() {
                    next_state_id = potential_id.unwrap();
                } else {
                    // TODO: This only works for CRNs right now. Need to generalize for VAS with custom formulas.
                    let mut rate_sum = 0.0;
                    rate_sum = model.transitions.iter()
                        .map(|trans| {
                            trans.get_sck_rate()
                        })
                        .sum();
                    prism_states.push(PrismStyleExplicitState::from_state(
                        next_state.clone(),
                        rate_sum,
                        format!("State {}", current_state_id),
                        Vec::new(),
                    ));
                }
                // Check if the transition is already in the current state's outgoing transitions
                if prism_states.get(current_state_id)
                    .map_or(true, |s| !s.next_states.iter().any(|tr| *tr == next_state_id))
                {
                    // Add the transition to the current state's outgoing transitions
                    let this_transition = PrismStyleExplicitTransition {
                        from_state: current_state_id,
                        to_state: next_state_id,
                        rate: t.get_sck_rate()
                    };
                    prism_states[current_state_id].next_states.push(next_state_id);
                    prism_transitions.push(this_transition);
                }
                // Move along the state space
                current_state = next_state.clone();
                current_state_id = next_state_id;
            } else {
                logging::messages::error(&format!("ERROR: Transition {} not found in model", transition_name));
                return;
            }
        }
    }
    // Add transitions to the absorbing state
    for i in 1..prism_states.len() {
        let transition_to_absorbing = PrismStyleExplicitTransition {
            from_state: i,
            to_state: absorbing_state_id,
            rate: prism_states[i].total_rate - prism_transitions
                .iter()
                .filter(|tr| tr.to_state != absorbing_state_id && prism_states[i].next_states.contains(&tr.to_state))
                .map(|tr| tr.rate)
                .sum::<f64>()
        };
        prism_transitions.push(transition_to_absorbing);
    }
    print_prism_files(model, &prism_states, &prism_transitions, output_file);
}