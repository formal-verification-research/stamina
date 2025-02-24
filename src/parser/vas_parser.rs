

use std::str::FromStr;

use super::*;
use vas_model::*;
use util::read_lines;

const VARIABLE_TERMS : &[&str] = &["species", "variable", "var"];
const TRANSITION_TERMS : &[&str] = &["reaction", "transition"];
const DECREASE_TERMS : &[&str] = &["consume", "decrease", "decrement"];
const INCREASE_TERMS : &[&str] = &["produce", "increase", "increment"];
const RATE_TERMS : &[&str] = &["rate", "const"];
const TARGET_TERMS : &[&str] = &["target", "goal", "prop", "check"];

#[derive(Copy, Clone, Debug)]
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

#[derive(Copy, Clone, Debug)]
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

// TODO: Add a correct error type, potentially still with a message, or just print the error?
pub fn parse_model(filename: String) -> Result<vas_model::VasModel, ModelParseError> {
	if let Ok(lines) = read_lines(filename) {
		let mut v: Vec<Box<vas_model::Variable>> = vec![];
		let mut t: Vec<Box<vas_model::Transition>> = vec![];
		let mut p: vas_model::Property = Property {
			variable: String::new(),
			operator: Operator::Equal,
			value: 0,
		};
		let mut current_transition: Option<String> = None;

		for (num, line) in lines.flatten().enumerate() {
			// Split the line into words and convert to a slice
			let words: &[&str] = &line.split_whitespace().collect::<Vec<&str>>()[..];

			if let Some(first_word) = words.get(0) {
				if VARIABLE_TERMS.contains(&first_word) {
					// Check the number of words
					match words.len() {
						2 => {
							// Handle case with 2 words
							let variable_name = words[1].to_string();
							v.push(Box::new(vas_model::Variable {
								variable_name,
								count: 0, // Default or unspecified initial count
							}));
						}
						4 => {
							// Handle case with 4 words
							if words[2] == "init" {
								if let Ok(count) = words[3].parse::<i128>() {
									let variable_name = words[1].to_string();
									v.push(Box::new(vas_model::Variable {
										variable_name,
										count,
									}));
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
							let transition_name = words[1].to_string();
							current_transition = Some(String::from(transition_name.clone()));
							t.push(Box::new(vas_model::Transition {
								increment: Vec::new(),
								decrement: Vec::new(),
								increment_vector: vec![Box::new(0); v.len()],
								decrement_vector: vec![Box::new(0); v.len()],
								transition_name: transition_name,
								transition_rate: 0.0,
							}));
						}
						_ => {
							return Err(ModelParseError::unexpected_token(num, &line));
						}
					}
				} else if DECREASE_TERMS.contains(&first_word) {
					if current_transition.is_some() {
						let species_name : String;
						let count;
						match words.len() {
							2 => {
								species_name = String::from(words[1]);
								count = 1;
							}
							3 => {
								species_name = String::from(words[1]);
								let count_s = words[2].parse::<u64>();
								if count_s.is_ok() {
									count = count_s.unwrap();
								}
								else {
									return Err(ModelParseError::expected_integer(num, &words[2]));
								}
							}
							_ => {
								return Err(ModelParseError::unexpected_token(num, &line));
							}
						}

						let index = v.clone().into_iter().position(|r| r.variable_name == species_name);
						if index.is_some() {
							if let Some(transition) = t
								.iter_mut()
								.find(|x| x.transition_name == current_transition.clone().unwrap()) { 
									transition.decrement.push(Box::new(Variable {
										variable_name: species_name,
										count: (count as i128),
									}));
									transition.decrement_vector[index.unwrap()] = Box::new(count);
								}
								else {
									return Err(ModelParseError::unspecified_transition(num, &current_transition.unwrap()));
								}
						}
						else {
							return Err(ModelParseError::unspecified_variable(num, words[1]));
						}
					}
					else {
						return Err(ModelParseError::general(num, 
							format!("Model parsing error: keyword {} used before declaring a transition.", first_word)));
					}
				} else if INCREASE_TERMS.contains(&first_word) {
					if current_transition.is_some() {
						let species_name : String;
						let count;
						match words.len() {
							2 => {
								species_name = String::from(words[1]);
								count = 1;
							}
							3 => {
								species_name = String::from(words[1]);
								let count_s = words[2].parse::<i128>();
								if count_s.is_ok() {
									count = count_s.unwrap();
								}
								else {
									return Err(ModelParseError::expected_integer(num, &words[2]));
								}
							}
							_ => {
								return Err(ModelParseError::unexpected_token(num, &line));
							}
						}

						let index = v.clone().into_iter().position(|r| r.variable_name == species_name);
						if index.is_some() {
							if let Some(transition) = t
								.iter_mut()
								.find(|x| x.transition_name == current_transition.clone().unwrap()) { 
									transition.increment.push(Box::new(Variable {
										variable_name: species_name,
										count: (count as i128),
									}));
									transition.increment_vector[index.unwrap()] = Box::new(count as u64);
								}
								else {
									return Err(ModelParseError::unspecified_transition(num, &current_transition.unwrap()));
								}
						}
						else {
							return Err(ModelParseError::unspecified_variable(num, &words[1]));
						}
					}
					else {
						return Err(ModelParseError::unexpected_token(num, first_word));
					}
				} else if RATE_TERMS.contains(&first_word) {
					if current_transition.is_some() {
						match words.len() {
							2 => {
								let count: f64;
								let count_s = words[1].parse::<f64>();
								if count_s.is_ok() {
									count = count_s.unwrap();
								}
								else {
									return Err(ModelParseError::expected_float(num, words[1]));
								}
								if let Some(transition) = t
									.iter_mut()
									.find(|x| x.transition_name == current_transition.clone().unwrap()) { 
										transition.transition_rate = count;
								}
								else {
									return Err(ModelParseError(num, &current_transition.unwrap()));
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
					match words.len() {
						4 => {
							let var = words[1];
							let op = Operator::from_str(words[2].trim());

							// let op: Result<Operator, String> = match words[2] {
							// 	">" => Ok(Operator::GreaterThan),
							// 	"<" => Ok(Operator::LessThan),
							// 	"==" => Ok(Operator::Equal),
							// 	"!=" => Ok(Operator::NotEqual),
							// 	">=" => Ok(Operator::GreaterThanOrEqual),
							// 	"<=" => Ok(Operator::LessThanOrEqual),
							// 	_ => return Err(format!("Model parsing error: Unexpected operator in target: {}", words[2])),
							// };

							let val = words[3].parse::<u64>().map_err(|_| format!("Model parsing error: Not an integer as target value: {}", words[2]));
							p = Property {
								variable: var.to_string(),
								operator: op.unwrap(),
								value: val.unwrap(),
							};
						}
						_ => {
							return Err(ModelParseError::unexpected_token(num, &line));
						}
					}
				} else {
					return Err(ModelParseError::unexpected_token(num, &first_word));
				}
			}
		}

		Ok(vas_model::VasModel {
			variables: v,
			transitions: t,
			property: p,
		})
	} else {
		Err(ModelParseError::general(0, &"line-by-line file parsing not Ok. Check your model file."))
	}
}
