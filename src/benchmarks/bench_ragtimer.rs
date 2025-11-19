use crate::builder::builder::Builder;
use crate::builder::ragtimer::ragtimer::{MagicNumbers, RagtimerApproach, RagtimerBuilder};
use crate::builder::ragtimer::rl_traces::default_magic_numbers;
use crate::model::vas_model::AbstractVas;
use crate::*;
use crate::{cycle_commute::commute::cycle_commute, model::vas_model::PrismVasModel};
use chrono::Local;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::time::Instant;
use sysinfo::{ProcessesToUpdate, System};

const OUTPUT_WORD: &str = "model";

/// Gets the list of .crn files in the specified directory
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
					// let model_name = model_path.file_stem().unwrap().to_str().unwrap();
					// let folder_name = path.file_name().unwrap().to_str().unwrap();
					let model_path_str = model_path.to_string_lossy().into_owned();
					crn_files.push(model_path_str);
				}
			}
		}
	}
	crn_files
}

/// This function runs the cycle commute demo for a given model and trace file.
/// It reads the models from the specified directory and creates a spreadsheet of results
/// plus a bash script to run PRISM on each generated model.
pub fn ragtimer_benchmark(
	model_dir: &Path,
	num_traces: usize,
	min_commute_depth: usize,
	max_commute_depth: usize,
	min_cycle_length: usize,
	max_cycle_length: usize,
) {
	// Collect all .crn files in the directory and its subdirectories
	let crn_files: Vec<String> = get_crn_files(model_dir);
	let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();

	let csv_path = format!("output/{}/benchmark_results.csv", timestamp);

	fs::create_dir_all(format!("output/{}", timestamp)).expect("Failed to create output directory");
	message!("CSV output created at output/{}", timestamp);
	let mut csv_file = OpenOptions::new()
		.create(true)
		.write(true)
		.append(true)
		.open(&csv_path)
		.expect("Failed to open CSV file");
	writeln!(
		csv_file,
		"model,commute_depth,cycle_length,time_ms_total,time_ms_ragtimer,time_ms_cc,bytes_total,bytes_ragtimer,bytes_cc,states_total,states_ragtimer,states_cc,output_file"
	).expect("Failed to write CSV header");

	let mut bash_file = OpenOptions::new()
		.create(true)
		.write(true)
		.truncate(true)
		.open(format!("output/{}/run_prism.sh", timestamp))
		.expect("Failed to open bash script file");

	// Declare most of the variables outside the loop to avoid stack overflows
	let mut explicit_model: PrismVasModel;
	let mut output_dir: String;
	let mut output_file: String;
	let mut bash_dir: String;
	let mut prop_dst: String;
	let mut sys = System::new();
	let current_pid = sysinfo::get_current_pid().unwrap();
	let mut start_memory: u64;
	let mut start_time: Instant;
	let mut ragtimer_start_memory: u64;
	let mut ragtimer_start_time: Instant;
	let mut ragtimer_end_memory: u64;
	let mut ragtimer_memory_usage: u64;
	let mut build_elapsed: u128;
	let mut ragtimer_state_count: usize;
	let mut approach_used: RagtimerApproach;
	let mut magic_numbers: MagicNumbers;
	let mut ragtimer_builder: RagtimerBuilder;
	let mut cycle_start_memory: u64;
	let mut cycle_start_time: Instant;
	let mut cycle_end_memory: u64;
	let mut cycle_memory_usage: u64;
	let mut cycle_elapsed: u128;
	let mut cycle_state_count: usize;
	let mut total_end_memory: u64;
	let mut total_memory_usage: u64;
	let mut total_elapsed: u128;
	let mut total_state_count: usize;
	let mut bash_command: String;
	let mut time_mem_file_path: String;
	let mut time_mem_file: std::fs::File;

	for model_file in crn_files {
		let model_name = Path::new(&model_file)
			.file_name()
			.and_then(|s| s.to_str())
			.unwrap_or("model")
			.replace(".", "_");
		let prop_src = Path::new(&model_file).with_extension("prop");

		if let Ok(mut abstract_model) = AbstractVas::from_file(&model_file) {
			// for approach in &[
			// 	RagtimerApproach::ReinforcementLearning(default_magic_numbers()),
			// 	// RagtimerApproach::RandomPathExploration,
			// 	RagtimerApproach::RandomDependencyGraph(num_traces),
			// ] {
			// 	let approach_word = match approach {
			// 		RagtimerApproach::ReinforcementLearning(_) => "rl",
			// 		RagtimerApproach::RandomPathExploration => "rpe",
			// 		RagtimerApproach::RandomDependencyGraph(_) => "rdg",
			// 	};
			// let approach = &RagtimerApproach::RandomDependencyGraph(num_traces);
			// let approach_word = "rdg";
			let approach = &RagtimerApproach::ReinforcementLearning(default_magic_numbers());
			let approach_word = "rl";
			message!(
				"Starting benchmarks for model: {} with approach: {:?}",
				model_name,
				approach_word
			);
			for cycle_length in min_cycle_length..=max_cycle_length {
				for commute_depth in min_commute_depth..=max_commute_depth {
					message!("{}", "━".repeat(80));
					message!("Running Ragtimer + C&C benchmark.");
					message!("model: {}", model_file);
					message!("commute depth: {}", commute_depth);
					message!("cycle length: {}", cycle_length);
					message!("{}", "━".repeat(80));
					message!("Model {} Parsed", model_name);
					output_dir = format!(
						"output/{}/{}/{}/cycle_{}/commute_{}/",
						timestamp, model_name, approach_word, cycle_length, commute_depth
					);
					fs::create_dir_all(&output_dir).expect("Failed to create output directory");
					output_file = Path::new(&output_dir)
						.join(OUTPUT_WORD)
						.to_string_lossy()
						.into_owned();
					bash_dir = format!(
						"{}/{}/cycle_{}/commute_{}/",
						model_name, approach_word, cycle_length, commute_depth
					);
					prop_dst = format!("{}.prop", output_file);
					if prop_src.exists() {
						fs::copy(&prop_src, &prop_dst).expect("Failed to copy .prop file");
						message!("Copied property file to {}", prop_dst);
					}

					explicit_model = PrismVasModel::from_abstract_model(&abstract_model);
					message!("Explicit Model Built");

					// Set start time and memory usage
					sys.refresh_processes(ProcessesToUpdate::Some(&[current_pid]), true);
					start_memory = sys
						.process(sysinfo::get_current_pid().unwrap())
						.map(|p| p.memory())
						.unwrap_or(0);
					start_time = Instant::now();

					// Time Ragtimer state space building
					ragtimer_start_time = Instant::now();
					sys.refresh_processes(ProcessesToUpdate::Some(&[current_pid]), true);
					ragtimer_start_memory = sys
						.process(sysinfo::get_current_pid().unwrap())
						.map(|p| p.memory())
						.unwrap_or(0);
					approach_used = (*approach).clone();
					if let RagtimerApproach::ReinforcementLearning(_) = *approach {
						magic_numbers = default_magic_numbers();
						magic_numbers.num_traces = num_traces;
						approach_used = RagtimerApproach::ReinforcementLearning(magic_numbers);
					}
					ragtimer_builder =
						RagtimerBuilder::new(&abstract_model, Some(approach_used.clone()));
					ragtimer_builder.build(&mut explicit_model);

					ragtimer_state_count = explicit_model.states.len();
					build_elapsed = ragtimer_start_time.elapsed().as_millis();
					sys.refresh_processes(ProcessesToUpdate::Some(&[current_pid]), true);
					ragtimer_end_memory = sys
						.process(sysinfo::get_current_pid().unwrap())
						.map(|p| p.memory())
						.unwrap_or(0);
					ragtimer_memory_usage = ragtimer_end_memory - ragtimer_start_memory;
					message!(
						"Traces added to explicit model with Ragtimer ({} ms)",
						build_elapsed
					);
					message!(
						"Ragtimer-specific memory usage: {:.3e} B",
						ragtimer_memory_usage as f64
					);
					drop(ragtimer_builder);

					// Time cycle and commute
					cycle_start_time = Instant::now();
					sys.refresh_processes(ProcessesToUpdate::Some(&[current_pid]), true);
					cycle_start_memory = sys
						.process(sysinfo::get_current_pid().unwrap())
						.map(|p| p.memory())
						.unwrap_or(0);
					cycle_commute(
						&mut abstract_model,
						&mut explicit_model,
						commute_depth,
						cycle_length,
					);
					// explicit_model.add_absorbing_transitions();
					cycle_state_count = explicit_model.states.len() - ragtimer_state_count;
					cycle_elapsed = cycle_start_time.elapsed().as_millis();
					sys.refresh_processes(ProcessesToUpdate::Some(&[current_pid]), true);
					cycle_end_memory = sys
						.process(sysinfo::get_current_pid().unwrap())
						.map(|p| p.memory())
						.unwrap_or(0);
					cycle_memory_usage = cycle_end_memory - cycle_start_memory;
					message!("CC added to explicit model ({} ms)", cycle_elapsed);
					message!(
						"CC-specific memory usage: {:.3e} B",
						cycle_memory_usage as f64
					);

					total_state_count = explicit_model.states.len();

					total_elapsed = start_time.elapsed().as_millis();
					message!("Total time for benchmark: {} ms", total_elapsed);

					sys.refresh_processes(ProcessesToUpdate::Some(&[current_pid]), true);
					total_end_memory = sys
						.process(sysinfo::get_current_pid().unwrap())
						.map(|p| p.memory())
						.unwrap_or(0);
					total_memory_usage = total_end_memory - start_memory;
					message!(
						"Total memory for benchmark: {:.3e} B",
						total_memory_usage as f64
					);

					explicit_model.add_absorbing_transitions();
					explicit_model.print_explicit_prism_files(&output_file);
					message!(
						"Current benchmark complete. Output written to {}",
						output_file
					);

					// Drop the large explicit_model on a separate thread with a bigger stack
					// to avoid stack overflow from deep/recursive Drop processing.
					{
						let model_to_drop = explicit_model;
						std::thread::Builder::new()
							.stack_size(32 * 1024 * 1024) // 32 MB stack
							.spawn(move || {
								drop(model_to_drop);
							})
							.expect("Failed to spawn thread to drop explicit_model")
							.join()
							.expect("Failed to join drop thread");
					}

					bash_command = format!(
												"/usr/bin/time -v -o {}prism_time.txt prism -importmodel {}{}.tra,sta,lab {}{}.prop -ctmc > {}prism_output.txt",
												bash_dir, bash_dir, OUTPUT_WORD, bash_dir, OUTPUT_WORD, bash_dir
											);
					writeln!(bash_file, "{}", bash_command)
						.expect("Failed to write bash command to script file");
					time_mem_file_path = format!("{}.stats", output_file);
					time_mem_file = OpenOptions::new()
						.create(true)
						.write(true)
						.truncate(true)
						.open(&time_mem_file_path)
						.expect("Failed to open time/memory file");
					writeln!(
						time_mem_file,
						"Benchmark Results for {}\n\
							Approach: {}\n\
							Commute Depth: {}\n\
							Cycle Length: {}\n\
							Total Time: {} ms\n\
							Ragtimer Time: {} ms\n\
							Cycle Commute Time: {} ms\n\
							Total Memory Used: {} B\n\
							Ragtimer Memory Used: {} B\n\
							Cycle Commute Memory Used: {} B\n\
							Total State Count: {}\n\
							Ragtimer State Count: {}\n\
							Cycle Commute State Count: {}\n",
						model_name,
						approach_word,
						commute_depth,
						cycle_length,
						total_elapsed,
						build_elapsed,
						cycle_elapsed,
						total_memory_usage as f64,
						ragtimer_memory_usage as f64,
						cycle_memory_usage as f64,
						total_state_count,
						ragtimer_state_count,
						cycle_state_count
					)
					.expect("Failed to write time/memory data");
					writeln!(
						csv_file,
						// model,approach,commute_depth,cycle_length,time_ms_total,time_ms_ragtimer,time_ms_cc,bytes_total,bytes_ragtimer,bytes_cc,states_total,states_ragtimer,states_cycle,output_file
						"{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
						model_name,
						approach_word,
						commute_depth,
						cycle_length,
						total_elapsed,
						build_elapsed,
						cycle_elapsed,
						total_memory_usage as f64,
						ragtimer_memory_usage as f64,
						cycle_memory_usage as f64,
						total_state_count,
						ragtimer_state_count,
						cycle_state_count,
						output_file
					)
					.expect("Failed to write benchmark results to CSV file");
				}
				// }
			}
		} else {
			error!("Could not parse model");
		}
	}
	message!("Benchmark complete. CSV output at {}", csv_path);
}
