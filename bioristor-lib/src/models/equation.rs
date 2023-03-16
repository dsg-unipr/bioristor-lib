use crate::{
    models::Model,
    params::{Currents, ModelParams},
};

/// Formulation of the mathematical model of the Bioristor device as an equation
/// that depends on a single variable: the concentration of ions in the electrolyte.
pub trait EquationModel: Model {
    /// Calculates the output value of the model for the given variables.
    ///
    /// # Arguments
    ///
    /// * `concentration` - Concentration of ions in the electrolyte [Molarity].
    ///
    /// # Returns
    ///
    /// The output value of the model.
    fn value(&self, concentration: f32) -> f32;

    /// Calculates the gradient of the error function.
    ///
    /// # Arguments
    ///
    /// * `concentration` - Concentration of ions in the electrolyte [Molarity].
    ///
    /// # Returns
    ///
    /// The first derivative of the error function.
    fn gradient(&self, concentration: f32) -> f32;

    /// Calculates the resistance given the concentration.
    ///
    /// # Arguments
    ///
    /// * `concentration` - The concentration of ions in the electrolyte [Molarity].
    ///
    /// # Returns
    ///
    /// The eletrical resistance of the wet PEDOT channel after being exposed
    ///     to the electrolyte [Ohm].
    fn resistance(&self, concentration: f32) -> f32;

    /// Calculates the water saturation given the concentration.
    ///
    /// # Arguments
    ///
    /// * `concentration` - The concentration of ions in the electrolyte [Molarity].
    ///
    /// # Returns
    ///
    /// The saturation of the water [dimensionless].
    fn saturation(&self, concentration: f32) -> f32;
}

/// Implementation of the mathematical model using a single-variable (i.e., the
/// concentration of ions in the electrolyte) equation.
///
/// # Example
///
/// ```
/// use bioristor_lib::models::{Model, Equation, EquationModel};
/// use bioristor_lib::params::{
///     Currents, ModelParams, ModulationParams, StemResistanceInvParams, Voltages,
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
/// let model = Equation::new(PARAMS, currents);
///
/// let concentration = 10.0;
/// let value = model.value(concentration);
///
/// let resistance = model.resistance(concentration);
/// let saturation = model.saturation(concentration);
/// ```
#[derive(Debug)]
pub struct Equation {
    /// Pre-calculated coefficients to compute the error function.
    func_coeffs: FuncCoeffs,

    /// Pre-calculated coefficients to compute the resistance.
    resistance_coeffs: ResistanceCoeffs,

    /// Pre-calculated coefficients to compute the saturation.
    saturation_coeffs: SaturationCoeffs,

    /// The output currents of the device.
    currents: Currents,

    /// The parameters of the mathematical model.
    params: ModelParams,
}

/// Pre-calculated coefficients to compute the error function.
#[derive(Debug)]
struct FuncCoeffs(f32, f32, f32, f32);

/// Pre-calculated coefficients to comput the resistance.
#[derive(Debug)]
struct ResistanceCoeffs(f32, f32, f32);

/// Pre-calculated coefficients to compute the saturation.
#[derive(Debug)]
struct SaturationCoeffs(f32, f32, f32);

impl Model for Equation {
    fn new(params: ModelParams, currents: Currents) -> Self {
        Equation {
            func_coeffs: FuncCoeffs(
                currents.i_gs_on,
                params.voltages.v_gs
                    * params.voltages.v_ds
                    * (currents.i_ds_off - currents.i_ds_on + currents.i_gs_on),
                params.voltages.v_gs
                    * currents.i_ds_off
                    * (params.voltages.v_ds - currents.i_ds_on * params.r_dry
                        + currents.i_gs_on * params.r_dry),
                currents.i_ds_off * params.r_dry * (currents.i_ds_on - currents.i_gs_on),
            ),
            resistance_coeffs: ResistanceCoeffs(
                params.r_dry
                    * params.voltages.v_ds
                    * (currents.i_ds_off - currents.i_ds_on + currents.i_gs_on),
                params.voltages.v_ds * (currents.i_ds_off - currents.i_ds_on + currents.i_gs_on),
                currents.i_ds_off
                    * (params.voltages.v_ds - currents.i_ds_on * params.r_dry
                        + currents.i_gs_on * params.r_dry),
            ),
            saturation_coeffs: SaturationCoeffs(
                params.voltages.v_ds * (currents.i_ds_off - currents.i_ds_on + currents.i_gs_on),
                currents.i_ds_off
                    * (params.voltages.v_ds - currents.i_ds_on * params.r_dry
                        + currents.i_gs_on * params.r_dry),
                currents.i_ds_off * params.r_dry * (currents.i_gs_on - currents.i_ds_on),
            ),
            currents,
            params,
        }
    }

