use nalgebra::DVector;

use crate::{
	logging::messages::*,
	model::vas_model::{self, AbstractVas, VasProperty, VasState, VasTransition, VasValue},
};

/// Temporary constants for debugging.
const DEBUG_DEPTH_LIMIT: usize = 5000;

/// A node in the dependency graph.
#[derive(Clone)]
struct GraphNode {
	transition: VasTransition,
	children: Vec<Box<GraphNode>>,
	parents: Vec<VasTransition>,
	executions: VasValue,
	enabled: bool,
	node_init: VasState,
	node_target: Vec<VasProperty>,
	decrement: bool,
}

/// A dependency graph containing only a root node.
#[derive(Clone)]
pub(crate) struct DependencyGraph {
	root: Box<GraphNode>,
}

/// This trait provides methods for building and manipulating the dependency graph.
impl GraphNode {
	/// Creates a new dependency graph (node) with the given transition and initial state.
	fn rec_build_graph(&mut self, vas: &AbstractVas, depth: usize) -> Result<(), String> {
		// Handle administrative tasks before building the node.
		if depth > DEBUG_DEPTH_LIMIT {
			error!("Error: Depth limit exceeded: {}", DEBUG_DEPTH_LIMIT);
			return Err(format!(
				"Error: Depth limit exceeded: {}",
				DEBUG_DEPTH_LIMIT
			));
		}
		let indentation = " ".repeat(depth as usize);
		debug_message!(
			"{}Building graph at node {} x{}",
			indentation,
			self.transition.transition_name,
			self.executions
		);
		// Check if the node is already enabled.
		if self.enabled {
			debug_message!("{}Node Enabled? {}", indentation, self.enabled);
			return Ok(());
		}
		// Create a new "initial state" for the child nodes.
		// This is the state after the child's parents have been applied to the model's initial state.
		let mut child_init = VasState::new(
			&self.node_init.vector + (&self.transition.update_vector * self.executions),
		);
		// Compute the adjustment vector: if update_vector[i] + enabled_bounds[i] != 0, subtract enabled_bounds[i] from child_init.vector[i]
		let adjustment = self
			.transition
			.update_vector
			.iter()
			.zip(self.transition.enabled_bounds.iter())
			.map(
				|(update, bound)| {
					if *update + *bound != 0 {
						-(*bound)
					} else {
						0
					}
				},
			)
			.collect::<Vec<_>>();
		child_init.vector += nalgebra::DVector::from_vec(adjustment);
		debug_message!(
			"{}child init {}",
			indentation,
			(0..child_init.vector.len())
				.map(|i| {
					let variable_name = vas
						.variable_names
						.get(i)
						.map_or("unknown", |name| name.as_str());
					format!("{}.{} ", variable_name, child_init.vector[i])
				})
				.collect::<String>()
		);
		// Similarly, compute the target values for the child nodes.
		// This is a set of targets that the child nodes must satisfy in order to enable its parents.
		let child_targets: Vec<VasProperty> = self
			.node_target
			.iter()
			.filter_map(|prop| {
				let reqd = if self.decrement {
					let initial_value = child_init.vector.get(prop.variable_index).unwrap();
					let consumed_here = 0 - self
						.transition
						.update_vector
						.get(prop.variable_index)
						.unwrap();
					debug_message!(
						"{}consumed_here {}.{}",
						indentation,
						vas.variable_names.get(prop.variable_index).unwrap(),
						consumed_here
					);
					debug_message!("{}initial_value {}", indentation, initial_value);
					prop.target_value + (consumed_here * self.executions)
				} else {
					let initial_value = child_init.vector.get(prop.variable_index).unwrap();
					let consumed_here = 0 + self
						.transition
						.update_vector
						.get(prop.variable_index)
						.unwrap();
					debug_message!(
						"{}produced_here {}.{}",
						indentation,
						vas.variable_names.get(prop.variable_index).unwrap(),
						consumed_here
					);
					debug_message!("{}initial_value {}", indentation, initial_value);
					prop.target_value - (consumed_here * self.executions)
				};
				if reqd != 0 {
					debug_message!("{}reqd {}", indentation, reqd);
					Some(VasProperty {
						variable_index: prop.variable_index,
						target_value: reqd,
					})
				} else {
					None
				}
			})
			.collect();
		// Any value that is consumed here more than it is produced, is a negative target.
		let mut negative_targets: Vec<VasProperty> = Vec::new();
		for i in 0..child_init.vector.len() {
			if child_init.vector[i] < 0 {
				debug_message!(
					"{}negative_target {}.{}",
					indentation,
					vas.variable_names.get(i).unwrap(),
					child_init.vector[i]
				);
				negative_targets.push(VasProperty {
					variable_index: i,
					target_value: -child_init.vector[i],
				});
			}
		}
		// Combine all the targets into a single vector.
		let mut all_targets = child_targets;
		all_targets.extend(negative_targets);
		debug_message!(
			"{}child targets {}",
			indentation,
			all_targets
				.iter()
				.map(|mm| format!(
					"{}.{} ",
					vas.variable_names.get(mm.variable_index).unwrap(),
					mm.target_value
				))
				.collect::<String>()
		);
		// For all targets, try and add children nodes that meet that target.
		for target in &all_targets {
			debug_message!(
				"{}Processing target {}.{}",
				indentation,
				vas.variable_names.get(target.variable_index).unwrap(),
				target.target_value
			);
			for trans in &vas.transitions {
				debug_message!(
					"{}Checking transition {}",
					indentation,
					trans.transition_name
				);
				if self
					.parents
					.iter()
					.all(|p| p.transition_name != trans.transition_name)
				{
					let mut this_child_targets: Vec<VasProperty> = Vec::new();
					let executions: VasValue;
					if (target.target_value > 0 && trans.update_vector[target.variable_index] > 0)
						|| (target.target_value < 0
							&& trans.update_vector[target.variable_index] < 0)
					{
						debug_message!(
							"{}Sign match for transition {} on target {}.{}",
							indentation,
							trans.transition_name,
							vas.variable_names.get(target.variable_index).unwrap(),
							target.target_value
						);
						this_child_targets.push(VasProperty {
							variable_index: target.variable_index,
							target_value: target.target_value,
						});
						executions = (target.target_value
							/ trans.update_vector[target.variable_index])
							.try_into()
							.unwrap();
						debug_message!("{}Executions calculated: {}", indentation, executions);
					} else {
						debug_message!(
							"{}Sign mismatch for transition {} on target {}.{}",
							indentation,
							trans.transition_name,
							vas.variable_names.get(target.variable_index).unwrap(),
							target.target_value
						);
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
						debug_message!(
							"{}Added child node for transition {}",
							indentation,
							trans.transition_name
						);
					}
				}
			}
		}
		// Collect all the children that have the same transition name and merge them.
		let mut merged_children: Vec<Box<GraphNode>> = Vec::new();

		for child in self.children.drain(..) {
			if let Some(existing_child) = merged_children
				.iter_mut()
				.find(|c| c.transition.transition_name == child.transition.transition_name)
			{
				if child.executions > existing_child.executions {
					*existing_child = child;
				}
			} else {
				merged_children.push(child);
			}
		}
		self.children = merged_children;
		// Recursively build the graph for each child node, propagating enabled status back up the graph.
		for child in &mut self.children {
			let _ = child.rec_build_graph(vas, depth + 1);
			if !child.enabled {
				self.enabled = false;
			}
		}
		Ok(())
	}
}

