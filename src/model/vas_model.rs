pub(crate) struct VasModel {
    // TODO: Might we want to use hashmaps instead? We can think about this later if we need an efficiency boost
    pub(super) variables: Vec<Box<Variable>>,
    pub(super) transitions: Vec<Box<Transition>>,
    pub(super) property: String, // TODO: maybe make a better data structure?
}

#[derive(Clone)]
pub(crate) struct Variable {
    pub(super) variable_name: String,
    pub(super) initial_count: u64,
}

pub(crate) struct Transition {
    pub(super) increment_vector: Vec<Box<u64>>,
	pub(super) decrement_vector: Vec<Box<u64>>,
	pub(super) transition_name: String,
    pub(super) transition_rate: f64,
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
                variable.variable_name, variable.initial_count));
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


