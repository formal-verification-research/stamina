use nalgebra::DVector;

use crate::{model::vas_model::{VasProperty, VasState, VasTransition}, AbstractVas};

use super::graph::{make_dependency_graph, DependencyGraph};


/// This trims the model by removing unnecessary variables and transitions
/// determined by the dependency graph.


// pub(crate) struct AbstractVas {
// 	pub(crate) variable_names: Box<[String]>,
// 	pub(crate) initial_states: Vec<VasState>,
// 	pub(crate) transitions: Vec<VasTransition>,
// 	pub(crate) m_type: ModelType,
// 	pub(crate) target: VasProperty,
// }

pub fn trim_model(model: AbstractVas, dg: DependencyGraph) -> AbstractVas {
    let mut variable_names = Vec::<String>::new();
	let mut initial_state = Vec::<u64>::new();
    let mut transitions = Vec::<VasTransition>::new();
    let dg_transitions = dg.get_transitions();
    
    // Collect the variables that are used in the dependency graph
    for i in 0..model.variable_names.len() {
        let mut is_used = false;
        print!("{}: ", model.variable_names[i]);
        for t in dg_transitions.iter() {
            if t.update_vector[i] != 0 || t.enabled_bounds[i] != 0 {
                is_used = true;
                println!("used by transition {}", t.transition_name);
                break;
            }
        }
        if is_used {
            variable_names.push(model.variable_names[i].clone());
            initial_state.push(model.initial_states[0].vector[i].try_into().unwrap());
        }
        else {
            println!("unused");
        }
    }

    // Collect the transitions that are used in the dependency graph,
    // and filter the update vector and enabled bounds to only include the used variables
    for t in dg_transitions {
        transitions.push(VasTransition {
            transition_name: t.transition_name,
            transition_id: t.transition_id,
            update_vector: t.update_vector
                .iter()
                .enumerate()
                .filter_map(|(i, &x)| if variable_names.contains(&model.variable_names[i]) { Some(x) } else { None })
                .collect::<Vec<_>>().into(),
            enabled_bounds: t.enabled_bounds
                .iter()
                .enumerate()
                .filter_map(|(i, &x)| if variable_names.contains(&model.variable_names[i]) { Some(x) } else { None })
                .collect::<Vec<_>>().into(),
            rate_const: t.rate_const,
            custom_rate_fn: t.custom_rate_fn,
        });
    }

    // Update the target property to match the trimmed model
    let target_index = variable_names.iter().position(|x| x == &model.variable_names[model.target.variable_index]).unwrap_or(0);
    let target = VasProperty {
        variable_index: target_index,
        target_value: model.target.target_value,
    };
    
    let trimmed_model = AbstractVas {
        variable_names: variable_names.into_boxed_slice(),
        initial_states: vec![VasState::new(DVector::from_vec(initial_state.into_iter().map(|x| x as i64).collect()))],
        transitions: transitions,
        m_type: model.m_type,
        target: target,
    };

    trimmed_model
}