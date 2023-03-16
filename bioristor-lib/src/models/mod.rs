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
        let params = self.params().mod_params;
        params.0 * concentration + params.1 * concentration.ln() + params.2
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
    fn modulation_gradient(&self, concentration: f32) -> f32 {
        let params = self.params().mod_params;
        params.0 + params.1 * concentration.recip()
    }

    /// Calculates the inverse (reciprocal) of the stem resistance.
    ///
    /// # Arguments
    ///
    /// * `concentration` - The concentration of ions in the electrolyte [Molarity].
    ///
    /// # Returns
    ///
    /// The reciprocal of the stem resistance [1 / Ohm].
    #[inline]
    fn stem_resistance_inv(&self, concentration: f32) -> f32 {
        let params = self.params().res_params;
        params.0 + params.1 * concentration.powf(0.955)
    }

    /// Calculates the gradient of the inverse of the stem resistance.
    ///
    /// # Arguments
    ///
    /// * `concentration` - The concentration of ions in the electrolyte [Molarity].
    ///
    /// # Returns
    ///
    /// The first derivative of the inverse of the stem resistance.
    #[inline]
    fn stem_resistance_inv_gradient(&self, concentration: f32) -> f32 {
        let params = self.params().res_params;
        params.1 * 0.955 * concentration.powf(-0.045)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::{
        Currents, ModelParams, ModulationParams, StemResistanceInvParams, Voltages,
    };

    fn mock_params() -> (ModelParams, Currents) {
        (
            ModelParams {
                mod_params: ModulationParams(1.0, 2.0, 3.0),
                r_dry: 4.0,
                res_params: StemResistanceInvParams(5.0, 6.0),
                voltages: Voltages {
                    v_ds: 7.0,
                    v_gs: 8.0,
                },
            },
            Currents {
                i_ds_off: 9.0,
                i_ds_on: 10.0,
                i_gs_on: 11.0,
            },
        )
    }

    struct ModelMock {
        params: ModelParams,
        currents: Currents,
    }

    impl Model for ModelMock {
        fn new(params: ModelParams, currents: Currents) -> Self {
            ModelMock { params, currents }
        }

        fn params(&self) -> &ModelParams {
            &self.params
        }

        fn currents(&self) -> &Currents {
            &self.currents
        }
    }

    #[test]
    fn test_modulation() {
        let (params, currents) = mock_params();
        let model = ModelMock::new(params, currents);

        assert!((model.modulation(10.0) - 17.605_17).abs() < 1e-4);
        assert!((model.modulation_gradient(10.0) - 1.2).abs() < 1e-9);
    }

    #[test]
    fn test_resistivity() {
        let (params, currents) = mock_params();
        let model = ModelMock::new(params, currents);

        assert!((model.stem_resistance_inv(10.0) - 59.094_26).abs() < 1e-4);
        assert!((model.stem_resistance_inv_gradient(10.0) - 5.166_002_6).abs() < 1e-6);
    }
}
