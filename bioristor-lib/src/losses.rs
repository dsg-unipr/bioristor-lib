#[allow(unused_imports)]
use micromath::F32Ext;

/// The loss function used to evaluate the model.
pub trait Loss {
    /// Evaluates the loss of the outputs of the model.
    ///
    /// # Arguments
    ///
    /// * `raw_value` - The raw output value of the model, i.e. the output of
    ///     the left and right function for each equation.
    ///
    /// # Returns
    ///
    /// The loss of the outputs of the model.
    fn evaluate(raw_value: &[(f32, f32); 3]) -> f32;
}

/// This loss function calculates the error as the maximum of the relative error
/// of the three equations of the model.
/// The relative error of an equation is calculated as follows:
/// `|left - right| / ( |left| + |right| )`.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct MaxRelative;

impl Loss for MaxRelative {
    fn evaluate(raw_value: &[(f32, f32); 3]) -> f32 {
        let [(a, b), (c, d), (e, f)] = raw_value;

        // The `f32::EPSILON` value is added to avoid division by zero.
        ((a - b).abs() / (a.abs() + b.abs() + f32::EPSILON)).max(
            ((c - d).abs() / (c.abs() + d.abs() + f32::EPSILON))
                .max((e - f).abs() / (e.abs() + f.abs() + f32::EPSILON)),
        )
    }
}

/// This loss function calculates the error as the mean of the relative error
/// of the three equations of the model.
/// The relative error of an equation is calculated as follows:
/// `|left - right| / ( |left| + |right| )`.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct MeanRelative;

impl Loss for MeanRelative {
    fn evaluate(raw_value: &[(f32, f32); 3]) -> f32 {
        let [(a, b), (c, d), (e, f)] = raw_value;

        // The `f32::EPSILON` value is added to avoid division by zero.
        ((a - b).abs() / (a.abs() + b.abs() + f32::EPSILON)
            + (c - d).abs() / (c.abs() + d.abs() + f32::EPSILON)
            + (e - f).abs() / (e.abs() + f.abs() + f32::EPSILON))
            * (1.0 / 3.0)
    }
}

/// This loss function calculates the error as the sum of the relative error
/// of the three equations of the model.
/// The relative error of an equation is calculated as follows:
/// `|left - right| / ( |left| + |right| )`.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct SumRelative;

impl Loss for SumRelative {
    fn evaluate(raw_value: &[(f32, f32); 3]) -> f32 {
        let [(a, b), (c, d), (e, f)] = raw_value;

        // The `f32::EPSILON` value is added to avoid division by zero.
        (a - b).abs() / (a.abs() + b.abs() + f32::EPSILON)
            + (c - d).abs() / (c.abs() + d.abs() + f32::EPSILON)
            + (e - f).abs() / (e.abs() + f.abs() + f32::EPSILON)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_max_relative() {
        let value = [(1.0, 2.0), (3.0, 4.0), (5.0, 6.0)];
        assert!((MaxRelative::evaluate(&value) - 0.333_333).abs() < 1e-6);

        let value = [(-1.0, 2.0), (-3.0, 4.0), (5.0, -6.0)];
        assert!((MaxRelative::evaluate(&value) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_mean_relative() {
        let value = [(1.0, 2.0), (3.0, 4.0), (5.0, 6.0)];
        assert!((MeanRelative::evaluate(&value) - 0.189_033).abs() < 1e-6);

        let value = [(-1.0, 2.0), (-3.0, 4.0), (5.0, -6.0)];
        assert!((MeanRelative::evaluate(&value) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_sum_relative() {
        let value = [(1.0, 2.0), (3.0, 4.0), (5.0, 6.0)];
        assert!((SumRelative::evaluate(&value) - 0.567_099).abs() < 1e-6);

        let value = [(-1.0, 2.0), (-3.0, 4.0), (5.0, -6.0)];
        assert!((SumRelative::evaluate(&value) - 3.0).abs() < 1e-9);
    }
}
