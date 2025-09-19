use std::collections::HashMap;

use z3::{ast, SatResult};

use crate::bmc::encoding::BMCEncoding;
use crate::bmc::unroller::Unroller;
use crate::bmc::vas_bmc::AbstractVasBmc;
use crate::dependency::trimmer::trim_model;
use crate::model::vas_model::AbstractVas;
use crate::model::vas_model::VasValue;
use crate::*;

/// Struct to hold the BMC encoding components
pub struct BMCBounds {
	pub lb_loose: HashMap<String, VasValue>,
	pub lb_tight: HashMap<String, VasValue>,
	pub ub_loose: HashMap<String, VasValue>,
	pub ub_tight: HashMap<String, VasValue>,
}

/// Builds variable bounds for an abstract VAS model for BMC.
impl BMCBounds {
	/// Constructs a new BMCEncoding from the given context, config, and unroller.
	pub fn from_encoding(
		model: &AbstractVas,
		encoding: &BMCEncoding,
		bits: u32,
		max_steps: u32,
		backward: bool,
	) -> Self {
		// Initialize the bounds
		let mut variable_bounds = BMCBounds {
			lb_loose: HashMap::new(),
			lb_tight: HashMap::new(),
			ub_loose: HashMap::new(),
			ub_tight: HashMap::new(),
		};
		// Do BMC to get the k-step reachable formula
		let (reachable_formula, steps) = encoding.run_bmc(max_steps, backward);
		if steps == 0 || steps >= max_steps {
			debug_message!("Steps: {}", steps);
			debug_message!("Reachable formula: {:?}", reachable_formula);
			panic!("BMC failed to find a reachable state within the maximum steps.");
		}
		// Get variable names and encodings
		let variable_names = model.variable_names.clone();
		let state_vars = encoding.unroller.state_vars.clone();
		// Initialize the Z3 solver and reset the unroller
		let solver = z3::Solver::new();
		let mut unroller = encoding.unroller.clone();
		// Step 1: Loosest upper bounds
		Self::loose_upper_bounds(
			model,
			bits,
			&mut variable_bounds,
			&solver,
			&mut unroller,
			&reachable_formula,
			&state_vars,
			&variable_names,
			steps,
		);

		// Step 2: Tightest upper bounds
		Self::tight_upper_bounds(
			model,
			bits,
			&mut variable_bounds,
			&solver,
			&mut unroller,
			&reachable_formula,
			&state_vars,
			&variable_names,
			steps,
		);

		// Step 3: Loosest lower bounds
		Self::loose_lower_bounds(
			model,
			bits,
			&mut variable_bounds,
			&solver,
			&mut unroller,
			&reachable_formula,
			&state_vars,
			&variable_names,
			steps,
		);

		// Step 4: Tightest lower bounds
		Self::tight_lower_bounds(
			model,
			bits,
			&mut variable_bounds,
			&solver,
			&mut unroller,
			&reachable_formula,
			&state_vars,
			&variable_names,
			steps,
		);

		// Print summary
		debug_message!("Summary of Bounds");
		debug_message!(
			"{:<20} {:<10} {:<10} {:<10} {:<10}",
			"Variable",
			"LB Loose",
			"LB Tight",
			"UB Loose",
			"UB Tight"
		);
		for s in variable_names.iter() {
			debug_message!(
				"{:<20} {:<10} {:<10} {:<10} {:<10}",
				s,
				variable_bounds.lb_loose[s],
				variable_bounds.lb_tight[s],
				variable_bounds.ub_loose[s],
				variable_bounds.ub_tight[s],
			);
		}
		variable_bounds
	}

