/// The parameters of the mathematical model.
#[derive(Debug, PartialEq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ModelParams {
    /// The geometrical characteristics of each vessel in the stem of the plant.
    pub geometrics: Geometrics,

    /// Eletrical resistance of the dry PEDOT channel before being exposed
    /// to the electrolyte [Ohm].
    pub r_ds_dry: f32,

    /// The number of vessels in the stem of the plant [dimensionless].
    pub vessels_number: f32,

    /// The input voltages of the device.
    pub voltages: Voltages,
}

/// The output currents of the device.
#[derive(Debug, PartialEq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Currents {
    /// Minimum current measured between drain and source [Ampere].
    pub i_ds_min: f32,

    /// Maximum current measured between drain and source [Ampere].
    pub i_ds_max: f32,

    /// Current measured between gate and source [Ampere].
    pub i_gs: f32,
}

/// The geometrical characteristics of each vessel in the stem of the plant.
#[derive(Debug, PartialEq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Geometrics {
    /// Cross sectional area of a vessel in the stem of the plant [Square Metre].
    pub cross_sectional_area: f32,

    /// Length of a vessel in the stem of the plant [Metre].
    pub length: f32,
}

/// The dependent variables of the model.
#[derive(Debug, PartialEq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Variables {
    /// Concentration of ions in the electrolyte [Molarity].
    pub concentration: f32,

    /// Eletrical resistance of the wet PEDOT channel after being exposed
    /// to the electrolyte [Ohm].
    pub resistance: f32,

    /// Saturation of the water in the system [dimensionless].
    pub saturation: f32,
}

/// The input voltages of the device.
#[derive(Debug, PartialEq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Voltages {
    /// Voltage applied between drain and source [Volt].
    pub v_ds: f32,

    /// Voltage applied between gate and source [Volt].
    pub v_gs: f32,
}
