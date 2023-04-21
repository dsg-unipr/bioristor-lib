#[allow(unused_imports)]
use micromath::F32Ext;

use crate::{
    algorithms::Algorithm,
    losses::Loss,
    models::{EquationModel, Model},
    params::Variables,
};

/// The parameters of the gradient descent algorithm.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct GradientDescentParams {
    /// The initial guessed value for the concentration.
    pub concentration_init: f32,

    /// The minimum value of the gradient at which the algorithm stops.
    pub grad_tolerance: f32,

    /// The initial learning rate.
    /// This is used in the first iteration and is updated in every iteration
    /// using the Barzilai–Borwein method.
    pub learning_rate_init: f32,

    /// The maximum number of iterations.
    pub max_iterations: usize,

    /// The error tolerance at which the algorithm stops.
    pub tolerance: f32,
}

/// Implementation of the gradient descent algorithm for the equation model.
///
/// # Type parameters
///
/// * `M` - The type of the model.
/// * `L` - The loss function to be used.
pub struct GradientDescentEquation<M: Model, L: Loss> {
    /// The parameters of the algorithm.
    params: GradientDescentParams,

    /// The model to be solved.
    model: M,

    _t: core::marker::PhantomData<L>,
}

impl<M, L> Algorithm<GradientDescentParams, M> for GradientDescentEquation<M, L>
where
    M: EquationModel,
    L: Loss<ModelOutput = f32>,
{
    /// Create a new instance of the gradient descent algorithm.
    ///
    /// # Arguments
    ///
    /// * `params` - The parameters of the algorithm.
    /// * `model` - The model to be solved by the algorithm.
    fn new(params: GradientDescentParams, model: M) -> Self {
        Self {
            params,
            model,
            _t: core::marker::PhantomData,
        }
    }

    /// Tries to solve the model for the given parameters using the gradient
    /// descent algorithm and returns the best solution found.
    ///
    /// # Returns
    ///
    /// * `Some((vars, loss))` - The variables and the loss of the solution.
    /// * `None` - If the algorithm could not find a solution.
    fn run(&self) -> Option<(Variables, f32)> {
        // The search for the minima of the squared function f²(x) is equivalent
        // to the search for the zeros in the initial function f(x).
        let gradient = |x: f32| -> f32 {
            let f = self.model.value(x);
            let df = self.model.gradient(x);
            2.0 * f * df
        };

        // Initialize variable with starting point.
        let mut c = self.params.concentration_init;
        let mut c_prev;

        let mut grad = gradient(c);
        let mut grad_prev;

        let mut learning_rate = self.params.learning_rate_init;

        // Initialize error with loss at starting point.
        let mut error = L::evaluate(self.model.value(c));

        // Loop until the maximum number of iterations is reached, the error
        // subceeds a certain tolerance, or the gradient becomes too small.
        let mut iterations = 0;
        while iterations < self.params.max_iterations
            && error > self.params.tolerance
            && grad.abs() > self.params.grad_tolerance
        {
            // Save previous values.
            c_prev = c;
            grad_prev = grad;

            // Update variable based on gradient and learning rate.
            c -= learning_rate * grad;
            grad = gradient(c);

            // Update learning rate using the Barzilai–Borwein method.
            learning_rate = ((c - c_prev) * (grad - grad_prev)).abs() / (grad - grad_prev).powi(2);

            error = L::evaluate(self.model.value(c));

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
    use crate::{
        losses::Absolute,
        models::Model,
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
    fn test_gradient_descent_equation() {
        let params = GradientDescentParams {
            concentration_init: 1.0,
            grad_tolerance: 1e-9,
            learning_rate_init: 0.2,
            max_iterations: 100,
            tolerance: 1e-6,
        };
        let model = EquationModelMock;

        let algorithm = GradientDescentEquation::<_, Absolute>::new(params, model);
        let (variables, error) = algorithm.run().unwrap();

        assert!((variables.concentration - 2.0).abs() < 1e-3);
        assert!((variables.resistance - 2.0).abs() < 1e-3);
        assert!((variables.saturation - 2.0).abs() < 1e-3);
        assert!(error.abs() < 1e-6);
    }
}
