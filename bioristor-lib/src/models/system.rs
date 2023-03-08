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

    fn jacobian(&self, variables: Variables) -> Matrix3<f32> {
        let m = self.modulation(variables.concentration);
        let dm = self.modulation_gradient(variables.concentration);
        let r = self.resistivity(variables.concentration);
        let dr = self.resistivity_gradient(variables.concentration);

        let denominator1 = (self.params.r_ds_dry
            - variables.saturation
                * (self.params.r_ds_dry
                    - (self.params.vessels_number * variables.resistance) / (m + 1.0)))
            .powi(2);
        let denominator2 = (self.params.r_ds_dry
            - variables.saturation
                * (self.params.r_ds_dry - self.params.vessels_number * variables.resistance))
            .powi(2);

        Matrix3::new(
            -(self.params.vessels_number
                * variables.resistance
                * variables.saturation
                * self.params.voltages.v_ds
                * dm)
                / ((m + 1.0).powi(2) * denominator1),
            (self.params.vessels_number * variables.saturation * self.params.voltages.v_ds)
                / ((m + 1.0) * denominator1),
            -(self.params.voltages.v_ds
                * (self.params.r_ds_dry
                    - (self.params.vessels_number * variables.resistance) / (m + 1.0)))
                / denominator1,
            0.0,
            (self.params.vessels_number * variables.saturation * self.params.voltages.v_ds)
                / denominator2,
            -(self.params.voltages.v_ds
                * (self.params.r_ds_dry - self.params.vessels_number * variables.resistance))
                / denominator2,
            (self.params.voltages.v_gs
                * self.params.geometrics.cross_sectional_area
                * self.params.vessels_number
                * variables.saturation
                * dr)
                / (self.params.geometrics.length * r.powi(2)),
            0.0,
            -(self.params.vessels_number
                * self.params.voltages.v_gs
                * self.params.geometrics.cross_sectional_area)
                / (self.params.geometrics.length * r),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::params::{Currents, Geometrics, Voltages};

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
    fn test_model() {
        let (params, currents) = mock_params();
        let model = System::new(params, currents);

        assert_eq!(model.params().geometrics.cross_sectional_area, 1.0);
        assert_eq!(model.params().geometrics.length, 2.0);
        assert_eq!(model.params().r_ds_dry, 3.0);
        assert_eq!(model.params().vessels_number, 4.0);
        assert_eq!(model.params().voltages.v_ds, 5.0);
        assert_eq!(model.params().voltages.v_gs, 6.0);
        assert_eq!(model.currents().i_ds_min, 7.0);
        assert_eq!(model.currents().i_ds_max, 8.0);
        assert_eq!(model.currents().i_gs, 9.0);
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
        assert!((value[0].0 - 8.0) < 1e-6);
        assert!((value[0].1 - 11.230_971) < 1e-6);
        assert!((value[1].0 - 7.0) < 1e-6);
        assert!((value[1].1 - 2.136_752) < 1e-6);
        assert!((value[2].0 - 9.0) < 1e-6);
        assert!((value[2].1 - 3.825_316) < 1e-6);
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
        assert!((jacobian.m11 + 0.578_667).abs() < 1e-6);
        assert!((jacobian.m12 - 0.702_668).abs() < 1e-6);
        assert!((jacobian.m13 + 2.517_893).abs() < 1e-6);
        assert!((jacobian.m21 - 0.0).abs() < 1e-6);
        assert!((jacobian.m22 - 1.095_77).abs() < 1e-6);
        assert!((jacobian.m23 + 2.008_912).abs() < 1e-6);
        assert!((jacobian.m31 + 0.000_621).abs() < 1e-6);
        assert!((jacobian.m32 - 0.0).abs() < 1e-6);
        assert!((jacobian.m33 + 12.751_054).abs() < 1e-6);
    }
}
