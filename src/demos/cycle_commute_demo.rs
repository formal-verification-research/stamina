// import crate::model::vas_trie::VasTrie;


/// For now, run this demo with 
/// cargo run -- cycle-commute -d models/ModifiedYeastPolarization/ModifiedYeastPolarization.crn -t models/ModifiedYeastPolarization/MYP_Trace.txt
use crate::model::vas_model::AbstractVas;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn cycle_commute_demo(model_file: &str, trace_file: &str, output_file: &str) {

    // Get the model
    let parsed_model = AbstractVas::from_file(model_file);
	if !parsed_model.is_ok() {
        println!("Error parsing model file: {}", model_file);
        return;
    }
    
    let model = parsed_model.unwrap();
    println!("MODEL PARSED\n\n");

    crate::cycle_commute::commute::cycle_commute(model, trace_file, output_file);

}