use nalgebra::DVector;

use crate::{model::vas_model::{self, VasProperty, VasState, VasTransition}, AbstractVas};
use std::io::{self, Write};

fn debug_println(s: String) {
	// if cfg!(debug_assertions) {
	// 	println!("{}", s);
	// }
}

#[derive(Clone)]
struct GraphNode {
	transition: VasTransition,
	children: Vec<Box<GraphNode>>,
	parents: Vec<VasTransition>,
	executions: u64,
	enabled: bool,
	node_init: VasState,
	node_target: Vec<VasProperty>,
	decrement: bool,
}

#[derive(Clone)]
pub(crate) struct DependencyGraph {
	root: Box<GraphNode>,
}

impl GraphNode {

	fn rec_build_graph(&mut self, vas: &AbstractVas, depth: u32) -> Result<(), String> {
		if depth > 500 {
			return Err("DEPTH OVER 500".to_string());
		}
		let spaces = " ".repeat(depth as usize);

		debug_println(format!("{}Building graph at node {} x{}", spaces, self.transition.transition_name, self.executions));
		
		if self.enabled {
			debug_println(format!("{}Node Enabled? {}", spaces, self.enabled));
			return Ok(());
		}

        let child_init = VasState::new(
            (&self.node_init.vector.map(|x| x as i128) + (&self.transition.update_vector * self.executions as i128))
                .map(|x| x as i64),
        );

		let child_init_str = (0..child_init.vector.len())
        .map(|i| {
            let variable_name = vas.variable_names.get(i).map_or("unknown", |name| name.as_str());
            format!("{}.{} ", variable_name, child_init.vector[i])
        })
        .collect::<String>();

		debug_println(format!("{}child init {}", spaces, child_init_str));

		let child_targets: Vec<VasProperty> = self.node_target.iter().filter_map(|prop| {
			let reqd = if self.decrement {
                let initial_value = child_init.vector.get(prop.variable_index).unwrap();
                let consumed_here = 0 - self.transition.update_vector.get(prop.variable_index).unwrap();
				debug_println(format!("{}consumed_here {}.{}", spaces, vas.variable_names.get(prop.variable_index).unwrap(), consumed_here));
				debug_println(format!("{}initial_value {}", spaces, initial_value));
				prop.target_value + (consumed_here * self.executions as i128)
			} else {
				let initial_value = child_init.vector.get(prop.variable_index).unwrap();
                let consumed_here = 0 + self.transition.update_vector.get(prop.variable_index).unwrap();
				debug_println(format!("{}produced_here {}.{}", spaces, vas.variable_names.get(prop.variable_index).unwrap(), consumed_here));
				debug_println(format!("{}initial_value {}", spaces, initial_value));
				prop.target_value - (consumed_here * self.executions as i128)
			};
			if reqd != 0 {
				debug_println(format!("{}reqd {}", spaces, reqd));
				Some(VasProperty {
                    variable_index: prop.variable_index,
                    target_value: reqd,
                })
			}
			else {
				None
			}
		}).collect();

		let mut negative_targets: Vec<VasProperty> = Vec::new();
        for i in 0..child_init.vector.len() {
            if child_init.vector[i] < 0 {
                debug_println(format!("{}negative_target {}.{}", spaces, vas.variable_names.get(i).unwrap(), child_init.vector[i]));
                negative_targets.push(VasProperty {
                    variable_index: i,
					target_value: -child_init.vector[i] as i128,
                });
            }
        }
		let mut all_targets = child_targets;
		all_targets.extend(negative_targets);

		debug_println(format!("{}child targets {}", spaces, 
            all_targets.iter().map(|mm| format!("{}.{} ", vas.variable_names.get(mm.variable_index).unwrap(), mm.target_value)).collect::<String>()
        ));

		for target in &all_targets {
			debug_println(format!("{}Processing target {}.{}", spaces, vas.variable_names.get(target.variable_index).unwrap(), target.target_value));
			for trans in &vas.transitions {
				debug_println(format!("{}Checking transition {}", spaces, trans.transition_name));
				if self.parents.iter().all(|p| p.transition_name != trans.transition_name) {
					// debug_println(format!("{}Transition {} is not in parents", spaces, trans.transition_name));

					let mut this_child_targets: Vec<VasProperty> = Vec::new();
					let mut executions: i128 = 0;

                    if (target.target_value > 0 && trans.update_vector[target.variable_index] > 0)
                        || (target.target_value < 0 && trans.update_vector[target.variable_index] < 0)
                    {
                        debug_println(format!(
                            "{}Sign match for transition {} on target {}.{}",
                            spaces,
                            trans.transition_name,
                            vas.variable_names.get(target.variable_index).unwrap(),
                            target.target_value
                        ));
                        this_child_targets.push(VasProperty {
                            variable_index: target.variable_index,
                            target_value: target.target_value,
                        });
                        executions = (target.target_value / trans.update_vector[target.variable_index]).try_into().unwrap();
                        debug_println(format!("{}Executions calculated: {}", spaces, executions));
                    } else {
                        // debug_println(format!(
                        //     "{}Sign mismatch for transition {} on target {}.{}",
                        //     spaces,
                        //     trans.transition_name,
                        //     vas.variable_names.get(target.variable_index).unwrap(),
                        //     target.target_value
                        // ));
                        continue;
                    }

                    if executions > 0 {
                        let mut child = GraphNode {
                            transition: trans.clone(),
                            children: Vec::new(),
                            parents: self.parents.clone(),
                            executions: executions.abs().try_into().unwrap(),
                            enabled: this_child_targets.is_empty(),
                            node_init: child_init.clone(),
                            node_target: this_child_targets.clone(),
                            decrement: executions < 0,
                        };
                        child.parents.push(self.transition.clone());
                        self.children.push(Box::new(child));
                        debug_println(format!("{}Added child node for transition {}", spaces, trans.transition_name));
                    }
				}
			}
		}

		let mut merged_children: Vec<Box<GraphNode>> = Vec::new();

		for child in self.children.drain(..) {
			if let Some(existing_child) = merged_children.iter_mut().find(|c| c.transition.transition_name == child.transition.transition_name) {
				if child.executions > existing_child.executions {
					*existing_child = child;
				}
			} else {
				merged_children.push(child);
			}
		}

		self.children = merged_children;

		for child in &mut self.children {
			let _ = child.rec_build_graph(vas, depth + 1);
			if !child.enabled {
				self.enabled = false;
			}
		}


		Ok(())
	}

}