	/// Computes tightest upper bounds for all the variables.
	fn tight_upper_bounds(
		model: &AbstractVas,
		bits: u32,
		variable_bounds: &mut BMCBounds,
		solver: &z3::Solver,
		unroller: &mut Unroller,
		reachable_formula: &ast::Bool,
		state_vars: &HashMap<String, ast::BV>,
		variable_names: &[String],
		steps: u32,
	) {
		for s in variable_names.iter() {
			let state_var = &state_vars[s];
			let state_var_index = model.variable_names.iter().position(|x| x == s).unwrap();
			debug_message!("Checking tight upper bound for {}", s);
			let mut min_bound: VasValue = model.initial_states.clone()[0].vector[state_var_index];
			let mut max_bound: VasValue = (1 << bits) - 1;
			let mut bound: VasValue = (1 << bits) - 1;
			// This loop does a binary search for the tightest upper bound
			loop {
				solver.reset();
				let bound_formula = unroller.at_all_times_and(
					&state_var.bvule(&ast::BV::from_i64(bound.try_into().unwrap(), bits)),
					steps,
				);
				let combined_formula = ast::Bool::and(&[&bound_formula, &reachable_formula]);
				solver.assert(&combined_formula);
				let status = solver.check();
				if status == SatResult::Sat {
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
					if bound <= min_bound {
						bound += 1;
						break;
					}
					min_bound = bound;
					bound = bound + ((max_bound - bound) / 2);
				}
			}
			variable_bounds.ub_tight.insert(s.clone(), bound);
			message!(
				"{} tight upper bound is: {}",
				s,
				variable_bounds.ub_tight[s]
			);
		}
	}

