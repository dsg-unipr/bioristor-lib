#[allow(unused_imports)]
use micromath::F32Ext;
use nalgebra::Vector3;

use crate::{
    losses::Loss,
    params::{Currents, ModelParams, Variables},
};

/// Mathematical model of the Bioristor device.
///
/// This trait is implemented by the [`ThreeEquations`] struct, that provides a
/// formulation of the mathematical model using a system of three equations that
/// depend on three variables: concentration, resistance, and saturation.
///
/// # Example
///
/// ```
/// use bioristor_lib::losses::MeanRelative;
/// use bioristor_lib::model::{Model, ThreeEquations};
/// use bioristor_lib::params::{Currents, Geometrics, ModelParams, Variables, Voltages};
///
/// const PARAMS: ModelParams = ModelParams {
///     geometrics: Geometrics {
///         cross_sectional_area: 1.0,
///         length: 2.0,
///     },
///     r_ds_dry: 3.0,
///     vessels_number: 4.0,
///     voltages: Voltages {
///         v_ds: 5.0,
///         v_gs: 6.0,
///     },
/// };
/// let currents = Currents {
///     i_ds_min: 7.0,
///     i_ds_max: 8.0,
///     i_gs: 9.0,
/// };
///
/// let model = ThreeEquations::new(PARAMS, currents);
///
/// let variables = Variables {
///     concentration: 10.0,
///     resistance: 11.0,
///     saturation: 12.0,
/// };
/// let value = model.value_with_loss::<MeanRelative>(&variables);
/// ```
pub trait Model {
    /// Creates a new instance of the model.
    ///
    /// # Arguments
    ///
    /// * `params` - The parameters of the mathematical model.
    /// * `currents` - The output currents of the devices,
    ///     i.e. the independent variables of the model.
    ///
    /// # Returns
    ///
    /// A new instance of the model.
    fn new(params: ModelParams, currents: Currents) -> Self;

    /// Calculates the value of the model and loss for the given variables.
    ///
    /// # Arguments
    ///
    /// * `variables` - The dependent variables of the model.
    ///
    /// # Returns
    ///
    /// A tuple containing the value of the model and the loss.
    fn value_with_loss<L: Loss>(&self, variables: &Variables) -> (Vector3<f32>, f32);
}

/// Implementation of the mathematical model a system of three equations that
/// depend on three variables: concentration, resistance, and saturation.
///
/// The output of the equation in implicit form is taken as the error of
/// a particular solution.
pub struct ThreeEquations {
    /// The parameters of the mathematical model.
    params: ModelParams,

    /// The output currents of the devices.
    currents: Currents,
}

impl ThreeEquations {
    fn raw_value(&self, variables: &Variables) -> [(f32, f32); 3] {
        [
            (
                self.currents.i_ds_max,
                self.currents.i_gs
                    + self.params.voltages.v_ds
                        / (self.params.r_ds_dry
                            + ((variables.resistance * self.params.vessels_number)
                                / (self.modulation(variables.concentration) + 1.0)
                                - self.params.r_ds_dry)
                                * variables.saturation),
            ),
            (
                self.currents.i_ds_min,
                self.params.voltages.v_ds
                    / (self.params.r_ds_dry
                        + (variables.resistance * self.params.vessels_number
                            - self.params.r_ds_dry)
                            * variables.saturation),
            ),
            (
                self.currents.i_gs,
                self.params.voltages.v_gs
                    * (self.params.geometrics.cross_sectional_area
                        / (self.resistivity(variables.concentration)
                            * self.params.geometrics.length))
                    * self.params.vessels_number
                    * variables.saturation,
            ),
        ]
    }
}

impl Model for ThreeEquations {
    fn new(params: ModelParams, currents: Currents) -> Self {
        Self { params, currents }
    }

    fn value_with_loss<L: Loss>(&self, variables: &Variables) -> (Vector3<f32>, f32) {
        let raw_value = self.raw_value(variables);
        (
            Vector3::new(
                raw_value[0].0 - raw_value[0].1,
                raw_value[1].0 - raw_value[1].1,
                raw_value[2].0 - raw_value[2].1,
            ),
            L::evaluate(&raw_value),
        )
    }
}

