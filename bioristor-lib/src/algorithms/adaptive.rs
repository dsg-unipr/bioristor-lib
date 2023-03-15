use crate::{
    algorithms::Algorithm,
    losses::Loss,
    models::{EquationModel, Model, SystemModel},
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

    /// The range of water saturation to search.
    pub saturation_range: FloatRange,

    /// The range of wet drain-source resistance to search.
    pub resistance_range: FloatRange,
}

/// Implementation of the adaptive algorithm for the equation model.
///
/// # Type parameters
///
/// * `M` - The model to be solved.
/// * `L` - The loss function to be used.
/// * `MINIMA` - The number of minima over which the algorithm will average and
///     finds the optimal values for the variables.
pub struct AdaptiveEquation<M: Model, L: Loss, const MINIMA: usize> {
    /// The parameters of the algorithm.
    params: AdaptiveParams,

    /// The model to be solved.
    model: M,

    _t: core::marker::PhantomData<L>,
}

impl<M, L, const MINIMA: usize> Algorithm<AdaptiveParams, M> for AdaptiveEquation<M, L, MINIMA>
where
    M: EquationModel,
    L: Loss<ModelOutput = f32>,
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
        // Best solutions found with their error.
        let mut best = BestOrderedList::<f32, MINIMA>::new();

        let mut support = self.params.concentration_init;

        for _ in 0..self.params.max_iterations {
            best.clear();

            let c_start = support / 10.0;
            let c_end = support * 10.0;

            // Perform a brute-force search.
            let range = FloatRange::new(c_start, c_end, self.params.concentration_steps);
            for concentration in range {
                // Evaluate the model for the given concentration.
                let error = L::evaluate(self.model.value(concentration));

                // Add the solution to the best solutions.
                best.add_solution((concentration, error));
            }

            let mean = best.mean_concentration();
            let center = (mean - c_start) / (c_end - c_start);

            if center > 0.5 {
                support *= 2.0;
            } else {
                support *= 0.5;
            }
        }

        let best = best.best();
        Some((
            Variables {
                concentration: best.0,
                resistance: self.model.resistance(best.0),
                saturation: self.model.saturation(best.0),
            },
            best.1,
        ))
    }
}

/// Implementation of the adaptive algorithm for the system model.
///
/// # Type parameters
///
/// * `M` - The type of the model.
/// * `L` - The type of the loss.
/// * `MINIMA` - The number of minima to keep track of.
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
        let mut best = BestOrderedList::<Variables, MINIMA>::new();

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
        losses::{Absolute, SumRelative},
        models::{Model, SystemModel},
        params::{Currents, ModelParams},
    };

    use super::*;

    struct EquationModelMock;

    impl Model for EquationModelMock {
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

    impl EquationModel for EquationModelMock {
        fn value(&self, concentration: f32) -> f32 {
            (concentration - 2.0).powi(2)
        }

        fn gradient(&self, concentration: f32) -> f32 {
            2.0 * (concentration - 2.0)
        }

        fn resistance(&self, concentration: f32) -> f32 {
            concentration
        }

        fn saturation(&self, concentration: f32) -> f32 {
            concentration
        }
    }

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
    fn test_adaptive_equation() {
        let params = AdaptiveParams {
            concentration_init: 1.0,
            concentration_steps: 500,
            max_iterations: 10,
            saturation_range: FloatRange::new(0.0, 10.0, 10),
            resistance_range: FloatRange::new(0.0, 10.0, 10),
        };
        let model = EquationModelMock;

        let algorithm = AdaptiveEquation::<_, Absolute, 5>::new(params, model);
        let (variables, error) = algorithm.run().unwrap();

        assert!((variables.concentration - 2.0).abs() < 1e-3);
        assert!((variables.resistance - 2.0).abs() < 1e-3);
        assert!((variables.saturation - 2.0).abs() < 1e-3);
        assert!(error.abs() < 1e-3);
    }

    #[test]
    fn test_adaptive_system() {
        let params = AdaptiveParams {
            concentration_init: 0.0,
            concentration_steps: 10,
            max_iterations: 10,
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
