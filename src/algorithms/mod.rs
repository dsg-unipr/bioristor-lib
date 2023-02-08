#[cfg(feature = "adaptive")]
mod adaptive;
#[cfg(feature = "brute-force")]
mod brute_force;

#[cfg(feature = "adaptive")]
pub use adaptive::*;
#[cfg(feature = "brute-force")]
pub use brute_force::*;

use crate::losses::Loss;
use crate::model::Model;
use crate::params::Variables;

/// Common interface for algorithm implementations.
///
/// # Type parameters
///
/// * `P` - The type of the parameters of the algoprithm.
/// * `M` - The type of the model.
/// * `L` - The type of the loss.
pub trait Algorithm<M: Model, P: Sized, L: Loss> {
    /// Create a new instance of the algorithm.
    ///
    /// # Arguments
    ///
    /// * `model` - The model to be solved by the algorithm.
    /// * `params` - The parameters of the algorithm.
    fn new(model: M, params: P) -> Self;

    /// Tries to solve the model for the given parameters using this algorithm
    /// and returns the best solution found.
    ///
    /// # Returns
    ///
    /// * `Some((vars, loss))` - The variables and the loss of the solution.
    /// * `None` - If the algorithm could not find a solution.
    fn run(&self) -> Option<(Variables, f32)>;
}
