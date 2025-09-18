use z3::{
	ast::{self},
	// Config, Context,
};

use crate::{
	bmc::{bounds::BMCBounds, encoding::BMCEncoding},
	model::model::AbstractModel,
	AbstractVas,
};

/// Trait for Abstract VAS models to provide BMC-related functionality.
pub(crate) trait AbstractVasBmc: AbstractModel {
	/// Sets up the Z3 context for BMC.
	// fn setup_z3(&mut self);
	/// Returns the formula for BMC plus the unroller.
	/// Order: (init_formula, transition_formula, target_formula, unroller)
	fn bmc_encoding(&self, bits: u32) -> BMCEncoding;
	/// Returns the variable bounds
	fn variable_bounds(
		&self,
		bmc_encoding: &BMCEncoding,
		bits: u32,
		max_steps: u32,
		backward: bool,
	) -> BMCBounds;
	/// Runs general BMC for the given number of steps.
	fn run_bmc(
		&self,
		bmc_encoding: &BMCEncoding,
		max_steps: u32,
		backward: bool,
	) -> (ast::Bool, u32);
}

impl AbstractVasBmc for AbstractVas {
	/// Sets up the Z3 context for BMC.
	// fn setup_z3(&mut self) {
	// 	let cfg = Config::new();
	// 	let ctx = Context::new(&cfg);
	// 	self.z3_context = Some(ctx);
	// }

	/// Returns the formula for BMC plus the unroller.
	/// Order: (context, config, init_formula, transition_formula, target_formula, unroller)
	fn bmc_encoding(&self, bits: u32) -> BMCEncoding {
		BMCEncoding::from_vas(self, bits)
	}

	/// Returns the variable bounds for the VAS model.
	/// It computes both loose and tight bounds for upper and lower limits of each variable.
	/// The bounds are calculated using a pre-computed BMC encoding of a VAS model.
	fn variable_bounds(
		&self,
		bmc_encoding: &BMCEncoding,
		bits: u32,
		max_steps: u32,
		backward: bool,
	) -> BMCBounds {
		BMCBounds::from_encoding(self, bmc_encoding, bits, max_steps, backward)
	}

	/// Runs general BMC for the given number of steps.
	fn run_bmc(
		&self,
		bmc_encoding: &BMCEncoding,
		max_steps: u32,
		backward: bool,
	) -> (ast::Bool, u32) {
		BMCEncoding::run_bmc(bmc_encoding, max_steps, backward)
	}
}
