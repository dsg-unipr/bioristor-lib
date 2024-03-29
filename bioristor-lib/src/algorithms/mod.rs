mod adaptive;
mod adaptive2;
mod brute_force;
mod gradient_descent;
mod neural_network;
mod newton;

pub use adaptive::*;
pub use adaptive2::*;
pub use brute_force::*;
pub use gradient_descent::*;
pub use neural_network::*;
pub use newton::*;

use crate::models::Model;
use crate::params::Variables;

/// Common interface for algorithm implementations.
///
/// # Type parameters
///
/// * `P` - The type of the parameters of the algoprithm.
/// * `M` - The type of the model.
pub trait Algorithm<P: Sized, M: Model> {
    /// Create a new instance of the algorithm.
    ///
    /// # Arguments
    ///
    /// * `params` - The parameters of the algorithm.
    /// * `model` - The model to be solved by the algorithm.
    fn new(params: P, model: M) -> Self;

    /// Tries to solve the model for the given parameters using this algorithm
    /// and returns the best solution found.
    ///
    /// # Returns
    ///
    /// * `Some((vars, loss))` - The variables and the loss of the solution.
    /// * `None` - If the algorithm could not find a solution.
    fn run(&self) -> Option<(Variables, f32)>;
}
