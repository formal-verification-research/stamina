mod model;
mod dependency;
mod logging;
mod parser;
mod property;
mod util;
mod validator;
mod bmc;

use bmc::formula::{print_satisfying_model, print_z3_encoding};
// use crate::parser;
use dependency::graph::make_dependency_graph;
use model::vas_model::AbstractVas;
use bmc::bounder::get_bounds;
// use bounder::z3_bounds::get_bounds;
use logging::messages::*;

use std::fs;
use std::path::Path;
use std::time::Instant;

const TIMEOUT_MINUTES: u64 = 10; // 

fn main() {
	
	
	let mut crn_files: Vec<String> = Vec::new();
	let dir_path = Path::new("models");
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
	
	// crn_files.push("ModifiedYeastPolarization/ModifiedYeastPolarization.crn".to_string());
	// crn_files.push("EnzymaticFutileCycle/EnzymaticFutileCycle.crn".to_string());
	
	for m in crn_files {
		println!("\n{}\n", "█".repeat(80));
		message(&format!("Model: models/{}", m));
		println!("Model: models/{}", m);
		let parsed_model = AbstractVas::from_file(&format!("models/{}", m));
		
		if parsed_model.is_ok() {
			let (model,property) = parsed_model.unwrap();
			// println!("{:?}", model.debug_print());
			println!("MODEL PARSED\n\n");
			println!("{}", model.nice_print());
			
			let dg = make_dependency_graph(&model);
			// dg.unwrap().pretty_print();
			if let Ok(Some(dependency_graph)) = &dg {
				dependency_graph.simple_print(&model);
				let trimmed_model = dependency::trimmer::trim_model(model.clone(), dependency_graph.clone());
				println!("{}", trimmed_model.nice_print());

				let start = Instant::now();
				let result = std::thread::spawn(move || {
					get_bounds(model.clone(), 8)
				});
				let timeout = std::time::Duration::from_secs(TIMEOUT_MINUTES*60);
				let (tx, rx) = std::sync::mpsc::channel();
				std::thread::spawn(move || {
					let _ = result.join();
					let _ = tx.send(());
				});
				if rx.recv_timeout(timeout).is_ok() {
					println!("get_bounds completed successfully");
				} else {
					println!("get_bounds timed out after {} minutes", TIMEOUT_MINUTES);
				}
				let duration = start.elapsed();
				println!("Time taken by get_bounds on regular model: {:?}", duration);
				
				let start = Instant::now();
				let result = std::thread::spawn(move || {
					get_bounds(trimmed_model.clone(), 8)
				});
				let timeout = std::time::Duration::from_secs(TIMEOUT_MINUTES*60);
				let (tx, rx) = std::sync::mpsc::channel();
				std::thread::spawn(move || {
					let _ = result.join();
					let _ = tx.send(());
				});
				if rx.recv_timeout(timeout).is_ok() {
					println!("get_bounds completed successfully");
				} else {
					println!("get_bounds timed out after {} minutes", TIMEOUT_MINUTES);
				}
				let duration = start.elapsed();
				println!("Time taken by get_bounds on trimmed model: {:?}", duration);
				
				
				// let start = Instant::now();
				// get_bounds(trimmed_model.clone(), 8);
				// let duration = start.elapsed();
				// println!("Time taken by get_bounds on trimmed model: {:?}", duration);

			} else {
				println!("Failed to create dependency graph");
			}
			// print_z3_encoding(model.clone(), bits, steps);
			// print_satisfying_model(model.clone(), bits, steps);
			
			
		}
		else {
			println!("parsing failed");
			if let Err(e) = parsed_model {
				println!("{}", e);
			}
			continue;
		}
		println!("\n{}\n", "█".repeat(80));
	}
	

	// let dep_graph = make_dependency_graph(&parsed_model.unwrap());

}
