#![allow(dead_code)]

mod bmc;
mod cycle_commute;
mod demos;
mod dependency;
mod logging;
mod model;
mod parser;
mod property;
// mod ragtimer;
mod builder;
mod trace;
mod util;
mod validator;

use clap::{Arg, Command};
use dependency::graph::make_dependency_graph;
use model::vas_model::AbstractVas;
use std::path::Path;

use crate::{
	builder::{builder::Builder, ragtimer::ragtimer::RagtimerBuilder},
	model::vas_model::PrismVasModel,
};

// use crate::ragtimer::rl_traces::print_traces_to_file;
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
					Arg::new("bits")
						.short('b')
						.long("bits")
						.value_name("BITS")
						.help("Sets the number of bits for variable representation (default 9)")
						.default_value("9"),
				)
				.arg(
					Arg::new("max_steps")
						.short('m')
						.long("max-steps")
						.value_name("MAX_STEPS")
						.help("Sets the maximum number of BMC steps to take (default 500)")
						.default_value("500"),
				)
				.arg(
					Arg::new("backward")
						.long("backward")
						.help("Run in backward mode if specified")
						.action(clap::ArgAction::SetTrue),
				),
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
				.about("Run the ragtimer tool (currently including only the RL Traces tool)")
				.arg(
					Arg::new("model")
						.short('d')
						.long("model")
						.value_name("MODEL")
						.help("Sets the model file (crn format)")
						.required(true),
				)
				.arg(
					Arg::new("qty")
						.short('q')
						.long("qty")
						.value_name("QTY")
						.help("Sets the number of traces to generate (default 100)")
						.default_value("100"),
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
						.help("Sets the model file (crn format)")
						.required(true),
				)
				// .arg(
				// 	Arg::new("trace")
				// 		.short('t')
				// 		.long("trace-file")
				// 		.value_name("TRACE")
				// 		.help("File containing white-space separated transition names for seed traces")
				// 		.required(true),
				// )
				.arg(
					Arg::new("output_file")
						.short('o')
						.long("output-file")
						.value_name("OUTPUT")
						.help("File to write the output to WITHOUT A FILE EXTENSION")
						.default_value("cycle_commute.txt"),
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
			message!("Running ragtimer with models_dir: {}", models_dir);
			let dir_path = Path::new(models_dir);
			let backward = sub_m.get_flag("backward");
			let bits = sub_m
				.get_one::<String>("bits")
				.and_then(|s| s.parse::<u32>().ok())
				.unwrap();
			let max_steps = sub_m
				.get_one::<String>("max_steps")
				.and_then(|s| s.parse::<u32>().ok())
				.unwrap();
			message!("Bits: {}, Max Steps: {}", bits, max_steps);
			demos::bmc_demo::bmc_demo(dir_path, bits, max_steps, backward);
		}
		Some(("dependency-graph", sub_m)) => {
			// TODO: Move this whole thing to a demo
			let model_file = sub_m.get_one::<String>("model").unwrap();
			message!("Running ragtimer with models: {}", model_file);
			let parsed_model = AbstractVas::from_file(model_file);
			if !parsed_model.is_ok() {
				error!("Error parsing model file: {}", model_file);
				return;
			}
			let parsed_model = parsed_model.unwrap();
			message!("MODEL PARSED\n\n");
			message!("{}", parsed_model.nice_print());
			let dg = make_dependency_graph(&parsed_model);
			if let Ok(Some(dependency_graph)) = &dg {
				dependency_graph.pretty_print(&parsed_model);
				dependency_graph.simple_print(&parsed_model);
				dependency_graph.original_print(&parsed_model);
			} else {
				error!("Error creating dependency graph.");
			}
		}
		Some(("ragtimer", sub_m)) => {
			message!("Ragtimer under development...");
			let _num_traces = sub_m
				.get_one::<String>("qty")
				.and_then(|s| s.parse::<usize>().ok())
				.unwrap();
			let model_file = sub_m.get_one::<String>("model").unwrap();
			message!("Running ragtimer with models: {}", model_file);
			let parsed_model = AbstractVas::from_file(model_file);
			if !parsed_model.is_ok() {
				error!("Error parsing model file: {}", model_file);
				return;
			}
			let parsed_model = parsed_model.unwrap();
			message!("MODEL PARSED\n\n");
			message!("{}", parsed_model.nice_print());
			let dg = make_dependency_graph(&parsed_model);
			if let Ok(Some(dependency_graph)) = &dg {
				dependency_graph.pretty_print(&parsed_model);
				let mut explicit_model = PrismVasModel::from_abstract_model(&parsed_model);
				let mut ragtimer_builder = RagtimerBuilder::new(&parsed_model, None);
				ragtimer_builder.build(&mut explicit_model);
			} else {
				error!("Error creating dependency graph.");
				return;
			}
		}
		Some(("cycle-commute", sub_m)) => {
			let model = sub_m.get_one::<String>("model").unwrap();
			// let trace = sub_m.get_one::<String>("trace").unwrap();
			let output_file = sub_m.get_one::<String>("output_file").unwrap();
			message!(
				"Running cycle-commute demo with model: {}",
				model,
				// trace
			);
			demos::cycle_commute_demo::cycle_commute_demo(model, output_file);
		}
		Some(("stamina", sub_m)) => {
			let models_dir = sub_m.get_one::<String>("models_dir").unwrap();
			let timeout = sub_m.get_one::<String>("timeout").unwrap();
			message!(
				"Running stamina with models_dir: {} and timeout: {}",
				models_dir,
				timeout
			);
			unimplemented!();
		}
		Some(("wayfarer", sub_m)) => {
			let models_dir = sub_m.get_one::<String>("models_dir").unwrap();
			let timeout = sub_m.get_one::<String>("timeout").unwrap();
			message!(
				"Running wayfarer with models_dir: {} and timeout: {}",
				models_dir,
				timeout
			);
			unimplemented!();
		}
		_ => {
			error!("No valid subcommand was used. Use --help for more information.");
		}
	}
}
