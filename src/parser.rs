use prusti_contracts::*;

struct VAS_Model {
    transitions: 
}

struct Transition {
    increment_vector: &[u64],
	decrement_vector: &[u64],
	transition_name: String,
    transition_rate: f64,
}

impl Transition {
    fn is_catalyst(&self, species_name: String) -> bool {
        // TODO
        unimplemented!()
    }
}


