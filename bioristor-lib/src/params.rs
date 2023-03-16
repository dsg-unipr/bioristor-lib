/// The parameters of the mathematical model.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ModelParams {
    /// The parameters of the modulation function.
    pub mod_params: ModulationParams,

    /// Eletrical resistance of the dry PEDOT channel before being exposed
    /// to the electrolyte [Ohm].
    pub r_dry: f32,

    /// The parameters of the inverse of stem resistance function.
    pub res_params: StemResistanceInvParams,

    /// The input voltages of the device.
    pub voltages: Voltages,
}

/// The output currents of the device.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Currents {
    /// Current measured between drain and source when the gate is off [Ampere].
    pub i_ds_off: f32,

    /// Current measured between drain and source when the gate is on [Ampere].
    pub i_ds_on: f32,

    /// Current measured between gate and source when the gate is on [Ampere].
    pub i_gs_on: f32,
}

/// The parameters of the modulation function.
/// The function is defined as:
/// ```text
/// a * x + b * ln(x) + c
/// ```
/// where `x` is the ion concentration, `a`, `b` and `c` are the parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ModulationParams(pub f32, pub f32, pub f32);

/// The parameters of the inverse of stem resistance function.
/// The function is defined as:
/// ```text
/// a + b * x^0.955
/// ```
/// where `x` is the ion concentration, `a` and `b` are the parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct StemResistanceInvParams(pub f32, pub f32);

/// The dependent variables of the model.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Variables {
    /// Concentration of ions in the electrolyte [Molarity].
    pub concentration: f32,

    /// Eletrical resistance of the wet PEDOT channel after being exposed
    /// to the electrolyte, when the gate is off [Ohm].
    pub resistance: f32,

    /// Saturation of the water in the system [dimensionless].
    pub saturation: f32,
}

/// The input voltages of the device.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Voltages {
    /// Voltage applied between drain and source [Volt].
    pub v_ds: f32,

    /// Voltage applied between gate and source [Volt].
    pub v_gs: f32,
}
