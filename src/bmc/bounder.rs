use std::collections::HashMap;

use z3::{ast::{self, Ast}, Config, Context, SatResult, Solver};

use crate::{bmc::{bmc::bmc, formula::build_z3_encoding}, logging::messages::*, model::vas_model::AbstractVas};

// TODO: this needs to be configurable by the user or calculated with the dependency graph.
/// Maximum number of BMC steps to take before giving up.
// This is a safety limit to prevent infinite loops in case of unreachable formulas.
const MAX_BMC_STEPS: u32 = 6000;

/// Calculates the bounds for each variable in the VAS model using BMC.
/// It computes both loose and tight bounds for upper and lower limits of each variable.
pub fn get_bounds(model:AbstractVas, bits: u32) {
    message(&format!("{}", "=".repeat(80)));
    message(&format!("VAS Variable Bound Calculator"));
    message(&format!("{}", "=".repeat(80)));
    // Z3 setup
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let (mut unroller, init_formula, trans_formula, target_formula) = build_z3_encoding(&model, bits, &ctx);
    let solver = Solver::new(&ctx);
    // SMT model encoding setup
    let mut init_constraints = vec![];
    let mut state_vars = HashMap::new();
    let mut next_vars = HashMap::new();
    let init = model.initial_states.clone();
    let vars = model.variable_names.clone();
    for i in 0..vars.len() {
        let state_var = ast::BV::new_const(&ctx, vars[i].clone(), bits);
        let next_var = ast::BV::new_const(&ctx, format!("{}_next", vars[i]), bits);
        state_vars.insert(vars[i].clone(), state_var.clone());
        next_vars.insert(vars[i].clone(), next_var.clone());
        init_constraints.push(Ast::_eq(&state_var, &ast::BV::from_i64(&ctx, init[0].vector[i], bits)));
    }
    // Run BMC to get a formula that represents the model alongside the number of steps, k
    let (formula, k) = bmc(init_formula, trans_formula, target_formula, unroller.clone(), MAX_BMC_STEPS);
    // Set up bound hashmaps
    let mut ub_loose: HashMap<String, i64> = HashMap::new();
    let mut ub_tight: HashMap<String, i64> = HashMap::new();
    let mut lb_loose: HashMap<String, i64> = HashMap::new();
    let mut lb_tight: HashMap<String, i64> = HashMap::new();
    // Step 1: Loosest upper bounds
    for s in vars.iter() {
        let state_var = &state_vars[s];
        let state_var_index = model.variable_names.iter().position(|x| x == s).unwrap();
        let mut min_bound = model.initial_states.clone()[0].vector[state_var_index];
        let mut max_bound = (1 << bits) - 1;
        let mut bound = 0;
        debug_message(&format!("Checking loose upper bound for {}", s));
        // This loop does a binary search for the loosest upper bound
        loop {
            solver.reset();
            let bound_formula = &unroller.at_all_times_or(&state_var.bvuge(&ast::BV::from_i64(&ctx, bound, bits)), k);
            let combined_formula = ast::Bool::and(&ctx, &[bound_formula, &formula]);
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
        ub_loose.insert(s.clone(), bound);
        message(&format!("{} loose upper bound is: {}", s, ub_loose[s]));
    }
    // Step 2: Tightest upper bounds
    for s in vars.iter() {
        let state_var = &state_vars[s];
        let state_var_index = model.variable_names.iter().position(|x| x == s).unwrap();
        let mut min_bound = model.initial_states.clone()[0].vector[state_var_index];
        let mut max_bound = (1 << bits) - 1;
        let mut bound = (1 << bits) - 1;
        debug_message(&format!("Checking tight upper bound for {}", s));
        // This loop does a binary search for the tightest upper bound
        loop {
            solver.reset();
            let bound_formula = &unroller.at_all_times_and(&state_var.bvule(&ast::BV::from_i64(&ctx, bound, bits)), k);
            let combined_formula = ast::Bool::and(&ctx, &[bound_formula, &formula]);
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
        ub_tight.insert(s.clone(), bound);
        message(&format!("{} tight upper bound is: {}", s, ub_tight[s]));
    }
    // Step 3: Loosest lower bounds
    for s in vars.iter() {
        let state_var = &state_vars[s];
        let state_var_index = model.variable_names.iter().position(|x| x == s).unwrap();
        let mut min_bound = 0;
        let mut max_bound = model.initial_states[0].vector[state_var_index];
        let mut bound = model.initial_states[0].vector[state_var_index];
        
        debug_message(&format!("Checking loose lower bound for {}", s));
        
        loop {
            if max_bound == 0 {
                bound = 0;
                break;
            }
            solver.reset();
            let bound_formula = &unroller.at_all_times_or(&state_var.bvule(&ast::BV::from_i64(&ctx, bound, bits)), k);
            let combined_formula = ast::Bool::and(&ctx, &[bound_formula, &formula]);
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
        lb_loose.insert(s.clone(), bound);
        message(&format!("{} loose lower bound is: {}", s, lb_loose[s]));
    }
    // Step 4: Tightest lower bounds
    for s in vars.iter() {
        let state_var = &state_vars[s];
        let state_var_index = model.variable_names.iter().position(|x| x == s).unwrap();
        let mut min_bound = 0;
        let mut max_bound = model.initial_states[0].vector[state_var_index];
        let mut bound = 0;
        debug_message(&format!("Checking tight lower bound for {}", s));
        // This loop does a binary search for the tightest lower bound
        loop {
            if max_bound == 0 {
                bound = 0;
                break;
            }
            solver.reset();
            let bound_formula = &unroller.at_all_times_and(&state_var.bvuge(&ast::BV::from_i64(&ctx, bound, bits)), k);
            let combined_formula = ast::Bool::and(&ctx, &[bound_formula, &formula]);
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
        lb_tight.insert(s.clone(), bound);
        message(&format!("{} tight lower bound is: {}", s, lb_tight[s]));
    }
    // Print summary
    message(&format!("{}", "=".repeat(80)));
    message(&format!("Summary of Bounds"));
    message(&format!("{}", "=".repeat(80)));
    message(&format!("{:<20} {:<10} {:<10} {:<10} {:<10}", "Variable", "LB Loose", "LB Tight", "UB Loose", "UB Tight"));
    message(&format!("{}", "-".repeat(80)));
    for s in vars.iter() {
        message(&format!(
            "{:<20} {:<10} {:<10} {:<10} {:<10}",
            s,
            lb_loose[s],
            lb_tight[s],
            ub_loose[s],
            ub_tight[s],
        ));
    }
    message(&format!("{}", "=".repeat(80)));
}