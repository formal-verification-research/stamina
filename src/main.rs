use model::parser::parse_model;

mod model;

fn main() {
	if let Ok(test_vas)  = parse_model(String::from("models/toy.crn")) {
		println!("{}", test_vas.to_string());
		println!("worked");
	}
	else {
		println!("failed");
	}

}
