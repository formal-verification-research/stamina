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
	let args = arguments::arg_parser::parse_args();
	arguments::cmd_executor::run_commands(&args);
}
