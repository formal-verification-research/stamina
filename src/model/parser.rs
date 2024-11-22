use std::str::FromStr;

use super::*;
use vas_model::*;

const VARIABLE_TERMS : &[&str] = &["species", "variable", "var"];
const TRANSITION_TERMS : &[&str] = &["reaction", "transition"];
const DECREASE_TERMS : &[&str] = &["consume", "decrease", "decrement"];
const INCREASE_TERMS : &[&str] = &["produce", "increase", "increment"];
const RATE_TERMS : &[&str] = &["rate", "const"];
const TARGET_TERMS : &[&str] = &["target", "goal", "prop", "check"];

// TODO: Add a correct error type, potentially still with a message, or just print the error?
pub fn parse_model(filename: String) -> Result<vas_model::VasModel, String> {
	if let Ok(lines) = util::read_lines(filename) {
		let mut v: Vec<Box<vas_model::Variable>> = vec![];
		let mut t: Vec<Box<vas_model::Transition>> = vec![];
		let mut p: vas_model::Property = Property {
			variable: String::new(),
			operator: Operator::Equal,
			value: 0,
		};
		let mut current_transition: Option<String> = None;

		for line in lines.flatten() {
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
									return Err(format!("Model parsing error: Initial count is not a valid number: {}", words[3]));
								}
							} else {
								return Err(format!("Model parsing error: Expected 'init' as the third word, found: {}", words[2]));
							}
						}
						_ => {
							return Err(format!("Model parsing error: Unexpected number of words for variable term: {} words in term <{}>", words.len(), line));
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
							return Err(format!("Model parsing error: Unexpected number of words for transition term: {} words in term <{}>", words.len(), line));
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
									return Err(format!("Model parsing error: Cannot parse into int: {}", words[2]));
								}
							}
							_ => {
								return Err(format!("Model parsing error: Unexpected number of words for decrease term: {} words in term <{}>", words.len(), line));
							}
						}

						let index = v.clone().into_iter().position(|r| r.variable_name == species_name);
						if index.is_some() {
							if let Some(transition) = t
								.iter_mut()
								.find(|x| x.transition_name == current_transition.clone().unwrap()) { 
									transition.decrement.push(Box::new(Variable {
										variable_name: current_transition.clone().unwrap(),
										count: (count as i128),
									}));
									transition.decrement_vector[index.unwrap()] = Box::new(count);
								}
								else {
									return Err(format!("Model parsing error: Transition {} not found.", current_transition.clone().unwrap()));
								}
						}
						else {
							return Err(format!("Model parsing error: Attempting to decrease species that does not exist: {}", words[1]));
						}
					}
					else {
						return Err(format!("Model parsing error: keyword {} used before declaring a transition.", first_word));
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
									return Err(format!("Model parsing error: Cannot parse into int: {}", words[2]));
								}
							}
							_ => {
								return Err(format!("Model parsing error: Unexpected number of words for increase term: {} words in term <{}>", words.len(), line));
							}
						}

						let index = v.clone().into_iter().position(|r| r.variable_name == species_name);
						if index.is_some() {
							if let Some(transition) = t
								.iter_mut()
								.find(|x| x.transition_name == current_transition.clone().unwrap()) { 
									transition.increment.push(Box::new(Variable {
										variable_name: current_transition.clone().unwrap(),
										count: (count as i128),
									}));
									transition.increment_vector[index.unwrap()] = Box::new(count as u64);
								}
								else {
									return Err(format!("Model parsing error: Transition {} not found.", current_transition.clone().unwrap()));
								}
						}
						else {
							return Err(format!("Model parsing error: Attempting to increase species that does not exist: {}", words[1]));
						}
					}
					else {
						return Err(format!("Model parsing error: keyword {} used before declaring a transition.", first_word));
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
									return Err(format!("Model parsing error: Cannot parse into float: {}", words[1]));
								}
								if let Some(transition) = t
									.iter_mut()
									.find(|x| x.transition_name == current_transition.clone().unwrap()) { 
										transition.transition_rate = count;
								}
								else {
									return Err(format!("Model parsing error: Transition {} not found.", current_transition.clone().unwrap()));
								}
							}
							_ => {
								return Err(format!("Model parsing error: Unexpected number of words for rate term: {} words in term <{}>", words.len(), line));
							}
						}
					}
					else {
						return Err(format!("Model parsing error: keyword {} used before declaring a transition.", first_word));
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
							return Err(format!("Model parsing error: Unexpected number of words for target term: {} words in term <{}>", words.len(), line));
						}
					}
				} else {
					return Err(format!("Unrecognized term: {}", first_word));
				}
			}
		}

		Ok(vas_model::VasModel {
			variables: v,
			transitions: t,
			property: p,
		})
	} else {
		Err(String::from("Model parsing error: line-by-line file parsing not Ok. Check your model file."))
	}
}
