use clap::*;

#[derive(Debug, Clone)]
enum Method {
    Stamina,
    Wayfarer,
	Ragtimer,
}

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Args {
	/// Filename
	#[arg(short, long)]
	filename : String,

	// Method of analysis to use
	// #[arg(short, long)]
	//method : Method,
}

fn main() {
	let args = Args::parse();
	println!("Got filename {}", args.filename);
}
