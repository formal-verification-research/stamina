use std::collections::HashMap;

use z3::{ast::{self, Ast}, Config, Context, SatResult, Solver};

use crate::{bmc::{unroller::Unroller, bmc::bmc}, AbstractVas};

// Because this uses BMC, it stops at the first SAT result, not necessarily going until k
pub fn print_z3_encoding(model: AbstractVas, bits: u32, steps: u32) {
    // Z3 setup
    let cfg = Config::new();
    let ctx = Context::new(&cfg);

    let (unroller, init_formula, trans_formula, target_formula) = build_z3_encoding(&model, bits, &ctx);
    
    let formula = bmc(init_formula, trans_formula, target_formula, unroller, steps);

    println!("{}", "=".repeat(80));
    println!("Z3 Encoding:");
    println!("{:?}", formula);
    println!("{}", "=".repeat(80));
    
}

// Because this uses BMC, it stops at the first SAT result, not necessarily going until k
pub fn print_satisfying_model(model: AbstractVas, bits: u32, steps: u32) {
    // Z3 setup
    let cfg = Config::new();
    let ctx = Context::new(&cfg);

    let (unroller, init_formula, trans_formula, target_formula) = build_z3_encoding(&model, bits, &ctx);
    
    let (formula, steps) = bmc(init_formula, trans_formula, target_formula, unroller, steps);

    println!("Model required {} steps", steps);
    let solver = Solver::new(&ctx);
    solver.assert(&formula);
    let status = solver.check();
    match status {
        SatResult::Sat => {
            println!("SAT: Satisfying model found");
            let model = solver.get_model();
            println!("Model: {:?}", model);
        },
        SatResult::Unsat => {
            println!("UNSAT: No satisfying model found");
        },
        _ => {
            println!("UNKNOWN: Unable to determine satisfiability");
        }
    }
    
}

// Returns (unroller, state_vars, next_vars, init_formula, transition_formula, target_formula)
pub fn build_z3_encoding<'ctx>(model: &AbstractVas, bits: u32, ctx: &'ctx Context) -> (Unroller<'ctx>, ast::Bool<'ctx>, ast::Bool<'ctx>, ast::Bool<'ctx>) {

    println!("{}", "=".repeat(80));
    println!("Z3 Encoder for VAS");
    println!("{}", "=".repeat(80));

    println!("Using {}-bit vectors", bits);

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
    println!("Encoded initial state");

    // Encode transitions (placeholder for actual encoding logic)
    let mut transition_constraints = vec![];
    
    for transition in &model.transitions {
        // println!("transition {}", transition.transition_name);
        
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
        // println!("Transition formula:\n{:?}", current_transition_formula);
        println!("Encoded transition {}", transition.transition_name);

        transition_constraints.push(current_transition_formula);
            
    }
    
    let transition_formula = ast::Bool::or(&ctx, &transition_constraints.iter().collect::<Vec<_>>());

    // Target formula
    let target = model.target.clone();

    println!("Target variable: {}", target.variable_index);
    println!("Target value: {}", target.target_value);
    println!("Vars: {:?}", vars);

    let target_variable_name = &vars[target.variable_index];
    let target_formula = ast::Ast::_eq(&state_vars[target_variable_name], &ast::BV::from_i64(&ctx, target.target_value.try_into().unwrap(), bits));
    println!("Encoded target state");

    let unroller = Unroller::new(state_vars.clone(), next_vars.clone());

    println!("Completed BMC encoding.");
    (unroller, init_formula, transition_formula, target_formula)

}