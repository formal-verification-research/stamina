use std::collections::HashMap;

use z3::ast::{self, Ast};

/// Unroller for BMC, which handles the state variables and their transitions.
#[derive(Debug, Clone)]
pub struct Unroller<'ctx> {
	pub state_vars: HashMap<String, ast::BV<'ctx>>,
	next_vars: HashMap<String, ast::BV<'ctx>>,
	var_cache: HashMap<(String, u32), ast::BV<'ctx>>,
	time_cache: Vec<HashMap<ast::BV<'ctx>, ast::BV<'ctx>>>,
}

impl<'ctx> Unroller<'ctx> {
	/// Constructs a new Unroller with the given state and next variables.
	pub fn new(
		state_vars: HashMap<String, ast::BV<'ctx>>,
		next_vars: HashMap<String, ast::BV<'ctx>>,
	) -> Self {
		Self {
			state_vars,
			next_vars,
			var_cache: HashMap::new(),
			time_cache: Vec::new(),
		}
	}

	/// Returns the state variable at a specific time step.
	pub fn at_time<T>(&mut self, term: &T, k: u32) -> T
	where
		T: Ast<'ctx> + Clone,
	{
		let cache = self.get_cache_at_time(k);
		term.substitute(&cache.iter().map(|(k, v)| (k, v)).collect::<Vec<_>>())
	}

	/// Returns the disjunction of state variable at all times up to k.
	pub fn at_all_times_or(&mut self, term: &ast::Bool<'ctx>, k: u32) -> ast::Bool<'ctx> {
		let mut terms = Vec::new();
		for i in 0..=k {
			terms.push(self.at_time(term, i));
		}
		ast::Bool::or(term.get_ctx(), &terms.iter().collect::<Vec<_>>())
	}

	/// Returns the conjunction of state variable at all times up to k.
	pub fn at_all_times_and(&mut self, term: &ast::Bool<'ctx>, k: u32) -> ast::Bool<'ctx> {
		let mut terms = Vec::new();
		for i in 0..=k {
			terms.push(self.at_time(term, i));
		}
		ast::Bool::and(term.get_ctx(), &terms.iter().collect::<Vec<_>>())
	}

	/// Returns the variable at a specific time step, caching it for future use.
	pub fn get_var(&mut self, v: &ast::BV<'ctx>, k: u32) -> ast::BV<'ctx> {
		let key = (v.to_string(), k);
		if let Some(var) = self.var_cache.get(&key) {
			return var.clone();
		}

		let v_k = ast::BV::new_const(
			v.get_ctx(),
			format!("{}@{}", v.to_string(), k),
			v.get_size(),
		);
		self.var_cache.insert(key, v_k.clone());
		v_k
	}

	/// Returns the cache at a specific time step.
	fn get_cache_at_time(&mut self, k: u32) -> &HashMap<ast::BV<'ctx>, ast::BV<'ctx>> {
		while self.time_cache.len() <= k as usize {
			let mut cache = HashMap::new();
			let t = self.time_cache.len() as u32;

			for (s, state_var) in self.state_vars.clone() {
				let s_t = self.get_var(&state_var, t);
				let n_t = self.get_var(&state_var, t + 1);
				cache.insert(state_var.clone(), s_t);
				cache.insert(self.next_vars[&s].clone(), n_t);
			}

			self.time_cache.push(cache);
		}
		&self.time_cache[k as usize]
	}
}
