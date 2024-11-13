use clap::*;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Args {
	/// Filename
	#[arg(short, long)]
	filename : String,
}

fn main() {
	let args = Args::parse();
	println!("Got filename {}", args.filename);
}