fn property_sat(prop: &VasProperty, state: &VasState) -> Result<bool,String>{
	if state.vector.len() < prop.variable_index {
		return Err(format!("Error: Index out of bounds for state vector: {} >= {}", prop.variable_index, state.vector.len()));
	}
	if state.vector[prop.variable_index] as i128 == prop.target_value {
		return Ok(true);
	}
    return Ok(false);
}

pub fn make_dependency_graph(vas: &vas_model::AbstractVas) -> Result<Option<DependencyGraph>, String> {

	debug_println(format!("Building a dependency graph."));

	// check if target is satisfied in the initial state; if not, build a root node.
	let initial_state = VasState::new(vas.initial_states[0].vector.clone());
	let initially_sat = property_sat(&vas.target, &initial_state);
	if initially_sat == Ok(true) {
		return Err(String::from("Error: Initial state satisfies the target property. Probability is 1 and this analysis is pointless."));
	}
	else if initially_sat.is_err() {
		return Err(String::from("Error: Cannot check initial state against target property."));
	}

	// figure out the executions on the artificial root node


    let target_variable = vas.target.variable_index;
    let initial_value = vas.initial_states[0].vector[target_variable];
    let target_value = vas.target.target_value;
	let target_difference = if (initial_value as i128) < target_value {
		target_value - (initial_value as i128)
	} else {
		(initial_value as i128) - target_value
	};
    let decrement = (initial_value as i128) > target_value;

	// debug_println(format!("");
    // TODO: Stoichiometry greater than one.
	debug_println(format!("Target Executions: {}", target_difference));
	
	// build a new root node
	let mut dependency_graph = DependencyGraph {
		root: {
		Box::new(GraphNode {
			transition: VasTransition {
				transition_id: 999999,
				transition_name: "ARTIFICIAL".to_string(),
				update_vector: DVector::zeros(vas.variable_names.len()),
				enabled_bounds: DVector::zeros(vas.variable_names.len()),
				rate_const: 0.0,
				custom_rate_fn: None, // make the artificial transition here
			},
			children: Vec::new(),
			parents: Vec::new(),
			executions: target_difference as u64,
			enabled: false,
			node_init: initial_state.clone(),
			node_target: vec![VasProperty {
                variable_index: target_variable,
                target_value: target_difference,
            }],
			decrement,
            })
        }
    };

	// handle the decrement case
	if dependency_graph.root.decrement {
		if let Some(first_target) = dependency_graph.root.node_target.first_mut() {
            first_target.target_value -= dependency_graph.root.node_init.vector[first_target.variable_index] as i128;
		}
	}

	debug_println(format!("decrement? {}", dependency_graph.root.decrement));

	let _ = dependency_graph.root.rec_build_graph(vas, 1);

	// if cfg!(debug_assertions) {
	// 	dependency_graph.pretty_print(vas);
	// }

	Ok(Some(dependency_graph))

}

