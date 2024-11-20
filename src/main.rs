mod model;
mod dependency;

use model::parser::parse_model;
use dependency::graph::make_dependency_graph;


fn main() {
	let parsed_model = parse_model(String::from("models/toy.crn"));
	if parsed_model.is_ok() {
		let model = parsed_model.unwrap();
		println!("{}", model.to_string());
        println!("parsing worked!");

        let dependency_graph = make_dependency_graph(&model);

	}
	else {
		println!("parsing failed");
		if let Err(e) = parsed_model {
			println!("{}", e);
		}
		return;
	}

	// let dep_graph = make_dependency_graph(&parsed_model.unwrap());

}
