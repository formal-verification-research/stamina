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

impl GraphNode {

	fn rec_build_graph(&mut self, vas: &VasModel, depth: u32) -> Result<(), String> {
		if depth > 5 {
			return Err("DEPTH OVER 5".to_string());
		}
		let spaces = " ".repeat(depth as usize);

		println!("{}Building graph at node {} x{}", spaces, self.transition.transition_name, self.executions);
		
		if self.enabled {
			println!("{}Node Enabled? {}", spaces, self.enabled);
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

		println!("{}child init {}", spaces, child_init.iter().map(|c| format!("{}.{} ", c.variable_name, c.count)).collect::<String>());

		let child_targets: Vec<Box<Variable>> = self.node_target.iter().filter_map(|x| {
			let binding = Box::new(Variable { variable_name: x.variable_name.clone(), count: 0 });
			let reqd = if self.decrement {
				let consumed_here = self.transition.decrement.iter().find(|inc| inc.variable_name == x.variable_name).unwrap_or(&binding);
				println!("{}consumed_here {}.{}", spaces, consumed_here.variable_name, consumed_here.count);
				- x.count + (consumed_here.count * self.executions as i128)
			} else {
				let produced_here = self.transition.increment.iter().find(|inc| inc.variable_name == x.variable_name).unwrap_or(&binding);
				println!("{}produced_here {}.{}", spaces, produced_here.variable_name, produced_here.count);
				x.count - (produced_here.count * self.executions as i128)
			};
			if reqd > 0 {
				Some(Box::new(Variable {
					variable_name: x.variable_name.clone(),
					count: reqd,
				}))
			} else {
				None
			}
		}).collect();
		// 	// let child_val = child_init.iter().find(|ci| ci.variable_name == x.variable_name).unwrap().count;
		// 	let binding = Box::new(Variable {variable_name:x.variable_name.clone(),count:0});
   		// 	let produced_here = self.transition.increment.iter().find(|inc| inc.variable_name == x.variable_name).unwrap_or(&binding);
		// 	println!("{}produced_here {}.{}",spaces,produced_here.variable_name,produced_here.count);
		// 	let reqd = x.count - (produced_here.count * self.executions as i128);
		// 	if reqd > 0 {
		// 		Some(Box::new(Variable {
		// 			variable_name: x.variable_name.clone(),
		// 			count: reqd,
		// 		}))
		// 	} else {
		// 		None
		// 	}
		// }).collect();

		let negative_targets: Vec<Box<Variable>> = child_init.iter().filter_map(|ci| {
			if ci.count < 0 {
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

		println!("{}child targets {}", spaces, all_targets.iter().map(|mm| format!("{}.{} ", mm.variable_name, mm.count)).collect::<String>());

		for t in &vas.transitions {
			if t.increment.iter().any(|i| all_targets.iter().any(|ct| ct.variable_name == i.variable_name)) {
				let mut this_child_targets = all_targets.clone();
				print!("{}{}: ", spaces, t.transition_name);

				println!("this child targets {}", this_child_targets.iter().map(|mm| format!("{}.{} ", mm.variable_name, mm.count)).collect::<String>());

				for x in &this_child_targets {
					if t.increment.iter().any(|v| v.variable_name == x.variable_name) {
						if self.parents.iter().all(|p| **p != t.transition_name) {
							if let Some(increment_variable) = t.increment.iter().find(|v| v.variable_name == x.variable_name) {
								let increment_count = increment_variable.count;
								let executions: u64 = if increment_count > 0 {
									(x.count / increment_count).try_into().unwrap()
								} else {
									0
								};

								let mut child = GraphNode {
									transition: t.clone(),
									children: Vec::new(),
									parents: self.parents.clone(),
									executions: executions,
									enabled: this_child_targets.is_empty(),
									node_init: child_init.clone(),
									node_target: this_child_targets.clone(),
									decrement: false
								};
								child.parents.push(Box::new(self.transition.transition_name.clone()));
								self.children.push(Box::new(child));
							}
						}
					}
				}
			}
		}

		// self.enabled = all_targets.iter().all(|v| v.count >= 0);

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

	println!("Building a dependency graph.");

	// check if target is satisfied in the initial state; if not, build a root node.
	let initially_sat = property_sat(&vas.property, &vas.variables);
	if initially_sat == Ok(true) {
		return Err(String::from("Error: Initial state satisfies the target property. Probability is 1 and this analysis is pointless."));
	}
	else if initially_sat.is_err() {
		return Err(String::from("Error: Cannot check initial state against target property."));
	}

	// print!("Targets: ");
	// figure out the executions on the artificial root node
	let target_executions = vas.variables.iter()
		.map(|x| 
			if x.variable_name == vas.property.variable {
				match vas.property.operator {
					vas_model::Operator::GreaterThan => {
						// print!(">{} ", (vas.property.value as i128 - x.count as i128) as u64);
						(vas.property.value as i128 - x.count as i128) as u64
					},
					vas_model::Operator::LessThan => {
						// print!("<{} ", (x.count as i128 - vas.property.value as i128) as u64);
						(x.count as i128 - vas.property.value as i128) as u64
					},
					vas_model::Operator::Equal => {
						if x.count < (vas.property.value as i128) {
							// print!("1={} ", (vas.property.value as i128 - x.count as i128) as u64);
							(vas.property.value as i128 - x.count as i128) as u64
						}
						else {
							// print!("2={} ", (x.count as i128 - vas.property.value as i128) as u64);
							(x.count as i128 - vas.property.value as i128) as u64
						}
					},
					vas_model::Operator::NotEqual => {
						if x.count < (vas.property.value as i128) {
							// print!("1!={} ", (x.count as i128 - vas.property.value as i128) as u64);
							(x.count as i128 - vas.property.value as i128) as u64
						}
						else {
							// print!("2!={} ", (vas.property.value as i128 - x.count as i128) as u64);
							(vas.property.value as i128 - x.count as i128) as u64
						}
					},
					vas_model::Operator::GreaterThanOrEqual => { //TODO: Figure out if I need to be off by one here.
						// print!(">={} ", (vas.property.value as i128 - x.count as i128) as u64);
						(vas.property.value as i128 - x.count as i128) as u64
					},
					vas_model::Operator::LessThanOrEqual => {
						// print!("<={} ", (x.count as i128 - vas.property.value as i128) as u64);
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
	
	// println!("");
	println!("Target Executions: {}", target_executions);
	
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
					vas.variables.iter().map(|x| 
						if x.variable_name == vas.property.variable {
							Box::new(Variable {
								variable_name: vas.property.variable.clone(),
								count: (vas.property.value as i128),
							})
						} else {
							Box::new(Variable {
								variable_name: vas.property.variable.clone(),
								count: 0,
							})
						}).collect()
					},
				// vec![Box::new(vas.property.variable.clone())],
				enabled: false,
				decrement: match vas.property.operator {
					vas_model::Operator::LessThan | vas_model::Operator::LessThanOrEqual => true,
					vas_model::Operator::GreaterThan | vas_model::Operator::GreaterThanOrEqual => false,
					vas_model::Operator::Equal | vas_model::Operator::NotEqual => {
						let initial_value = vas.variables.iter().find(|v| v.variable_name == vas.property.variable).map_or(0, |v| v.count);
						initial_value > vas.property.value as i128
					},
				}
				// {
				// 	vas.variables.iter().map(|x|
				// 		Box::new(x.count)
				// 	).collect()
				// },
				// node_target: {
				// 	vas.variables.iter().map(|x|
				// 		Box::new(x.count)
				// 	).collect()
				// },
			})
		},
	};

	println!("decrement? {}", dependency_graph.root.decrement);

	let _ = dependency_graph.root.rec_build_graph(vas, 1);

	// dependency_graph.pretty_print();

	Ok(dependency_graph)

}

impl DependencyGraph {
	pub fn pretty_print(&self) {
		fn print_node(node: &GraphNode, depth: usize) {
			let indent = " ".repeat(depth * 2);
			println!("{}Node: {}", indent, node.transition.transition_name);
			println!("{}  Executions: {}", indent, node.executions);
			println!("{}  Enabled: {}", indent, node.enabled);
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
			println!("{}{} x {} for {}", indent, node.transition.transition_name, node.executions, node.node_target.iter().map(|t| format!("{}.{} ",t.variable_name,t.count)).collect::<String>());
			for child in &node.children {
				print_node(child, depth + 1);
			}
		}

		print_node(&self.root, 0);
	}
}