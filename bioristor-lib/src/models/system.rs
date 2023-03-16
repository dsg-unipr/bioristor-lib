#[allow(unused_imports)]
use micromath::F32Ext;
use nalgebra::Matrix3;

use crate::{
    models::Model,
    params::{Currents, ModelParams, Variables},
};

/// Formulation of the mathematical model as a system of three equations that
/// depend on three variables: ions concentration, resistance of the wet channel
/// when the gate is off, and water saturation.
pub trait SystemModel: Model {
    /// Calculates the output value of the model for the given variables.
    ///
    /// # Arguments
    ///
    /// * `variables` - The dependent variables of the mathematical model.
    ///
    /// # Returns
    ///
    /// The output value of the model.
    fn value(&self, variables: Variables) -> [(f32, f32); 3];

    /// Calculates the Jacobian matrix of the model for the given variables.
    ///
    /// # Arguments
    ///
    /// * `variables` - The dependent variables of the model.
    ///
    /// # Returns
    ///
    /// The Jacobian matrix of the model.
    fn jacobian(&self, variables: Variables) -> Matrix3<f32>;
}

/// Implementation of the mathematical model using a system of three equations
/// that depend on three variables: ions concentration, resistance of the wet
/// channel when the gate is off, and water saturation.
///
/// # Example
///
/// ```
/// use bioristor_lib::losses::{Loss, MeanRelative};
/// use bioristor_lib::models::{Model, System, SystemModel};
/// use bioristor_lib::params::{
///     Currents, ModelParams, ModulationParams, StemResistanceInvParams, Variables, Voltages,
/// };
///
/// const PARAMS: ModelParams = ModelParams {
///     mod_params: ModulationParams(1.0, 2.0, 3.0),
///     r_dry: 4.0,
///     res_params: StemResistanceInvParams(5.0, 6.0),
///     voltages: Voltages {
///         v_ds: 7.0,
///         v_gs: 8.0,
///     },
/// };
/// let currents = Currents {
///     i_ds_off: 9.0,
///     i_ds_on: 10.0,
///     i_gs_on: 11.0,
/// };
///
/// let model = System::new(PARAMS, currents);
///
/// let variables = Variables {
///     concentration: 10.0,
///     resistance: 11.0,
///     saturation: 12.0,
/// };
/// let value = model.value(variables);
/// let error = MeanRelative::evaluate(value);
/// ```
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct System {
    /// The parameters of the mathematical model.
    params: ModelParams,

    /// The output currents of the devices.
    currents: Currents,
}

impl Model for System {
    fn new(params: ModelParams, currents: Currents) -> Self {
        Self { params, currents }
    }

    fn params(&self) -> &ModelParams {
        &self.params
    }

    fn currents(&self) -> &Currents {
        &self.currents
    }
}

impl SystemModel for System {
    fn value(&self, variables: Variables) -> [(f32, f32); 3] {
        [
            (
                self.currents.i_ds_on,
                self.currents.i_gs_on
                    + self.params.voltages.v_ds
                        / (self.params.r_dry
                            + variables.saturation
                                * (variables.resistance
                                    / (self.modulation(variables.concentration) + 1.0)
                                    - self.params.r_dry)),
            ),
            (
                self.currents.i_ds_off,
                self.params.voltages.v_ds
                    / (self.params.r_dry
                        + variables.saturation * (variables.resistance - self.params.r_dry)),
            ),
            (
                self.currents.i_gs_on,
                self.params.voltages.v_gs
                    * variables.saturation
                    * self.stem_resistance_inv(variables.concentration),
            ),
        ]
    }

    fn jacobian(&self, variables: Variables) -> Matrix3<f32> {
        let m = self.modulation(variables.concentration);
        let dm = self.modulation_gradient(variables.concentration);
        let r = self.stem_resistance_inv(variables.concentration);
        let dr = self.stem_resistance_inv_gradient(variables.concentration);

        let denominator1 = (self.params.r_dry
            - variables.saturation * (self.params.r_dry - variables.resistance / (m + 1.0)))
            .powi(2);
        let denominator2 = (self.params.r_dry
            + variables.saturation * (variables.resistance - self.params.r_dry))
            .powi(2);

        Matrix3::new(
            -(variables.resistance * variables.saturation * self.params.voltages.v_ds * dm)
                / ((m + 1.0).powi(2) * denominator1),
            (variables.saturation * self.params.voltages.v_ds) / ((m + 1.0) * denominator1),
            -(self.params.voltages.v_ds * (self.params.r_dry - variables.resistance / (m + 1.0)))
                / denominator1,
            0.0,
            (variables.saturation * self.params.voltages.v_ds) / denominator2,
            (self.params.voltages.v_ds * (variables.resistance - self.params.r_dry)) / denominator2,
            -variables.saturation * self.params.voltages.v_gs * dr,
            0.0,
            -self.params.voltages.v_gs * r,
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::params::{Currents, ModulationParams, StemResistanceInvParams, Voltages};

    use super::*;

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

    #[test]
    fn test_model() {
        let (params, currents) = mock_params();
        let model = System::new(params, currents);

        assert_eq!(model.params().mod_params.0, 1.0);
        assert_eq!(model.params().mod_params.1, 2.0);
        assert_eq!(model.params().mod_params.2, 3.0);
        assert_eq!(model.params().r_dry, 4.0);
        assert_eq!(model.params().res_params.0, 5.0);
        assert_eq!(model.params().res_params.1, 6.0);
        assert_eq!(model.params().voltages.v_ds, 7.0);
        assert_eq!(model.params().voltages.v_gs, 8.0);
        assert_eq!(model.currents().i_ds_off, 9.0);
        assert_eq!(model.currents().i_ds_on, 10.0);
        assert_eq!(model.currents().i_gs_on, 11.0);
    }

    #[test]
    fn test_value() {
        let (params, currents) = mock_params();
        let model = System::new(params, currents);

        let variables = Variables {
            concentration: 0.1,
            resistance: 0.2,
            saturation: 0.3,
        };
        let value = model.value(variables);
        assert!((value[0].0 - 10.0) < 1e-9);
        assert!((value[0].1 - 13.610_743_8) < 1e-6);
        assert!((value[1].0 - 9.0) < 1e-9);
        assert!((value[1].1 - 2.447_552_4) < 1e-6);
        assert!((value[2].0 - 11.0) < 1e-9);
        assert!((value[2].1 - 13.597_211) < 1e-5);
    }

    #[test]
    fn test_jacobian() {
        let (params, currents) = mock_params();
        let model = System::new(params, currents);

        let variables = Variables {
            concentration: 0.1,
            resistance: 0.2,
            saturation: 0.3,
        };
        let jacobian = model.jacobian(variables);
        assert!((jacobian.m11 + 4.807_57).abs() < 1e-5);
        assert!((jacobian.m12 + 0.578_247_87).abs() < 1e-6);
        assert!((jacobian.m13 + 4.280_346_3).abs() < 1e-6);
        assert!((jacobian.m21 - 0.0).abs() < 1e-6);
        assert!((jacobian.m22 - 0.256_736_27).abs() < 1e-6);
        assert!((jacobian.m23 + 3.251_992_7).abs() < 1e-6);
        assert!((jacobian.m31 + 15.253_372).abs() < 1e-6);
        assert!((jacobian.m32 - 0.0).abs() < 1e-6);
        assert!((jacobian.m33 + 45.324_03).abs() < 1e-5);
    }
}
