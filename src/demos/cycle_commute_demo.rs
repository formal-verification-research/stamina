use crate::model::vas_model::AbstractVas;
use crate::*;

/// This function runs the cycle commute demo for a given model and trace file.
/// It reads the model from the specified file, processes the trace file,
/// and writes the output to the specified output file.
/// It is not meant to be used by an end user, but rather as a demo or proof of concept for the cycle commute functionality.
/// For now, run this demo with
/// cargo run -- cycle-commute -d models/ModifiedYeastPolarization/ModifiedYeastPolarization.crn -t models/ModifiedYeastPolarization/MYP_Trace.txt
pub fn cycle_commute_demo(model_file: &str, trace_file: &str, output_file: &str) {
	if let Ok(model) = AbstractVas::from_file(model_file) {
		debug_message!("Model Parsed");
		crate::cycle_commute::commute::cycle_commute(&model, trace_file, output_file);
	} else {
		error!("Could not parse model");
	}
}
