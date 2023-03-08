pub use equation::*;
pub use system::*;

mod equation;
mod system;

#[allow(unused_imports)]
use micromath::F32Ext;

use crate::params::{Currents, ModelParams};

/// Common trait for all the formulations of the mathematical model
/// of the Bioristor device.
///
/// This trait is implemented by the [`Equation`] and [`System`] structs, that
/// provide a formulation of the mathematical model of the Bioristor device.
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

    /// Returns a reference to the parameters of the mathematical model.
    ///
    /// # Returns
    ///
    /// A reference to the parameters of the mathematical model.
    fn params(&self) -> &ModelParams;

    /// Returns a reference to the output currents of the device.
    ///
    /// # Returns
    ///
    /// A reference to the output currents of the device.
    fn currents(&self) -> &Currents;

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

    /// Calculates the gradient of the modulation of the channel.
    ///
    /// # Arguments
    ///
    /// * `concentration` - The concentration of ions in the electrolyte [Molarity].
    ///
    /// # Returns
    ///
    /// The first derivative of the modulation of the channel.
    #[inline]
    fn modulation_gradient(&self, _: f32) -> f32 {
        7.0
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
        let concentration = concentration * 58e3;
        0.0123 + 3647.5 / concentration.powf(0.955)
    }

    /// Calculates the gradient of the resistivity of a path.
    ///
    /// # Arguments
    ///
    /// * `concentration` - The concentration of ions in the electrolyte [Molarity].
    ///
    /// # Returns
    ///
    /// The first derivative of the resistivity of a path.
    #[inline]
    fn resistivity_gradient(&self, concentration: f32) -> f32 {
        // Transform concentration to Parts Per Million (PPM).
        let concentration = concentration * 58e3;
        -3483.3625 / concentration.powf(1.955)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::{Currents, Geometrics, ModelParams, Voltages};

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

    struct ModelMock;

    impl Model for ModelMock {
        fn new(_: ModelParams, _: Currents) -> Self {
            ModelMock
        }

        fn params(&self) -> &ModelParams {
            todo!()
        }

        fn currents(&self) -> &Currents {
            todo!()
        }
    }

    #[test]
    fn test_modulation() {
        let (params, currents) = mock_params();
        let model = ModelMock::new(params, currents);

        assert!((model.modulation(10.0) - 70.0).abs() < 1e-9);
        assert!((model.modulation_gradient(20.0) - 7.0).abs() < 1e-9);
    }

    #[test]
    fn test_resistivity() {
        let (params, currents) = mock_params();
        let model = ModelMock::new(params, currents);

        assert!((model.resistivity(1.0) - 0.115_320).abs() < 1e-6);
        assert!((model.resistivity_gradient(2.0) + 0.000_000_438).abs() < 1e-9);
    }
}
