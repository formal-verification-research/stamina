use std::collections::HashMap;

use nalgebra::DVector;
use rand::{seq::SliceRandom, Rng};

use crate::{
    builder::ragtimer::ragtimer::{MagicNumbers, RagtimerBuilder, RagtimerMethod::ReinforcementLearning, RewardValue}, dependency::graph::DependencyGraph, logging::messages::{error, warning}, model::{model::ExplicitModel, vas_model::{PrismVasModel, VasProbOrRate, VasStateVector, VasTransition, VasValue}}, trace::{self, trace_trie::{self, TraceTrieNode}}
};

const MAX_TRACE_LENGTH: usize = 10000;

/// This is the builder for the Ragtimer tool, specifically for the RL Traces method.
/// It implements the `Builder` trait and provides methods to build the explicit state space
/// using reinforcement learning traces.
impl<'a> RagtimerBuilder<'a> {

    /// Function to set default magic numbers for the RL traces method.
    pub fn default_magic_numbers(&mut self) -> MagicNumbers {
        MagicNumbers {
            dependency_reward: 1.0,
            base_reward: 0.1,
            base_trace_reward: 0.01,
            smallest_history_window: 50
        }
    }

    /// Initializes the rewards for each transition in the model based on the dependency graph.
    /// For now, it initializes all rewards to zero, then adds DEPENDENCY_REWARD to the reward of any transition
    /// that appears in the dependency graph.
    fn initialize_rewards(&mut self, dependency_graph: &DependencyGraph) -> HashMap<usize, RewardValue> {
        let mut rewards = HashMap::new();
        let magic_numbers = match &self.method {
            ReinforcementLearning(magic_numbers) => magic_numbers,
            _ => panic!("RagtimerBuilder::add_rl_traces called with non-RL method"),
        };
        let model = self.abstract_model;
        let all_transitions = model.transitions.clone();
        let dep_transitions = dependency_graph.get_transitions();
        for transition in all_transitions {
            rewards.insert(transition.transition_id, magic_numbers.base_reward);
        }
        for transition in dep_transitions {
            if let Some(reward) = rewards.get_mut(&transition.transition_id) {
                *reward += magic_numbers.dependency_reward;
            }
        }
        rewards
    }

    /// Maintains rewards at a reasonable level with the following rules:
    /// 1. If a reaction is in the dependency graph, it should have a reward of at least DEPENDENCY_REWARD.
    // TODO: Adjust this more as time goes on (run many tests to see what works best)
    fn maintain_rewards(
        &mut self,
        rewards: &mut HashMap<usize, RewardValue>,
        dependency_graph: &DependencyGraph,
        magic_numbers: &MagicNumbers,
    ) {
        for transition in dependency_graph.get_transitions() {
            if let Some(reward) = rewards.get_mut(&transition.transition_id) {
                if *reward < magic_numbers.dependency_reward {
                    *reward = magic_numbers.dependency_reward;
                }
            }
        }
    }

    /// Returns a list of transition IDs that are enabled in the current state.
    fn get_available_transitions(
        &self, 
        current_state: &VasStateVector
    ) -> Vec<usize> {
        let x = self.abstract_model
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
        // debug_message(&format!("Available transitions: {:?}", x));
        x
    }

    /// Calculates the transition probability for a given transition in the context
    /// of the current state under the SCK assumption for CRN models.
    fn crn_transition_probability(
       &self,
        current_state: &VasStateVector,
        transition: &VasTransition,
    ) -> VasProbOrRate {
        let mut total_outgoing_rate = 0.0;
        let available_transitions = self.get_available_transitions(current_state);
        for t in available_transitions {
            if let Some(vas_transition) = self.abstract_model.get_transition_from_id(t) {
                let mut this_transition_rate = 0.0;
                for (_, &current_value) in current_state.iter().enumerate() {
                    this_transition_rate += vas_transition.rate_const * (current_value as VasProbOrRate);
                }

                total_outgoing_rate += this_transition_rate;
            } else {
                error(&format!("Transition ID {} not found in model.", t));
                return 0.0; // If the transition is not found, return 0 probability
            }
        }
        let mut transition_rate = 0.0;
        for (_, &current_value) in current_state.iter().enumerate() {
            transition_rate += transition.rate_const * (current_value as VasProbOrRate);
        }

        transition_rate / total_outgoing_rate
    }


    /// Generates a single trace based on the rewards and magic numbers.
    /// This function will be called multiple times to generate traces for the RL traces method.
    fn generate_single_trace(
        &mut self,
        rewards: &HashMap<usize, RewardValue>,
        magic_numbers: &MagicNumbers,
        depth: usize,
    ) -> (Vec<usize>, VasProbOrRate) {
        let mut trace = Vec::new();
        let mut trace_probability = 1.0;
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
                error(&format!(
                    "Current state length {} is less than target variable index {}",
                    current_state.len(),
                    vas_target.variable_index
                ));
            }
            // Get available transitions
            let available_transitions = self.get_available_transitions(&current_state);
            if available_transitions.is_empty() {
                // No available transitions, warn the user and break out of the loop
                warning(&format!("No available transitions found in state {:?}. Ending trace generation.", current_state));
                break;
            }
            // Shuffle the available transitions to add randomness
            let mut shuffled_transitions = available_transitions.clone();
            shuffled_transitions.shuffle(&mut rand::rng());
            // Find the total reward for the available transitions
            let total_reward: RewardValue = shuffled_transitions.iter()
                .filter_map(|&t_id| rewards.get(&t_id))
                .sum();
            // Pick a transition based on the rewards and magic numbers
            for (_, &transition) in shuffled_transitions.iter().enumerate() {
                // debug_message(&format!("Considering transition {} ({}/{})", transition, index + 1, shuffled_transitions.len()));
                let transition_reward = rewards.get(&transition).unwrap_or(&0.0);
                // debug_message(&format!("Considering transition {} with reward {}", transition, transition_reward));
                let selection_probability: RewardValue = if total_reward > 0.0 {
                    transition_reward / total_reward
                } else {
                    *transition_reward
                };
                if rand::rng().random::<RewardValue>() < selection_probability {
                    if let Some(vas_transition) = self.abstract_model.get_transition_from_id(transition) {
                        current_state =
                            current_state + vas_transition.update_vector.clone();
                        trace.push(transition);
                        // debug_message(&format!("Transition {} selected with reward {:.3e}. Current state updated to: {:?}", transition, transition_reward, current_state));
                        trace_probability *=
                            self.crn_transition_probability(&current_state, &vas_transition);
                    } else {
                        error(&format!("Transition ID {} not found in model.", transition));
                    }
                    break;
                }
            }
        }

        (trace, trace_probability)
    }

    /// High-level function that builds the explicit state space with RL traces.
    pub fn add_rl_traces(
        &mut self, 
        explicit_model: &mut PrismVasModel,
        dependency_graph: Option<&DependencyGraph>,
    ) {
        let magic_numbers = match &self.method {
            ReinforcementLearning(magic_numbers) => magic_numbers,
            _ => panic!("RagtimerBuilder::add_rl_traces called with non-RL method"),
        };
        let trace_trie = TraceTrieNode::new();
    }

}