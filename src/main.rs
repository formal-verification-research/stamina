mod model;
mod dependency;
mod logging;
mod parser;
mod property;
mod util;
mod validator;
mod bmc;
mod demos;
mod cycle_commute;

use std::path::Path;
use clap::{Arg, Command};
use dependency::graph::make_dependency_graph;
use model::vas_model::AbstractVas;

const TIMEOUT_MINUTES: &str = "10"; // 

fn main() {
	let matches = Command::new("practice")
		.version("0.0.1")
		.author("Formal Verification Research at Utah State University")
		.about("More details coming soon")
		.subcommand(
			Command::new("bounds")
				.about("Run the variable bounding tool")
				.arg(
					Arg::new("models_dir")
						.short('d')
						.long("models-dir")
						.value_name("DIR")
						.help("Sets the directory containing model folders")
						.default_value("models"),
				)
				.arg(
					Arg::new("timeout")
						.short('t')
						.long("timeout")
						.value_name("MINUTES")
						.help("Timeout in minutes for get_bounds")
						.default_value(TIMEOUT_MINUTES),
				)
		)
		.subcommand(
			Command::new("dependency-graph")
				.about("Run the variable bounding tool")
				.arg(
					Arg::new("model")
						.short('m')
						// .long("model")
						.value_name("FILE")
						.help("Sets the model file")
						.required(true),
				)
		)
		.subcommand(
			Command::new("ragtimer")
				.about("Run the ragtimer tool")
				.arg(
					Arg::new("models_dir")
						.required(true)
						.short('d')
						.long("models-dir")
						.value_name("DIR")
						.help("Sets the directory containing model folders")
						.default_value("models"),
				)
				.arg(
					Arg::new("timeout")
						.short('t')
						.long("timeout")
						.value_name("MINUTES")
						.help("Timeout in minutes for get_bounds")
						.default_value(TIMEOUT_MINUTES),
				)
		)
		.subcommand(
			Command::new("cycle-commute")
				.about("Run the Cycle & Commute tool")
				.arg(
					Arg::new("model")
						.short('d')
						.long("model-file")
						.value_name("MODEL")
						.help("Sets the directory containing model folders")
						.required(true),
				)
				.arg(
					Arg::new("trace")
						.short('t')
						.long("trace-file")
						.value_name("TRACE")
						.help("File containing white-space separated transition names for seed traces")
						.required(true),
				)
				.arg(
					Arg::new("output_file")
						.short('o')
						.long("output-file")
						.value_name("OUTPUT")
						.help("File to write the output to WITHOUT A FILE EXTENSION")
						.default_value("cycle_commute"),
				)
		)
		.subcommand(
			Command::new("stamina")
				.about("Run the stamina tool")
				.arg(
					Arg::new("models_dir")
						.required(true)
						.short('d')
						.long("models-dir")
						.value_name("DIR")
						.help("Sets the directory containing model folders")
						.default_value("models"),
				)
				.arg(
					Arg::new("timeout")
						.short('t')
						.long("timeout")
						.value_name("MINUTES")
						.help("Timeout in minutes for get_bounds")
						.default_value(TIMEOUT_MINUTES),
				)
		)
		.subcommand(
			Command::new("wayfarer")
				.about("Run the wayfarer tool")
				.arg(
					Arg::new("models_dir")
						.required(true)
						.short('d')
						.long("models-dir")
						.value_name("DIR")
						.help("Sets the directory containing model folders")
						.default_value("models"),
				)
				.arg(
					Arg::new("timeout")
						.short('t')
						.long("timeout")
						.value_name("MINUTES")
						.help("Timeout in minutes for get_bounds")
						.default_value(TIMEOUT_MINUTES),
				)
		)
		.get_matches();

	match matches.subcommand() {
		Some(("bounds", sub_m)) => {
			let models_dir = sub_m.get_one::<String>("models_dir").unwrap();
			let timeout = sub_m.get_one::<String>("timeout").unwrap();
			println!("Running ragtimer with models_dir: {} and timeout: {}", models_dir, timeout);
			let dir_path = Path::new(models_dir);
			demos::bmc_demo::bmc_demo(dir_path, timeout.parse::<u64>().unwrap());
		}
		Some(("dependency-graph", sub_m)) => {
			let model_file = sub_m.get_one::<String>("model").unwrap();
			println!("Running ragtimer with models: {}", model_file);
			
			let parsed_model = AbstractVas::from_file(model_file);
			if !parsed_model.is_ok() {
				println!("Error parsing model file: {}", model_file);
				return;
			}
			let parsed_model = parsed_model.unwrap();
			println!("MODEL PARSED\n\n");
			println!("{}", parsed_model.nice_print());
			let dg = make_dependency_graph(&parsed_model);
			if let Ok(Some(dependency_graph)) = &dg {
				dependency_graph.pretty_print(&parsed_model);
				dependency_graph.simple_print(&parsed_model);
				// let trimmed_model = dependency::trimmer::trim_model(parsed_model.clone(), dependency_graph.clone());
				// println!("{}", trimmed_model.nice_print());
			} else {
				println!("Error creating dependency graph.");
			}
		}
		Some(("ragtimer", sub_m)) => {
			let models_dir = sub_m.get_one::<String>("models_dir").unwrap();
			let timeout = sub_m.get_one::<String>("timeout").unwrap();
			println!("Running ragtimer with models_dir: {} and timeout: {}", models_dir, timeout);
			println!("Ragtimer is not yet implemented in Practice.");
			unimplemented!();
		}
		Some(("cycle-commute", sub_m)) => {
			let model = sub_m.get_one::<String>("model").unwrap();
			let trace = sub_m.get_one::<String>("trace").unwrap();
			let output_file = sub_m.get_one::<String>("output_file").unwrap();
			println!("Running cycle-commute with model: {} and trace: {}", model, trace);
			demos::cycle_commute_demo::cycle_commute_demo(model, trace, output_file);
		}
		Some(("stamina", sub_m)) => {
			let models_dir = sub_m.get_one::<String>("models_dir").unwrap();
			let timeout = sub_m.get_one::<String>("timeout").unwrap();
			println!("Running stamina with models_dir: {} and timeout: {}", models_dir, timeout);
			// Call your stamina logic here
		}
		Some(("wayfarer", sub_m)) => {
			let models_dir = sub_m.get_one::<String>("models_dir").unwrap();
			let timeout = sub_m.get_one::<String>("timeout").unwrap();
			println!("Running wayfarer with models_dir: {} and timeout: {}", models_dir, timeout);
			// Call your wayfarer logic here
		}
		_ => {
			println!("No valid subcommand was used. Use --help for more information.");
		}
	}


	// if let Some(matches) = matches.subcommand_matches("ragtimer") {
}
