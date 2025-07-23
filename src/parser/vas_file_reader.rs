use std::fmt;

use metaverify::trusted;
use nalgebra::DVector;

use crate::{
	logging::messages::*,
	model::{
		model::{AbstractModel, ProbabilityOrRate},
		vas_model::{AbstractVas, VasProperty, VasState, VasTransition, VasValue},
	},
	util::util::read_lines,
};

const VARIABLE_TERMS: &[&str] = &["species", "variable", "var"];
const INITIAL_TERMS: &[&str] = &["initial", "init"];
const TRANSITION_TERMS: &[&str] = &["reaction", "transition"];
const DECREASE_TERMS: &[&str] = &["consume", "decrease", "decrement"];
const INCREASE_TERMS: &[&str] = &["produce", "increase", "increment"];
const RATE_TERMS: &[&str] = &["rate", "const"];
const TARGET_TERMS: &[&str] = &["target", "goal", "prop", "check"];

#[trusted]
#[derive(Clone, Debug)]
enum ModelParseErrorType {
	InvalidInitialVariableCount(String), // Variable name
	InitUnspecified(String),             // The initial value for a variable is unspecified
	UnexpextedTokenError(String),        // A token is found we were not expecting
	ExpectedInteger(String),             // We expected an integer, we got this
	ExpectedFloat(String),               // We expected a float, we got this
	UnspecifiedTransitionError(String),  // The name of the transition
	UnspecifiedVariableError(String),    // The name of the variable
	GeneralParseError(String),           // Description
}
#[trusted]
impl ToString for ModelParseErrorType {
	#[trusted]
	fn to_string(&self) -> String {
		match self {
			Self::InvalidInitialVariableCount(count) => {
				format!("Invalid initial count: `{}`.", count)
			}
			Self::InitUnspecified(var_name) => {
				format!("The initial value for `{}` is unspecified.", var_name)
			}
			Self::UnexpextedTokenError(token) => format!("Unexpexted token: `{}`.", token),
			Self::ExpectedInteger(value) => format!("Expected integer, got `{}`.", value),
			Self::ExpectedFloat(value) => format!("Expected float, got `{}`.", value),
			Self::UnspecifiedTransitionError(transition) => {
				format!("Unspecified transition: `{}`.", transition)
			}
			Self::UnspecifiedVariableError(var) => format!("Unspecified variable: `{}`", var),
			Self::GeneralParseError(desc) => format!("General Parse Error: {}", desc),
		}
	}
}
#[trusted]
#[derive(Clone, Debug)]
pub struct ModelParseError {
	line: usize,
	etype: ModelParseErrorType,
}
#[trusted]
impl fmt::Display for ModelParseError {
	#[trusted]
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let (line_num, line_content) = self.line();
		let col = self.column();
		let err_str = self.to_string();
		let marker = if col.is_some() {
			let col_n = col.unwrap();
			format!(
				"{}^{}",
				" ".repeat(col_n as usize),
				"-".repeat(line_content.len() - col_n as usize - 1)
			)
		} else {
			"^".repeat(line_content.len())
		};
		write!(
			f,
			"[Parse Error] Error in model parsing. Unable to parse model!\n{}: {}\n{}\n{}",
			line_num, line_content, marker, err_str
		)
	}
}
#[trusted]
impl ModelParseError {
	#[trusted]
	fn line(&self) -> (usize, String) {
		(self.line, self.etype.to_string())
	}
	#[trusted]
	fn column(&self) -> Option<u32> {
		None
	}
	#[trusted]
	fn invalid_init(line: usize, count: &dyn ToString) -> Self {
		Self {
			line,
			etype: ModelParseErrorType::InvalidInitialVariableCount(count.to_string()),
		}
	}
	#[trusted]
	fn init_unspecified(line: usize, name: &dyn ToString) -> Self {
		Self {
			line,
			etype: ModelParseErrorType::InitUnspecified(name.to_string()),
		}
	}
	#[trusted]
	fn unexpected_token(line: usize, token: &dyn ToString) -> Self {
		Self {
			line,
			etype: ModelParseErrorType::UnexpextedTokenError(token.to_string()),
		}
	}
	#[trusted]
	fn expected_integer(line: usize, value: &dyn ToString) -> Self {
		Self {
			line,
			etype: ModelParseErrorType::ExpectedInteger(value.to_string()),
		}
	}
	#[trusted]
	fn expected_float(line: usize, value: &dyn ToString) -> Self {
		Self {
			line,
			etype: ModelParseErrorType::ExpectedFloat(value.to_string()),
		}
	}
	#[trusted]
	fn unspecified_transition(line: usize, tname: &dyn ToString) -> Self {
		Self {
			line,
			etype: ModelParseErrorType::UnspecifiedTransitionError(tname.to_string()),
		}
	}
	#[trusted]
	fn unspecified_variable(line: usize, vname: &dyn ToString) -> Self {
		Self {
			line,
			etype: ModelParseErrorType::UnspecifiedVariableError(vname.to_string()),
		}
	}
	#[trusted]
	fn general(line: usize, desc: &dyn ToString) -> Self {
		Self {
			line,
			etype: ModelParseErrorType::GeneralParseError(desc.to_string()),
		}
	}
}
#[trusted]
fn get_variable_id(v: &[String], name: &str) -> Option<usize> {
	v.iter().position(|r| r == name)
}

