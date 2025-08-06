use crate::model::vas_model::AbstractVas;
use crate::cycle_commute::commute::cycle_commute;
use crate::*;

/// This function runs the cycle commute demo for a given model and trace file.
/// It reads the model from the specified file, processes the trace file,
/// and writes the output to the specified output file.
/// It is not meant to be used by an end user, but rather as a demo or proof of concept for the cycle commute functionality.
/// For now, run this demo with
/// cargo run -- cycle-commute -d models/ModifiedYeastPolarization/ModifiedYeastPolarization.crn -t models/ModifiedYeastPolarization/MYP_Trace.txt
pub fn cycle_commute_demo(model_file: &str, output_file: &str) {
	if let Ok(mut abstract_model) = AbstractVas::from_file(model_file) {
		debug_message!("Model Parsed");
		let mut explicit_model = PrismVasModel::from_abstract_model(&abstract_model);
		debug_message!("Explicit Model Built");
		let mut ragtimer_builder = RagtimerBuilder::new(&abstract_model, None);
		ragtimer_builder.build(&mut explicit_model);
		debug_message!("Traces added to explicit model with Ragtimer");
		cycle_commute(&mut abstract_model, &mut explicit_model, output_file);
		debug_message!("Cycle commute demo complete. Output written to {}", output_file);
	} else {
		error!("Could not parse model");
	}
}
