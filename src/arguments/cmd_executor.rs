use std::path::Path;

use crate::{
	builder::{builder::Builder, ragtimer::ragtimer::RagtimerBuilder},
	demos,
	dependency::graph::make_dependency_graph,
	logging::messages::*,
	model::vas_model::{AbstractVas, PrismVasModel},
};

pub fn run_commands(matches: &clap::ArgMatches) {
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
		Some(("cycle-commute-benchmark", sub_m)) => {
			let models_dir = sub_m.get_one::<String>("models_dir").unwrap();
			message!("Running ragtimer with models_dir: {}", models_dir);
			let dir_path = Path::new(models_dir);
			let (min_commute_depth, max_commute_depth, min_cycle_length, max_cycle_length) =
				if sub_m.get_flag("default") {
					// Set recommended default values
					(0, 12, 0, 8)
				} else {
					let min_commute_depth = sub_m
						.get_one::<String>("min_commute_depth")
						.and_then(|s| s.parse::<usize>().ok())
						.unwrap();
					let max_commute_depth = sub_m
						.get_one::<String>("max_commute_depth")
						.and_then(|s| s.parse::<usize>().ok())
						.unwrap();
					let min_cycle_length = sub_m
						.get_one::<String>("min_cycle_length")
						.and_then(|s| s.parse::<usize>().ok())
						.unwrap();
					let max_cycle_length = sub_m
						.get_one::<String>("max_cycle_length")
						.and_then(|s| s.parse::<usize>().ok())
						.unwrap();
					(
						min_commute_depth,
						max_commute_depth,
						min_cycle_length,
						max_cycle_length,
					)
				};
			message!(
				"Max Commute Depth: {}, Max Cycle Length: {}",
				max_commute_depth,
				max_cycle_length
			);
			demos::cycle_commute_benchmark::cycle_commute_benchmark(
				dir_path,
				min_commute_depth,
				max_commute_depth,
				min_cycle_length,
				max_cycle_length,
			);
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
			let max_commute_depth = sub_m
				.get_one::<String>("max_commute_depth")
				.and_then(|s| s.parse::<usize>().ok())
				.unwrap();
			let max_cycle_length = sub_m
				.get_one::<String>("max_cycle_length")
				.and_then(|s| s.parse::<usize>().ok())
				.unwrap();
			message!(
				"Running cycle-commute demo with model: {}",
				model,
				// trace
			);
			demos::cycle_commute_demo::cycle_commute_demo(
				model,
				output_file,
				max_commute_depth,
				max_cycle_length,
			);
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