/// Checks if a given property is satisfied in the current state.
fn property_sat(prop: &VasProperty, state: &VasState) -> Result<bool, String> {
	if state.vector.len() < prop.variable_index {
		return Err(format!(
			"Error: Index out of bounds for state vector: {} >= {}",
			prop.variable_index,
			state.vector.len()
		));
	}
	if state.vector[prop.variable_index] == prop.target_value {
		return Ok(true);
	}
	return Ok(false);
}

/// Top-level function to create a dependency graph from an abstract VAS model.
pub fn make_dependency_graph(
	vas: &vas_model::AbstractVas,
) -> Result<Option<DependencyGraph>, String> {
	debug_message!("Building a dependency graph.");
	// check if target is satisfied in the initial state; if not, build a root node.
	let initial_state = VasState::new(vas.initial_states[0].vector.clone());
	let initially_sat = property_sat(&vas.target, &initial_state);
	if initially_sat == Ok(true) {
		return Err(String::from("Error: Initial state satisfies the target property. Probability is 1 and this analysis is pointless."));
	} else if initially_sat.is_err() {
		return Err(String::from(
			"Error: Cannot check initial state against target property.",
		));
	}
	// figure out the executions on the artificial root node
	let target_variable = vas.target.variable_index;
	let initial_value = vas.initial_states[0].vector[target_variable];
	let target_value = vas.target.target_value;
	let target_difference = if (initial_value) < target_value {
		target_value - (initial_value)
	} else {
		(initial_value) - target_value
	};
	let decrement = (initial_value) > target_value;
	// TODO: Handle stoichiometry greater than one.
	debug_message!("Target Executions: {}", target_difference);
	// Build a new root (abstract transition) node
	let mut dependency_graph = DependencyGraph {
		root: {
			Box::new(GraphNode {
				transition: VasTransition {
					transition_id: usize::MAX,
					transition_name: "ARTIFICIAL".to_string(),
					update_vector: DVector::zeros(vas.variable_names.len()),
					enabled_bounds: DVector::zeros(vas.variable_names.len()),
					rate_const: 0.0,
					custom_rate_fn: None, // make the artificial transition here
				},
				children: Vec::new(),
				parents: Vec::new(),
				executions: target_difference,
				enabled: false,
				node_init: initial_state.clone(),
				node_target: vec![VasProperty {
					variable_index: target_variable,
					target_value: target_difference,
				}],
				decrement,
			})
		},
	};
	// handle the case where it is desired to decrease a value to reach a target
	if dependency_graph.root.decrement {
		if let Some(first_target) = dependency_graph.root.node_target.first_mut() {
			first_target.target_value -=
				dependency_graph.root.node_init.vector[first_target.variable_index];
		}
	}
	debug_message!("decrement? {}", dependency_graph.root.decrement);
	// Start building the graph from the root node.
	let _ = dependency_graph.root.rec_build_graph(vas, 1);
	Ok(Some(dependency_graph))
}

