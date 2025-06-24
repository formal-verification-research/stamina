use std::collections::HashMap;

use z3::{
	ast::{self, Ast},
	Context, SatResult, Solver,
};

use crate::{bmc::unroller::Unroller, logging::messages::*, AbstractVas};

/// Struct to hold the BMC encoding components
pub struct BMCEncoding<'a> {
	pub init_formula: ast::Bool<'a>,
	pub target_formula: ast::Bool<'a>,
	pub transition_formula: ast::Bool<'a>,
	pub unroller: Unroller<'a>,
}

/// Builds an encoding for an abstract VAS model for BMC.
impl<'a> BMCEncoding<'a> {
	/// Constructs a new BMCEncoding from the given context, config, and unroller.
	pub fn from_vas(model: &'a AbstractVas, ctx: &'a Context, bits: u32) -> Self {
		debug_message!("Building BMC encoding for VAS model");
		// Load the state variables
		let model_variables = model.variable_names.clone();
		let mut bmc_current_variables = HashMap::new();
		let mut bmc_next_variables = HashMap::new();
		// Load the initial constraints
		let model_init = model.initial_states.clone();
		let mut bmc_init_constraints = Vec::new();
		// Encode the Z3 bit-vector variables for initial constraints and state variables
		for i in 0..model_variables.len() {
			let state_var = ast::BV::new_const(&ctx, model_variables[i].clone(), bits);
			let next_var = ast::BV::new_const(&ctx, format!("{}_next", model_variables[i]), bits);
			bmc_current_variables.insert(model_variables[i].clone(), state_var.clone());
			bmc_next_variables.insert(model_variables[i].clone(), next_var.clone());
			bmc_init_constraints.push(Ast::_eq(
				&state_var,
				&ast::BV::from_i64(&ctx, model_init[0].vector[i].try_into().unwrap(), bits),
			));
		}
		debug_message!("Encoded variables for BMC:\n{:?}", bmc_current_variables);
		// Build the initial formula by conjoining the constraints
		let bmc_init_formula =
			ast::Bool::and(&ctx, &bmc_init_constraints.iter().collect::<Vec<_>>());
		debug_message!("Encoded initial state for BMC:\n{:?}", bmc_init_formula);

		// Encode the target formula
		let model_target = model.target.clone();
		let bmc_target_formula = ast::Ast::_eq(
			&bmc_current_variables[&model_variables[model_target.variable_index]],
			&ast::BV::from_i64(&ctx, model_target.target_value.try_into().unwrap(), bits),
		);
		// Encode the transitions one-by-one
		let mut bmc_transition_constraints = Vec::new();
		for transition_i in &model.transitions {
			let mut transition_i_constraints = Vec::new();
			// Loop over the updates in the transition, taking note of enabled bounds
			for (i, update) in transition_i.update_vector.iter().enumerate() {
				// Get the name of the variable impacted by this update
				let bmc_current_variable = &bmc_current_variables[&model_variables[i]];
				let bmc_next_variable = &bmc_next_variables[&model_variables[i]];
				// If the update is zero and the enabled bounds are also zero, mark no change
				if transition_i.update_vector[i] == 0 && transition_i.enabled_bounds[i] == 0 {
					transition_i_constraints
						.push(ast::Ast::_eq(bmc_next_variable, bmc_current_variable));
					continue;
				}
				// Encode the guard for the transition
				if transition_i.enabled_bounds[i] > 0 {
					let guard_constraint = bmc_current_variable.bvuge(&ast::BV::from_i64(
						&ctx,
						transition_i.enabled_bounds[i].try_into().unwrap(),
						bits,
					));
					transition_i_constraints.push(guard_constraint);
				}
				// Encode the update for the transition
				transition_i_constraints.push(if *update > 0 {
					ast::Ast::_eq(
						bmc_next_variable,
						&bmc_current_variable.bvadd(&ast::BV::from_i64(
							&ctx,
							(*update).try_into().unwrap(),
							bits,
						)),
					)
				} else {
					ast::Ast::_eq(
						bmc_next_variable,
						&bmc_current_variable.bvsub(&ast::BV::from_i64(
							&ctx,
							(*update).try_into().unwrap(),
							bits,
						)),
					)
				});
			}
			// Combine all constraints for this transition and add it to the transition constraints
			bmc_transition_constraints.push(ast::Bool::and(
				&ctx,
				&transition_i_constraints.iter().collect::<Vec<_>>(),
			));
			debug_message!(
				"Encoded transition {}:\n{:?}",
				transition_i.transition_name,
				bmc_transition_constraints.last()
			);
		}
		let bmc_transition_formula =
			ast::Bool::or(&ctx, &bmc_transition_constraints.iter().collect::<Vec<_>>());

		// Build the unroller
		let unroller = Unroller::new(bmc_current_variables, bmc_next_variables);

		BMCEncoding {
			init_formula: bmc_init_formula,
			target_formula: bmc_target_formula,
			transition_formula: bmc_transition_formula,
			unroller,
		}
	}

	/// Performs symbolic BMC and returns the result formula and number of steps taken on a tuple.
	/// `max_steps`: The max number of steps
	/// `init_formula`: The initial formula representing the system
	/// `transition_formula`: The transition formula modifying the system
	/// `target_formula`: The formula identifying the target
	/// `unroller`: The unroller used
	pub fn run_bmc(&self, ctx: &'a Context, max_steps: u32) -> (ast::Bool<'a>, u32) {
		debug_message!("Bounded Model Checking to {} steps", max_steps);
		let (init_formula, transition_formula, target_formula, unroller) = (
			&self.init_formula,
			&self.transition_formula,
			&self.target_formula,
			&mut self.unroller.clone(),
		);
		// let ctx = init_formula.get_ctx();
		let solver = Solver::new(&ctx);
		let mut formula = unroller.at_time(init_formula, 0);
		let mut max_k = 0;
		// Do the full unrolling to k steps
		for k in 0..max_steps {
			max_k = k;
			debug_message!("-- TIME {:3} --", k);

			let step_formula =
				&ast::Bool::and(&ctx, &[&formula, &unroller.at_time(&target_formula, k)]);
			// println!("Step formula:\n{:?}", step_formula);
			solver.reset();
			solver.assert(step_formula);
			let status = solver.check();

			if status == SatResult::Sat {
				// println!("Status: SAT");
				formula = ast::Bool::and(&ctx, &[&formula, &unroller.at_time(&target_formula, k)]);
				break;
			} else {
				// println!("Status: UNSAT");
				formula =
					ast::Bool::and(&ctx, &[&formula, &unroller.at_time(&transition_formula, k)]);
			}
		}
		debug_message!("Finished BMC with actual step count of {}", max_k);
		(formula, max_k)
	}
}
