

use std::str::FromStr;

use crate::{model::{model::AbstractModel, vas_model::{AbstractVas, VasTransition}}, property::property::Property, util::util::read_lines};

use super::*;
// use vas_model::*;
// use util::read_lines;

const VARIABLE_TERMS : &[&str] = &["species", "variable", "var"];
const INITIAL_TERMS : &[&str] = &["initial", "init"];
const TRANSITION_TERMS : &[&str] = &["reaction", "transition"];
const DECREASE_TERMS : &[&str] = &["consume", "decrease", "decrement"];
const INCREASE_TERMS : &[&str] = &["produce", "increase", "increment"];
const RATE_TERMS : &[&str] = &["rate", "const"];
const TARGET_TERMS : &[&str] = &["target", "goal", "prop", "check"];

#[derive(Clone, Debug)]
enum ModelParseErrorType {
	InvalidInitialVariableCount(String), // Variable name
	InitUnspecified(String), // The initial value for a variable is unspecified
	UnexpextedTokenError(String), // A token is found we were not expecting
	ExpectedInteger(String), // We expected an integer, we got this
	ExpectedFloat(String), // We expected a float, we got this
	UnspecifiedTransitionError(String), // The name of the transition
	UnspecifiedVariableError(String), // The name of the variable
	GeneralParseError(String), // Description
}

impl ToString for ModelParseErrorType {
	fn to_string(&self) -> String {
	    match self {
			Self::InvalidInitialVariableCount(count) => format!("Invalid initial count: `{}`.", count),
			Self::InitUnspecified(var_name) => format!("The initial value for `{}` is unspecified.", var_name),
			Self::UnexpextedTokenError(token) => format!("Unexpexted token: `{}`.", token),
			Self::ExpectedInteger(value) => format!("Expected integer, got `{}`.", value),
			Self::ExpectedFloat(value) => format!("Expected float, got `{}`.", value),
			Self::UnspecifiedTransitionError(transition) => format!("Unspecified transition: `{}`.", transition),
			Self::UnspecifiedVariableError(var) => format!("Unspecified variable: `{}`", var),
			Self::GeneralParseError(desc) => format!("General Parse Error: {}", desc),
	        
	    }
	}
}

#[derive(Clone, Debug)]
struct ModelParseError {
	line: u32,
	etype: ModelParseErrorType,
}

impl ModelParseError {
	fn invalid_init(line: u32, count: &dyn ToString) -> Self {
		Self {
			line: line,
			etype: ModelParseErrorType::InvalidInitialVariableCount(count.to_string()),
		}
	}

	fn init_unspecified(line: u32, name: &dyn ToString) -> Self {
		Self {
			line: line,
			etype: ModelParseErrorType::InitUnspecified(name.to_string()),
		}
	}

	fn unexpected_token(line: u32, token: &dyn ToString) -> Self {
		Self {
			line: line,
			etype: ModelParseErrorType::UnexpextedTokenError(token.to_string()),
		}
	}

	fn expected_integer(line: u32, value: &dyn ToString) -> Self {
		Self {
			line: line,
			etype: ModelParseErrorType::ExpectedInteger(value.to_string()),
		}
	}

	fn expected_float(line: u32, value: &dyn ToString) -> Self {
		Self {
			line: line,
			etype: ModelParseErrorType::ExpectedFloat(value.to_string()),
		}
	}

	fn unspecified_transition(line: u32, tname: &dyn ToString) -> Self {
		Self {
			line: line,
			etype: ModelParseErrorType::UnspecifiedTransitionError(tname.to_string()),
		}
	}

	fn unspecified_variable(line: u32, vname: &dyn ToString) -> Self {
		Self {
			line: line,
			etype: ModelParseErrorType::UnspecifiedVariableError(vname.to_string()),
		}
	}

	fn general(line: u32, desc: &dyn ToString) -> Self {
		Self {
			line: line,
			etype: ModelParseErrorType::GeneralParseError(desc.to_string()),
		}		
	}
}

