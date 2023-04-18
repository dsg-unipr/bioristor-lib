use crate::{
    algorithms::Algorithm,
    losses::Loss,
    models::{EquationModel, Model},
    params::Variables,
    utils::{BestOrderedList, FloatRange},
};

/// The parameters of the adaptive algorithm.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Adaptive2Params {
    /// The range of concentrations to search.
    pub concentration_range: FloatRange,

    /// The maximum number of iterations.
    pub max_iterations: usize,

    /// The factor by which the range of concentrations is reduced after each
    /// iteration.
    pub reduction_factor: f32,

    /// The range of wet drain-source resistance to search.
    pub resistance_range: FloatRange,

    /// The range of water saturation to search.
    pub saturation_range: FloatRange,

    /// The error tolerance at which the algorithm stops.
    pub tolerance: f32,
}

/// Implementation of the adaptive algorithm v2 for the equation model.
///
/// # Type parameters
///
/// * `M` - The model to be solved.
/// * `L` - The loss function to be used.
/// * `MINIMA` - The number of minima over which the algorithm will average and
///     finds the optimal values for the variables.
pub struct Adaptive2Equation<M: Model, L: Loss, const MINIMA: usize> {
    /// The parameters of the algorithm.
    params: Adaptive2Params,

    /// The model to be solved.
    model: M,

    _t: core::marker::PhantomData<L>,
}

impl<M, L, const MINIMA: usize> Algorithm<Adaptive2Params, M> for Adaptive2Equation<M, L, MINIMA>
where
    M: EquationModel,
    L: Loss<ModelOutput = f32>,
{
    /// Create a new instance of the adaptive algorithm v2.
    ///
    /// # Arguments
    ///
    /// * `params` - The parameters of the algorithm.
    /// * `model` - The model to be solved by the algorithm.
    fn new(params: Adaptive2Params, model: M) -> Self {
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
        let mut best_list = BestOrderedList::<f32, MINIMA>::new();

        let mut range = self.params.concentration_range.clone();
        let mut range_semi_width = (range.end - range.start) * 0.5;
        let range_min = range.start;
        let range_max = range.end;
        let range_steps = range.steps;

        let mut error = f32::INFINITY;

        let mut iteration = 0;
        while iteration < self.params.max_iterations && error > self.params.tolerance {
            best_list.clear();

            // Perform a brute-force search.
            for concentration in range {
                // Evaluate the model for the given concentration.
                let err = L::evaluate(self.model.value(concentration));

                // Add the solution to the best solutions.
                best_list.add_solution((concentration, err));
            }

            let mean = best_list.mean_concentration();
            error = L::evaluate(self.model.value(mean));

            range_semi_width *= self.params.reduction_factor;
            range = FloatRange::new(
                (mean - range_semi_width).max(range_min),
                (mean + range_semi_width).min(range_max),
                range_steps,
            );

            iteration += 1;
        }

        let best = best_list.best();
        Some((
            Variables {
                concentration: best,
                resistance: self.model.resistance(best),
                saturation: self.model.saturation(best),
            },
            L::evaluate(self.model.value(best)),
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        losses::Absolute,
        models::{EquationModel, Model},
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

    #[test]
    fn test_adaptive2_equation() {
        let params = Adaptive2Params {
            concentration_range: FloatRange::new(0.0, 10.0, 10),
            max_iterations: 10,
            reduction_factor: 0.5,
            resistance_range: FloatRange::new(0.0, 10.0, 10),
            saturation_range: FloatRange::new(0.0, 10.0, 10),
            tolerance: 1e-3,
        };
        let model = EquationModelMock;

        let algorithm = Adaptive2Equation::<_, Absolute, 5>::new(params, model);
        let (variables, error) = algorithm.run().unwrap();

        assert!((variables.concentration - 2.0).abs() < 1e-3);
        assert!((variables.resistance - 2.0).abs() < 1e-3);
        assert!((variables.saturation - 2.0).abs() < 1e-3);
        assert!(error.abs() < 1e-3);
    }
}
