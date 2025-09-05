use crate::cycle_commute::commute::cycle_commute;
use crate::model::vas_model::AbstractVas;
use std::fs;
use std::path::Path;
use chrono::Local;
use std::time::Instant;
use sysinfo::{System, SystemExt, ProcessExt};
use std::fs::OpenOptions;
use std::io::Write;
use crate::*;

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
/// It reads the model from the specified file, processes the trace file,
/// and writes the output to the specified output file.
/// It is not meant to be used by an end user, but rather as a demo or proof of concept for the cycle commute functionality.
/// For now, run this demo with
/// cargo run -- cycle-commute-benchmark --models-dir <dir> --min-commute-depth <min> --max-commute-depth <max> --min-cycle-length <min> --max-cycle-length <max>
/// cargo run -- cycle-commute-benchmark --default -m benchmark_models
pub fn cycle_commute_benchmark(
	model_dir: &Path,
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
		"model,commute_depth,cycle_length,time_ms_total,time_ms_ragtimer,time_ms_cc,bytes_total,bytes_ragtimer,bytes_cc,output_file"
	).expect("Failed to write CSV header");

	let mut bash_file = OpenOptions::new()
		.create(true)
		.write(true)
		.truncate(true)
		.open(format!("output/{}/run_all.sh", timestamp))
		.expect("Failed to open bash script file");

	for model_file in crn_files {
		let model_name = Path::new(&model_file)
		.file_name()
		.and_then(|s| s.to_str())
		.unwrap_or("model")
		.replace(".", "_");
		for cycle_length in min_cycle_length..=max_cycle_length {
			for commute_depth in min_commute_depth..=max_commute_depth {
				message!("Running cycle commute benchmark on model: {} with commute depth: {} and cycle length: {}", model_file, commute_depth, cycle_length);
				if let Ok(mut abstract_model) = AbstractVas::from_file(&model_file) {
					message!("Model {} Parsed", model_name);
					let output_dir = format!("output/{}/{}/cycle_{}/commute_{}/", timestamp, model_name, cycle_length, commute_depth);
					let output_word = "model";
					fs::create_dir_all(&output_dir).expect("Failed to create output directory");
					let output_file = Path::new(&output_dir).join(output_word).to_string_lossy().into_owned();
					let mut explicit_model = PrismVasModel::from_abstract_model(&abstract_model);
					message!("Explicit Model Built");
					
					
					let mut ragtimer_builder = RagtimerBuilder::new(&abstract_model, None);
					ragtimer_builder.build(&mut explicit_model);
					message!("Traces added to explicit model with Ragtimer");

					// Set start time and memory usage
					let mut sys = System::new();
					sys.refresh_process(sysinfo::get_current_pid().unwrap());
					let start_memory = sys.process(sysinfo::get_current_pid().unwrap())
						.map(|p| p.memory())
						.unwrap_or(0);
					let start_time = Instant::now();

					// Time Ragtimer state space building
					let ragtimer_start_time = Instant::now();
					let ragtimer_start_memory = sys.process(sysinfo::get_current_pid().unwrap())
						.map(|p| p.memory())
						.unwrap_or(0);
					let mut ragtimer_builder = RagtimerBuilder::new(&abstract_model, None);
					ragtimer_builder.build(&mut explicit_model);
					let build_elapsed = ragtimer_start_time.elapsed().as_millis();
					let ragtimer_end_memory = sys.process(sysinfo::get_current_pid().unwrap())
						.map(|p| p.memory())
						.unwrap_or(0);
					let ragtimer_memory_usage = ragtimer_end_memory - ragtimer_start_memory;
					message!("Traces added to explicit model with Ragtimer ({} ms)", build_elapsed);
					message!("Ragtimer-specific memory usage: {:.3e} B", ragtimer_memory_usage as f64);

					// Time cycle and commute
					let cycle_start_time = Instant::now();
					let cycle_start_memory = sys.process(sysinfo::get_current_pid().unwrap())
						.map(|p| p.memory())
						.unwrap_or(0);
					cycle_commute(
						&mut abstract_model,
						&mut explicit_model,
						commute_depth,
						cycle_length,
					);
					let cycle_elapsed = cycle_start_time.elapsed().as_millis();
					let cycle_end_memory = sys.process(sysinfo::get_current_pid().unwrap())
						.map(|p| p.memory())
						.unwrap_or(0);
					let cycle_memory_usage = cycle_end_memory - cycle_start_memory;
					message!("CC added to explicit model ({} ms)", cycle_elapsed);
					message!("CC-specific memory usage: {:.3e} B", cycle_memory_usage as f64);

					let total_elapsed = start_time.elapsed().as_millis();
					message!("Total time for benchmark: {} ms", total_elapsed);

					sys.refresh_process(sysinfo::get_current_pid().unwrap());
					let total_end_memory = sys.process(sysinfo::get_current_pid().unwrap())
						.map(|p| p.memory())
						.unwrap_or(0);
					let total_memory_usage = total_end_memory - start_memory;
					message!("Total memory for benchmark: {:.3e} B", total_memory_usage as f64);

					explicit_model.print_explicit_prism_files(&output_file);
					message!(
						"Current benchmark complete. Output written to {}",
						output_file
					);

					let bash_dir = format!("{}/cycle_{}/commute_{}/", model_name, cycle_length, commute_depth);
					let prop_dst = format!("{}.prop", output_file);

					let prop_src = Path::new(&model_file)
						.with_extension("prop");
					if prop_src.exists() {
						fs::copy(&prop_src, &prop_dst).expect("Failed to copy .prop file");
						message!("Copied property file to {}", prop_dst);
					}

					let bash_command = format!(
											"/usr/bin/time -v -o {}prism_time.txt prism -importmodel {}{}.tra,sta,lab {}{}.prop -ctmc > {}prism_output.txt",
											bash_dir, bash_dir, output_word, bash_dir, output_word, bash_dir
										);
					writeln!(bash_file, "{}", bash_command)
						.expect("Failed to write bash command to script file");

					let time_mem_file_path = format!("{}.stats", output_file);
					let mut time_mem_file = OpenOptions::new()
						.create(true)
						.write(true)
						.truncate(true)
						.open(&time_mem_file_path)
						.expect("Failed to open time/memory file");
					writeln!(
						time_mem_file,
						"Benchmark Results for {}\n\
						Commute Depth: {}\n\
						Cycle Length: {}\n\
						Total Time: {} ms\n\
						Ragtimer Time: {} ms\n\
						Cycle Commute Time: {} ms\n\
						Total Memory Used: {} B\n\
						Ragtimer Memory Used: {} B\n\
						Cycle Commute Memory Used: {} B\n",
						model_name,
						commute_depth,
						cycle_length,
						total_elapsed,
						build_elapsed,
						cycle_elapsed,
						total_memory_usage as f64,
						ragtimer_memory_usage as f64,
						cycle_memory_usage as f64
					).expect("Failed to write time/memory data");

					writeln!(
						csv_file,
						// model,commute_depth,cycle_length,time_ms_total,time_ms_ragtimer,time_ms_cc,bytes_total,bytes_ragtimer,bytes_cc,output_file
						"{},{},{},{},{},{},{},{},{},{}",
						model_name,
						commute_depth,
						cycle_length,
						total_elapsed,
						build_elapsed,
						cycle_elapsed,
						total_memory_usage as f64,
						ragtimer_memory_usage as f64,
						cycle_memory_usage as f64,
						output_file
					).expect("Failed to write CSV row");
				} else {
					error!("Could not parse model");
				}
			}
		}
	}
	message!("Benchmark complete. CSV output at {}", csv_path);
}
