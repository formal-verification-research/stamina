use std::{fs::File, io::{BufRead, BufReader}, thread::current};

use nalgebra::DVector;

use crate::model::{vas_model::{AbstractVas, VasTransition}, vas_trie};
use std::io::Write;

struct CCState {
    state_vector: DVector<i64>,
    total_rate: f64,
    label: String,
    next_states: Vec<usize>,
}
struct CCTransition {
    from_state: usize,
    to_state: usize,
    rate: f64,
}

fn get_outgoing_rate(t: &VasTransition) -> f64 {
    t.rate_const * t.enabled_bounds.iter()
        .filter(|&&r| r != 0)
        .map(|&r| -(r as f64))
        .product::<f64>()
}

pub fn cycle_commute(model: AbstractVas, trace_file: &str, output_file: &str) {
    // Read the trace list
    let trace_file = match File::open(trace_file) {
        Ok(f) => f,
        Err(e) => {
            println!("Error opening trace file: {}", e);
            return;
        }
    };
    
    let mut current_state = model.initial_states[0].vector.clone();
    let mut current_state_id = 1;
    
    let mut prism_states: Vec<CCState> = Vec::new();
    let mut prism_transitions: Vec<CCTransition> = Vec::new();
    
    // State trie for super quick lookups
    let mut state_trie = vas_trie::VasTrie::new();
    state_trie.insert(&current_state, 0);

    // Create the absorbing state
    let absorbing_state = DVector::from_element(current_state.len(), 0);
    let absorbing_state_id = 0;

    // Add the absorbing state to the prism states
    prism_states.insert(absorbing_state_id, CCState {
        state_vector: absorbing_state,
        total_rate: 0.0,
        label: "SINK".to_string(),
        next_states: Vec::new(),
    });
    
    let trace_reader = BufReader::new(trace_file);
    for trace in trace_reader.lines() {
        
        let trace = match trace {
            Ok(t) => t,
            Err(e) => {
                println!("Error reading trace line: {}", e);
                continue;
            }
        };

        current_state = model.initial_states[0].vector.clone(); // Reset current state for each trace
        current_state_id = 1; // Reset current state ID for each trace

        // Build the state space from the seed trace
        let transitions: Vec<&str> = trace.split_whitespace().collect();
        for transition_name in transitions {
            // Apply the transition to the current state
            let transition = model.get_transition_from_name(transition_name);
            if let Some(t) = transition {
                // Update the current state based on the transition
                let next_state = (current_state.clone().cast::<i128>() + t.update_vector.clone()).clone().cast::<i64>();
                let mut next_state_id = current_state_id + 1;
                if next_state.iter().any(|&x| x < 0) {
                    println!("ERROR: Next state contains non-positive values: {:?}", next_state);
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
                            get_outgoing_rate(trans)
                        })
                        .sum();
                    prism_states.push(CCState {
                        state_vector: next_state.clone(),
                        total_rate: rate_sum,
                        label: format!("State {}", current_state_id),
                        next_states: Vec::new(),
                    });
                }
                // Check if the transition is already in the current state's outgoing transitions
                if prism_states.get(current_state_id)
                    .map_or(true, |s| !s.next_states.iter().any(|tr| *tr == next_state_id))
                {
                    // Add the transition to the current state's outgoing transitions
                    let this_transition = CCTransition {
                        from_state: current_state_id,
                        to_state: next_state_id,
                        rate: get_outgoing_rate(t)
                    };
                    prism_states[current_state_id].next_states.push(next_state_id);
                    prism_transitions.push(this_transition);
                }

                current_state = next_state.clone();
                current_state_id = next_state_id;
            } else {
                println!("ERROR: Transition {} not found in model", transition_name);
                return;
            }

        }
    }

    // Add transitions to the absorbing state
    for i in 1..prism_states.len() {
        let transition_to_absorbing = CCTransition {
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


    // Write .sta file
    let mut sta_file = match File::create(format!("{}.sta", output_file)) {
        Ok(f) => f,
        Err(e) => {
            println!("Error creating .sta file: {}", e);
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
            println!("Error creating .tra file: {}", e);
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

    // rate_finder = lambda state : rate_const * np.prod([state[i] ** rate_mul_vector[i] for i in range(len(rate_mul_vector))])
    // Output results to the specified output file
    // This is a placeholder; actual implementation would depend on what results you want to output
    println!("Resulting explicit state space written to: {}.sta, .tra", output_file);
    println!("Check this with the following command:\n
        prism -importtrans {}.tra -importstates {}.sta -ctmc", output_file, output_file);
    
}