impl DependencyGraph {
	pub fn pretty_print(&self, vas: &AbstractVas) {
		fn print_node(vas: &AbstractVas, node: &GraphNode, depth: usize) {
			let indent = " ".repeat(depth * 2);
			println!("{}Node: {}", indent, node.transition.transition_name);
			println!("{}  Executions: {}", indent, node.executions);
			println!("{}  Enabled: {}", indent, node.enabled);
			if node.decrement {
				println!("{}  Decrement", indent);
			}
			println!("{}  Init Variables:", indent);

			for var in 0..node.node_init.vector.len() {
                println!("{}    {}: {}", indent, node.node_init.vector.get(var).unwrap(), vas.variable_names.get(var).unwrap());
			}
			println!("{}  Target Variables:", indent);
            for target in node.node_target.iter() {
                // let var = node.node_target.get(target).unwrap();
                // println!("{}    {}: {}", indent, target.variable_name, target.count);
			// for var in &node.node_target {
                println!("{}    {}: {}", indent, vas.variable_names.get(target.variable_index).unwrap(), target.target_value);
				// println!("{}    {}: {}", indent, var.variable_name, var.count);
			}
			for child in &node.children {
				print_node(vas, child, depth + 1);
			}
		}

		print_node(vas, &self.root, 0);
	}
    pub fn simple_print(&self, vas: &AbstractVas) {
		println!("===================");
		println!("Dependency Graph");
        fn print_node(vas: &AbstractVas, node: &GraphNode, depth: usize) {
			let indent = " ".repeat(depth * 2);
            println!("{}Node: {} (Executions: {})", indent, node.transition.transition_name, node.executions);
            for child in &node.children {
				print_node(vas, child, depth + 1);
            }
        }
        print_node(vas, &self.root, 0);
		println!("===================\n");
		
    }
    pub fn vec_transitions(&self) -> Vec<VasTransition> {
        fn collect_transitions(node: &GraphNode, transitions: &mut Vec<VasTransition>) {
            transitions.push(node.transition.clone());
            for child in &node.children {
                collect_transitions(child, transitions);
            }
        }
        let mut transitions = Vec::new();
        collect_transitions(&self.root, &mut transitions);
        transitions
    }

	pub fn get_transitions(&self) -> Vec<VasTransition> {
		let mut transitions = Vec::new();
		fn traverse(node: &GraphNode, transitions: &mut Vec<VasTransition>) {
			if node.transition.transition_name != "ARTIFICIAL" {
				transitions.push(node.transition.clone());
			}
			for child in &node.children {
			traverse(child, transitions);
			}
		}
		traverse(&self.root, &mut transitions);
		transitions
	}
}