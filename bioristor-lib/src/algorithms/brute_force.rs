use core::marker::PhantomData;

use crate::algorithms::Algorithm;
use crate::losses::Loss;
use crate::model::Model;
use crate::params::Variables;
use crate::utils::FloatRange;

/// Implementation of the brute force algorithm.
///
/// # Type parameters
///
/// * `M` - The type of the model.
/// * `L` - The type of the loss.
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct BruteForce<M: Model, L: Loss> {
    /// The model to be solved.
    model: M,

    /// The parameters of the algorithm.
    params: BruteForceParams,

    _t: PhantomData<L>,
}

impl<M: Model, L: Loss> Algorithm<M, BruteForceParams, L> for BruteForce<M, L> {
    /// Create a new instance of the brute force algorithm.
    ///
    /// # Arguments
    ///
    /// * `model` - The model to be solved by the algorithm.
    /// * `params` - The parameters of the algorithm.
    fn new(model: M, params: BruteForceParams) -> Self {
        Self {
            model,
            params,
            _t: PhantomData,
        }
    }

    /// Tries to solve the model for the given parameters using the brute force
    /// algorithm and returns the best solution found.
    ///
    /// # Returns
    ///
    /// * `Some((vars, loss))` - The variables and the loss of the solution.
    /// * `None` - If the algorithm could not find a solution.
    fn run(&self) -> Option<(Variables, f32)> {
        let mut best: Option<(Variables, f32)> = None;

        for c in self.params.concentration_range.clone() {
            for r in self.params.resistance_range.clone() {
                for s in self.params.saturation_range.clone() {
                    let vars = Variables {
                        concentration: c,
                        resistance: r,
                        saturation: s,
                    };

                    let (_, error) = self.model.value_with_loss::<L>(&vars);

                    if let Some((_, best_error)) = best {
                        if error < best_error {
                            best = Some((vars, error));
                        }
                    } else {
                        best = Some((vars, error));
                    }
                }
            }
        }

        best
    }
}

/// The parameters of the brute force algorithm.
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct BruteForceParams {
    /// The range of concentrations to search.
    pub concentration_range: FloatRange,

    /// The range of wet drain-source resistance to search.
    pub resistance_range: FloatRange,

    /// The range of water saturation to search.
    pub saturation_range: FloatRange,
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
    fn test_brute_force() {
        let params = BruteForceParams {
            concentration_range: FloatRange::new(0.0, 1.0, 10),
            resistance_range: FloatRange::new(0.0, 1.0, 10),
            saturation_range: FloatRange::new(0.0, 1.0, 10),
        };
        let model = ModelMock;

        let algorithm = BruteForce::<_, losses::SumRelative>::new(model, params);
        let (vars, error) = algorithm.run().unwrap();

        assert_eq!(vars.concentration, 0.0);
        assert_eq!(vars.resistance, 0.0);
        assert_eq!(vars.saturation, 0.0);
        assert_eq!(error, 0.0);
    }
}
