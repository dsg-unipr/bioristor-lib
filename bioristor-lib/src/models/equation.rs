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
/// use bioristor_lib::params::{Currents, Geometrics, ModelParams, Voltages};
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
                currents.i_gs,
                params.vessels_number
                    * params.voltages.v_gs
                    * params.geometrics.cross_sectional_area
                    * params.voltages.v_ds
                    * (currents.i_ds_min - currents.i_ds_max + currents.i_gs),
                params.vessels_number
                    * params.voltages.v_gs
                    * params.geometrics.cross_sectional_area
                    * currents.i_ds_min
                    * (params.voltages.v_ds - currents.i_ds_max * params.r_ds_dry
                        + currents.i_gs * params.r_ds_dry),
                currents.i_ds_min
                    * params.r_ds_dry
                    * params.geometrics.length
                    * (currents.i_ds_max - currents.i_gs),
            ),
            resistance_coeffs: ResistanceCoeffs(
                params.r_ds_dry
                    * params.voltages.v_ds
                    * (currents.i_ds_min - currents.i_ds_max + currents.i_gs),
                params.vessels_number
                    * params.voltages.v_ds
                    * (currents.i_ds_min - currents.i_ds_max + currents.i_gs),
                params.vessels_number
                    * currents.i_ds_min
                    * (params.voltages.v_ds - currents.i_ds_max * params.r_ds_dry
                        + currents.i_gs * params.r_ds_dry),
            ),
            saturation_coeffs: SaturationCoeffs(
                params.voltages.v_ds * (currents.i_ds_min - currents.i_ds_max + currents.i_gs),
                currents.i_ds_min
                    * (params.voltages.v_ds - currents.i_ds_max * params.r_ds_dry
                        + currents.i_gs * params.r_ds_dry),
                currents.i_ds_min * params.r_ds_dry * (currents.i_gs - currents.i_ds_max),
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
        let r = self.resistivity(concentration);

        self.func_coeffs.0
            + (self.func_coeffs.1 + self.func_coeffs.2 * m) / (self.func_coeffs.3 * r * m)
    }

    fn gradient(&self, concentration: f32) -> f32 {
        let m = self.modulation(concentration);
        let r = self.resistivity(concentration);
        let dm = self.modulation_gradient(concentration);
        let dr = self.resistivity_gradient(concentration);

        -(self.func_coeffs.1 / self.func_coeffs.3) * ((m * dr + dm * r) / (m * m * r * r))
            - (self.func_coeffs.2 / self.func_coeffs.3) * (dr / (r * r))
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
    use crate::params::{Currents, Geometrics, Voltages};

    fn mock_params() -> (Currents, ModelParams) {
        (
            Currents {
                i_ds_min: 1.0,
                i_ds_max: 2.0,
                i_gs: 3.0,
            },
            ModelParams {
                geometrics: Geometrics {
                    cross_sectional_area: 4.0,
                    length: 5.0,
                },
                r_ds_dry: 6.0,
                vessels_number: 7.0,
                voltages: Voltages {
                    v_ds: 8.0,
                    v_gs: 9.0,
                },
            },
        )
    }

    #[test]
    fn test_model() {
        let (currents, params) = mock_params();
        let model = Equation::new(params, currents);

        assert_eq!(model.func_coeffs.0, 3.0);
        assert_eq!(
            model.func_coeffs.1,
            7.0 * 9.0 * 4.0 * 8.0 * (1.0 - 2.0 + 3.0)
        );
        assert_eq!(
            model.func_coeffs.2,
            7.0 * 9.0 * 4.0 * 1.0 * (8.0 - 2.0 * 6.0 + 3.0 * 6.0)
        );
        assert_eq!(model.func_coeffs.3, 1.0 * 6.0 * 5.0 * (2.0 - 3.0));

        assert_eq!(model.resistance_coeffs.0, 6.0 * 8.0 * (1.0 - 2.0 + 3.0));
        assert_eq!(model.resistance_coeffs.1, 7.0 * 8.0 * (1.0 - 2.0 + 3.0));
        assert_eq!(
            model.resistance_coeffs.2,
            7.0 * 1.0 * (8.0 - 2.0 * 6.0 + 3.0 * 6.0)
        );

        assert_eq!(model.saturation_coeffs.0, 8.0 * (1.0 - 2.0 + 3.0));
        assert_eq!(
            model.saturation_coeffs.1,
            1.0 * (8.0 - 2.0 * 6.0 + 3.0 * 6.0)
        );
        assert_eq!(model.saturation_coeffs.2, 1.0 * 6.0 * (3.0 - 2.0));

        assert_eq!(model.currents().i_ds_min, 1.0);
        assert_eq!(model.currents().i_ds_max, 2.0);
        assert_eq!(model.currents().i_gs, 3.0);

        assert_eq!(model.params().geometrics.cross_sectional_area, 4.0);
        assert_eq!(model.params().geometrics.length, 5.0);
        assert_eq!(model.params().r_ds_dry, 6.0);
        assert_eq!(model.params().vessels_number, 7.0);
        assert_eq!(model.params().voltages.v_ds, 8.0);
        assert_eq!(model.params().voltages.v_gs, 9.0);
    }

    #[test]
    fn test_value() {
        let (currents, params) = mock_params();
        let model = Equation::new(params, currents);

        assert!((model.value(1.0) + 1_183.264_2).abs() < 1e-3);
    }

    #[test]
    fn test_gradient() {
        let (currents, params) = mock_params();
        let model = Equation::new(params, currents);

        assert!((model.gradient(1.0) - 166.475_77).abs() < 1e-4);
    }

    #[test]
    fn test_resistance() {
        let (currents, params) = mock_params();
        let model = Equation::new(params, currents);

        assert!((model.resistance(1.0) - 0.962_406_01).abs() < 1e-6);
    }

    #[test]
    fn test_saturation() {
        let (currents, params) = mock_params();
        let model = Equation::new(params, currents);

        assert!((model.saturation(1.0) - 2.714_285_7).abs() < 1e-6);
    }
}
