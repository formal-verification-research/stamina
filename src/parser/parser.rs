use std::fmt;

use super::model::AbstractModel;

pub(crate) trait ModelParseError: Copy, Clone, ToString {
	/// The line number where the error occurred
	fn line(&self) -> (u64, String);
	/// The column where the error occurred (not all errors can provide this)
	fn column(&self) -> Option<u64>;
}

/// A nice pretty way to print parse errors with helpful 
impl fmt::Display for ModelParseError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let line_num, line_content = self.line();
		let col = self.column();
		let err_str = self.to_string();
		let marker = if col.is_some() {
			let col_n = col.unwrap();
			format!("{}^{}", " ".repeat(coln), "-".repeat(line_content.len() - coln - 1)
		} else {
			"^".repeat(line_content.len())
		};
		write!(f, "[Parse Error] Error in model parsing. Unable to parse model!\n{}: {}\n{}\n{}", line_num, line_content, marker, err_str);

	}
}

pub(crate) trait Parser {
	type ModelType: AbstractModel;
	type ParserErrorType: ModelParseError;

	fn parse(filename: &str) -> Result<ModelType, ParseErrorType>;

	fn parse_or_panic(filename: &str) -> ModelType {
		let model = Parser::parse(filename);
		match model {
			Ok(model) => { return model; },
			Err(parse_error) => { panic!("{parse_error:?}"); },
		};
	}
}
