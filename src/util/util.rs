use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

/// Reads a file into a list of lines. Returns an error if the file was unable to open
pub(crate) fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
	P: AsRef<Path>,
{
	let file = File::open(filename)?;
	Ok(io::BufReader::new(file).lines())
}