impl ThreeEquations {
    /// Calculates the modulation of the channel.
    ///
    /// # Arguments
    ///
    /// * `concentration` - The concentration of ions in the electrolyte [Molarity].
    ///
    /// # Returns
    ///
    /// The modulation of the channel.
    #[inline]
    fn modulation(&self, concentration: f32) -> f32 {
        7.0 * concentration
    }

    /// Calculates the resistivity of a path.
    ///
    /// # Arguments
    ///
    /// * `concentration` - The concentration of ions in the electrolyte [Molarity].
    ///
    /// # Returns
    ///
    /// The resistivity of a path [Ohm * Meter].
    #[inline]
    fn resistivity(&self, concentration: f32) -> f32 {
        // Transform concentration to Parts Per Million (PPM).
        0.0123 + 3647.5 / (58e3 * concentration).powf(0.955)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        losses,
        params::{Currents, Geometrics, Voltages},
    };

    use super::*;

    fn mock_params() -> (ModelParams, Currents) {
        (
            ModelParams {
                geometrics: Geometrics {
                    cross_sectional_area: 1.0,
                    length: 2.0,
                },
                r_ds_dry: 3.0,
                vessels_number: 4.0,
                voltages: Voltages {
                    v_ds: 5.0,
                    v_gs: 6.0,
                },
            },
            Currents {
                i_ds_min: 7.0,
                i_ds_max: 8.0,
                i_gs: 9.0,
            },
        )
    }

    #[test]
    fn test_three_equations() {
        let (params, currents) = mock_params();
        let model = ThreeEquations::new(params, currents);

        assert_eq!(model.params.geometrics.cross_sectional_area, 1.0);
        assert_eq!(model.params.geometrics.length, 2.0);
        assert_eq!(model.params.r_ds_dry, 3.0);
        assert_eq!(model.params.vessels_number, 4.0);
        assert_eq!(model.params.voltages.v_ds, 5.0);
        assert_eq!(model.params.voltages.v_gs, 6.0);
        assert_eq!(model.currents.i_ds_min, 7.0);
        assert_eq!(model.currents.i_ds_max, 8.0);
        assert_eq!(model.currents.i_gs, 9.0);
    }

    #[test]
    fn test_raw_value() {
        let (params, currents) = mock_params();
        let model = ThreeEquations::new(params, currents);

        let variables = Variables {
            concentration: 0.1,
            resistance: 0.2,
            saturation: 0.3,
        };
        let raw_values = model.raw_value(&variables);
        assert!((raw_values[0].0 - 8.0) < 1e-6);
        assert!((raw_values[0].1 - 11.230_971) < 1e-6);
        assert!((raw_values[1].0 - 7.0) < 1e-6);
        assert!((raw_values[1].1 - 2.136_752) < 1e-6);
        assert!((raw_values[2].0 - 9.0) < 1e-6);
        assert!((raw_values[2].1 - 3.825_316) < 1e-6);
    }

    #[test]
    fn test_value_with_loss() {
        let (params, currents) = mock_params();
        let model = ThreeEquations::new(params, currents);

        let variables = Variables {
            concentration: 0.1,
            resistance: 0.2,
            saturation: 0.3,
        };
        let (values, loss) = model.value_with_loss::<losses::SumRelative>(&variables);
        assert!((values.x + 3.230_971).abs() < 1e-6);
        assert!((values.y - 4.863_247).abs() < 1e-6);
        assert!((values.z - 5.174_683).abs() < 1e-6);
        assert!((loss - 1.103_756).abs() < 1e-6)
    }

    #[test]
    fn test_modulation() {
        let (params, currents) = mock_params();
        let model = ThreeEquations::new(params, currents);

        assert!((model.modulation(10.0) - 70.0).abs() < 1e-6);
    }

    #[test]
    fn test_resistivity() {
        let (params, currents) = mock_params();
        let model = ThreeEquations::new(params, currents);

        assert!((model.resistivity(1.0) - 0.115_320).abs() < 1e-6);
    }
}
