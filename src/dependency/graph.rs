// use prusti_contracts::*;
use crate::model::vas_model::{self, Property, Transition, Variable, VasModel};

pub(crate) struct DependencyGraph {
	root: Box<GraphNode>
}

struct GraphNode {
	transition: Box<Transition>,
	children: Vec<Box<GraphNode>>,
	parents: Vec<Box<String>>,
	executions: u64,
	enabled: bool,
	node_init: Vec<Box<Variable>>,
	node_target: Vec<Box<Variable>>,
	// decrement: bool,
	// node_target: Vec<Box<u64>>,
}

impl GraphNode {
	// fn is_enabled(&self, state: &[u64]) -> bool {
	// 	(0..state.len()).try_fold(true, |_acc, i| {
	// 		if state[i] >= *self.transition.decrement_vector[i] {
	// 			Some(true)
	// 		}
	// 		else {
	// 			None
	// 		}
	// 	}).is_some()
	// }

	fn rec_build_graph(&mut self, vas: &VasModel, depth:u32) -> Result<(), String> {
		
		if depth > 30 {
			return Err("DEPTH OVER 30".to_string());
		}
		let spaces = " ".repeat(depth as usize);

		println!("{}Building graph at node {} x{}", 
			spaces, 
			self.transition.transition_name,
			self.executions
		);

		println!("{}Node Enabled? {}", spaces, self.enabled);

		if self.enabled {
			return Ok(());
		}
		
		//TODO: Handle catalysts
		let child_init: Vec<Box<Variable>> = self.node_init.iter().map(|x|
			{
				let mut ci = Box::new(Variable {
					variable_name: x.variable_name.clone(),
					count: x.count,
				});
				self.transition.decrement.iter().for_each(|y|
					if x.variable_name == y.variable_name {
						ci.count = ci.count - (y.count * self.executions as i128);
					}
				);
				self.transition.increment.iter().for_each(|y|
					if x.variable_name == y.variable_name {
						ci.count = ci.count + (y.count * self.executions as i128);
					}
				);
				// println!("{}ci({})={}",spaces,ci.variable_name,ci.count);
				ci
			}
		).collect();
		
		println!("{}child init {}", spaces, child_init.iter().map(|c| format!("{}.{} ",c.variable_name.clone(),c.count)).collect::<String>());

		// let child_targets: Vec<Box<Variable>> = self.node_target.iter().map(|x|
		// 	{
		// 		let ct = Box::new(Variable {
		// 			variable_name: x.variable_name.clone(),
		// 			count: {
		// 				if x.count > 0 {
		// 					x.count - child_init.iter().find(|y| y.variable_name == x.variable_name).unwrap().count
		// 				}
		// 				else {
		// 					0
		// 				}
		// 			},
		// 		});
		// 		println!("{}ct({})={}",spaces,ct.variable_name,ct.count);
		// 		ct
		// 	}
		// ).collect();

		// collect the targets we haven't met with the current transition
		// let mut child_targets: Vec<Box<Variable>> = self.node_target.iter()
		// 	.filter_map(|x| {
		// 		let count = if x.count > 0 {
		// 			x.count - child_init.iter().find(|y| y.variable_name == x.variable_name).unwrap().count
		// 		} else {
		// 			0
		// 		};

		// 		if count > 0 {
		// 			Some(Box::new(Variable {
		// 				variable_name: x.variable_name.clone(),
		// 				count,
		// 			}))
		// 		} else {
		// 			None
		// 		}
		// 	})
		// 	.collect();
		
		// println!("{} t={}", spaces, t.transition_name);
		// println!("{} x.name={}", spaces, x.variable_name);
		// println!("{} var name={}", spaces, t.increment.iter().map(|yy| yy.variable_name.clone() + " ").collect::<String>());
		// println!("{} t.incrementany={}", spaces, format!("{}",t.increment.iter().any(|v: &Box<Variable>| v.variable_name == x.variable_name)));


		// 
		
		// 	{
		// 		println!("{}child targets {}", spaces, child_targets.iter().map(|mm| mm.variable_name.clone() + "." + format!("{}",mm.count).as_str() + " ").collect::<String>());
				
		// 	});


		//TODO: Handle decrement and catalyst

		println!("{}node targets {}", spaces, self.node_target.iter().map(|mm| mm.variable_name.clone() + "." + format!("{}",mm.count).as_str() + " ").collect::<String>());

		let child_targets: Vec<Box<Variable>> = self.node_target.iter()
			.filter_map(|x| {
				// let child_val = child_init.iter().find(|ci| ci.variable_name == x.variable_name).unwrap().count;
				// let reqd = x.count - child_val;
				let reqd = x.count;

				if reqd > 0 {
					Some(Box::new(Variable {
						variable_name: x.variable_name.clone(),
						count: reqd,
					}))
				} else {
					None
				}
			})
			.collect();

			println!("{}child targets {}", spaces, child_targets.iter().map(|mm| mm.variable_name.clone() + "." + format!("{}",mm.count).as_str() + " ").collect::<String>());


		let _ = vas.transitions.iter().for_each(|t|
			{
				// check if it's even worth looking at the transition
				if t.increment.iter().any(|i| {
					child_targets.iter().any(|ct| {
						ct.variable_name == i.variable_name
					}
					)
				}) {

					let mut this_child_targets = child_targets.clone();

					print!("{}{}: ", spaces, t.transition_name);

					t.decrement.iter().for_each(|d| {
						child_init.iter().filter(|ci| ci.variable_name == d.variable_name).for_each(|ci| {
							// child_init.iter().for_each(|ci| {
							if ci.count < (d.count as u64 * self.executions).into() {
								// println!("{}t={}",spaces,t.transition_name);
								// println!("{} d={}.{}",spaces,d.variable_name,d.count * self.executions as i128);
								// println!("{}  ci={}.{}",spaces,ci.variable_name,ci.count);
								print!("ci={}.{} ",ci.variable_name,ci.count);
								this_child_targets.push(Box::new(Variable {
									variable_name: ci.variable_name.clone(),
									count: (d.count * self.executions as i128 - ci.count).into()
								}));
							}
						});
					});

					println!("this child targets {}", this_child_targets.iter().map(|mm| mm.variable_name.clone() + "." + format!("{}",mm.count).as_str() + " ").collect::<String>());

					this_child_targets.iter().for_each(|x| {
						if t.increment.iter().any(|v: &Box<Variable>| v.variable_name == x.variable_name) {
							if self.parents.iter().all(|p: &Box<String>| **p != t.transition_name) {

								if let Some(increment_variable) = t.increment.iter().find(|v| v.variable_name == x.variable_name) {
									let increment_count = increment_variable.count; // Get the count of the increment variable
									let executions: u64 = if increment_count > 0 {
										(x.count / increment_count).try_into().unwrap() // Calculate the number of executions
									} else {
										0 // Handle the case where increment_count is 0 to avoid division by zero
									};
				
									let mut child = GraphNode {
										transition: t.clone(),
										children: Vec::new(),
										parents: self.parents.clone(),
										executions: executions, // Use the calculated executions
										enabled: this_child_targets.is_empty(),
										// enabled: child_init.iter().filter(|y: &&Box<Variable>| y.variable_name == x.variable_name).map(|y| y.count).sum::<i128>() >= executions.into(),
										node_init: child_init.clone(),
										node_target: this_child_targets.clone(),
										// decrement: todo!(),
									};
									child.parents.push(Box::new(self.transition.transition_name.clone()));
									self.children.push(Box::new(child));
								}
							}
						}
					})
				}
			});
					






		
		for child in &mut self.children {
			if depth > 2 {
				println!("TOO MUCH DEPTH");
				return Err("Too much depth".to_string());
			}
			let _ = child.rec_build_graph(vas, depth+1);
		}
		Err("Not Finished".to_string())
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
				// decrement: false //TODO: Add decrement logic
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

	let _ = dependency_graph.root.rec_build_graph(vas, 1);

	Err("Not Finished".to_string())

}

