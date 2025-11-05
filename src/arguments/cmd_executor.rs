use crate::{
	arguments::default_args::*,
	benchmarks::bench_ragtimer::ragtimer_benchmark,
	bmc::{bounds::bound_model, encoding::unroll_model},
	builder::ragtimer::{
		ragtimer::{ragtimer, RagtimerApproach},
		rl_traces::default_magic_numbers,
	},
	dependency::graph::make_dependency_graph,
	logging::messages::*,
	model::vas_model::AbstractVas,
};

pub fn run_commands(args: &clap::ArgMatches) {
	match args.subcommand() {
		// Benchmark set
		Some(("benchmark", sub_m)) => {
			// let model = sub_m.get_one::<String>("model");
			let dir = sub_m.get_one::<String>("dir");
			let num_traces = sub_m
				.get_one::<String>("num-traces")
				.and_then(|s| s.parse::<usize>().ok())
				.unwrap_or(DEFAULT_NUM_TRACES.parse::<usize>().unwrap());
			let cycle_length = sub_m
				.get_one::<String>("cycle-length")
				.and_then(|s| s.parse::<usize>().ok())
				.unwrap_or(DEFAULT_CYCLE_LENGTH.parse::<usize>().unwrap());
			let commute_depth = sub_m
				.get_one::<String>("commute-depth")
				.and_then(|s| s.parse::<usize>().ok())
				.unwrap_or(DEFAULT_COMMUTE_DEPTH.parse::<usize>().unwrap());
			let _timeout = sub_m
				.get_one::<String>("timeout")
				.and_then(|s| s.parse::<usize>().ok())
				.unwrap_or(DEFAULT_TIMEOUT_SECONDS.parse::<usize>().unwrap());

			ragtimer_benchmark(
				dir.unwrap().as_ref(),
				num_traces,
				0,
				commute_depth,
				0,
				cycle_length,
			);

			// The following code used to allow either a single model file or a directory of models.
			// For now, we requires a directory of models, but I left the old code commented out to
			// allow updating benchmarks in the future.
			// let model_files: Vec<String> = if let Some(dir_path) = dir {
			// 	let path = Path::new(dir_path);
			// 	if !path.exists() || !path.is_dir() {
			// 		error!(
			// 			"Specified directory does not exist or is not a directory: {}",
			// 			dir_path
			// 		);
			// 		return;
			// 	}
			// 	let mut files = Vec::new();
			// 	for entry in walkdir::WalkDir::new(path)
			// 		.into_iter()
			// 		.filter_map(|e| e.ok())
			// 		.filter(|e| e.file_type().is_file())
			// 	{
			// 		let file_name = entry.file_name().to_string_lossy();
			// 		if file_name.ends_with(".crn") || file_name.ends_with(".vas") {
			// 			files.push(entry.path().to_string_lossy().to_string());
			// 		}
			// 	}
			// 	files
			// // } else if let Some(model_file) = model {
			// // 	vec![model_file.clone()]
			// } else {
			// 	error!("Either --model or --dir must be specified for benchmarking.");
			// 	return;
			// };

			// for model_file in model_files {
			// 	message!(
			// 		"Benchmarking Model: {}, Traces: {}, Cycle Length: {}, Commute Depth: {}, Timeout: {}s",
			// 		model_file, num_traces, cycle_length, commute_depth, timeout
			// 	);
			// 	// unimplemented!();
			// 	// Call the benchmarking function for each model_file here
			// }
		}
		Some(("bmc", sub_m)) => {
			let model_file = sub_m.get_one::<String>("model").unwrap();
			let steps = sub_m
				.get_one::<String>("steps")
				.and_then(|s| s.parse::<u32>().ok())
				.unwrap();
			let bits = sub_m
				.get_one::<String>("bits")
				.and_then(|s| s.parse::<u32>().ok())
				.unwrap();
			let output = sub_m
				.get_one::<String>("output")
				.map(|s| s.to_string())
				.unwrap_or_else(|| format!("{}.smt2", model_file));
			let check = sub_m.get_flag("check");
			let timeout = sub_m
				.get_one::<String>("timeout")
				.and_then(|s| s.parse::<usize>().ok())
				.unwrap_or(DEFAULT_TIMEOUT_SECONDS.parse::<usize>().unwrap());
			message!(
				"Preparing BMC Unrolled Encoding on model: {}, Steps: {}, Bits: {}, Output: {}, Timeout: {}s",
				model_file,
				steps,
				bits,
				output,
				timeout
			);
			unroll_model(model_file, steps, bits, &output, check);
		}
		Some(("bounds", sub_m)) => {
			let model_file = sub_m.get_one::<String>("model").unwrap();
			let bits = sub_m
				.get_one::<String>("bits")
				.and_then(|s| s.parse::<u32>().ok())
				.unwrap();
			let max_steps = sub_m
				.get_one::<String>("max-steps")
				.and_then(|s| s.parse::<u32>().ok())
				.unwrap_or(DEFAULT_BOUNDER_STEPS.parse::<u32>().unwrap());
			let trim = sub_m.get_flag("trim");
			let timeout = sub_m
				.get_one::<String>("timeout")
				.and_then(|s| s.parse::<usize>().ok())
				.unwrap_or(DEFAULT_TIMEOUT_SECONDS.parse::<usize>().unwrap());
			message!(
				"Running bounds checking on model: {}, Bits: {}, Max Steps: {}, Timeout: {}s",
				model_file,
				bits,
				max_steps,
				timeout
			);
			bound_model(model_file, bits, max_steps, trim);
		}
		Some(("cycle-commute", sub_m)) => {
			let model = sub_m.get_one::<String>("model").unwrap();
			let trace = sub_m.get_one::<String>("trace").unwrap();
			let cycle_length = sub_m
				.get_one::<String>("max-cycle-length")
				.and_then(|s| s.parse::<usize>().ok())
				.unwrap();
			let commute_depth = sub_m
				.get_one::<String>("max-commute-depth")
				.and_then(|s| s.parse::<usize>().ok())
				.unwrap();
			let output = sub_m.get_one::<String>("output").unwrap();
			let timeout = sub_m
				.get_one::<String>("timeout")
				.and_then(|s| s.parse::<usize>().ok())
				.unwrap_or(DEFAULT_TIMEOUT_SECONDS.parse::<usize>().unwrap());
			message!(
				"Running Cycle & Commute on model: {}, Trace: {}, Max Cycle Length: {}, Max Commute Depth: {}, Output: {}, Timeout: {}s",
				model, trace, cycle_length, commute_depth, output, timeout
			);
			error!("Cycle & Commute is not yet implemented for reading traces from files.");
			unimplemented!();
			// Run cycle-commute here
		}
		Some(("dependency-graph", sub_m)) => {
			let model_file = sub_m.get_one::<String>("model").unwrap();
			let output = sub_m.get_one::<String>("output").unwrap();
			let timeout = sub_m
				.get_one::<String>("timeout")
				.and_then(|s| s.parse::<usize>().ok())
				.unwrap_or(DEFAULT_TIMEOUT_SECONDS.parse::<usize>().unwrap());
			message!(
				"Generating Dependency Graph for model: {}, Output: {}, Timeout: {}s",
				model_file,
				output,
				timeout
			);
			// TODO: Put this into its own function in the dependency module
			if let Ok(model) = AbstractVas::from_file(model_file) {
				message!("Successfully parsed model file: {}", model_file);
				// Generate and display the dependency graph
				let dependency_graph = match make_dependency_graph(&model) {
					Ok(Some(dg)) => dg,
					Ok(None) => {
						error!(
							"Failed to create dependency graph for model: {}",
							model_file
						);
						return;
					}
					Err(e) => {
						error!(
							"Error creating dependency graph for model: {}: {}",
							model_file, e
						);
						return;
					}
				};
				message!("Dependency graph created for model: {}", model_file);
				message!("Constructed the following dependency graph:");
				dependency_graph.simple_print(&model);
				// Write the dependency graph to the specified output file
				let output_file = format!("{}.dependencygraph.txt", output);
				let original_style_output = dependency_graph.original_print(&model);
				if let Err(e) = std::fs::write(&output_file, original_style_output) {
					error!(
						"Error writing dependency graph to file {}: {}",
						output_file, e
					);
				} else {
					message!("Dependency graph written to file: {}", output_file);
				}
			} else {
				error!("Error parsing model file: {}", model_file);
			}
		}
		Some(("ragtimer", sub_m)) => {
			let model = sub_m.get_one::<String>("model").unwrap();
			let approach: &String = sub_m.get_one::<String>("approach").unwrap();
			let output = sub_m.get_one::<String>("output").unwrap();
			let num_traces = sub_m
				.get_one::<String>("num-traces")
				.and_then(|s| s.parse::<usize>().ok())
				.unwrap_or(DEFAULT_NUM_TRACES.parse::<usize>().unwrap());
			let cycle_length = sub_m
				.get_one::<String>("cycle-length")
				.and_then(|s| s.parse::<usize>().ok())
				.unwrap_or(DEFAULT_CYCLE_LENGTH.parse::<usize>().unwrap());
			let commute_depth = sub_m
				.get_one::<String>("commute-depth")
				.and_then(|s| s.parse::<usize>().ok())
				.unwrap_or(DEFAULT_COMMUTE_DEPTH.parse::<usize>().unwrap());
			let timeout = sub_m
				.get_one::<String>("timeout")
				.and_then(|s| s.parse::<usize>().ok())
				.unwrap_or(DEFAULT_TIMEOUT_SECONDS.parse::<usize>().unwrap());
			message!(
				"Running Ragtimer on model: {}, Approach: {}, Traces: {}, Cycle Length: {}, Commute Depth: {}, Timeout: {}s",
				model, approach, num_traces, cycle_length, commute_depth, timeout
			);
			// Run ragtimer based on approach
			match approach.as_str() {
				"RL" => {
					message!("Ragtimer with Reinforcement Learning");
					let mut magic_numbers = default_magic_numbers();
					magic_numbers.num_traces = num_traces;
					ragtimer(
						model,
						RagtimerApproach::ReinforcementLearning(magic_numbers),
						cycle_length,
						commute_depth,
						output,
					);
				}
				"random" => {
					message!("Ragtimer with Random approach is not yet implemented.");
					unimplemented!();
				}
				"shortest" => {
					message!("Ragtimer with Shortest approach is not yet implemented.");
					unimplemented!();
				}
				_ => {
					error!(
						"Invalid approach: {}. Must be one of: RL, random, shortest.",
						approach
					);
					return;
				}
			}
		}
		Some(("stamina", _sub_m)) => {
			error!("Stamina is not yet implemented.");
			// Implement Stamina functionality here
		}
		Some(("wayfarer", _sub_m)) => {
			error!("Wayfarer is not yet implemented.");
			// Implement Wayfarer functionality here
		}
		_ => {
			error!("No valid subcommand was used. Use --help for more information.");
		}
	}

	// // Old Stuff
	// match args.subcommand() {

	// 	Some(("cycle-commute-benchmark", sub_m)) => {
	// 		let models_dir = sub_m.get_one::<String>("models_dir").unwrap();
	// 		message!("Running ragtimer with models_dir: {}", models_dir);
	// 		let dir_path = Path::new(models_dir);
	// 		let (min_commute_depth, max_commute_depth, min_cycle_length, max_cycle_length) =
	// 			if sub_m.get_flag("default") {
	// 				// Set recommended default values
	// 				(0, 12, 0, 8)
	// 			} else {
	// 				let min_commute_depth = sub_m
	// 					.get_one::<String>("min_commute_depth")
	// 					.and_then(|s| s.parse::<usize>().ok())
	// 					.unwrap();
	// 				let max_commute_depth = sub_m
	// 					.get_one::<String>("max_commute_depth")
	// 					.and_then(|s| s.parse::<usize>().ok())
	// 					.unwrap();
	// 				let min_cycle_length = sub_m
	// 					.get_one::<String>("min_cycle_length")
	// 					.and_then(|s| s.parse::<usize>().ok())
	// 					.unwrap();
	// 				let max_cycle_length = sub_m
	// 					.get_one::<String>("max_cycle_length")
	// 					.and_then(|s| s.parse::<usize>().ok())
	// 					.unwrap();
	// 				(
	// 					min_commute_depth,
	// 					max_commute_depth,
	// 					min_cycle_length,
	// 					max_cycle_length,
	// 				)
	// 			};
	// 		message!(
	// 			"Max Commute Depth: {}, Max Cycle Length: {}",
	// 			max_commute_depth,
	// 			max_cycle_length
	// 		);
	// 		demos::cycle_commute_benchmark::cycle_commute_benchmark(
	// 			dir_path,
	// 			min_commute_depth,
	// 			max_commute_depth,
	// 			min_cycle_length,
	// 			max_cycle_length,
	// 		);
	// 	}
	// 	Some(("dependency-graph", sub_m)) => {
	// 		// TODO: Move this whole thing to a demo
	// 		let model_file = sub_m.get_one::<String>("model").unwrap();
	// 		message!("Running ragtimer with models: {}", model_file);
	// 		let parsed_model = AbstractVas::from_file(model_file);
	// 		if !parsed_model.is_ok() {
	// 			error!("Error parsing model file: {}", model_file);
	// 			return;
	// 		}
	// 		let parsed_model = parsed_model.unwrap();
	// 		message!("MODEL PARSED\n\n");
	// 		message!("{}", parsed_model.nice_print());
	// 		let dg = make_dependency_graph(&parsed_model);
	// 		if let Ok(Some(dependency_graph)) = &dg {
	// 			dependency_graph.pretty_print(&parsed_model);
	// 			dependency_graph.simple_print(&parsed_model);
	// 			dependency_graph.original_print(&parsed_model);
	// 		} else {
	// 			error!("Error creating dependency graph.");
	// 		}
	// 	}
	// 	Some(("ragtimer", sub_m)) => {
	// 		message!("Ragtimer under development...");
	// 		let _num_traces = sub_m
	// 			.get_one::<String>("qty")
	// 			.and_then(|s| s.parse::<usize>().ok())
	// 			.unwrap();
	// 		let model_file = sub_m.get_one::<String>("model").unwrap();
	// 		message!("Running ragtimer with models: {}", model_file);
	// 		let parsed_model = AbstractVas::from_file(model_file);
	// 		if !parsed_model.is_ok() {
	// 			error!("Error parsing model file: {}", model_file);
	// 			return;
	// 		}
	// 		let parsed_model = parsed_model.unwrap();
	// 		message!("MODEL PARSED\n\n");
	// 		message!("{}", parsed_model.nice_print());
	// 		let dg = make_dependency_graph(&parsed_model);
	// 		if let Ok(Some(dependency_graph)) = &dg {
	// 			dependency_graph.pretty_print(&parsed_model);
	// 			let mut explicit_model = PrismVasModel::from_abstract_model(&parsed_model);
	// 			let mut ragtimer_builder = RagtimerBuilder::new(&parsed_model, None);
	// 			ragtimer_builder.build(&mut explicit_model);
	// 		} else {
	// 			error!("Error creating dependency graph.");
	// 			return;
	// 		}
	// 	}
	// 	Some(("cycle-commute", sub_m)) => {
	// 		let model = sub_m.get_one::<String>("model").unwrap();
	// 		// let trace = sub_m.get_one::<String>("trace").unwrap();
	// 		let output_file = sub_m.get_one::<String>("output_file").unwrap();
	// 		let max_commute_depth = sub_m
	// 			.get_one::<String>("max_commute_depth")
	// 			.and_then(|s| s.parse::<usize>().ok())
	// 			.unwrap();
	// 		let max_cycle_length = sub_m
	// 			.get_one::<String>("max_cycle_length")
	// 			.and_then(|s| s.parse::<usize>().ok())
	// 			.unwrap();
	// 		message!(
	// 			"Running cycle-commute demo with model: {}",
	// 			model,
	// 			// trace
	// 		);
	// 		demos::cycle_commute_demo::cycle_commute_demo(
	// 			model,
	// 			output_file,
	// 			max_commute_depth,
	// 			max_cycle_length,
	// 		);
	// 	}
	// 	Some(("stamina", sub_m)) => {
	// 		let models_dir = sub_m.get_one::<String>("models_dir").unwrap();
	// 		let timeout = sub_m.get_one::<String>("timeout").unwrap();
	// 		message!(
	// 			"Running stamina with models_dir: {} and timeout: {}",
	// 			models_dir,
	// 			timeout
	// 		);
	// 		unimplemented!();
	// 	}
	// 	Some(("wayfarer", sub_m)) => {
	// 		let models_dir = sub_m.get_one::<String>("models_dir").unwrap();
	// 		let timeout = sub_m.get_one::<String>("timeout").unwrap();
	// 		message!(
	// 			"Running wayfarer with models_dir: {} and timeout: {}",
	// 			models_dir,
	// 			timeout
	// 		);
	// 		unimplemented!();
	// 	}
	// 	_ => {
	// 		error!("No valid subcommand was used. Use --help for more information.");
	// 	}
	// }
}
