mod model;
mod dependency;

use model::parser::parse_model;
use dependency::graph::make_dependency_graph;

use std::fs;
use std::path::Path;

fn main() {

	let mut crn_files: Vec<String> = Vec::new();

	// let dir_path = Path::new("models");
	// for entry in fs::read_dir(dir_path).unwrap() {
    //     let entry = entry.unwrap();
    //     let path = entry.path();
    //     if path.is_dir() {
    //         for model_entry in fs::read_dir(&path).unwrap() {
    //             let model_entry = model_entry.unwrap();
    //             let model_path = model_entry.path();

    //             if model_path.is_file() && model_path.extension().unwrap().to_str().unwrap() == "crn" {
    //                 let model_name = model_path.file_stem().unwrap().to_str().unwrap();
    //                 let folder_name = path.file_name().unwrap().to_str().unwrap();
    //                 crn_files.push(format!("{}/{}.crn", folder_name, model_name));
    //             }
    //         }
    //     }
	// }

	crn_files.push("ModifiedYeastPolarization/ModifiedYeastPolarization.crn".to_string());
	crn_files.push("EnzymaticFutileCycle/EnzymaticFutileCycle.crn".to_string());

	for m in crn_files {
		println!("\n\n\nModel: models/{}",m);
		let parsed_model = parse_model(format!("models/{}",m));
		if parsed_model.is_ok() {
			let model = parsed_model.unwrap();
			println!("{}", model.to_string());
			println!("parsing worked!");
	
			let dg = make_dependency_graph(&model);
			// dg.unwrap().pretty_print();
			dg.unwrap().simple_print();
	
		}
		else {
			println!("parsing failed");
			if let Err(e) = parsed_model {
				println!("{}", e);
			}
			continue;
		}
	}
	

	// let dep_graph = make_dependency_graph(&parsed_model.unwrap());

}
