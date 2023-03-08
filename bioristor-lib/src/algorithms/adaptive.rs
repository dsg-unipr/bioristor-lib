use crate::{
    algorithms::Algorithm,
    losses::Loss,
    models::{Model, SystemModel},
    params::Variables,
    utils::{BestOrderedList, FloatRange},
};

/// The parameters of the adaptive algorithm.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct AdaptiveParams {
    /// The initial guessed value for the concentration.
    pub concentration_init: f32,

    /// The number of steps in which the concentration interval is divided.
    pub concentration_steps: usize,

    /// The maximum number of iterations.
    pub max_iterations: usize,

    /// The number of minima over which the algorithm will average and finds the
    /// optimal values for the variables.
    pub minima_number: usize,

    /// The range of water saturation to search.
    pub saturation_range: FloatRange,

    /// The range of wet drain-source resistance to search.
    pub resistance_range: FloatRange,
}

/// Implementation of the adaptive algorithm for the system model.
///
/// # Type parameters
///
/// * `M` - The type of the model.
/// * `L` - The type of the loss.
/// * `MINIMA` - The number of minima to keep track of.
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct AdaptiveSystem<M: Model, L: Loss, const MINIMA: usize> {
    /// The parameters of the algorithm.
    params: AdaptiveParams,

    /// The model to be solved.
    model: M,

    _t: core::marker::PhantomData<L>,
}

impl<M, L, const MINIMA: usize> Algorithm<AdaptiveParams, M> for AdaptiveSystem<M, L, MINIMA>
where
    M: SystemModel,
    L: Loss<ModelOutput = [(f32, f32); 3]>,
{
    /// Create a new instance of the adaptive algorithm.
    ///
    /// # Arguments
    ///
    /// * `params` - The parameters of the algorithm.
    /// * `model` - The model to be solved by the algorithm.
    fn new(params: AdaptiveParams, model: M) -> Self {
        Self {
            params,
            model,
            _t: core::marker::PhantomData,
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

        let mut support = self.params.concentration_init;

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
                        let error = L::evaluate(self.model.value(vars));

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
    fn test_adaptive_system() {
        let params = AdaptiveParams {
            concentration_init: 0.0,
            concentration_steps: 10,
            max_iterations: 10,
            minima_number: 5,
            saturation_range: FloatRange::new(0.0, 10.0, 10),
            resistance_range: FloatRange::new(0.0, 10.0, 10),
        };
        let model = SystemModelMock;

        let algorithm = AdaptiveSystem::<_, SumRelative, 5>::new(params, model);
        let (vars, error) = algorithm.run().unwrap();

        assert_eq!(vars.concentration, 0.0);
        assert_eq!(vars.resistance, 0.0);
        assert_eq!(vars.saturation, 0.0);
        assert_eq!(error, 0.0);
    }
}