#[trusted]
// Build two variable objects (names and initial values)
fn build_variables(
	raw_data: Vec<(usize, String)>,
) -> Result<(Box<[String]>, Box<[VasValue]>), ModelParseError> {
	let mut variable_names = Vec::<String>::new();
	let mut initial_state = Vec::<VasValue>::new();
	for declaration in raw_data.iter() {
		let words: &[&str] = &declaration.1.split_whitespace().collect::<Vec<&str>>()[..];
		let variable_name;
		let variable_init;
		match words.len() {
			2 => {
				// Handle case with just variable names (i.e., initial value is assumed to be 0)
				variable_name = words[1].to_string();
				variable_init = 0;
			}
			4 => {
				// Handle case with initialization (i.e., initial value follows word "init")
				if INITIAL_TERMS.contains(&words[2]) {
					variable_name = words[1].to_string();
					if let Ok(count) = words[3].parse::<VasValue>() {
						variable_init = count;
					} else {
						return Err(ModelParseError::invalid_init(
							declaration.0.try_into().unwrap(),
							&words[3],
						));
					}
				} else {
					return Err(ModelParseError::init_unspecified(
						declaration.0.try_into().unwrap(),
						&words[1],
					));
				}
			}
			_ => {
				return Err(ModelParseError::unexpected_token(
					declaration.0.try_into().unwrap(),
					&declaration.1,
				));
			}
		}
		variable_names.push(variable_name);
		initial_state.push(variable_init);
	}
	Ok((
		variable_names.into_boxed_slice(),
		initial_state.into_boxed_slice(),
	))
}

