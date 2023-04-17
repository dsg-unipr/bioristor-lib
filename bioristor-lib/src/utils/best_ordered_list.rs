use crate::params::Variables;

/// An ordered list of the best solutions found so far.
///
/// # Type parameters
///
/// * `S` - The type of a solution.
/// * `N` - The number of solutions to keep.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct BestOrderedList<S: Sized, const N: usize> {
    data: [(S, f32); N],
}

impl<const N: usize> Default for BestOrderedList<f32, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> BestOrderedList<f32, N> {
    /// Create a new instance of the list.
    #[inline]
    pub fn new() -> Self {
        BestOrderedList::<f32, N> {
            data: [(0.0, f32::INFINITY); N],
        }
    }

    /// Clear the list.
    #[inline]
    pub fn clear(&mut self) {
        self.data = [(0.0, f32::INFINITY); N];
    }

    /// Add a new solution to the list if it is better than the worst solution
    /// currently in the list.
    ///
    /// # Arguments
    ///
    /// * `solution` - The solution to add in the form `(variable, error)`.
    #[inline]
    pub fn add_solution(&mut self, solution: (f32, f32)) {
        if solution.1 < self.data[N - 1].1 {
            self.data[N - 1] = solution;
            self.data
                .sort_unstable_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        }
    }

    /// Get the mean concentration of the solutions in the list.
    ///
    /// # Returns
    ///
    /// The mean concentration.
    #[inline]
    pub fn mean_concentration(&self) -> f32 {
        let n = self.data.iter().filter(|(_, e)| e.is_finite()).count() as f32;
        return self
            .data
            .iter()
            .filter(|(_, e)| e.is_finite())
            .map(|(var, _)| var)
            .sum::<f32>()
            / n;
    }

    /// Get the best solution calculated as the mean of the solutions in the list.
    ///
    /// # Returns
    ///
    /// The best solution.
    #[inline]
    pub fn best(&self) -> f32 {
        let mut concentration = 0.0;

        let mut n = 0;
        for (var, _) in self.data.iter().filter(|(_, e)| e.is_finite()) {
            concentration += var;
            n += 1;
        }

        let n_inv = 1.0 / n as f32;
        concentration * n_inv
    }
}

impl<const N: usize> Default for BestOrderedList<Variables, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> BestOrderedList<Variables, N> {
    const DEFAULT: (Variables, f32) = (
        Variables {
            concentration: 0.0,
            resistance: 0.0,
            saturation: 0.0,
        },
        f32::INFINITY,
    );

    /// Create a new instance of the list.
    #[inline]
    pub fn new() -> Self {
        BestOrderedList::<Variables, N> {
            data: [Self::DEFAULT; N],
        }
    }

    /// Clear the list.
    #[inline]
    pub fn clear(&mut self) {
        self.data = [Self::DEFAULT; N];
    }

    /// Add a new solution to the list if it is better than the worst solution
    /// currently in the list.
    ///
    /// # Arguments
    ///
    /// * `solution` - The solution to add.
    #[inline]
    pub fn add_solution(&mut self, solution: (Variables, f32)) {
        if solution.1 < self.data[N - 1].1 {
            self.data[N - 1] = solution;
            self.data
                .sort_unstable_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        }
    }

    /// Get the mean concentration of the solutions in the list.
    ///
    /// # Returns
    ///
    /// The mean concentration.
    #[inline]
    pub fn mean_concentration(&self) -> f32 {
        let n = self.data.iter().filter(|(_, e)| e.is_finite()).count() as f32;
        return self
            .data
            .iter()
            .filter(|(_, e)| e.is_finite())
            .map(|(v, _)| v.concentration)
            .sum::<f32>()
            / n;
    }

