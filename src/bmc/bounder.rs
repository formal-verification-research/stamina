use std::collections::HashMap;

use z3::{ast::{self, Ast}, Config, Context, SatResult, Solver};

use crate::{bmc::{bmc::bmc, formula::{build_z3_encoding, print_satisfying_model, print_z3_encoding}}, model::vas_model::AbstractVas};

const MAX_BMC_STEPS: u32 = 6000;

pub fn get_bounds(model:AbstractVas, bits: u32) {
    println!("{}", "=".repeat(80));
    println!("VAS Variable Bound Calculator");
    println!("{}", "=".repeat(80));

    // Z3 setup
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let (mut unroller, init_formula, trans_formula, target_formula) = build_z3_encoding(&model, bits, &ctx);
    let solver = Solver::new(&ctx);
    
    // Model setup
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

    
    let (mut formula, k) = bmc(init_formula, trans_formula, target_formula, unroller.clone(), MAX_BMC_STEPS);
    // println!("BMC formula:\n{:?}", formula);

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

        println!("Checking loose upper bound for {}", s);

        loop {
            // println!("{}-{}-{}", min_bound, bound, max_bound);
            solver.reset();
            let bound_formula = &unroller.at_all_times_or(&state_var.bvuge(&ast::BV::from_i64(&ctx, bound, bits)), k);
            // println!("Bound formula:\n{:?}", bound_formula);
            let combined_formula = ast::Bool::and(&ctx, &[bound_formula, &formula]);
            // println!("Combined formula:\n{:?}", combined_formula);
            solver.assert(&combined_formula);
            let status = solver.check();

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
        let mut min_bound = model.initial_states.clone()[0].vector[state_var_index];
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
            let status = solver.check();

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
            let status = solver.check();

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
            let status = solver.check();

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