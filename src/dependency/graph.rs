// use prusti_contracts::*;
use crate::model::vas_model::{self, Property, Transition, Variable, VasModel};

pub(crate) struct DependencyGraph {
	root: Box<GraphNode>,
}

struct GraphNode {
	transition: Box<Transition>,
	children: Vec<Box<GraphNode>>,
	parents: Vec<Box<String>>,
	executions: u64,
	enabled: bool,
	node_init: Vec<Box<Variable>>,
	node_target: Vec<Box<Variable>>,
	decrement: bool,
}

fn debug_println(s: String) {
	if cfg!(debug_assertions) {
		println!("{}", s);
	}
}

impl GraphNode {

	fn rec_build_graph(&mut self, vas: &VasModel, depth: u32) -> Result<(), String> {
		if depth > 500 {
			return Err("DEPTH OVER 500".to_string());
		}
		let spaces = " ".repeat(depth as usize);

		debug_println(format!("{}Building graph at node {} x{}", spaces, self.transition.transition_name, self.executions));
		
		if self.enabled {
			debug_println(format!("{}Node Enabled? {}", spaces, self.enabled));
			return Ok(());
		}

		let child_init: Vec<Box<Variable>> = self.node_init.iter().map(|x| {
			let mut ci = Box::new(Variable {
				variable_name: x.variable_name.clone(),
				count: x.count,
			});
			for y in &self.transition.decrement {
				if x.variable_name == y.variable_name {
					ci.count -= y.count * self.executions as i128;
				}
			}
			for y in &self.transition.increment {
				if x.variable_name == y.variable_name {
					ci.count += y.count * self.executions as i128;
				}
			}
			ci
		}).collect();

		debug_println(format!("{}child init {}", spaces, child_init.iter().map(|c| format!("{}.{} ", c.variable_name, c.count)).collect::<String>()));

		let child_targets: Vec<Box<Variable>> = self.node_target.iter().filter_map(|x| {
			let binding = Box::new(Variable { variable_name: "ignore_me".to_string(), count: 0 });
			let reqd = if self.decrement {
				let initial_value = child_init.iter().find(|init| init.variable_name == x.variable_name).map_or(0, |init| init.count);
				let consumed_here = self.transition.decrement.iter().find(|inc: &&Box<Variable>|inc.variable_name == x.variable_name).unwrap_or(&binding);
				debug_println(format!("{}consumed_here {}.{}", spaces, consumed_here.variable_name, consumed_here.count));
				debug_println(format!("{}initial_value {}", spaces, initial_value));
				x.count + (consumed_here.count * self.executions as i128)
			} else {
				let initial_value = child_init.iter().find(|init| init.variable_name == x.variable_name).map_or(0, |init| init.count);
				let produced_here: &Box<Variable> = self.transition.increment.iter().find(|inc| inc.variable_name == x.variable_name).unwrap_or(&binding);
				debug_println(format!("{}produced_here {}.{}", spaces, produced_here.variable_name, produced_here.count));
				debug_println(format!("{}initial_value {}", spaces, initial_value));
				x.count - (produced_here.count * self.executions as i128)
			};
			if reqd != 0 {
				debug_println(format!("{}reqd {}", spaces, reqd));
				Some(Box::new(Variable {
					variable_name: x.variable_name.clone(),
					count: reqd,
				}))
			}
			else {
				None
			}
			// if reqd != 0 {
			// } else {
			// 	None
			// }
		}).collect();

		let negative_targets: Vec<Box<Variable>> = child_init.iter().filter_map(|ci| {
			if ci.count < 0 {
				debug_println(format!("{}negative_target {}.{}", spaces, ci.variable_name, ci.count));
				Some(Box::new(Variable {
					variable_name: ci.variable_name.clone(),
					count: -ci.count,
				}))
			} else {
				None
			}
		}).collect();

		let mut all_targets = child_targets;
		all_targets.extend(negative_targets);

		debug_println(format!("{}child targets {}", spaces, all_targets.iter().map(|mm| format!("{}.{} ", mm.variable_name, mm.count)).collect::<String>()));

		for target in &all_targets {
			debug_println(format!("{}Processing target {}.{}", spaces, target.variable_name, target.count));
			for trans in &vas.transitions {
				debug_println(format!("{}Checking transition {}", spaces, trans.transition_name));
				if self.parents.iter().all(|p| **p != trans.transition_name) {
					debug_println(format!("{}Transition {} is not in parents", spaces, trans.transition_name));

					let mut this_child_targets: Vec<Box<Variable>> = Vec::new();
					let mut executions: i128 = 0;
	
					// update the child targets and executions
	
					if target.count > 0 {
						debug_println(format!("{}Target count is positive", spaces));
						if trans.increment.iter().any(|i| all_targets.iter().any(|ct| ct.variable_name == i.variable_name)) {
							debug_println(format!("{}Transition {} has increment affecting target", spaces, trans.transition_name));
							this_child_targets.push(target.clone());
							if let Some(increment_variable) = trans.increment.iter().find(|v| v.variable_name == target.variable_name) {
								let increment_count = increment_variable.count;
								executions = if increment_count > 0 {
									(target.count / increment_count).try_into().unwrap()
								} else {
									0
								};
								debug_println(format!("{}Executions calculated: {}", spaces, executions));
							}
						}
						else {
							continue;
						}
					} else {
						debug_println(format!("{}Target count is non-positive", spaces));
						if trans.decrement.iter().any(|i| all_targets.iter().any(|ct| ct.variable_name == i.variable_name)) {
							debug_println(format!("{}Transition {} has decrement affecting target", spaces, trans.transition_name));
							this_child_targets.push(target.clone());
							if let Some(decrement_variable) = trans.decrement.iter().find(|v| v.variable_name == target.variable_name) {
								let decrement_count = decrement_variable.count;
								executions = if decrement_count > 0 {
									(target.count / decrement_count).try_into().unwrap()
								} else {
									0
								};
								debug_println(format!("{}Executions calculated: {}", spaces, executions));
							}
						}
						else {
							continue;
						}
					}

					let mut child = GraphNode {
						transition: trans.clone(),
						children: Vec::new(),
						parents: self.parents.clone(),
						executions: executions.abs().try_into().unwrap(),
						enabled: this_child_targets.is_empty(),
						node_init: child_init.clone(),
						node_target: this_child_targets.clone(),
						decrement: executions < 0 
					};
					child.parents.push(Box::new(self.transition.transition_name.clone()));
					self.children.push(Box::new(child));
					debug_println(format!("{}Added child node for transition {}", spaces, trans.transition_name));
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

fn property_sat(prop: &Property, state: &Vec<Box<vas_model::Variable>>) -> Result<bool,String>{
	let result =state.iter().any(
		|x| if x.variable_name == prop.variable {
			match prop.operator {
				vas_model::Operator::GreaterThan => x.count > (prop.value as i128),
				vas_model::Operator::LessThan => x.count < (prop.value as i128),
				vas_model::Operator::Equal => x.count == (prop.value as i128),
				vas_model::Operator::NotEqual => x.count != (prop.value as i128),
				vas_model::Operator::GreaterThanOrEqual => x.count >= (prop.value as i128),
				vas_model::Operator::LessThanOrEqual => x.count <= (prop.value as i128),
			}
		} else {false}
	);
	Ok(result)
}

pub fn make_dependency_graph(vas: &vas_model::VasModel) -> Result<DependencyGraph, String> {

	debug_println(format!("Building a dependency graph."));

	// check if target is satisfied in the initial state; if not, build a root node.
	let initially_sat = property_sat(&vas.property, &vas.variables);
	if initially_sat == Ok(true) {
		return Err(String::from("Error: Initial state satisfies the target property. Probability is 1 and this analysis is pointless."));
	}
	else if initially_sat.is_err() {
		return Err(String::from("Error: Cannot check initial state against target property."));
	}

	// figure out the executions on the artificial root node
	let target_executions = vas.variables.iter()
		.map(|x| 
			if x.variable_name == vas.property.variable {
				match vas.property.operator {
					vas_model::Operator::GreaterThan => {
						(vas.property.value as i128 - x.count as i128) as u64
					},
					vas_model::Operator::LessThan => {
						(x.count as i128 - vas.property.value as i128) as u64
					},
					vas_model::Operator::Equal => {
						if x.count < (vas.property.value as i128) {
							(vas.property.value as i128 - x.count as i128) as u64
						}
						else {
							(x.count as i128 - vas.property.value as i128) as u64
						}
					},
					vas_model::Operator::NotEqual => {
						if x.count < (vas.property.value as i128) {
							(x.count as i128 - vas.property.value as i128) as u64
						}
						else {
							(vas.property.value as i128 - x.count as i128) as u64
						}
					},
					vas_model::Operator::GreaterThanOrEqual => { //TODO: Figure out if I need to be off by one here.
						(vas.property.value as i128 - x.count as i128) as u64
					},
					vas_model::Operator::LessThanOrEqual => {
						(x.count as i128 - vas.property.value as i128) as u64
					},
					// _ => 0
				}
			} else {
				0
			}
		)
		.max()
		.unwrap_or(9999); // Default to 0 if no valid differences are found
	
	// debug_println(format!("");
	debug_println(format!("Target Executions: {}", target_executions));
	
	// build a new root node
	let mut dependency_graph = DependencyGraph {
		root: {
			Box::new(GraphNode {
				transition: Box::new(Transition { // make the artificial transition here
					increment: Vec::new(),
					decrement: Vec::new(),
					increment_vector: Vec::new(),
					decrement_vector: Vec::new(),
					transition_name: "PostUntil".to_string(),
					transition_rate: 0.0,
				}),
				children: Vec::new(),
				parents: Vec::new(),
				executions: target_executions,
				node_init: vas.variables.clone(),
				node_target: {
							let mut targ = Vec::new();
							targ.push(Box::new(Variable {
								variable_name: vas.property.variable.clone(),
								count: (vas.property.value as i128),
							}));
							targ
					},
				enabled: false,
				decrement: match vas.property.operator {
					vas_model::Operator::LessThan | vas_model::Operator::LessThanOrEqual => true,
					vas_model::Operator::GreaterThan | vas_model::Operator::GreaterThanOrEqual => false,
					vas_model::Operator::Equal | vas_model::Operator::NotEqual => {
						let initial_value = vas.variables.iter().find(|v| v.variable_name == vas.property.variable).map_or(0, |v| v.count);
						initial_value > vas.property.value as i128
					},
				}
			})
		},
	};

	// handle the decrement case
	if dependency_graph.root.decrement {
		if let Some(first_target) = dependency_graph.root.node_target.first_mut() {
			first_target.count = first_target.count - dependency_graph.root.node_init.iter().find(|x| x.variable_name == first_target.variable_name).unwrap().count;
		}
	}

	debug_println(format!("decrement? {}", dependency_graph.root.decrement));

	let _ = dependency_graph.root.rec_build_graph(vas, 1);

	if cfg!(debug_assertions) {
		dependency_graph.pretty_print();
	}

	Ok(dependency_graph)

}

impl DependencyGraph {
	pub fn pretty_print(&self) {
		fn print_node(node: &GraphNode, depth: usize) {
			let indent = " ".repeat(depth * 2);
			println!("{}Node: {}", indent, node.transition.transition_name);
			println!("{}  Executions: {}", indent, node.executions);
			println!("{}  Enabled: {}", indent, node.enabled);
			if node.decrement {
				println!("{}  Decrement", indent);
			}
			println!("{}  Init Variables:", indent);
			for var in &node.node_init {
				println!("{}    {}: {}", indent, var.variable_name, var.count);
			}
			println!("{}  Target Variables:", indent);
			for var in &node.node_target {
				println!("{}    {}: {}", indent, var.variable_name, var.count);
			}
			for child in &node.children {
				print_node(child, depth + 1);
			}
		}

		print_node(&self.root, 0);
	}
	pub fn simple_print(&self) {
		fn print_node(node: &GraphNode, depth: usize) {
			let indent = "|".repeat(depth);
			let targets: Vec<(String, i128)> = node.node_target.iter().map(|t| (t.variable_name.clone(), t.count)).collect();
			println!("{}{} {} times to produce {:?}", indent, node.transition.transition_name, node.executions, targets);
			for child in &node.children {
				print_node(child, depth + 1);
			}
		}

		print_node(&self.root, 0);
	}
}