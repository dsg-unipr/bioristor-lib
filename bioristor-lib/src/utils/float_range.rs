/// An implementation of a number range able to handle floating point numbers
/// and providing a way to iterate over the range for a fixed number of steps.
///
/// # Examples
///
/// ```
/// use bioristor_lib::utils::FloatRange;
///
/// let range = FloatRange::new(0.0, 1.0, 10usize);
///
/// for i in range {
///    println!("{}", i);
/// }
/// ```
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct FloatRange {
    /// The lower bound of the range (inclusive).
    pub start: f32,

    /// The upper bound of the range (exclusive).
    pub end: f32,

    /// The number of steps in which the interval is divided.
    pub steps: usize,
}

impl FloatRange {
    /// Creates a new float range.
    ///
    /// # Arguments
    ///
    /// * `start` - The lower bound of the range (inclusive).
    /// * `end` - The upper bound of the range (exclusive).
    /// * `steps` - The number of steps in which the interval is divided.
    pub const fn new(start: f32, end: f32, steps: usize) -> Self {
        Self { start, end, steps }
    }
}

impl IntoIterator for FloatRange {
    type Item = f32;
    type IntoIter = FloatRangeIter;

    fn into_iter(self) -> Self::IntoIter {
        FloatRangeIter {
            value: self.start,
            remaining_steps: self.steps,
            increment: (self.end - self.start) / self.steps as f32,
        }
    }
}

/// An iterator over a range of floating point numbers.
///
/// # Examples
///
/// ```
/// use bioristor_lib::utils::FloatRange;
///
/// let range = FloatRange::new(0.0, 1.0, 10usize);
/// let mut iter = range.into_iter();
///
/// assert!((iter.next().unwrap()).abs() < 1e-12);
/// assert!((iter.next().unwrap() - 0.1).abs() < 1e-6);
/// assert!((iter.next().unwrap() - 0.2).abs() < 1e-6);
/// assert!((iter.next().unwrap() - 0.3).abs() < 1e-6);
/// assert!((iter.next().unwrap() - 0.4).abs() < 1e-6);
/// assert!((iter.next().unwrap() - 0.5).abs() < 1e-6);
/// assert!((iter.next().unwrap() - 0.6).abs() < 1e-6);
/// assert!((iter.next().unwrap() - 0.7).abs() < 1e-6);
/// assert!((iter.next().unwrap() - 0.8).abs() < 1e-6);
/// assert!((iter.next().unwrap() - 0.9).abs() < 1e-6);
/// assert_eq!(iter.next(), None);
/// ```
#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct FloatRangeIter {
    /// The current value of the iterator.
    value: f32,

    /// The number of remaining steps.
    remaining_steps: usize,

    /// The increment between two consecutive values in the range.
    increment: f32,
}

impl Iterator for FloatRangeIter {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining_steps > 0usize {
            let value = self.value;
            self.value += self.increment;
            self.remaining_steps -= 1usize;
            Some(value)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_float_range() {
        let range = FloatRange::new(0.0, 1.0, 10usize);

        let mut count = 0usize;
        for _ in range.clone() {
            count += 1usize;
        }
        assert_eq!(count, 10usize);

        let mut iter = range.into_iter();
        assert!((iter.next().unwrap()).abs() < 1e-12);
        assert!((iter.next().unwrap() - 0.1).abs() < 1e-6);
        assert!((iter.next().unwrap() - 0.2).abs() < 1e-6);
        assert!((iter.next().unwrap() - 0.3).abs() < 1e-6);
        assert!((iter.next().unwrap() - 0.4).abs() < 1e-6);
        assert!((iter.next().unwrap() - 0.5).abs() < 1e-6);
        assert!((iter.next().unwrap() - 0.6).abs() < 1e-6);
        assert!((iter.next().unwrap() - 0.7).abs() < 1e-6);
        assert!((iter.next().unwrap() - 0.8).abs() < 1e-6);
        assert!((iter.next().unwrap() - 0.9).abs() < 1e-6);
        assert_eq!(iter.next(), None);
    }
}
