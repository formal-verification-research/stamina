use crate::builder::builder::Builder;
use crate::builder::ragtimer::ragtimer::RagtimerBuilder;
use crate::cycle_commute::commute::cycle_commute;
use crate::model::vas_model::{AbstractVas, PrismVasModel};
use crate::*;

/// This function runs the cycle commute demo for a given model and trace file.
/// It reads the model from the specified file, processes the trace file,
/// and writes the output to the specified output file.
/// It is not meant to be used by an end user, but rather as a demo or proof of concept for the cycle commute functionality.
/// For now, run this demo with
/// cargo run -- cycle-commute --model-file toy_model/ModifiedYeastPolarization/ModifiedYeastPolarization.crn --max-commute-depth 2 --max-cycle-length 2
pub fn cycle_commute_demo(
	model_file: &str,
	output_file: &str,
	max_commute_depth: usize,
	max_cycle_length: usize,
) {
	if let Ok(mut abstract_model) = AbstractVas::from_file(model_file) {
		debug_message!("Model Parsed");
		let mut explicit_model = PrismVasModel::from_abstract_model(&abstract_model);
		debug_message!("Explicit Model Built");
		let mut ragtimer_builder = RagtimerBuilder::new(&abstract_model, None);
		ragtimer_builder.build(&mut explicit_model);
		debug_message!("Traces added to explicit model with Ragtimer");
		cycle_commute(
			&mut abstract_model,
			&mut explicit_model,
			max_commute_depth,
			max_cycle_length,
		);
		explicit_model.print_explicit_prism_files(output_file);
		debug_message!(
			"Cycle commute demo complete. Output written to {}",
			output_file
		);
	} else {
		error!("Could not parse model");
	}
}
