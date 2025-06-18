use z3::{
	ast::{self},
	Config, Context,
};

use crate::{
	bmc::{bounds::BMCBounds, encoding::BMCEncoding},
	model::model::AbstractModel,
	AbstractVas,
};

// TODO: make this configurable by the user or calculated with the dependency graph.
const NUM_BITS: u32 = 9; // Default number of bits for variable representation
pub const MAX_BMC_STEPS: u32 = 1000; // Maximum number of BMC steps to take before giving up

/// Trait for Abstract VAS models to provide BMC-related functionality.
pub(crate) trait AbstractVasBmc<'a>: AbstractModel {
	/// Sets up the Z3 context for BMC.
	fn setup_z3(&mut self);
	/// Returns the formula for BMC plus the unroller.
	/// Order: (init_formula, transition_formula, target_formula, unroller)
	fn bmc_encoding(&'a self) -> BMCEncoding<'a>;
	/// Returns the variable bounds
	fn variable_bounds(&'a self, bmc_encoding: &BMCEncoding) -> BMCBounds;
	/// Runs general BMC for the given number of steps.
	fn run_bmc(&'a self, bmc_encoding: &BMCEncoding<'a>) -> (ast::Bool<'a>, u32);
}

impl<'a> AbstractVasBmc<'a> for AbstractVas {
	/// Sets up the Z3 context for BMC.
	fn setup_z3(&mut self) {
		let cfg = Config::new();
		let ctx = Context::new(&cfg);
		self.z3_context = Some(ctx);
	}

	/// Returns the formula for BMC plus the unroller.
	/// Order: (context, config, init_formula, transition_formula, target_formula, unroller)
	fn bmc_encoding(&'a self) -> BMCEncoding<'a> {
		let ctx: &'a Context = self
			.z3_context
			.as_ref()
			.expect("Z3 context not initialized");
		BMCEncoding::from_vas(self, ctx, NUM_BITS)
	}

	/// Returns the variable bounds for the VAS model.
	/// It computes both loose and tight bounds for upper and lower limits of each variable.
	/// The bounds are calculated using a pre-computed BMC encoding of a VAS model.
	fn variable_bounds(&'a self, bmc_encoding: &BMCEncoding) -> BMCBounds {
		let ctx: &'a Context = self
			.z3_context
			.as_ref()
			.expect("Z3 context not initialized");
		BMCBounds::from_encoding(self, bmc_encoding, ctx, NUM_BITS)
	}

	fn run_bmc(&'a self, bmc_encoding: &BMCEncoding<'a>) -> (ast::Bool<'a>, u32) {
		let ctx: &'a Context = self
			.z3_context
			.as_ref()
			.expect("Z3 context not initialized");
		BMCEncoding::run_bmc(bmc_encoding, ctx, MAX_BMC_STEPS)
	}
}
