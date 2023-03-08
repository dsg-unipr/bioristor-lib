use crate::{
    algorithms::Algorithm,
    losses::Loss,
    models::{Model, SystemModel},
    params::Variables,
    utils::FloatRange,
};

/// The parameters of the brute force algorithm.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct BruteForceParams {
    /// The range of concentrations to search.
    pub concentration_range: FloatRange,

    /// The range of wet drain-source resistance to search.
    pub resistance_range: FloatRange,

    /// The range of water saturation to search.
    pub saturation_range: FloatRange,
}

/// Implementation of the brute force algorithm for the system model.
///
/// # Type parameters
///
/// * `M` - The type of the model.
/// * `L` - The type of the loss.
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct BruteForceSystem<M: Model, L: Loss> {
    /// The parameters of the algorithm.
    params: BruteForceParams,

    /// The model to be solved.
    model: M,

    _t: core::marker::PhantomData<L>,
}

impl<M, L> Algorithm<BruteForceParams, M> for BruteForceSystem<M, L>
where
    M: SystemModel,
    L: Loss<ModelOutput = [(f32, f32); 3]>,
{
    /// Create a new instance of the brute force algorithm.
    ///
    /// # Arguments
    ///
    /// * `params` - The parameters of the algorithm.
    /// * `model` - The model to be solved by the algorithm.
    fn new(params: BruteForceParams, model: M) -> Self {
        Self {
            params,
            model,
            _t: core::marker::PhantomData,
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

                    let error = L::evaluate(self.model.value(vars));

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

#[cfg(test)]
mod tests {
    use crate::{
        losses::SumRelative,
        models::{Model, SystemModel},
        params::{Currents, ModelParams},
    };

    use super::*;

    struct SystemModelMock;

    impl Model for SystemModelMock {
        fn new(_: ModelParams, _: Currents) -> Self {
            Self
        }

        fn params(&self) -> &ModelParams {
            unimplemented!()
        }

        fn currents(&self) -> &Currents {
            unimplemented!()
        }
    }

    impl SystemModel for SystemModelMock {
        fn value(&self, vars: Variables) -> [(f32, f32); 3] {
            [
                (vars.concentration, 0.0),
                (vars.resistance, 0.0),
                (vars.saturation, 0.0),
            ]
        }

        fn jacobian(&self, _: Variables) -> nalgebra::Matrix3<f32> {
            unimplemented!()
        }
    }

    #[test]
    fn test_brute_force() {
        let params = BruteForceParams {
            concentration_range: FloatRange::new(0.0, 1.0, 10),
            resistance_range: FloatRange::new(0.0, 1.0, 10),
            saturation_range: FloatRange::new(0.0, 1.0, 10),
        };
        let model = SystemModelMock;

        let algorithm = BruteForceSystem::<_, SumRelative>::new(params, model);
        let (vars, error) = algorithm.run().unwrap();

        assert_eq!(vars.concentration, 0.0);
        assert_eq!(vars.resistance, 0.0);
        assert_eq!(vars.saturation, 0.0);
        assert_eq!(error, 0.0);
    }
}
