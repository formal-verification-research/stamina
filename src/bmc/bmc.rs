use z3::{ast::{self, Ast}, SatResult, Solver};

use crate::bmc::unroller::Unroller;


// Returns (formula, steps)
pub fn bmc<'a>(init_formula: ast::Bool<'a>, trans_formula: ast::Bool<'a>, target_formula: ast::Bool<'a>, mut unroller: Unroller<'a>, steps: u32) -> (ast::Bool<'a>, u32) {
    println!("{}", "=".repeat(80));
    println!("Bounded Model Checking to {} steps", steps);
    println!("{}", "=".repeat(80));

    let ctx = init_formula.get_ctx();
    let solver = Solver::new(&ctx);
    let mut formula = unroller.at_time(&init_formula, 0);
    let mut max_k = 0;

    for k in 0..steps {
        max_k = k;
        println!("-- TIME {:3} --", k);

        let step_formula = &ast::Bool::and(&ctx, &[&formula, &unroller.at_time(&target_formula, k)]);
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
            formula = ast::Bool::and(&ctx, &[&formula, &unroller.at_time(&trans_formula, k)]);
        }

    }

    // println!("Final formula:\n{:?}", formula);
    println!("Final steps: {}", max_k);

    (formula, max_k)

}