// Build the transition objects
#[trusted]
fn build_transitions(
	raw_data: Vec<Vec<(usize, std::string::String)>>,
	variable_names: &Box<[String]>,
) -> Result<Vec<<AbstractVas as AbstractModel>::TransitionType>, ModelParseError> {
	let mut transitions = Vec::<<AbstractVas as AbstractModel>::TransitionType>::new();
	let num_variables = variable_names.len();
	let mut transition_id: usize = 0;

	for declaration in raw_data {
		let mut transition_name = String::new();
		let mut increment = vec![VasValue::from(0); num_variables].into_boxed_slice();
		let mut decrement = vec![VasValue::from(0); num_variables].into_boxed_slice();
		let mut rate_const: ProbabilityOrRate = 0.0;

		for line in declaration.iter() {
			let words: &[&str] = &line.1.split_whitespace().collect::<Vec<&str>>()[..];
			let first_word = words.get(0).unwrap_or(&"");
			if TRANSITION_TERMS.contains(first_word) {
				if words.len() == 2 {
					transition_name = words[1].to_string();
				} else {
					return Err(ModelParseError::unexpected_token(
						line.0.try_into().unwrap(),
						&line.1,
					));
				}
			} else if DECREASE_TERMS.contains(first_word) {
				let variable_name;
				let decrease_count: VasValue;
				match words.len() {
					2 => {
						variable_name = words[1].to_string();
						decrease_count = 1;
					}
					3 => {
						variable_name = words[1].to_string();
						if let Ok(count) = words[2].parse::<VasValue>() {
							if count < 0 {
								// TODO: This error, and presumably others, just cause a stack overflow. Need to fix this.
								return Err(ModelParseError::unexpected_token(
									line.0.try_into().unwrap(),
									&line.1,
								));
							} else {
								decrease_count = count;
							};
						} else {
							return Err(ModelParseError::unexpected_token(
								line.0.try_into().unwrap(),
								&line.1,
							));
						}
					}
					_ => {
						return Err(ModelParseError::unexpected_token(
							line.0.try_into().unwrap(),
							&line.1,
						));
					}
				}
				// update the transition
				if let Some(index) = get_variable_id(&variable_names, &variable_name) {
					if decrement[index] != 0 {
						return Err(ModelParseError::general(line.0.try_into().unwrap(), &format!("Model parsing error: variable {} decreases by multiple declared values in the same transition.", variable_name)));
					}
					decrement[index] = decrease_count;
				} else {
					return Err(ModelParseError::unspecified_variable(
						line.0.try_into().unwrap(),
						&variable_name,
					));
				}
			} else if INCREASE_TERMS.contains(first_word) {
				let variable_name;
				let increase_count;
				match words.len() {
					2 => {
						variable_name = words[1].to_string();
						increase_count = 1;
					}
					3 => {
						variable_name = words[1].to_string();
						if let Ok(count) = words[2].parse::<VasValue>() {
							increase_count = if count < 0 {
								return Err(ModelParseError::unexpected_token(
									line.0.try_into().unwrap(),
									&line.1,
								));
							} else {
								count
							};
						} else {
							return Err(ModelParseError::unexpected_token(
								line.0.try_into().unwrap(),
								&line.1,
							));
						}
					}
					_ => {
						return Err(ModelParseError::unexpected_token(
							line.0.try_into().unwrap(),
							&line.1,
						));
					}
				}
				// update the transition
				if let Some(index) = get_variable_id(&variable_names.clone(), &variable_name) {
					if increment[index] != 0 {
						return Err(ModelParseError::general(line.0.try_into().unwrap(), &format!("Model parsing error: variable {} increases by multiple declared values in the same transition.", variable_name)));
					}
					increment[index] = increase_count;
				} else {
					return Err(ModelParseError::unspecified_variable(
						line.0.try_into().unwrap(),
						&variable_name,
					));
				}
			} else if RATE_TERMS.contains(first_word) {
				if words.len() == 2 {
					if let Ok(rate) = words[1].parse::<ProbabilityOrRate>() {
						rate_const = rate;
					} else {
						return Err(ModelParseError::expected_float(
							line.0.try_into().unwrap(),
							&words[1],
						));
					}
				} else {
					return Err(ModelParseError::unexpected_token(
						line.0.try_into().unwrap(),
						&line.1,
					));
				}
			} else {
				return Err(ModelParseError::unexpected_token(
					line.0.try_into().unwrap(),
					&line.1,
				));
			}
		}

		let transition = VasTransition::new(
			transition_id,
			transition_name,
			increment,
			decrement,
			rate_const,
		);

		transitions.push(transition);

		transition_id += 1;
	}

	// For now, we'll return an error as a placeholder
	Ok(transitions)
}

