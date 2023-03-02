//! Profiler for Cortex-M microcontrollers.
//!
//! This implementation is strongly inspired by the [`ep-systick`] crate.
//!
//! This profiler depends on the [`SYST`] hardware, common to most Cortex-M devices.
//!
//! The profiler's configured resolution is the same as the core clock.
//!
//! The cycle count clock is free-running, so overflows are likely if you have
//! long running functions to profile.
//! To mitigate this, this profiler uses a [`u64`] counter and the [`SysTick`] exception.
//! You can expect an exception to fire every 2^24 clock cycles.
//!
//! [`ep-systick`]: https://crates.io/crates/ep-systick
//! [`SYST`]: `cortex_m::peripheral::SYST`
//! [`SysTick`]: `cortex_m::peripheral::scb::Exception::SysTick`

#![no_std]

use core::sync::atomic::{AtomicU32, Ordering};

use cortex_m::peripheral::{syst::SystClkSource, SYST};
use cortex_m_rt::exception;

/// Tracker of `systick` cycle count overflows to extend systick's 24 bit timer.
static ROLLOVER_COUNT: AtomicU32 = AtomicU32::new(0);

/// The reload value of the [`systick`](cortex_m::peripheral::SYST) peripheral.
/// Also is the max it can go: 2^24.
const SYSTICK_RELOAD: u32 = 0x00FF_FFFF;

/// The resolution of [`systick`](cortex_m::peripheral::SYST): 2^24.
const SYSTICK_RESOLUTION: u64 = 0x0100_0000;

/// Profiler based on [`SysTick`](cortex_m::peripheral::SYST)
/// for Cortex-M microcontrollers.
///
/// # Example
///
/// ```no_run
/// use cortex_m::peripheral::Peripherals;
/// use cortex_m_rt::entry;
///
/// use profiler::{cycles_to_ms, Profiler};
///
/// let cp = Peripherals::take().unwrap();
/// let mut syst = cp.SYST;
/// let profiler = Profiler::new(syst);
///
/// // Do some work.
///
/// let cycles = profiler.cycles();
/// let duration_ms = cycles_to_ms::<1_000_000>(cycles);
/// ```
pub struct Profiler {
    systick: SYST,
}

impl Profiler {
    /// Setup the SysTick counter and start counting CPU cycles.
    ///
    /// # Parameters
    ///
    /// * `systick`: The [`SysTick`] peripheral.
    pub fn new(mut systick: SYST) -> Self {
        // Reset the rollover count.
        ROLLOVER_COUNT.store(0, Ordering::Relaxed);

        // Configure SysTick counter.
        systick.disable_counter();
        systick.set_clock_source(SystClkSource::Core);
        systick.clear_current();
        systick.set_reload(SYSTICK_RELOAD);
        systick.enable_counter();

        // Enable SysTick interrupt.
        systick.enable_interrupt();

        Self { systick }
    }

    /// Releases the system timer (SysTick) resource
    pub fn free(mut self) -> SYST {
        // Disable SysTick interrupt.
        self.systick.disable_interrupt();

        self.systick
    }

    /// Returns the number of CPU cycles since the profiler was started.
    ///
    /// # Returns
    ///
    /// The number of CPU cycles since the profiler was started.
    #[inline]
    pub fn cycles(&self) -> u64 {
        // Read the clock & ROLLOVER_COUNT. We read `SYST` twice because we need to detect
        // if we've rolled over, and if we have make sure we have the right value for ROLLOVER_COUNT.
        let first = SYST::get_current();
        let rollover_count = ROLLOVER_COUNT.load(Ordering::Acquire) as u64;
        let second = SYST::get_current();

        // Since the SYSTICK counter is a count down timer, check if first is larger than second.
        if first > second {
            // The usual case: we did not roll over between the first and second reading,
            // and because of that, we also know we got a valid read on ROLLOVER_COUNT.
            rollover_count * SYSTICK_RESOLUTION + (SYSTICK_RELOAD - first) as u64
        } else {
            // We rolled over sometime between the first and second read. We may or may not have
            // caught the right ROLLOVER_COUNT, so grab that again and then use the second reading.
            let rollover_count = ROLLOVER_COUNT.load(Ordering::Acquire) as u64;
            rollover_count * SYSTICK_RESOLUTION + (SYSTICK_RELOAD - second) as u64
        }
    }
}

#[exception]
fn SysTick() {
    ROLLOVER_COUNT.fetch_add(1, Ordering::Release);
}

/// Converts the number of CPU cycles to milliseconds.
///
/// # Parameters
///
/// * `cycles`: The number of CPU cycles.
///
/// # Returns
///
/// The number of milliseconds.
///
/// # Type parameters
///
/// * `FREQ`: The frequency of the CPU in Hz.
#[inline]
pub fn cycles_to_ms<const FREQ: u32>(cycles: u64) -> u32 {
    (cycles as f32 * (1_000_f32 / FREQ as f32)) as u32
}

/// Converts the number of CPU cycles to microseconds.
///
/// # Parameters
///
/// * `cycles`: The number of CPU cycles.
///
/// # Returns
///
/// The number of microseconds.
///
/// # Type parameters
///
/// * `FREQ`: The frequency of the CPU in Hz.
#[inline]
pub fn cycles_to_us<const FREQ: u32>(cycles: u64) -> u32 {
    (cycles as f32 * (1_000_000_f32 / FREQ as f32)) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cycles_to_ms() {
        assert_eq!(cycles_to_ms::<1_000_000>(1_000_000), 1_000);
        assert_eq!(cycles_to_ms::<1_000_000>(1_000_000_000), 1_000_000);
    }

    #[test]
    fn test_cycles_to_us() {
        assert_eq!(cycles_to_us::<1_000_000>(1_000), 1_000);
        assert_eq!(cycles_to_us::<1_000_000>(1_000_000), 1_000_000);
    }
}
