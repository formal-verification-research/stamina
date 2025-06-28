pub(crate) enum ModelCheckingResult {
	NoResult, // checking has not yet been performed or was unable to occur
	LowerBound(f64), // A lower bound (Pmin)
	UpperBound(f64), // An upper bound (Pmax)
	ExactProbability(f64), // The exact probability
	ProbabilityRange(f64, f64), // A probabilistic range of Pmin to Pmax
	VariableValueResult(i64), // A result representing a variable value
}
impl ModelCheckingResult {
	// TODO: I would like to have these perform the model checking
	// but idk if that makes sense
	#[trusted]
	fn new() -> Self {
		unimplemented!();
	}
	#[trusted]
	fn valid(&self) -> bool {
		match self {
			NoResult => true,
			// Probabilities must be in the range of 0.0-1.0
			LowerBound(pmin) => pmin >= 0.0 && pmin <= 1.0,
			UpperBound(pmax) => pmax >= 0.0 && pmax <= 1.0,
			ExactProbability(p) => p >= 0.0 && p <= 1.0,
			// With the range result we add the additional restriction
			// of pmax >= pmin (in addition to both being valid probabilistic ranges
			ProbabilityRange(pmin, pmax) => pmin >= 0.0 && pmin <= 1.0
									  && pmax >= 0.0 && pmax <= 1.0
									  && pmax >= pmin,
			// We do not try to validate other types of results
			_ => true
		}
	}
}
