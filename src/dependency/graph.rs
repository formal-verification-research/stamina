// use prusti_contracts::*;
use crate::model::vas_model::{self, Property, Transition};

pub(crate) struct DependencyGraph {
	root: Box<GraphNode>
}

struct GraphNode {
	transition: Box<vas_model::Transition>,
	children: Vec<Box<GraphNode>>,
	parents: Vec<Box<GraphNode>>,
	executions: u64,
	variables_desired: Vec<Box<String>>,
	enabled: bool,
}

impl GraphNode {
	// #[pure]
	// #[requires(state.len() == self.transition.increment_vector.len())]
	// #[requires(state.len() == self.transition.decrement_vector.len())]
	// // #[ensures(result <==>
	// // 		  //forall(|i: usize| {
	// // 	//i < state.len() ==> (state[i] >= self.transition.decrement_vector[i])
	// // }))]
	fn is_enabled(&self, state: &[u64]) -> bool {
		(0..state.len()).try_fold(true, |_acc, i| {
			// body_invariant!(i < state.len());
			if state[i] >= *self.transition.decrement_vector[i] {
				Some(true)
			}
			else {
				None
			}
		}).is_some()
	}

	fn rec_build_graph(&self) -> Result<GraphNode, String> {
		Err("Not Finished".to_string())
	}

}

fn property_sat(prop: &Property, state: &Vec<Box<vas_model::Variable>>) -> Result<bool,String>{
	let result =state.iter().any(
		|x| if x.variable_name == prop.variable {
			match prop.operator {
				vas_model::Operator::GreaterThan => x.count > prop.value,
				vas_model::Operator::LessThan => x.count < prop.value,
				vas_model::Operator::Equal => x.count == prop.value,
				vas_model::Operator::NotEqual => x.count != prop.value,
				vas_model::Operator::GreaterThanOrEqual => x.count >= prop.value,
				vas_model::Operator::LessThanOrEqual => x.count <= prop.value,
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
						if x.count < vas.property.value {
							// print!("1={} ", (vas.property.value as i128 - x.count as i128) as u64);
							(vas.property.value as i128 - x.count as i128) as u64
						}
						else {
							// print!("2={} ", (x.count as i128 - vas.property.value as i128) as u64);
							(x.count as i128 - vas.property.value as i128) as u64
						}
					},
					vas_model::Operator::NotEqual => {
						if x.count < vas.property.value {
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
					_ => 0
				}
			} else {
				0
			}
		)
		.max()
		.unwrap_or(9999); // Default to 0 if no valid differences are found
	
	// println!("");
	println!("Target Executions: {}", target_executions);
	
	// create new targets (currently just stores the target in a vector)
	// let mut new_targets: Vec<Box<Property>> = Vec::new();
	// new_targets.push(Box::new(vas.property.clone()));

	// build a new root node
	let dependency_graph = DependencyGraph {
		root: {
			Box::new(GraphNode {
				transition: Box::new(Transition { // make the artificial transition here
					increment_vector: Vec::new(),
					decrement_vector: Vec::new(),
					transition_name: "PostUntil".to_string(),
					transition_rate: 0.0,
				}),
				children: Vec::new(),
				parents: Vec::new(),
				executions: target_executions,
				variables_desired: vec![Box::new(vas.property.variable.clone())],
				enabled: false,
			})
		},
	};

	dependency_graph.root.rec_build_graph();

	
	Err("Not Finished".to_string())

}