    /// Get the best solution calculated as the mean of the solutions in the list.
    ///
    /// # Returns
    ///
    /// The best solution.
    #[inline]
    pub fn best(&self) -> (Variables, f32) {
        let mut concentration = 0.0;
        let mut resistance = 0.0;
        let mut saturation = 0.0;
        let mut error = 0.0;
        let mut n = 0;
        for (vars, err) in self.data.iter().filter(|(_, e)| e.is_finite()) {
            concentration += vars.concentration;
            resistance += vars.resistance;
            saturation += vars.saturation;
            error += err;
            n += 1;
        }
        let n_inv = 1.0 / n as f32;
        (
            Variables {
                concentration: concentration * n_inv,
                resistance: resistance * n_inv,
                saturation: saturation * n_inv,
            },
            error * n_inv,
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::params::Variables;

    use super::*;

    #[test]
    fn test_default() {
        let list: BestOrderedList<f32, 3> = Default::default();
        for i in 0..3 {
            assert_eq!(list.data[i].0, 0.0);
            assert_eq!(list.data[i].1, f32::INFINITY);
        }

        let list: BestOrderedList<Variables, 3> = Default::default();
        for i in 0..3 {
            assert_eq!(list.data[i].0.concentration, 0.0);
            assert_eq!(list.data[i].0.resistance, 0.0);
            assert_eq!(list.data[i].0.saturation, 0.0);
            assert_eq!(list.data[i].1, f32::INFINITY);
        }
    }

    #[test]
    fn test_new() {
        let list = BestOrderedList::<f32, 3>::new();
        for i in 0..3 {
            assert_eq!(list.data[i].0, 0.0);
            assert_eq!(list.data[i].1, f32::INFINITY);
        }

        let list = BestOrderedList::<Variables, 3>::new();
        for i in 0..3 {
            assert_eq!(list.data[i].0.concentration, 0.0);
            assert_eq!(list.data[i].0.resistance, 0.0);
            assert_eq!(list.data[i].0.saturation, 0.0);
            assert_eq!(list.data[i].1, f32::INFINITY);
        }
    }

    #[test]
    fn test_clear() {
        let mut list = BestOrderedList::<f32, 3>::new();
        list.data[0] = (1.0, 0.0);
        list.clear();

        for i in 0..3 {
            assert_eq!(list.data[i].0, 0.0);
            assert_eq!(list.data[i].1, f32::INFINITY);
        }

        let mut list = BestOrderedList::<Variables, 3>::new();
        list.data[0] = (
            Variables {
                concentration: 1.0,
                resistance: 2.0,
                saturation: 3.0,
            },
            0.0,
        );
        list.clear();

        for i in 0..3 {
            assert_eq!(list.data[i].0.concentration, 0.0);
            assert_eq!(list.data[i].0.resistance, 0.0);
            assert_eq!(list.data[i].0.saturation, 0.0);
            assert_eq!(list.data[i].1, f32::INFINITY);
        }
    }

    #[test]
    fn test_add_solution() {
        let mut list = BestOrderedList::<f32, 3>::new();

        list.add_solution((0.0, 0.0));
        assert_eq!(list.data[0].0, 0.0);
        assert_eq!(list.data[0].1, 0.0);
        assert_eq!(list.data[1].0, 0.0);
        assert_eq!(list.data[1].1, f32::INFINITY);
        assert_eq!(list.data[2].0, 0.0);
        assert_eq!(list.data[2].1, f32::INFINITY);

        list.add_solution((1.0, 1.0));
        assert_eq!(list.data[0].0, 0.0);
        assert_eq!(list.data[0].1, 0.0);
        assert_eq!(list.data[1].0, 1.0);
        assert_eq!(list.data[1].1, 1.0);
        assert_eq!(list.data[2].0, 0.0);
        assert_eq!(list.data[2].1, f32::INFINITY);

        list.add_solution((2.0, 2.0));
        assert_eq!(list.data[0].0, 0.0);
        assert_eq!(list.data[0].1, 0.0);
        assert_eq!(list.data[1].0, 1.0);
        assert_eq!(list.data[1].1, 1.0);
        assert_eq!(list.data[2].0, 2.0);
        assert_eq!(list.data[2].1, 2.0);

        list.add_solution((3.0, 3.0));
        assert_eq!(list.data[0].0, 0.0);
        assert_eq!(list.data[0].1, 0.0);
        assert_eq!(list.data[1].0, 1.0);
        assert_eq!(list.data[1].1, 1.0);
        assert_eq!(list.data[2].0, 2.0);
        assert_eq!(list.data[2].1, 2.0);

        list.add_solution((4.0, 0.5));
        assert_eq!(list.data[0].0, 0.0);
        assert_eq!(list.data[0].1, 0.0);
        assert_eq!(list.data[1].0, 4.0);
        assert_eq!(list.data[1].1, 0.5);
        assert_eq!(list.data[2].0, 1.0);
        assert_eq!(list.data[2].1, 1.0);

        let mut list = BestOrderedList::<Variables, 3>::new();

        list.add_solution((
            Variables {
                concentration: 0.0,
                resistance: 0.0,
                saturation: 0.0,
            },
            0.0,
        ));
        assert_eq!(list.data[0].0.concentration, 0.0);
        assert_eq!(list.data[0].0.resistance, 0.0);
        assert_eq!(list.data[0].0.saturation, 0.0);
        assert_eq!(list.data[0].1, 0.0);
        assert_eq!(list.data[1].0.concentration, 0.0);
        assert_eq!(list.data[1].0.resistance, 0.0);
        assert_eq!(list.data[1].0.saturation, 0.0);
        assert_eq!(list.data[1].1, f32::INFINITY);
        assert_eq!(list.data[2].0.concentration, 0.0);
        assert_eq!(list.data[2].0.resistance, 0.0);
        assert_eq!(list.data[2].0.saturation, 0.0);
        assert_eq!(list.data[2].1, f32::INFINITY);

        list.add_solution((
            Variables {
                concentration: 1.0,
                resistance: 1.0,
                saturation: 1.0,
            },
            1.0,
        ));
        assert_eq!(list.data[0].0.concentration, 0.0);
        assert_eq!(list.data[0].0.resistance, 0.0);
        assert_eq!(list.data[0].0.saturation, 0.0);
        assert_eq!(list.data[0].1, 0.0);
        assert_eq!(list.data[1].0.concentration, 1.0);
        assert_eq!(list.data[1].0.resistance, 1.0);
        assert_eq!(list.data[1].0.saturation, 1.0);
        assert_eq!(list.data[1].1, 1.0);
        assert_eq!(list.data[2].0.concentration, 0.0);
        assert_eq!(list.data[2].0.resistance, 0.0);
        assert_eq!(list.data[2].0.saturation, 0.0);
        assert_eq!(list.data[2].1, f32::INFINITY);

        list.add_solution((
            Variables {
                concentration: 2.0,
                resistance: 2.0,
                saturation: 2.0,
            },
            2.0,
        ));
        assert_eq!(list.data[0].0.concentration, 0.0);
        assert_eq!(list.data[0].0.resistance, 0.0);
        assert_eq!(list.data[0].0.saturation, 0.0);
        assert_eq!(list.data[0].1, 0.0);
        assert_eq!(list.data[1].0.concentration, 1.0);
        assert_eq!(list.data[1].0.resistance, 1.0);
        assert_eq!(list.data[1].0.saturation, 1.0);
        assert_eq!(list.data[1].1, 1.0);
        assert_eq!(list.data[2].0.concentration, 2.0);
        assert_eq!(list.data[2].0.resistance, 2.0);
        assert_eq!(list.data[2].0.saturation, 2.0);
        assert_eq!(list.data[2].1, 2.0);

        list.add_solution((
            Variables {
                concentration: 3.0,
                resistance: 3.0,
                saturation: 3.0,
            },
            3.0,
        ));
        assert_eq!(list.data[0].0.concentration, 0.0);
        assert_eq!(list.data[0].0.resistance, 0.0);
        assert_eq!(list.data[0].0.saturation, 0.0);
        assert_eq!(list.data[0].1, 0.0);
        assert_eq!(list.data[1].0.concentration, 1.0);
        assert_eq!(list.data[1].0.resistance, 1.0);
        assert_eq!(list.data[1].0.saturation, 1.0);
        assert_eq!(list.data[1].1, 1.0);
        assert_eq!(list.data[2].0.concentration, 2.0);
        assert_eq!(list.data[2].0.resistance, 2.0);
        assert_eq!(list.data[2].0.saturation, 2.0);
        assert_eq!(list.data[2].1, 2.0);

        list.add_solution((
            Variables {
                concentration: 4.0,
                resistance: 4.0,
                saturation: 4.0,
            },
            0.5,
        ));
        assert_eq!(list.data[0].0.concentration, 0.0);
        assert_eq!(list.data[0].0.resistance, 0.0);
        assert_eq!(list.data[0].0.saturation, 0.0);
        assert_eq!(list.data[0].1, 0.0);
        assert_eq!(list.data[1].0.concentration, 4.0);
        assert_eq!(list.data[1].0.resistance, 4.0);
        assert_eq!(list.data[1].0.saturation, 4.0);
        assert_eq!(list.data[1].1, 0.5);
        assert_eq!(list.data[2].0.concentration, 1.0);
        assert_eq!(list.data[2].0.resistance, 1.0);
        assert_eq!(list.data[2].0.saturation, 1.0);
        assert_eq!(list.data[2].1, 1.0);
    }

    #[test]
    fn test_mean_concentration() {
        let mut list = BestOrderedList::<f32, 3>::new();
        list.data = [(0.0, 0.0), (1.0, 1.0), (2.0, 2.0)];
        assert_eq!(list.mean_concentration(), 1.0);

        list.data = [(0.0, 0.0), (1.0, 1.0), (0.0, f32::INFINITY)];
        assert_eq!(list.mean_concentration(), 0.5);

        let mut list = BestOrderedList::<Variables, 3>::new();
        list.data = [
            (
                Variables {
                    concentration: 0.0,
                    resistance: 0.0,
                    saturation: 0.0,
                },
                0.0,
            ),
            (
                Variables {
                    concentration: 1.0,
                    resistance: 1.0,
                    saturation: 1.0,
                },
                1.0,
            ),
            (
                Variables {
                    concentration: 2.0,
                    resistance: 2.0,
                    saturation: 2.0,
                },
                2.0,
            ),
        ];
        assert_eq!(list.mean_concentration(), 1.0);

        list.data = [
            (
                Variables {
                    concentration: 0.0,
                    resistance: 0.0,
                    saturation: 0.0,
                },
                0.0,
            ),
            (
                Variables {
                    concentration: 1.0,
                    resistance: 1.0,
                    saturation: 1.0,
                },
                1.0,
            ),
            (
                Variables {
                    concentration: 0.0,
                    resistance: 0.0,
                    saturation: 0.0,
                },
                f32::INFINITY,
            ),
        ];
        assert_eq!(list.mean_concentration(), 0.5);
    }

    #[test]
    fn test_best() {
        let mut list = BestOrderedList::<f32, 3>::new();
        list.data = [(0.0, 0.0), (1.0, 1.0), (2.0, 2.0)];
        let best = list.best();
        assert_eq!(best, 1.0);

        list.data = [(0.0, 0.0), (1.0, 1.0), (0.0, f32::INFINITY)];
        let best = list.best();
        assert_eq!(best, 0.5);

        let mut list = BestOrderedList::<Variables, 3>::new();
        list.data = [
            (
                Variables {
                    concentration: 0.0,
                    resistance: 0.0,
                    saturation: 0.0,
                },
                0.0,
            ),
            (
                Variables {
                    concentration: 1.0,
                    resistance: 1.0,
                    saturation: 1.0,
                },
                1.0,
            ),
            (
                Variables {
                    concentration: 2.0,
                    resistance: 2.0,
                    saturation: 2.0,
                },
                2.0,
            ),
        ];
        let best = list.best();
        assert_eq!(best.0.concentration, 1.0);
        assert_eq!(best.0.resistance, 1.0);
        assert_eq!(best.0.saturation, 1.0);
        assert_eq!(best.1, 1.0);

        list.data = [
            (
                Variables {
                    concentration: 0.0,
                    resistance: 0.0,
                    saturation: 0.0,
                },
                0.0,
            ),
            (
                Variables {
                    concentration: 1.0,
                    resistance: 1.0,
                    saturation: 1.0,
                },
                1.0,
            ),
            (
                Variables {
                    concentration: 0.0,
                    resistance: 0.0,
                    saturation: 0.0,
                },
                f32::INFINITY,
            ),
        ];
        let best = list.best();
        assert_eq!(best.0.concentration, 0.5);
        assert_eq!(best.0.resistance, 0.5);
        assert_eq!(best.0.saturation, 0.5);
        assert_eq!(best.1, 0.5);
    }
}
