use super::default_args::*;
use clap::{Arg, Command};

pub fn parse_args() -> clap::ArgMatches {
	let matches = Command::new("staminats")
		.version("0.0.1")
		.author("Formal Verification Research at Utah State University")
		.about("More details coming soon")
		// Benchmark commands
        .subcommand(
            Command::new("benchmark")
            .about("Runs a benchmark set. This mode is under development. Currently, it benchmarks only Ragtimer and its dependencies.")
			.arg(
				Arg::new("dir")
					.short('d')
					.long("dir")
					.value_name("DIR")
					.help("Set a directory with multiple models (alternative to --model)")
					.required(true)
			)
            .arg(
                Arg::new("num-traces")
                    .long("num-traces")
                    .value_name("NUM_TRACES")
                    .help(&format!("Sets the number of traces to generate (default {})", DEFAULT_NUM_TRACES))
                    .default_value(DEFAULT_NUM_TRACES),
            )
            .arg(
                Arg::new("cycle-length")
                    .long("cycle-length")
                    .value_name("CYCLE_LENGTH")
                    .help(&format!("Sets the cycle length (default {})", DEFAULT_CYCLE_LENGTH))
                    .default_value(DEFAULT_CYCLE_LENGTH),
            )
            .arg(
                Arg::new("commute-depth")
                    .long("commute-depth")
                    .value_name("COMMUTE_DEPTH")
                    .help(&format!("Sets the commute depth (default {})", DEFAULT_COMMUTE_DEPTH))
                    .default_value(DEFAULT_COMMUTE_DEPTH),
            )
            .arg(
            Arg::new("timeout")
                .short('t')
                .long("timeout")
                .value_name("TIMEOUT")
                .help(&format!("Timeout in seconds for each tool (default {})", DEFAULT_TIMEOUT_SECONDS))
                .default_value(DEFAULT_TIMEOUT_SECONDS),
            )
        )
        // BMC commands
        .subcommand(
            Command::new("bmc")
            .about("Runs bounded model checking (BMC) on a given model")
            .arg(
                Arg::new("model")
                    .short('m')
                    .long("model")
                    .value_name("MODEL")
                    .help("Sets the input model file (required)")
                    .required(true),
            )
            .arg(
                Arg::new("steps")
                    .long("steps")
                    .value_name("STEPS")
                    .help("Sets the max number of unrolling steps for BMC (required)")
                    .required(true),
            )
            .arg(
                Arg::new("bits")
                    .long("bits")
                    .value_name("BITS")
                    .help(&format!("Sets the number of bits to use for BMC (default {})", DEFAULT_BOUNDER_BITS))
                    .default_value(DEFAULT_BOUNDER_BITS),
            )
            .arg(
                Arg::new("output")
                    .long("output")
                    .value_name("OUTPUT")
                    .help("Sets the output filename (default <model>.smt2)")
                    .required(false),
            )
			.arg(
				Arg::new("check")
					.long("check")
					.help("Use z3 to check the output model (equivalent to `z3 <model>.smt2`)")
					.action(clap::ArgAction::SetTrue),
			)
            .arg(
            Arg::new("timeout")
                .short('t')
                .long("timeout")
                .value_name("TIMEOUT")
                .help(&format!("Timeout in seconds for each tool (default {})", DEFAULT_TIMEOUT_SECONDS))
                .default_value(DEFAULT_TIMEOUT_SECONDS),
            )
        )
        // Bounder Commands
        .subcommand(
            Command::new("bounds")
            .about("Run the variable bounding tool using BMC and bit vectors")
            .arg(
                Arg::new("model")
                    .short('m')
                    .long("model")
                    .value_name("MODEL")
                    .help("Sets the input model file (required)")
                    .required(true),
            )
            .arg(
                Arg::new("bits")
                    .short('b')
                    .long("bits")
                    .value_name("BITS")
					.help(&format!("Sets the number of bits to use for BMC (default {})", DEFAULT_BOUNDER_BITS))
                    .default_value(DEFAULT_BOUNDER_BITS),
            )
            .arg(
                Arg::new("max-steps")
                    .long("max-steps")
                    .value_name("MAX_STEPS")
                    .help(&format!("Sets the limit on the number of steps (default {})", DEFAULT_BOUNDER_STEPS))
                    .default_value(DEFAULT_BOUNDER_STEPS),
            )
			.arg(
				Arg::new("trim")
					.long("trim")
					.help("Trim model based on dependency graph before bounding")
					.action(clap::ArgAction::SetTrue),
			)
            .arg(
            Arg::new("timeout")
                .short('t')
                .long("timeout")
                .value_name("TIMEOUT")
                .help(&format!("Timeout in seconds for each tool (default {})", DEFAULT_TIMEOUT_SECONDS))
                .default_value(DEFAULT_TIMEOUT_SECONDS),
            )
        )
        // Cycle & Commute commands
        .subcommand(
            Command::new("cycle-commute")
            .about("Build explicit state space from input trace(s) and expand using Cycle & Commute")
            .arg(
                Arg::new("model")
                .short('m')
                .long("model")
                .value_name("MODEL")
                .help("Sets the input model file (required)")
                .required(true),
            )
            .arg(
                Arg::new("trace")
                .long("trace")
                .value_name("TRACE")
                .help("Provide a tab-separated list of transitions (required)")
                .required(true),
            )
            .arg(
                Arg::new("cycle-length")
                .long("cycle-length")
                .value_name("CYCLE_LENGTH")
                .help(&format!("Set the maximum Cycle & Commute cycle length (default {})", DEFAULT_CYCLE_LENGTH))
                .default_value(DEFAULT_CYCLE_LENGTH),
            )
            .arg(
                Arg::new("commute-depth")
                .long("commute-depth")
                .value_name("COMMUTE_DEPTH")
                .help(&format!("Set the maximum Cycle & Commute recursion depth (default {})", DEFAULT_COMMUTE_DEPTH))
                .default_value(DEFAULT_COMMUTE_DEPTH),
            )
            .arg(
                Arg::new("output")
                .short('o')
                .long("output")
                .value_name("OUTPUT")
                .help(&format!("Set the output file name without extensions (default {})", DEFAULT_OUTPUT_NAME))
                .default_value(DEFAULT_OUTPUT_NAME),
            )
            .arg(
                Arg::new("timeout")
                .short('t')
                .long("timeout")
                .value_name("TIMEOUT")
                .help(&format!("Set the time limit per-model in seconds (default {})", DEFAULT_TIMEOUT_SECONDS))
                .default_value(DEFAULT_TIMEOUT_SECONDS),
            ),
        )
        .subcommand(
            Command::new("dependency-graph")
            .about("Builds a dependency graph from the specified model and outputs it in plain text")
            .arg(
                Arg::new("model")
                .short('m')
                .long("model")
                .value_name("MODEL")
                .help("Sets the input model file (required)")
                .required(true),
            )
            .arg(
                Arg::new("output")
                .short('o')
                .long("output")
                .value_name("OUTPUT")
                .help(&format!("Sets the output file name without extensions (default {})", DEFAULT_OUTPUT_NAME))
                .default_value(DEFAULT_OUTPUT_NAME),
            )
            .arg(
                Arg::new("timeout")
                .short('t')
                .long("timeout")
                .value_name("TIMEOUT")
                .help(&format!("Set the time limit per-model in seconds (default {})", DEFAULT_TIMEOUT_SECONDS))
                .default_value(DEFAULT_TIMEOUT_SECONDS),
            ),
        )
        .subcommand(
            Command::new("ragtimer")
                .about("Build explicit state space from input model and generate traces using Ragtimer approaches")
                .arg(
                    Arg::new("model")
                        .short('m')
                        .long("model")
                        .value_name("MODEL")
                        .help("Sets the input model file (required)")
                        .required(true),
                )
                .arg(
                    Arg::new("approach")
                        .short('a')
                        .long("approach")
                        .value_name("APPROACH")
                        .help("Sets the trace generation approach: RL, shortest, or random (default RL)")
                        .default_value("RL"),
                )
                .arg(
                    Arg::new("num-traces")
                        .long("num-traces")
                        .value_name("NUM_TRACES")
                        .help(&format!("Sets the number of traces to generate (default {})", DEFAULT_NUM_TRACES))
                        .default_value(DEFAULT_NUM_TRACES),
                )
                .arg(
                    Arg::new("cycle-length")
                        .long("cycle-length")
                        .value_name("CYCLE_LENGTH")
                        .help(&format!("Sets the maximum Cycle & Commute cycle length (default {})", DEFAULT_CYCLE_LENGTH))
                        .default_value(DEFAULT_CYCLE_LENGTH),
                )
                .arg(
                    Arg::new("commute-depth")
                        .long("commute-depth")
                        .value_name("COMMUTE_DEPTH")
                        .help(&format!("Sets the maximum Cycle & Commute recursion depth (default {})", DEFAULT_COMMUTE_DEPTH))
                        .default_value(DEFAULT_COMMUTE_DEPTH),
                )
				.arg(
					Arg::new("output")
						.short('o')
						.long("output")
						.value_name("OUTPUT")
						.help(&format!("Sets the output file name without extensions (default {})", DEFAULT_OUTPUT_NAME))
						.default_value(DEFAULT_OUTPUT_NAME),
				)
                .arg(
                    Arg::new("timeout")
                        .short('t')
                        .long("timeout")
                        .value_name("TIMEOUT")
                        .help(&format!("Set the time limit per-model in seconds (default {})", DEFAULT_TIMEOUT_SECONDS))
                        .default_value(DEFAULT_TIMEOUT_SECONDS),
                ),
				// TODO: Add options for RL magic number import (from some kind of file)
        )
		.subcommand(
            Command::new("wayfarer")
                .about("Wayfarer is not yet implemented")
		)
		.subcommand(
            Command::new("stamina")
                .about("Stamina is not yet implemented")
		)
		.get_matches();
	matches
}
