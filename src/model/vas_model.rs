use std::fmt::Display;
use std::str::FromStr;

#[derive(Clone)]
pub(crate) enum Operator {
	GreaterThan,
	LessThan,
	Equal,
	NotEqual,
	GreaterThanOrEqual,
	LessThanOrEqual,
}

impl Display for Operator {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let op_str = match self {
			Operator::GreaterThan => ">",
			Operator::LessThan => "<",
			Operator::Equal => "==",
			Operator::NotEqual => "!=",
			Operator::GreaterThanOrEqual => ">=",
			Operator::LessThanOrEqual => "<=",
		};
		write!(f, "{}", op_str)
	}
}
impl FromStr for Operator {
	type Err = &'static str;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			">" => Ok(Operator::GreaterThan),
			"<" => Ok(Operator::LessThan),
			"=" => Ok(Operator::Equal),
			"==" => Ok(Operator::Equal),
			"!=" => Ok(Operator::NotEqual),
			">=" => Ok(Operator::GreaterThanOrEqual),
			"<=" => Ok(Operator::LessThanOrEqual),
			_ => Err("Invalid operator"),
		}
	}
}


pub(crate) struct Property {
	pub(crate) variable: String,
	pub(crate) operator: Operator,
	pub(crate) value: u64,
}

impl Display for Property {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{} {} {}", self.variable, self.operator, self.value)
	}
}
impl Clone for Property {
	fn clone(&self) -> Self {
		Property {
			variable: self.variable.clone(),
			operator: self.operator.clone(),
			value: self.value,
		}
	}
}

pub(crate) struct VasModel {
	// TODO: Might we want to use hashmaps instead? We can think about this later if we need an efficiency boost
	pub(crate) variables: Vec<Box<Variable>>,
	pub(crate) transitions: Vec<Box<Transition>>,
	pub(crate) property: Property, 
}

#[derive(Clone)]
pub(crate) struct Variable {
	pub(crate) variable_name: String,
	pub(crate) count: i128,
}

pub(crate) struct Transition {
	pub(crate) increment: Vec<Box<Variable>>,
	pub(crate) decrement: Vec<Box<Variable>>,
	pub(crate) increment_vector: Vec<Box<u64>>,
	pub(crate) decrement_vector: Vec<Box<u64>>,
	pub(crate) transition_name: String,
	pub(crate) transition_rate: f64,
}

impl VasModel {
	pub fn to_string(&self) -> String {
		let mut result = String::new();

		// Add the property
		result.push_str(&format!("Property: {}\n", self.property));

		// Add variables
		result.push_str("Variables:\n");
		for variable in &self.variables {
			result.push_str(&format!("  - Name: {}, Initial Count: {}\n", 
				variable.variable_name, variable.count));
		}

		// Add transitions
		result.push_str("Transitions:\n");
		for transition in &self.transitions {
			result.push_str(&format!("  - Name: {}, Rate: {}\n", 
				transition.transition_name, transition.transition_rate));
			result.push_str("    Increment Vector: [");
			result.push_str(&transition.increment_vector.iter()
				.map(|x| x.to_string())
				.collect::<Vec<String>>()
				.join(", "));
			result.push_str("]\n");
			result.push_str("    Decrement Vector: [");
			result.push_str(&transition.decrement_vector.iter()
				.map(|x| x.to_string())
				.collect::<Vec<String>>()
				.join(", "));
			result.push_str("]\n");
		}

		result
	}
}




// impl Transition {
//     fn is_catalyst(&self, species_name: String) -> bool {
//         // TODO
//         unimplemented!()
//     }
//     fn to_string(&self) -> String {
//         self.transition_name.clone()
//     }
// }