	/// Computes loosest upper bounds for all the variables.
	fn loose_upper_bounds(
		model: &AbstractVas,
		bits: u32,
		variable_bounds: &mut BMCBounds,
		solver: &z3::Solver,
		unroller: &mut Unroller,
		reachable_formula: &ast::Bool,
		state_vars: &HashMap<String, ast::BV>,
		variable_names: &[String],
		steps: u32,
	) {
		for variable_name in variable_names.iter() {
			let state_var = &state_vars[variable_name];
			let state_var_index = model
				.variable_names
				.iter()
				.position(|x| x == variable_name)
				.unwrap();
			debug_message!("Checking loose upper bound for {}", variable_name);
			let mut min_bound: VasValue = model.initial_states.clone()[0].vector[state_var_index];
			let mut max_bound: VasValue = (1 << bits) - 1;
			let mut bound: VasValue = 0;
			// This loop does a binary search for the loosest upper bound
			loop {
				solver.reset();
				let bound_formula = unroller.at_all_times_or(
					&state_var.bvuge(&ast::BV::from_i64(bound.try_into().unwrap(), bits)),
					steps,
				);
				let combined_formula = ast::Bool::and(&[&bound_formula, &reachable_formula]);
				solver.assert(&combined_formula);
				let status = solver.check();
				if status == SatResult::Sat {
					if bound >= max_bound {
						break;
					}
					min_bound = bound;
					bound = bound + ((max_bound - bound + 1) / 2);
				} else {
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
			variable_bounds
				.ub_loose
				.insert(variable_name.clone(), bound);
			message!(
				"{} loose upper bound is: {}",
				variable_name,
				variable_bounds.ub_loose[variable_name]
			);
		}
	}

	/// Computes tightest lower bounds for all the variables.
	fn tight_lower_bounds(
		model: &AbstractVas,
		bits: u32,
		variable_bounds: &mut BMCBounds,
		solver: &z3::Solver,
		unroller: &mut Unroller,
		reachable_formula: &ast::Bool,
		state_vars: &HashMap<String, ast::BV>,
		variable_names: &[String],
		steps: u32,
	) {
		for s in variable_names.iter() {
			let state_var = &state_vars[s];
			let state_var_index = model.variable_names.iter().position(|x| x == s).unwrap();
			debug_message!("Checking tight lower bound for {}", s);
			let mut min_bound: VasValue = 0;
			let mut max_bound: VasValue = model.initial_states[0].vector[state_var_index];
			let mut bound: VasValue = 0;
			// This loop does a binary search for the tightest lower bound
			loop {
				if max_bound == 0 {
					bound = 0;
					break;
				}
				solver.reset();
				let bound_formula = unroller.at_all_times_and(
					&state_var.bvuge(&ast::BV::from_i64(bound.try_into().unwrap(), bits)),
					steps,
				);
				let combined_formula = ast::Bool::and(&[&bound_formula, &reachable_formula]);
				solver.assert(&combined_formula);
				let status = solver.check();
				if status == SatResult::Sat {
					if bound >= max_bound {
						break;
					}
					min_bound = bound;
					bound = bound + ((max_bound - bound + 1) / 2);
				} else {
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
			variable_bounds.lb_tight.insert(s.clone(), bound);
			message!(
				"{} tight lower bound is: {}",
				s,
				variable_bounds.lb_tight[s]
			);
		}
	}

	/// Computes loosest lower bounds for all the variables.
	fn loose_lower_bounds(
		model: &AbstractVas,
		bits: u32,
		variable_bounds: &mut BMCBounds,
		solver: &z3::Solver,
		unroller: &mut Unroller,
		reachable_formula: &ast::Bool,
		state_vars: &HashMap<String, ast::BV>,
		variable_names: &[String],
		steps: u32,
	) {
		for s in variable_names.iter() {
			let state_var = &state_vars[s];
			let state_var_index = model.variable_names.iter().position(|x| x == s).unwrap();
			debug_message!("Checking loose lower bound for {}", s);
			let mut min_bound: VasValue = 0;
			let mut max_bound: VasValue = model.initial_states[0].vector[state_var_index];
			let mut bound: VasValue = model.initial_states[0].vector[state_var_index];

			loop {
				if max_bound == 0 {
					bound = 0;
					break;
				}
				solver.reset();
				let bound_formula = unroller.at_all_times_or(
					&state_var.bvule(&ast::BV::from_i64(bound.try_into().unwrap(), bits)),
					steps,
				);
				let combined_formula = ast::Bool::and(&[&bound_formula, &reachable_formula]);
				solver.assert(&combined_formula);
				let status = solver.check();

				if status == SatResult::Sat {
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
					if bound <= min_bound {
						bound += 1;
						break;
					}
					min_bound = bound;
					bound = bound + ((max_bound - bound) / 2);
				}
			}
			variable_bounds.lb_loose.insert(s.clone(), bound);
			message!(
				"{} loose lower bound is: {}",
				s,
				variable_bounds.lb_loose[s]
			);
		}
	}
}

pub fn bound_model(
	model_file: &str,
	bits: u32,
	max_steps: u32,
	trim: bool,
) {
	// Run the bounds checking
	// TODO: Allow model trimming based on dependency graph
	if let Ok(model) = AbstractVas::from_file(model_file) {
		message!("Successfully parsed model file: {}", model_file);
		if trim {
			let dependency_graph = match crate::dependency::graph::make_dependency_graph(&model) {
				Ok(Some(dg)) => dg,
				Ok(None) => {
					error!("Failed to create dependency graph for model: {}", model_file);
					return;
				}
				Err(e) => {
					error!("Error creating dependency graph for model: {}: {}", model_file, e);
					return;
				}
			};
			let trimmed_model = trim_model(&model, dependency_graph);
			message!("Using trimmed model based on dependency graph.");
			debug_message!("Trimmed Model: {}", trimmed_model.nice_print());
			let bmc_encoding = trimmed_model.bmc_encoding(bits);
			let _ = trimmed_model.variable_bounds(&bmc_encoding, bits, max_steps, false);
			// TODO: print the bound object rather than printing in-process
			message!("Bounding completed successfully on trimmed model.");
		}
		else {
			message!("Using original model without trimming.");
			debug_message!("Model: {}", model.nice_print());
			let bmc_encoding = model.bmc_encoding(bits);
			let _ = model.variable_bounds(&bmc_encoding, bits, max_steps, false);
			// TODO: print the bound object rather than printing in-process
			message!("Bounding completed successfully on original model.");
		}
	} else {
		error!("Error parsing model file: {}", model_file);
		return;
	};
}