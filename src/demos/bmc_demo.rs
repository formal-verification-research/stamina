use crate::dependency;
use crate::dependency::graph::make_dependency_graph;
use crate::model::vas_model::AbstractVas;
use crate::bmc::bounder::get_bounds;
use crate::logging::messages::*;

use std::fs;
use std::path::Path;
use std::time::Instant;

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
				if model_path.is_file() && model_path.extension().unwrap().to_str().unwrap() == "crn" {
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
pub fn bmc_demo(crn_model_directory: &Path, timeout_minutes: u64) {
    // This function is a placeholder for the actual BMC demo logic
    message(&format!("Running BMC demo..."));
	// Collect all .crn files in the directory and its subdirectories
	let mut crn_files: Vec<String> = get_crn_files(crn_model_directory);
	// Uncomment the following lines to test specific models manually instead of all models in the directory:
	// let mut crn_files: Vec<String> = Vec::new();
	// crn_files.push("ModifiedYeastPolarization/ModifiedYeastPolarization.crn".to_string());
	// crn_files.push("EnzymaticFutileCycle/EnzymaticFutileCycle.crn".to_string());
	for m in crn_files {
		// Parse each model file
		message(&format!("\n{}\n", "█".repeat(80)));
		message(&format!("Model: models/{}", m));
		let parsed_model = AbstractVas::from_file(&format!("models/{}", m));
		if parsed_model.is_ok() {
			let model = parsed_model.unwrap();
			message(&format!("Finished parsing model: {}", m));
			debug_message(&format!("Model: {}", model.nice_print()));
			// Build the dependency graph
			let dg = make_dependency_graph(&model);
			// dg.unwrap().pretty_print();
			if let Ok(Some(dependency_graph)) = &dg {
				message(&format!("Dependency graph created for model: {}", m));
				debug_message(&format!("Dependency graph: {:?}", dependency_graph.nice_print(&model)));
				// Trim the model using the dependency graph
				let trimmed_model = dependency::trimmer::trim_model(model.clone(), dependency_graph.clone());
				message(&format!("Trimmed model created for model: {}", m));
				debug_message(&format!("{}", trimmed_model.nice_print()));
				let start = Instant::now();
				let result = std::thread::spawn(move || {
					// TODO: Implement a calculator instead of a fixed number of bits
					get_bounds(model.clone(), 8)
				});
				let timeout = std::time::Duration::from_secs(timeout_minutes*60);
				let (tx, rx) = std::sync::mpsc::channel();
				std::thread::spawn(move || {
					let _ = result.join();
					let _ = tx.send(());
				});
				if rx.recv_timeout(timeout).is_ok() {
					debug_message(&format!("get_bounds completed successfully"));
				} else {
					warning(&format!("get_bounds timed out after {} minutes", timeout_minutes));
				}
				let duration = start.elapsed();
				message(&format!("Time taken by get_bounds on regular model: {:?}", duration));
				let start = Instant::now();
				let result = std::thread::spawn(move || {
					get_bounds(trimmed_model.clone(), 8)
				});
				let timeout = std::time::Duration::from_secs(timeout_minutes*60);
				let (tx, rx) = std::sync::mpsc::channel();
				std::thread::spawn(move || {
					let _ = result.join();
					let _ = tx.send(());
				});
				if rx.recv_timeout(timeout).is_ok() {
					debug_message(&format!("get_bounds completed successfully"));
				} else {
					warning(&format!("get_bounds timed out after {} minutes", timeout_minutes));
				}
				let duration = start.elapsed();
				message(&format!("Time taken by get_bounds on trimmed model: {:?}", duration));
			} else {
				message(&format!("Failed to create dependency graph"));
			}
		}
		else {
			error(&format!("parsing failed"));
			if let Err(e) = parsed_model {
				error(&format!("{}", e));
			}
			continue;
		}
		error(&format!("\n{}\n", "█".repeat(80)));
	}
}
