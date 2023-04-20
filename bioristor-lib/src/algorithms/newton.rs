#[allow(unused_imports)]
use micromath::F32Ext;

use crate::{
    algorithms::Algorithm,
    losses::Loss,
    models::{EquationModel, Model},
    params::Variables,
};

/// The parameters of the Newton's method.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct NewtonParams {
    /// The initial guessed value for the concentration.
    pub concentration_init: f32,

    /// The minimum value of the gradient at which the algorithm stops.
    pub grad_tolerance: f32,

    /// The maximum number of iterations.
    pub max_iterations: usize,

    /// The error tolerance at which the algorithm stops.
    pub tolerance: f32,
}

/// Implementation of the Newton's method.
///
/// # Type parameters
///
/// * `M` - The type of the model.
/// * `L` - The loss function to be used.
pub struct NewtonEquation<M: Model, L: Loss> {
    /// The parameters of the algorithm.
    params: NewtonParams,

    /// The model to be solved.
    model: M,

    _t: core::marker::PhantomData<L>,
}

impl<M, L> Algorithm<NewtonParams, M> for NewtonEquation<M, L>
where
    M: EquationModel,
    L: Loss<ModelOutput = f32>,
{
    /// Create a new instance of the Newton's method.
    ///
    /// # Arguments
    ///
    /// * `params` - The parameters of the algorithm.
    /// * `model` - The model to be solved by the algorithm.
    fn new(params: NewtonParams, model: M) -> Self {
        Self {
            params,
            model,
            _t: core::marker::PhantomData,
        }
    }

    /// Tries to solve the model for the given parameters using the Newton's
    /// method and returns the best solution found.
    ///
    /// # Returns
    ///
    /// * `Some((vars, loss))` - The variables and the loss of the solution.
    /// * `None` - If the algorithm could not find a solution.
    fn run(&self) -> Option<(Variables, f32)> {
        // Initialize variable and gradient with starting point.
        let mut c = self.params.concentration_init;
        let mut grad = self.model.gradient(c);

        // Initialize the value of the function at starting point.
        let mut value = self.model.value(c);
        let mut error = L::evaluate(value);

        // Loop until the maximum number of iterations is reached, the error
        // subceeds a certain tolerance, or the gradient becomes too small.
        let mut iterations = 0;
        while iterations < self.params.max_iterations
            && error > self.params.tolerance
            && grad.abs() > self.params.grad_tolerance
        {
            // Update variable and gradient.
            c -= value / grad;
            grad = self.model.gradient(c);

            // Update the function value and loss.
            value = self.model.value(c);
            error = L::evaluate(value);

            iterations += 1;
        }

        Some((
            Variables {
                concentration: c,
                resistance: self.model.resistance(c),
                saturation: self.model.saturation(c),
            },
            error,
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::losses::Absolute;
    use crate::models::Model;
    use crate::params::{Currents, ModelParams};

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
        fn value(&self, x: f32) -> f32 {
            x.cos() - x.powi(3)
        }

        fn gradient(&self, x: f32) -> f32 {
            -3.0 * x.powi(2) - x.sin()
        }

        fn resistance(&self, x: f32) -> f32 {
            x
        }

        fn saturation(&self, x: f32) -> f32 {
            x
        }
    }

    #[test]
    fn test_newton_equation() {
        let params = NewtonParams {
            concentration_init: 0.5,
            grad_tolerance: 1e-6,
            max_iterations: 20,
            tolerance: 1e-6,
        };
        let model = EquationModelMock;

        let algorithm = NewtonEquation::<_, Absolute>::new(params, model);
        let (variables, error) = algorithm.run().unwrap();

        assert!((variables.concentration - 0.865_474_03).abs() < 1e-6);
        assert!((variables.resistance - 0.865_474_03).abs() < 1e-6);
        assert!((variables.saturation - 0.865_474_03).abs() < 1e-6);
        assert!(error.abs() < 1e-6);
    }
}