    fn currents(&self) -> &Currents {
        &self.currents
    }

    fn params(&self) -> &ModelParams {
        &self.params
    }
}

impl EquationModel for Equation {
    fn value(&self, concentration: f32) -> f32 {
        let m = self.modulation(concentration);
        let r = self.stem_resistance_inv(concentration);

        self.func_coeffs.0
            + (self.func_coeffs.1 * r + self.func_coeffs.2 * r * m) / (self.func_coeffs.3 * m)
    }

    fn gradient(&self, concentration: f32) -> f32 {
        let m = self.modulation(concentration);
        let r = self.stem_resistance_inv(concentration);
        let dm = self.modulation_gradient(concentration);
        let dr = self.stem_resistance_inv_gradient(concentration);

        (self.func_coeffs.1 * dr + self.func_coeffs.2 * (m * dr + dm * r))
            / (self.func_coeffs.3 * m)
            - ((self.func_coeffs.1 + self.func_coeffs.2 * m) * r * dm)
                / (self.func_coeffs.3 * m * m)
    }

    fn resistance(&self, concentration: f32) -> f32 {
        let m = self.modulation(concentration);

        (self.resistance_coeffs.0 * (m + 1.0))
            / (self.resistance_coeffs.1 + self.resistance_coeffs.2 * m)
    }

    fn saturation(&self, concentration: f32) -> f32 {
        let m = self.modulation(concentration);

        (self.saturation_coeffs.0 + self.saturation_coeffs.1 * m) / (self.saturation_coeffs.2 * m)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::{Currents, ModulationParams, StemResistanceInvParams, Voltages};

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
        let model = Equation::new(params, currents);

        assert_eq!(model.func_coeffs.0, 11.0);
        assert_eq!(model.func_coeffs.1, 8.0 * 7.0 * (9.0 - 10.0 + 11.0));
        assert_eq!(
            model.func_coeffs.2,
            8.0 * 9.0 * (7.0 - 10.0 * 4.0 + 11.0 * 4.0)
        );
        assert_eq!(model.func_coeffs.3, 9.0 * 4.0 * (10.0 - 11.0));

        assert_eq!(model.resistance_coeffs.0, 4.0 * 7.0 * (9.0 - 10.0 + 11.0));
        assert_eq!(model.resistance_coeffs.1, 7.0 * (9.0 - 10.0 + 11.0));
        assert_eq!(
            model.resistance_coeffs.2,
            9.0 * (7.0 - 10.0 * 4.0 + 11.0 * 4.0)
        );

        assert_eq!(model.saturation_coeffs.0, 7.0 * (9.0 - 10.0 + 11.0));
        assert_eq!(
            model.saturation_coeffs.1,
            9.0 * (7.0 - 10.0 * 4.0 + 11.0 * 4.0)
        );
        assert_eq!(model.saturation_coeffs.2, 9.0 * 4.0 * (11.0 - 10.0));

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
        let model = Equation::new(params, currents);

        assert!((model.value(1.0) + 273.777_77).abs() < 1e-4);
    }

    #[test]
    fn test_gradient() {
        let (params, currents) = mock_params();
        let model = Equation::new(params, currents);

        assert!((model.gradient(1.0) + 116.26).abs() < 1e-3);
    }

    #[test]
    fn test_resistance() {
        let (params, currents) = mock_params();
        let model = Equation::new(params, currents);

        assert!((model.resistance(1.0) - 3.004_291_8).abs() < 1e-6);
    }

    #[test]
    fn test_saturation() {
        let (params, currents) = mock_params();
        let model = Equation::new(params, currents);

        assert!((model.saturation(1.0) - 3.236_111_1).abs() < 1e-6);
    }
}
