use std::fmt;

use metaverify::trusted;

use crate::model::model::ModelType;
use crate::model::*;
use crate::parser::parser::model::AbstractModel;

#[trusted]
pub(crate) trait ModelParseError: ToString {
	/// The line number where the error occurred
	fn line(&self) -> (u64, String);
	/// The column where the error occurred (not all errors can provide this)
	fn column(&self) -> Option<u64>;
}

/// A wrapper type for ModelParseError to implement fmt::Display
#[trusted]
impl fmt::Display for dyn ModelParseError {
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
pub(crate) trait Parser {
	type ModelType: AbstractModel + Into<ModelType>;
	type ParserErrorType: ModelParseError + fmt::Debug;

	#[trusted]
	fn parse(filename: &str) -> Result<Self::ModelType, Self::ParserErrorType>;

	#[trusted]
	fn parse_or_panic(filename: &str) -> ModelType {
		let model = Self::parse(filename);
		match model {
			Ok(model) => {
				return model.into();
			}
			Err(parse_error) => {
				panic!("{parse_error:?}");
			}
		};
	}
}
