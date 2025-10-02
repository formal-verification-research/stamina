#![allow(dead_code)]

mod arguments;
mod bmc;
mod builder;
mod cycle_commute;
mod demos;
mod dependency;
mod logging;
mod model;
mod parser;
mod property;
mod trace;
mod util;
mod validator;

fn main() {
	// Get the current date and time for logging purposes
	let now = chrono::Local::now();
	let hostname = hostname::get().unwrap_or_else(|_| "(unknown)".into());

	// Parse command line arguments
	let args = arguments::arg_parser::parse_args();

	// Print welcome message
	println!(
		"
 ╺┳╸╻ ╻┏━╸   ┏━┓╺┳╸┏━┓┏┳┓╻┏┓╻┏━┓   ╺┳╸┏━┓┏━┓╻  ┏━┓┏━╸╺┳╸
  ┃ ┣━┫┣╸    ┗━┓ ┃ ┣━┫┃┃┃┃┃┗┫┣━┫    ┃ ┃ ┃┃ ┃┃  ┗━┓┣╸  ┃ 
  ╹ ╹ ╹┗━╸   ┗━┛ ╹ ╹ ╹╹ ╹╹╹ ╹╹ ╹    ╹ ┗━┛┗━┛┗━╸┗━┛┗━╸ ╹ 
    "
	);
	message!("Welcome to the Stamina Toolset!");
	message!("Repository: https://github.com/formal-verification-research/stamina-toolset");
	message!("Documentation: https://github.com/formal-verification-research/stamina-toolset/tree/main/docs");
	message!("For help, use the --help flag or consult the documentation.");
	info!(
		"This execution began on host {} at {}",
		hostname.to_string_lossy(),
		now.format("%Y-%m-%d %H:%M:%S%.3f")
	);

	// Execute commands based on parsed arguments
	arguments::cmd_executor::run_commands(&args);
}