struct TempTransition {
	transition_id: usize,
	transition_name: String,
	increment: Box<[u64]>,
	decrement: Box<[u64]>,
	transition_rate: f64,
}
struct TempVariable {
	variable_id: usize,
	variable_name: String,
	initial_value: u64,
}

fn get_temp_variable_id(v: &Vec<TempVariable>, name: &str) -> Option<usize> {
	v.iter().position(|r| r.variable_name == name)
}

/// Parses a VAS model from a file
pub fn parse_model(filename: String) -> Result<AbstractModel, ModelParseError> {
	
	let lines = read_lines(&filename).map_err(|_| ModelParseError::general(0, &"line-by-line file parsing not Ok. Check your model file."))?;

	let model: AbstractVas = AbstractVas::new();
	let num_states: usize = 0;
	let num_trans: usize = 0;

	let mut variables: Vec<TempVariable> = vec![];

	let mut transitions: Vec<TempTransition> = vec![];

	let mut property: String;

	// let mut p: vas_model::Property = Property {
	// 	variable: String::new(),
	// 	operator: Operator::Equal,
	// 	value: 0,
	// };

	let mut num_variables: usize = 0;
	let mut num_transitions: usize = 0;
	let mut current_transition: Option<&TempTransition> = None;

	for (num, line) in lines.flatten().enumerate() {
		// Split the line into words and convert to a slice
		let words: &[&str] = &line.split_whitespace().collect::<Vec<&str>>()[..];

		if let Some(first_word) = words.get(0) {
			if VARIABLE_TERMS.contains(&first_word) {
				// If there is already a transition, there is an error in the model
				if current_transition.is_some() {
					return Err(ModelParseError::general(num, &"Model parsing error: variable keyword used after declaring a transition."));
				}
				// Check the number of words
				match words.len() {
					2 => {
						// Handle case with just variable names (i.e., initial value is assumed to be 0)
						let variable_name = words[1].to_string();
						let variable_init = 0;
						variables.push(TempVariable {
							variable_id: num_variables,
							variable_name: variable_name,
							initial_value: variable_init,
						});
						num_variables += 1;
					}
					4 => {
						// Handle case with initialization (i.e., initial value follows word "init")
						if INITIAL_TERMS.contains(&words[2]) {
							if let Ok(count) = words[3].parse::<i128>() {
								let variable_name = words[1].to_string();
								let variable_init = count;
								variables.push(TempVariable {
									variable_id: num_variables,
									variable_name: variable_name,
									initial_value: variable_init,
								});
								num_variables += 1;
							} else {
								return Err(ModelParseError::invalid_init(num, &words[3]));
							}
						} else {
							return Err(ModelParseError::init_unspecified(num, &words[1]));
						}
					}
					_ => {
						return Err(ModelParseError::unexpected_token(num, &line));
					}
				}
			} else if TRANSITION_TERMS.contains(&first_word) {
				match words.len() {
					2 => {
						// transition should just have a name
						let transition_name = words[1].to_string();
						transitions.push(TempTransition {
							transition_id: num_transitions,
							transition_name: transition_name,
							increment: vec![0; variables.len()].into_boxed_slice(),
							decrement: vec![0; variables.len()].into_boxed_slice(),
							transition_rate: 0.0,
						});
						current_transition = Some(&transitions[num_transitions]);
						num_transitions += 1;
					}
					_ => {
						return Err(ModelParseError::unexpected_token(num, &line));
					}
				}
			} else if DECREASE_TERMS.contains(&first_word) {
				if current_transition.is_some() {
					// get the variable to decrease
					let variable_name: String = if words.len() >= 2 {
						words[1].to_string()
					} else {
						return Err(ModelParseError::unexpected_token(num, &line));
					};
					let variable_id = if let Ok(vid) = get_temp_variable_id(&variables, &variable_name) {
						vid
					} else {
						return Err(ModelParseError::unspecified_variable(num, &variable_name));
					};
					// get the count to decrease
					let count = match words.len() {
						2 => {
							1
						}
						3 => {
							if let Ok(count_s) = words[2].parse::<u64>() {
								count_s
							} else {
								return Err(ModelParseError::expected_integer(num, &words[2]));
							}
						}
						_ => {
							return Err(ModelParseError::unexpected_token(num, &line));
						}
					}; 
					// update the transition
					let transition = current_transition.unwrap();
					if transition.decrement[variable_id] != 0 {
						return Err(ModelParseError::general(num, &format!("Model parsing error: variable {} increases by multiple declared values in the same transition.", variable_name)));
					}
					transition.decrement[variable_id] = count;	
				}
				else {
					return Err(ModelParseError::general(num, 
						&format!("Model parsing error: keyword {} used before declaring a transition.", first_word)));
				}
			} else if INCREASE_TERMS.contains(&first_word) {
				if current_transition.is_some() {
					// get the variable to decrease
					let variable_name: String = if words.len() >= 2 {
						words[1].to_string()
					} else {
						return Err(ModelParseError::unexpected_token(num, &line));
					};
					let variable_id = if let Ok(vid) = get_temp_variable_id(&variables, &variable_name) {
						vid
					} else {
						return Err(ModelParseError::unspecified_variable(num, &variable_name));
					};
					// get the count to decrease
					let count = match words.len() {
						2 => {
							1
						}
						3 => {
							if let Ok(count_s) = words[2].parse::<u64>() {
								count_s
							} else {
								return Err(ModelParseError::expected_integer(num, &words[2]));
							}
						}
						_ => {
							return Err(ModelParseError::unexpected_token(num, &line));
						}
					}; 
					// update the transition
					let transition = current_transition.unwrap();
					if transition.increment[variable_id] != 0 {
						return Err(ModelParseError::general(num, &format!("Model parsing error: variable {} increases by multiple declared values in the same transition.", variable_name)));
					}
					transition.increment[variable_id] = count;	
				}
				else {
					return Err(ModelParseError::general(num, 
						&format!("Model parsing error: keyword {} used before declaring a transition.", first_word)));
				}
			} else if RATE_TERMS.contains(&first_word) {
				if current_transition.is_some() {
					match words.len() {
						2 => {
							if current_transition.is_some() {
								let rate: f64 = words[1].parse::<f64>().map_err(|_| ModelParseError::expected_float(num, &words[1]))?;
								// update the transition
								let transition = current_transition.unwrap();
								if transition.rate != 0.0 {
									return Err(ModelParseError::general(num, &format!("Model parsing error: rate has multiple declared values in the same transition.")));
								}
								transition.transition_rate = rate;
							}
							else {
								return Err(ModelParseError::general(num, 
									&format!("Model parsing error: keyword {} used before declaring a transition.", first_word)));
							}
						}
						_ => {
							return Err(ModelParseError::unexpected_token(num, &line));
						}
					}
				}
				else {
					return Err(ModelParseError::unexpected_token(num, &first_word));
				}
			} else if TARGET_TERMS.contains(&first_word) {
				println!("Found target term: {}", first_word);
				println!("No property parsing implemented yet.");
			} else {
				return Err(ModelParseError::unexpected_token(num, &first_word));
			}
		}
	}

	// Convert the temporary variables and transitions into the final model
	let mut final_variables: Box<[String]> = vec!["".to_string(); variables.len()].into_boxed_slice();
	let mut final_transitions: Vec<VasTransition> = vec![];

	for v in variables {
		final_variables[v.variable_id] = v.variable_name;
	}

	for t in transitions {
		let final_transition = VasTransition::new();
		final_transition.set_vectors(t.increment, t.decrement);
		final_transition.set_rate(t.rate);
	}

}