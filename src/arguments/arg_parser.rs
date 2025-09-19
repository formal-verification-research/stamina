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
				Arg::new("model")
					.short('m')
					.long("model")
					.value_name("MODEL")
					.help("Sets the model file (VAS format)")
					.required(false)
					.conflicts_with("dir")
			)
			.arg(
				Arg::new("dir")
					.short('d')
					.long("dir")
					.value_name("DIR")
					.help("Set a directory with multiple models (alternative to --model)")
					.required(false)
					.conflicts_with("model")
			)
			.arg(
				Arg::new("input")
					.required(true)
					.hide(true)
					.action(clap::ArgAction::Set)
					.value_parser(clap::builder::ValueParser::os_string())
					.help("Exactly one of --model or --dir is required")
					.requires_if("model", "model")
					.requires_if("dir", "dir")
			)
            .arg(
                Arg::new("num-traces")
                    .long("num-traces")
                    .value_name("NUM_TRACES")
                    .help(&format!("Sets the number of traces to generate (default {})", DEFAULT_NUM_TRACES))
                    .default_value("10000"),
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
                Arg::new("output")
                    .long("output")
                    .value_name("OUTPUT")
                    .help("Sets the output directory (default <model>.smt2)")
                    .required(false),
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
                        .default_value("10000"),
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
                    Arg::new("timeout")
                        .short('t')
                        .long("timeout")
                        .value_name("TIMEOUT")
                        .help(&format!("Set the time limit per-model in seconds (default {})", DEFAULT_TIMEOUT_SECONDS))
                        .default_value(DEFAULT_TIMEOUT_SECONDS),
                ),
        )
		.subcommand(
            Command::new("wayfarer")
                .about("Wayfarer is not yet implemented")
		)
		.subcommand(
            Command::new("stamina")
                .about("Stamina is not yet implemented")
		)


        // .subcommand(
		// 	Command::new("bounds")
		// 		.about("Run the variable bounding tool")
		// 		.arg(
		// 			Arg::new("models_dir")
		// 				.short('d')
		// 				.long("models-dir")
		// 				.value_name("DIR")
		// 				.help("Sets the directory containing model folders")
		// 				.default_value("models"),
		// 		)
		// 		.arg(
		// 			Arg::new("bits")
		// 				.short('b')
		// 				.long("bits")
		// 				.value_name("BITS")
		// 				.help("Sets the number of bits for variable representation (default 9)")
		// 				.default_value("9"),
		// 		)
		// 		.arg(
		// 			Arg::new("max_steps")
		// 				.short('m')
		// 				.long("max-steps")
		// 				.value_name("MAX_STEPS")
		// 				.help("Sets the maximum number of BMC steps to take (default 500)")
		// 				.default_value("500"),
		// 		)
		// 		.arg(
		// 			Arg::new("backward")
		// 				.long("backward")
		// 				.help("Run in backward mode if specified")
		// 				.action(clap::ArgAction::SetTrue),
		// 		),
		// )
		// .subcommand(
		// 	Command::new("cycle-commute-benchmark")
		// 		.about("Run the cycle commute benchmark")
		// 		.arg(
		// 			Arg::new("models_dir")
		// 				.short('m')
		// 				.long("models-dir")
		// 				.value_name("DIR")
		// 				.help("Sets the directory containing model folders")
		// 				.default_value("models"),
		// 		)
		// 		.arg(
		// 			Arg::new("min_commute_depth")
		// 				.short('d')
		// 				.long("min-commute-depth")
		// 				.value_name("MIN_COMMUTE_DEPTH")
		// 				.help("Sets the minimum commute depth")
		// 				.default_value("3"),
		// 		)
		// 		.arg(
		// 			Arg::new("max_commute_depth")
		// 				.short('D')
		// 				.long("max-commute-depth")
		// 				.value_name("MAX_COMMUTE_DEPTH")
		// 				.help("Sets the maximum commute depth")
		// 				.default_value("3"),
		// 		)
		// 		.arg(
		// 			Arg::new("min_cycle_length")
		// 				.short('c')
		// 				.long("min-cycle-length")
		// 				.value_name("MIN_CYCLE_LENGTH")
		// 				.help("Sets the minimum cycle length")
		// 				.default_value("3"),
		// 		)
		// 		.arg(
		// 			Arg::new("max_cycle_length")
		// 				.short('C')
		// 				.long("max-cycle-length")
		// 				.value_name("MAX_CYCLE_LENGTH")
		// 				.help("Sets the maximum cycle length")
		// 				.default_value("3"),
		// 		)
		// 		.arg(
		// 			Arg::new("default")
		// 				.long("default")
		// 				.help("Set all parameters to default recommended values")
		// 				.action(clap::ArgAction::SetTrue),
		// 		),
		// )
		// .subcommand(
		// 	Command::new("dependency-graph")
		// 		.about("Run the variable bounding tool")
		// 		.arg(
		// 			Arg::new("model")
		// 				.short('m')
		// 				// .long("model")
		// 				.value_name("FILE")
		// 				.help("Sets the model file")
		// 				.required(true),
		// 		),
		// )
		// .subcommand(
		// 	Command::new("ragtimer")
		// 		.about("Run the ragtimer tool (currently including only the RL Traces tool)")
		// 		.arg(
		// 			Arg::new("model")
		// 				.short('d')
		// 				.long("model")
		// 				.value_name("MODEL")
		// 				.help("Sets the model file (crn format)")
		// 				.required(true),
		// 		)
		// 		.arg(
		// 			Arg::new("qty")
		// 				.short('q')
		// 				.long("qty")
		// 				.value_name("QTY")
		// 				.help("Sets the number of traces to generate (default 100)")
		// 				.default_value("100"),
		// 		)
		// 		.arg(
		// 			Arg::new("timeout")
		// 				.short('t')
		// 				.long("timeout")
		// 				.value_name("MINUTES")
		// 				.help("Timeout in minutes for get_bounds")
		// 				.default_value(TIMEOUT_MINUTES),
		// 		),
		// )
		// .subcommand(
		// 	Command::new("cycle-commute")
		// 		.about("Run the Cycle & Commute tool")
		// 		.arg(
		// 			Arg::new("model")
		// 				.short('d')
		// 				.long("model-file")
		// 				.value_name("MODEL")
		// 				.help("Sets the model file (crn format)")
		// 				.required(true),
		// 		)
		// 		// .arg(
		// 		// 	Arg::new("trace")
		// 		// 		.short('t')
		// 		// 		.long("trace-file")
		// 		// 		.value_name("TRACE")
		// 		// 		.help("File containing white-space separated transition names for seed traces")
		// 		// 		.required(true),
		// 		// )
		// 		.arg(
		// 			Arg::new("output_file")
		// 				.short('o')
		// 				.long("output-file")
		// 				.value_name("OUTPUT")
		// 				.help("File to write the output to WITHOUT A FILE EXTENSION")
		// 				.default_value("cycle_commute_output"),
		// 		)
		// 		.arg(
		// 			Arg::new("max_commute_depth")
		// 				.short('c')
		// 				.long("max-commute-depth")
		// 				.value_name("MAX_COMMUTE_DEPTH")
		// 				.help("Max recursion depth for commuting transitions")
		// 				.default_value("3"),
		// 		)
		// 		.arg(
		// 			Arg::new("max_cycle_length")
		// 				.short('l')
		// 				.long("max-cycle-length")
		// 				.value_name("MAX_CYCLE_LENGTH")
		// 				.help("Max length of cycles to consider")
		// 				.default_value("3"),
		// 		),
		// )
		// .subcommand(
		// 	Command::new("stamina")
		// 		.about("Run the stamina tool")
		// 		.arg(
		// 			Arg::new("models_dir")
		// 				.required(true)
		// 				.short('d')
		// 				.long("models-dir")
		// 				.value_name("DIR")
		// 				.help("Sets the directory containing model folders")
		// 				.default_value("models"),
		// 		)
		// 		.arg(
		// 			Arg::new("timeout")
		// 				.short('t')
		// 				.long("timeout")
		// 				.value_name("MINUTES")
		// 				.help("Timeout in minutes for get_bounds")
		// 				.default_value(TIMEOUT_MINUTES),
		// 		),
		// )
		// .subcommand(
		// 	Command::new("wayfarer")
		// 		.about("Run the wayfarer tool")
		// 		.arg(
		// 			Arg::new("models_dir")
		// 				.required(true)
		// 				.short('d')
		// 				.long("models-dir")
		// 				.value_name("DIR")
		// 				.help("Sets the directory containing model folders")
		// 				.default_value("models"),
		// 		)
		// 		.arg(
		// 			Arg::new("timeout")
		// 				.short('t')
		// 				.long("timeout")
		// 				.value_name("MINUTES")
		// 				.help("Timeout in minutes for get_bounds")
		// 				.default_value(TIMEOUT_MINUTES),
		// 		),
		// )
		.get_matches();
	matches
}