/// These methods provide functionality to print the dependency graph in various formats.
/// TODO: These should be unified into a single printout and a single JSON format.
impl DependencyGraph {
	/// Prints the dependency graph in its original format.
	/// This uses println! instead of message to simplify Beckey's work.
	pub fn original_print(&self, vas: &AbstractVas) {
		fn print_node(vas: &AbstractVas, node: &GraphNode, depth: usize) {
			let mut node_str = String::new();
			node_str.push_str(&format!(
				"{}{}",
				"|".repeat(depth),
				node.transition.transition_name
			));
			node_str.push_str(&format!(
				" {} times to {} ",
				node.executions,
				if node.decrement { "consume" } else { "produce" }
			));
			let targets_str = node
				.node_target
				.iter()
				.map(|target| {
					format!(
						"('{}',{})",
						vas.variable_names.get(target.variable_index).unwrap(),
						target.target_value
					)
				})
				.collect::<Vec<_>>()
				.join(", ");
			node_str.push_str(&format!("[{}]", targets_str));
			println!("{}", node_str);
			// message(&format!("{}", node_str));
			for child in &node.children {
				print_node(vas, child, depth + 1);
			}
		}

		print_node(vas, &self.root, 0);
	}
	/// Pretty prints the dependency graph in a human-readable format.
	pub fn pretty_print(&self, vas: &AbstractVas) {
		fn print_node(vas: &AbstractVas, node: &GraphNode, depth: usize) {
			let indent = " ".repeat(depth * 2);
			message!("{}Node: {}", indent, node.transition.transition_name);
			message!("{}  Executions: {}", indent, node.executions);
			message!("{}  Enabled: {}", indent, node.enabled);
			if node.decrement {
				message!("{}  Decrement", indent);
			}
			message!(
				"{}  Init Variables: [{}]",
				indent,
				node.node_init
					.vector
					.iter()
					.map(|v| v.to_string())
					.collect::<Vec<_>>()
					.join(", ")
			);
			message!("{}  Target Variables:", indent);
			for target in node.node_target.iter() {
				message!(
					"{}    {}: {}",
					indent,
					vas.variable_names.get(target.variable_index).unwrap(),
					target.target_value
				);
			}
			for child in &node.children {
				print_node(vas, child, depth + 1);
			}
		}
		print_node(vas, &self.root, 0);
	}

	/// Prints a simple representation of the dependency graph.
	pub fn simple_print(&self, vas: &AbstractVas) {
		message!("===================");
		message!("Dependency Graph");
		fn print_node(vas: &AbstractVas, node: &GraphNode, depth: usize) {
			let indent = " ".repeat(depth * 2);
			message!(
				"{}Node: {} (Executions: {})",
				indent,
				node.transition.transition_name,
				node.executions
			);
			for child in &node.children {
				print_node(vas, child, depth + 1);
			}
		}
		print_node(vas, &self.root, 0);
		message!("===================\n");
	}

	/// Nicely formats the dependency graph as a string for better readability.
	pub fn nice_print(&self, vas: &AbstractVas) -> String {
		let mut output = String::new();
		fn print_node(vas: &AbstractVas, node: &GraphNode, depth: usize, output: &mut String) {
			let indent = "  ".repeat(depth);
			output.push_str(&format!(
				"{}- {} (x{})\n",
				indent, node.transition.transition_name, node.executions
			));
			for target in &node.node_target {
				output.push_str(&format!(
					"{}    target: {} = {}\n",
					indent,
					vas.variable_names
						.get(target.variable_index)
						.unwrap_or(&"unknown".to_string()),
					target.target_value
				));
			}
			for child in &node.children {
				print_node(vas, child, depth + 1, output);
			}
		}
		print_node(vas, &self.root, 0, &mut output);
		output
	}

	/// Gives a vector of all the transitions in the dependency graph.
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
