use std::collections::HashMap;
use z3::{ast::{self, Ast}, Config, Context, SatResult, Solver};

use crate::{model::{self, vas_model}, AbstractVas};

pub fn get_bounds(model: AbstractVas, bits: u32) {
	println!("{}", "=".repeat(80));
	println!("CRN Variable Bound Calculator");
	println!("{}", "=".repeat(80));

	println!("Using {} bit vectors", bits);

	// Z3 setup
	let cfg = Config::new();
	let ctx = Context::new(&cfg);

	// Dependency graph (placeholder)
	// let dependency = false;
	// let dependency_graph;
	// if dependency {
	//     println!("{}", "-".repeat(80));
	//     println!("Building the dependency graph");
	//     println!("{}", "-".repeat(80));
	//     dependency_graph = make_dependency_graph(&model).unwrap();
	// } else {
	//     dependency_graph = None;
	// }

	println!("{}", "-".repeat(80));
	println!("Building the Z3 model");
	println!("{}", "-".repeat(80));

	// State variables and initialization
	let mut init_constraints = vec![];
	let mut state_vars = HashMap::new();
	let mut next_vars = HashMap::new();
	let init = model.initial_states.clone();
	let vars = model.variable_names.clone();
	
	// This only handles the first initial state
	assert!(init.len() == 1, "Only one initial state is supported");

	for i in 0..vars.len() {
		let state_var = ast::BV::new_const(&ctx, vars[i].clone(), bits);
		let next_var = ast::BV::new_const(&ctx, format!("{}_next", vars[i]), bits);
		state_vars.insert(vars[i].clone(), state_var.clone());
		next_vars.insert(vars[i].clone(), next_var.clone());
		init_constraints.push(Ast::_eq(&state_var, &ast::BV::from_i64(&ctx, init[0].vector[i], bits)));
	}

	let init_formula = ast::Bool::and(&ctx, &init_constraints.iter().collect::<Vec<_>>());

	// // Frame condition
	// let frame_cond = |vars: &Vec<&String>| {
	//     vars.iter().fold(ast::Bool::from_bool(&ctx, true), |acc, var| {
	//         ast::Bool::and(&ctx, &[&acc, &ast::Ast::_eq(&next_vars[*var], &state_vars[*var])])
	//     })
	// };

	// Encode transitions (placeholder for actual encoding logic)
	let mut transition_constraints = vec![];
	
	for transition in &model.transitions {
		println!("transition {}", transition.transition_name);
		
		// Handle consumption and production
		let mut current_transition_constraints = vec![];

		for (i, update) in transition.update_vector.iter().enumerate() {
			let state_var = &state_vars[&vars[i]];
			let next_var = &next_vars[&vars[i]];

			// Consumption
			if transition.enabled_bounds[i] > 0 {
				let consumption_constraint = state_var.bvuge(&ast::BV::from_i64(
					&ctx,
					transition.enabled_bounds[i].try_into().unwrap(),
					bits,
				));
				current_transition_constraints.push(consumption_constraint);
			}
			
			let update_constraint = if *update > 0 {
				// Update Vector
				ast::Ast::_eq(
				next_var,
				&state_var.bvadd(&ast::BV::from_i64(&ctx, (*update).try_into().unwrap(), bits)),
				)
			} else if *update < 0 {
				// Update Vector
				ast::Ast::_eq(
				next_var,
				&state_var.bvsub(&ast::BV::from_i64(&ctx, (-*update).try_into().unwrap(), bits)),
				)
			} else {
				ast::Ast::_eq(
				next_var,
				state_var,
				)
			};
			current_transition_constraints.push(update_constraint);
		}

		
		// Combine all constraints for this transition
		let current_transition_formula = ast::Bool::and(&ctx, &current_transition_constraints.iter().collect::<Vec<_>>());
		println!("Transition formula:\n{:?}", current_transition_formula);

		transition_constraints.push(current_transition_formula);
			
	}
	
	let trans_formula = ast::Bool::or(&ctx, &transition_constraints.iter().collect::<Vec<_>>());
	// println!("Transition formula:\n{:?}", trans_formula);

	// Target formula
	let target = model.target.clone();

	let target_variable_name = &vars[target.variable_index];
	let target_formula = ast::Ast::_eq(&state_vars[target_variable_name], &ast::BV::from_i64(&ctx, target.target_value.try_into().unwrap(), bits));

	// Initialize the unroller
	let mut unroller = Unroller::new(state_vars.clone(), next_vars.clone());

	// Initial formula
	let mut formula = unroller.at_time(&init_formula, 0);

	// Create a Z3 solver
	let solver = Solver::new(&ctx);

	// println!("Initial formula:");
	// println!("{:?}", formula);
	
	// Assert the initial formula
	solver.assert(&formula);
	let mut status = solver.check();
	
	// Debugging: print the current status
	match status {
		SatResult::Sat => println!("Status 1: SAT"),
		SatResult::Unsat => println!("Status 1: UNSAT"),
		SatResult::Unknown => println!("Status 1: UNKNOWN"),
	}
	
	// Start the BMC process
	let mut k = 0;
	let mut unreachable_formula = formula.clone();
	
	loop {
		if k > 999 {
			println!("Reached maximum iterations");
			break;
		}
		
		println!("-- TIME {:3} --", k);
		
		// Assume a goal at time k
		// let assump = ast::Bool::new_const(&ctx, format!("assump_{}", k));
		// solver.push();
		// let step_formula = &assump.implies(&unroller.at_time(&target_formula, k));
		// let step_formula = &ast::Bool::and(&ctx, &[&formula, step_formula]);
		let step_formula = &ast::Bool::and(&ctx, &[&formula, &unroller.at_time(&target_formula, k)]);
		
		// println!("{:?}", step_formula);

		solver.reset();
		solver.assert(step_formula);
		status = solver.check();

		if status == SatResult::Sat {
			println!("Status: SAT");
			// Remember the whole formula
			formula = ast::Bool::and(&ctx, &[&formula, &unroller.at_time(&target_formula, k)]);
			break;
		} else {
			println!("Status: UNSAT");
			// solver.assert(&assump.not());
			// solver.assert(&unroller.at_time(&trans_formula, k));
			formula = ast::Bool::and(&ctx, &[&formula, &unroller.at_time(&trans_formula, k)]);
			unreachable_formula = ast::Bool::and(&ctx, &[&unreachable_formula, &unroller.at_time(&trans_formula, k)]);
			k += 1;
		}

	}

	let sat_model = solver.get_model();
	println!("Satisfying Model:");
	println!("{:?}", sat_model);

	println!();
	println!("{}", "-".repeat(80));
	println!("Bounding the species counts");
	println!("{}", "-".repeat(80));

	// Function to calculate bounds
	let mut ub_loose: HashMap<String, i64> = HashMap::new();
	let mut ub_tight: HashMap<String, i64> = HashMap::new();
	let mut lb_loose: HashMap<String, i64> = HashMap::new();
	let mut lb_tight: HashMap<String, i64> = HashMap::new();

	// Clear the solver context
	solver.reset();

	// Assert the extended unreachable formula
	solver.assert(&formula);

	let mut status = solver.check();
	
	// Debugging: print the current status
	match status {
		SatResult::Sat => println!("Status 2: SAT"),
		SatResult::Unsat => println!("Status 2: UNSAT"),
		SatResult::Unknown => println!("Status 2: UNKNOWN"),
	}

	// Step 1: Loosest upper bounds
	for s in vars.iter() {
		let state_var = &state_vars[s];
		let mut min_bound = 0;
		let mut max_bound = (1 << bits) - 1;
		let mut bound = 0;

		println!("Checking loose upper bound for {}", s);

		loop {
			// println!("{}-{}-{}", min_bound, bound, max_bound);
			solver.reset();
			let bound_formula = &unroller.at_all_times_or(&state_var.bvuge(&ast::BV::from_i64(&ctx, bound, bits)), k);
			// println!("Bound formula:\n{:?}", bound_formula);
			let combined_formula = ast::Bool::and(&ctx, &[bound_formula, &formula]);
			// println!("Combined formula:\n{:?}", combined_formula);
			solver.assert(&combined_formula);
			status = solver.check();

			if status == SatResult::Sat {
				// println!("   SAT");
				if bound >= max_bound {
					break;
				}
				min_bound = bound;
				bound = bound + ((max_bound - bound + 1) / 2);
			} else {
				// println!("   UNSAT");
				if bound >= max_bound {
					bound -= 1;
				}
				max_bound = bound;
				if bound == (1 << bits) - 1 {
					bound -= 1;
				} else {
					bound = bound - ((bound - min_bound) / 2);
				}

			}
		}
		ub_loose.insert(s.clone(), bound);
		println!("{} loose upper bound is: {}", s, ub_loose[s]);
	}
	
	// Step 2: Tightest upper bounds
	for s in vars.iter() {
		let state_var = &state_vars[s];
		let state_var_index = model.variable_names.iter().position(|x| x == s).unwrap();
		let mut min_bound = model.initial_states[0].vector[state_var_index];
		let mut max_bound = (1 << bits) - 1;
		let mut bound = (1 << bits) - 1;

		println!("Checking tight upper bound for {}", s);

		loop {
			// println!("{}-{}-{}", min_bound, bound, max_bound);
			solver.reset();
			let bound_formula = &unroller.at_all_times_and(&state_var.bvule(&ast::BV::from_i64(&ctx, bound, bits)), k);
			// println!("Bound formula:\n{:?}", bound_formula);
			let combined_formula = ast::Bool::and(&ctx, &[bound_formula, &formula]);
			// println!("Combined formula:\n{:?}", combined_formula);
			solver.assert(&combined_formula);
			status = solver.check();

			if status == SatResult::Sat {
				// println!("   SAT");
				if bound <= min_bound {
					break;
				}
				max_bound = bound;
				if bound == 1 {
					bound = 0;
				} else {
					bound = bound - ((bound - min_bound + 1) / 2);
				}
			} else {
				// println!("   UNSAT");
				if bound <= min_bound {
					bound += 1;
					break;
				}
				min_bound = bound;
				bound = bound + ((max_bound - bound) / 2);
			}
		}
		ub_tight.insert(s.clone(), bound);
		println!("{} tight upper bound is: {}", s, ub_tight[s]);
	}

	// Step 3: Loosest lower bounds
	for s in vars.iter() {
		let state_var = &state_vars[s];
		let state_var_index = model.variable_names.iter().position(|x| x == s).unwrap();
		let mut min_bound = 0;
		let mut max_bound = model.initial_states[0].vector[state_var_index];
		let mut bound = model.initial_states[0].vector[state_var_index];
		
		println!("Checking loose lower bound for {}", s);
		
		loop {
			if max_bound == 0 {
				bound = 0;
				break;
			}
			// println!("{}-{}-{}", min_bound, bound, max_bound);
			solver.reset();
			let bound_formula = &unroller.at_all_times_or(&state_var.bvule(&ast::BV::from_i64(&ctx, bound, bits)), k);
			// println!("Bound formula:\n{:?}", bound_formula);
			let combined_formula = ast::Bool::and(&ctx, &[bound_formula, &formula]);
			// println!("Combined formula:\n{:?}", combined_formula);
			solver.assert(&combined_formula);
			status = solver.check();

			if status == SatResult::Sat {
				// println!("   SAT");
				if bound <= min_bound {
					break;
				}
				max_bound = bound;
				if bound == 1 {
					bound = 0;
				} else {
					bound = bound - ((bound - min_bound + 1) / 2);
				}
			} else {
				// println!("   UNSAT");
				if bound <= min_bound {
					bound += 1;
					break;
				}
				min_bound = bound;
				bound = bound + ((max_bound - bound) / 2);
			}
		}
		lb_loose.insert(s.clone(), bound);
		println!("{} loose lower bound is: {}", s, lb_loose[s]);
	}

	// Step 4: Tightest lower bounds
	for s in vars.iter() {
		let state_var = &state_vars[s];
		let state_var_index = model.variable_names.iter().position(|x| x == s).unwrap();
		let mut min_bound = 0;
		let mut max_bound = model.initial_states[0].vector[state_var_index];
		let mut bound = 0;
		
		println!("Checking tight lower bound for {}", s);
		
		loop {
			if max_bound == 0 {
				bound = 0;
				break;
			}
			// println!("{}-{}-{}", min_bound, bound, max_bound);
			solver.reset();
			let bound_formula = &unroller.at_all_times_and(&state_var.bvuge(&ast::BV::from_i64(&ctx, bound, bits)), k);
			// println!("Bound formula:\n{:?}", bound_formula);
			let combined_formula = ast::Bool::and(&ctx, &[bound_formula, &formula]);
			// println!("Combined formula:\n{:?}", combined_formula);
			solver.assert(&combined_formula);
			status = solver.check();

			if status == SatResult::Sat {
				// println!("   SAT");
				if bound >= max_bound {
					break;
				}
				min_bound = bound;
				bound = bound + ((max_bound - bound + 1) / 2);
			} else {
				// println!("   UNSAT");
				if bound >= max_bound {
					bound -= 1;
					break;
				}
				max_bound = bound;
				if bound == (1 << bits) - 1 {
					bound -= 1;
				} else {
					bound = bound - ((bound - min_bound) / 2);
				}
			}
		}
		lb_tight.insert(s.clone(), bound);
		println!("{} tight lower bound is: {}", s, lb_tight[s]);
	}



	// Print summary
	println!("{}", "=".repeat(80));
	println!("Summary of Bounds");
	println!("{}", "=".repeat(80));
	println!("{:<20} {:<10} {:<10} {:<10} {:<10}", "Variable", "LB Loose", "LB Tight", "UB Loose", "UB Tight");
	println!("{}", "-".repeat(80));
	for s in vars.iter() {
		println!(
			"{:<20} {:<10} {:<10} {:<10} {:<10}",
			s,
			lb_loose[s],
			lb_tight[s],
			ub_loose[s],
			ub_tight[s],
		);
	}
	println!("{}", "=".repeat(80));

}
