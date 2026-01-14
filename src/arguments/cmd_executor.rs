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
			let model = sub_m.get_one::<String>("model").unwrap();
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
			let approach = sub_m.get_one::<String>("approach").unwrap();
			let output = sub_m
				.get_one::<String>("output")
				.map(|s| s.to_string())
				.unwrap_or_else(|| DEFAULT_BENCHMARK_OUTPUT.to_string());
			let _timeout = sub_m
				.get_one::<String>("timeout")
				.and_then(|s| s.parse::<usize>().ok())
				.unwrap_or(DEFAULT_TIMEOUT_SECONDS.parse::<usize>().unwrap());
			match approach.as_str() {
				"RL" => {
					message!("Ragtimer with Reinforcement Learning");
					let mut magic_numbers = default_magic_numbers();
					magic_numbers.num_traces = num_traces;
					ragtimer_benchmark(
						model,
						cycle_length,
						commute_depth,
						RagtimerApproach::ReinforcementLearning(magic_numbers),
						&output,
					);
				}
				"random" => {
					message!("Ragtimer with Random approach is not yet implemented.");
					unimplemented!();
				}
				"shortest" => {
					message!("Ragtimer with Random Dependency Graph path approach");
					ragtimer_benchmark(
						model,
						cycle_length,
						commute_depth,
						RagtimerApproach::RandomDependencyGraph(num_traces),
						&output,
					);
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
					message!("Ragtimer with Shortest path approach");
					ragtimer(
						model,
						RagtimerApproach::RandomDependencyGraph(num_traces),
						cycle_length,
						commute_depth,
						output,
					);
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
}
