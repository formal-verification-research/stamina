use std::collections::HashMap;

use z3::{ast, SatResult};

use crate::bmc::encoding::BMCEncoding;
use crate::bmc::vas_bmc::MAX_BMC_STEPS;
use crate::model::vas_model::VasValue;
use crate::AbstractVas;
use crate::*;

/// Struct to hold the BMC encoding components
pub struct BMCBounds {
	pub lb_loose: HashMap<String, VasValue>,
	pub lb_tight: HashMap<String, VasValue>,
	pub ub_loose: HashMap<String, VasValue>,
	pub ub_tight: HashMap<String, VasValue>,
}

/// Builds variable bounds for an abstract VAS model for BMC.
impl<'a> BMCBounds {
	/// Constructs a new BMCEncoding from the given context, config, and unroller.
	pub fn from_encoding(
		model: &'a AbstractVas,
		encoding: &'a BMCEncoding<'a>,
		ctx: &'a z3::Context,
		bits: u32,
	) -> Self {
		// Initialize the bounds
		let mut variable_bounds = BMCBounds {
			lb_loose: HashMap::new(),
			lb_tight: HashMap::new(),
			ub_loose: HashMap::new(),
			ub_tight: HashMap::new(),
		};
		// Do BMC to get the k-step reachable formula
		let (reachable_formula, steps) = encoding.run_bmc(ctx, MAX_BMC_STEPS);
		if steps == 0 || steps >= MAX_BMC_STEPS {
			debug_message!("Steps: {}", steps);
			debug_message!("Reachable formula: {:?}", reachable_formula);
			panic!("BMC failed to find a reachable state within the maximum steps.");
		}
		// Get variable names and encodings
		let variable_names = model.variable_names.clone();
		let state_vars = encoding.unroller.state_vars.clone();
		// Initialize the Z3 solver and reset the unroller
		let solver = z3::Solver::new(ctx);
		let mut unroller = encoding.unroller.clone();
		// Step 1: Loosest upper bounds
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
					&state_var.bvuge(&ast::BV::from_i64(&ctx, bound.try_into().unwrap(), bits)),
					steps,
				);
				let combined_formula = ast::Bool::and(&ctx, &[&bound_formula, &reachable_formula]);
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
		// Step 2: Tightest upper bounds
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
					&state_var.bvule(&ast::BV::from_i64(&ctx, bound.try_into().unwrap(), bits)),
					steps,
				);
				let combined_formula = ast::Bool::and(&ctx, &[&bound_formula, &reachable_formula]);
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
		// Step 3: Loosest lower bounds
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
					&state_var.bvule(&ast::BV::from_i64(&ctx, bound.try_into().unwrap(), bits)),
					steps,
				);
				let combined_formula = ast::Bool::and(&ctx, &[&bound_formula, &reachable_formula]);
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
		// Step 4: Tightest lower bounds
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
					&state_var.bvuge(&ast::BV::from_i64(&ctx, bound.try_into().unwrap(), bits)),
					steps,
				);
				let combined_formula = ast::Bool::and(&ctx, &[&bound_formula, &reachable_formula]);
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
}
