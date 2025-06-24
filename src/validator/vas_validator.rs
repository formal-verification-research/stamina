use metaverify::*;

use crate::model::vas_model::{AbstractVas, VasProperty, VasTransition, VasValue};
use ::std::collections::HashMap;
use colored::{ColoredString, Colorize};

#[trusted]
fn check_variable_names(variable_names: &Box<[String]>) -> Vec<String> {
	let mut errors = Vec::new();
	let empty_variable_names = variable_names.iter().filter(|name| name.is_empty()).count();
	if empty_variable_names > 0 {
		let unnamed_indices: Vec<usize> = variable_names
			.iter()
			.enumerate()
			.filter(|(_, name)| name.is_empty())
			.map(|(index, _)| index)
			.collect();
		errors.push(format!(
			"{} variables have empty names at indices: {:?}",
			empty_variable_names, unnamed_indices
		));
	}
	let mut name_counts = HashMap::new();
	for name in variable_names.iter() {
		*name_counts.entry(name).or_insert(0) += 1;
	}
	let duplicates: Vec<_> = name_counts
		.iter()
		.filter(|&(_, &count)| count > 1)
		.map(|(name, _)| name.clone())
		.collect();
	if !duplicates.is_empty() {
		errors.push(format!("Duplicate variable names found: {:?}", duplicates));
	}
	errors
}

#[trusted]
fn initial_state_neq_target(initial_state: Box<[VasValue]>, property: &VasProperty) -> Vec<String> {
	let mut errors = Vec::new();
	if initial_state[property.variable_index] == property.target_value {
		errors.push(format!(
			"Initial state [ {} ] satisfies target with value {}",
			initial_state
				.iter()
				.map(|x| format!("{}", x))
				.collect::<Vec<String>>()
				.join(" "),
			property.target_value
		));
	}
	errors
}

#[trusted]
// TODO: To check the SCK assumption, I think we actually need the increment/decrement vectors
fn check_sck_assumption(transitions: Vec<VasTransition>) -> Vec<String> {
	let mut errors = Vec::new();

	for transition in transitions {
		let total_update: VasValue = transition.update_vector.iter().map(|&val| val).sum();
		if total_update > 3 || total_update < -3 {
			errors.push(format!(
				"Transition {} has an update vector with total change > 3",
				transition.transition_name
			));
		}

		if transition.update_vector.iter().any(|&val| val > 2) {
			errors.push(format!(
				"Transition {} has an update with a value > 2",
				transition.transition_name
			));
		}
	}
	errors
}

#[trusted]
fn check_rate_constant(transitions: Vec<VasTransition>) -> Vec<String> {
	let mut errors = Vec::new();
	for transition in transitions {
		if transition.rate_const <= 0.0 {
			errors.push(format!(
				"Transition {} has a non-positive rate constant {}",
				transition.transition_name, transition.rate_const
			));
		}
	}
	errors
}

#[trusted]
pub fn write_outcome(test_name: &str, errors: Vec<String>) -> String {
	let fail = "FAIL".red();
	let pass = "PASS".green();
	fn explain(text: String) -> ColoredString {
		text.purple()
	}
	let mut result = String::new();
	if errors.is_empty() {
		result.push_str(&format!("[{}]\t", pass));
		result.push_str(&format!("{}\n", test_name));
	} else {
		result.push_str(&format!("[{}]\t", fail));
		result.push_str(&format!("{}\n", test_name));
		for error in errors {
			result.push_str(&format!("\t{}\n", explain(error)));
		}
	}
	result
}

#[trusted]
pub fn validate_vas(model: &AbstractVas, property: &VasProperty) -> Result<String, String> {
	let mut result = String::new();

	result.push_str("===============================================\n");
	result.push_str("              Vas Model Validation             \n");
	result.push_str("===============================================\n");

	// Perform Tests
	result.push_str(&write_outcome(
		"Check Variable Names",
		check_variable_names(&model.variable_names),
	));
	let initial_state: Box<[VasValue]> = model.initial_states[0]
		.vector
		.iter()
		.map(|&val| val)
		.collect::<Vec<VasValue>>()
		.into_boxed_slice();
	result.push_str(&write_outcome(
		"Check Initial State != Target",
		initial_state_neq_target(initial_state, &property),
	));
	result.push_str(&write_outcome(
		"Check SCK Assumption (CRNs Only)",
		check_sck_assumption(model.transitions.clone()),
	));
	result.push_str(&write_outcome(
		"Check Rate Constant",
		check_rate_constant(model.transitions.clone()),
	));

	Ok(result)
}

#[trusted]
pub fn validate_vas_property(property: VasProperty) -> Result<String, String> {
	// Implement the validation logic for the VAS property
	Ok("Property validation successful".to_string())
}
