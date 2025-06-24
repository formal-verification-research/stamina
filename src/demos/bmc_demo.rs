use crate::bmc::vas_bmc::AbstractVasBmc;
use crate::dependency;
use crate::dependency::graph::make_dependency_graph;
use crate::model::vas_model::AbstractVas;
use crate::*;

use std::fs;
use std::path::Path;

/// Gets the list of .crn files in the models directory
fn get_crn_files(dir_path: &Path) -> Vec<String> {
	let mut crn_files: Vec<String> = Vec::new();
	for entry in fs::read_dir(dir_path).unwrap() {
		let entry = entry.unwrap();
		let path = entry.path();
		if path.is_dir() {
			for model_entry in fs::read_dir(&path).unwrap() {
				let model_entry = model_entry.unwrap();
				let model_path = model_entry.path();
				if model_path.is_file()
					&& model_path.extension().unwrap().to_str().unwrap() == "crn"
				{
					let model_name = model_path.file_stem().unwrap().to_str().unwrap();
					let folder_name = path.file_name().unwrap().to_str().unwrap();
					crn_files.push(format!("{}/{}.crn", folder_name, model_name));
				}
			}
		}
	}
	crn_files
}

/// Runs the BMC demo on all models in the specified directory.
/// The directory should contain subdirectories with .crn files.
/// This is not meant to be used by an end user, but rather as a demo
/// or proof of concept for the BMC functionality.
pub fn bmc_demo(crn_model_directory: &Path) {
	// This function is a placeholder for the actual BMC demo logic
	message!("Running BMC demo...");
	// Collect all .crn files in the directory and its subdirectories
	let crn_files: Vec<String> = get_crn_files(crn_model_directory);
	// Uncomment the following lines to test specific models manually instead of all models in the directory:
	// let mut crn_files: Vec<String> = Vec::new();
	// crn_files.push("ModifiedYeastPolarization/ModifiedYeastPolarization.crn".to_string());
	// crn_files.push("EnzymaticFutileCycle/EnzymaticFutileCycle.crn".to_string());
	for m in crn_files {
		// Parse each model file
		message!("Model: {}", m);
		let model_path = crn_model_directory.join(&m);
		let parsed_model = AbstractVas::from_file(model_path.to_str().unwrap());
		if parsed_model.is_ok() {
			let mut model = parsed_model.unwrap();
			message!("Finished parsing model: {}", m);
			debug_message!("Model: {}", model.nice_print());
			// Build the dependency graph
			let dg = make_dependency_graph(&model);
			// dg.unwrap().pretty_print();
			if let Ok(Some(dependency_graph)) = &dg {
				message!("Dependency graph created for model: {}", m);
				debug_message!(
					"Dependency graph: {:?}",
					dependency_graph.nice_print(&model)
				);

				model.setup_z3();
				let bmc_encoding = model.bmc_encoding();
				let _ = model.variable_bounds(&bmc_encoding);
				message!("Bounding completed successfully on original model.");

				// Trim the model using the dependency graph
				let mut trimmed_model =
					dependency::trimmer::trim_model(&model, dependency_graph.clone());
				message!("Trimmed model created for model: {}", m);
				debug_message!("{}", trimmed_model.nice_print());

				trimmed_model.setup_z3();
				let bmc_encoding = trimmed_model.bmc_encoding();
				let _ = trimmed_model.variable_bounds(&bmc_encoding);
				message!("Bounding completed successfully on trimmed model.");
			}
		}
	}
}
