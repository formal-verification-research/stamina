use model::parser::parse_model;

mod model;

fn main() {
	let res = parse_model(String::from("models/toy.crn"));
	if res.is_ok() {
		println!("{}", res.unwrap().to_string());
		println!("parsing worked!");
	}
	else {
		println!("parsing failed");
		if let Err(e) = res {
			println!("{}", e);
		}
	}

}
