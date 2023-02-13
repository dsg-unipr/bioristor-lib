use core::marker::PhantomData;
use core::usize;

use crate::algorithms::Algorithm;
use crate::losses::Loss;
use crate::model::Model;
use crate::params::Variables;
use crate::utils::{BestOrderedList, FloatRange};

/// Implementation of the adaptive algorithm.
///
/// # Type parameters
///
/// * `M` - The type of the model.
/// * `L` - The type of the loss.
/// * `MINIMA` - The number of minima to keep track of.
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Adaptive<M: Model, L: Loss, const MINIMA: usize> {
    /// The model to be solved.
    model: M,

    /// The parameters of the algorithm.
    params: AdaptiveParams,

    _t: PhantomData<L>,
}

impl<M: Model, L: Loss, const MINIMA: usize> Algorithm<M, AdaptiveParams, L>
    for Adaptive<M, L, MINIMA>
{
    /// Create a new instance of the adaptive algorithm.
    ///
    /// # Arguments
    ///
    /// * `model` - The model to be solved by the algorithm.
    /// * `params` - The parameters of the algorithm.
    fn new(model: M, params: AdaptiveParams) -> Self {
        Self {
            params,
            model,
            _t: PhantomData,
        }
    }

    /// Tries to solve the model for the given parameters using the adaptive
    /// algorithm and returns the best solution found.
    ///
    /// # Returns
    ///
    /// * `Some((vars, loss))` - The variables and the loss of the solution.
    /// * `None` - If the algorithm could not find a solution.
    fn run(&self) -> Option<(Variables, f32)> {
        let mut best: BestOrderedList<MINIMA> = Default::default();

        let mut support = self.params.concentration_guess;

        for _ in 0..self.params.max_iterations {
            best.clear();

            let c_start = support / 10.0;
            let c_end = support * 10.0;

            for c in FloatRange::new(c_start, c_end, self.params.concentration_steps) {
                for s in self.params.saturation_range.clone() {
                    for r in self.params.resistance_range.clone() {
                        // Evaluate the model for the given variables.
                        let vars = Variables {
                            concentration: c,
                            resistance: r,
                            saturation: s,
                        };
                        let (_, error) = self.model.value_with_loss::<L>(&vars);

                        // Add the solution to the best solutions.
                        best.add_solution((vars, error));
                    }
                }
            }

            let mean = best.mean_concentration();
            let center = (mean - c_start) / (c_end - c_start);

            if center > 0.5 {
                support *= 2.0;
            } else {
                support *= 0.5;
            }
        }

        Some(best.best())
    }
}

/// The parameters of the adaptive algorithm.
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct AdaptiveParams {
    /// The initial guessed value for the concentration.
    pub concentration_guess: f32,

    /// The number of steps in which the concentration interval is divided.
    pub concentration_steps: usize,

    /// The maximum number of iterations.
    pub max_iterations: usize,

    /// The range of water saturation to search.
    pub saturation_range: FloatRange,

    /// The range of wet drain-source resistance to search.
    pub resistance_range: FloatRange,
}

#[cfg(test)]
mod tests {
    use nalgebra::Vector3;

    use crate::losses;
    use crate::model::Model;
    use crate::params::{Currents, ModelParams};

    use super::*;

    struct ModelMock;

    impl Model for ModelMock {
        fn new(_: ModelParams, _: Currents) -> Self {
            Self
        }

        fn value_with_loss<L: Loss>(&self, variables: &Variables) -> (Vector3<f32>, f32) {
            (
                Vector3::new(
                    variables.concentration,
                    variables.resistance,
                    variables.saturation,
                ),
                L::evaluate(&[
                    (variables.concentration, 0.0),
                    (variables.resistance, 0.0),
                    (variables.saturation, 0.0),
                ]),
            )
        }
    }

    #[test]
    fn test_adaptive() {
        let params = AdaptiveParams {
            concentration_guess: 0.0,
            concentration_steps: 10,
            saturation_range: FloatRange::new(0.0, 10.0, 10),
            resistance_range: FloatRange::new(0.0, 10.0, 10),
            max_iterations: 10,
        };
        let model = ModelMock;

        let algorithm = Adaptive::<_, losses::SumRelative, 5>::new(model, params);
        let (vars, error) = algorithm.run().unwrap();

        assert_eq!(vars.concentration, 0.0);
        assert_eq!(vars.resistance, 0.0);
        assert_eq!(vars.saturation, 0.0);
        assert_eq!(error, 0.0);
    }
}