fn build_property(
	raw_data: Vec<(usize, String)>,
	variable_names: Box<[String]>,
) -> Result<VasProperty, ModelParseError> {
	if raw_data.len() != 1 {
		return Err(ModelParseError::general(
			0,
			&"Model parsing error: property must be a single line.",
		));
	}
	let words: &[&str] = &raw_data[0].1.split_whitespace().collect::<Vec<&str>>()[..];
	let variable_name = words.get(1).unwrap_or(&"");
	let variable_index = get_variable_id(&*variable_names, variable_name);
	if variable_index.is_none() {
		return Err(ModelParseError::unspecified_variable(
			raw_data[0].0.try_into().unwrap(),
			&variable_name,
		));
	}
	let variable_index = variable_index.unwrap();
	let target_value = if words.len() == 4 {
		if let Ok(value) = words[3].parse::<VasValue>() {
			value
		} else {
			return Err(ModelParseError::expected_integer(
				raw_data[0].0.try_into().unwrap(),
				&words[2],
			));
		}
	} else {
		return Err(ModelParseError::unexpected_token(
			raw_data[0].0.try_into().unwrap(),
			&raw_data[0].1,
		));
	};

	let property = VasProperty {
		variable_index,
		target_value,
	};

	Ok(property)
}

pub fn build_model(filename: &str) -> Result<AbstractVas, ModelParseError> {
	// Initialize everything
	let lines = read_lines(&filename).map_err(|_| {
		ModelParseError::general(
			0,
			&"line-by-line file parsing not Ok. Check your model file.",
		)
	})?;

	// Setup strings for the various things
	let mut variable_lines = Vec::<(usize, String)>::new();
	let mut transition_lines = Vec::<Vec<(usize, String)>>::new();
	let mut property_lines = Vec::<(usize, String)>::new();
	let mut current_transition = Vec::<(usize, String)>::new();

	for (num, line) in lines.flatten().enumerate() {
		// Split the line into words and convert to a slice, then sort the line by first words
		let words: &[&str] = &line.split_whitespace().collect::<Vec<&str>>()[..];
		let first_word = words.get(0).unwrap_or(&"");

		// VARIABLE_TERMS
		// INITIAL_TERMS
		// TRANSITION_TERMS
		// DECREASE_TERMS
		// INCREASE_TERMS
		// RATE_TERMS
		// TARGET_TERMS

		// Check the first word against the keywords
		if VARIABLE_TERMS.contains(first_word) {
			variable_lines.push((num, line));
		} else if TRANSITION_TERMS.contains(first_word) {
			if current_transition.is_empty() {
				current_transition = vec![(num, line)];
			} else {
				transition_lines.push(current_transition);
				current_transition = vec![(num, line)];
			}
		} else if DECREASE_TERMS.contains(first_word) || INCREASE_TERMS.contains(first_word) {
			current_transition.push((num, line));
		} else if RATE_TERMS.contains(first_word) {
			current_transition.push((num, line));
		} else if TARGET_TERMS.contains(first_word) {
			property_lines.push((num, line));
		} else if line != "" {
			return Err(ModelParseError::unexpected_token(
				num.try_into().unwrap(),
				&line,
			));
		}
		// TODO: Implement better error handling at initial parse.
	}
	transition_lines.push(current_transition);

	// Parse the variables and initial states
	let (variable_names, initial_states) = match build_variables(variable_lines) {
		Ok(result) => result,
		Err(e) => return Err(e),
	};

	// Read the property
	let target = build_property(property_lines, variable_names.clone())?;

	// Read the transitions
	let transitions = match build_transitions(transition_lines, &variable_names) {
		Ok(result) => result,
		Err(e) => {
			error!("ERROR DURING TRANSITION PARSING:\n{}", e.to_string());
			return Err(e);
		}
	};

	// Return the model
	let model = AbstractVas::new(
		variable_names,
		vec![VasState::new(DVector::from_vec(initial_states.to_vec()))],
		transitions,
		target,
	);

	Ok(model)
}
