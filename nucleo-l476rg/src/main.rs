#![no_main]
#![no_std]

use defmt_rtt as _; // global logger
use panic_probe as _; // panic handler

use stm32l4xx_hal::{delay::Delay, pac, prelude::*};

use bioristor_lib::{
    algorithms::{Adaptive, AdaptiveParams, Algorithm},
    losses::MeanRelative,
    model::{Model, ThreeEquations},
    params::{Currents, Geometrics, ModelParams, Voltages},
    utils::FloatRange,
};

const ALG_PARAMS: AdaptiveParams = AdaptiveParams {
    concentration_guess: 1e-3,
    concentration_steps: 100,
    max_iterations: 4,
    resistance_range: FloatRange::new(0.0, 5.0, 100),
    saturation_range: FloatRange::new(0.0, 1.0, 100),
};

const MODEL_PARAMS: ModelParams = ModelParams {
    geometrics: Geometrics {
        cross_sectional_area: 2e-8,
        length: 5e-3,
    },
    r_ds_dry: 10.0,
    vessels_number: 100.0,
    voltages: Voltages {
        v_ds: -0.05,
        v_gs: 0.5,
    },
};

const CORE_FREQ: u32 = 80_000_000;

#[cortex_m_rt::entry]
fn main() -> ! {
    // Retrieve device and core peripherals.
    let dp: pac::Peripherals = pac::Peripherals::take().unwrap();
    let cp: pac::CorePeripherals = pac::CorePeripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    let mut pwr = dp.PWR.constrain(&mut rcc.apb1r1);

    // Configure clocks.
    let clocks = rcc
        .cfgr
        .sysclk(CORE_FREQ.Hz())
        .freeze(&mut flash.acr, &mut pwr);

    defmt::info!("Bioristor application");

    // Setup LED.
    let mut gpioa = dp.GPIOA.split(&mut rcc.ahb2);
    let mut led = gpioa
        .pa5
        .into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper);

    let mut delay = Delay::new(cp.SYST, clocks);
    delay.delay_ms(1000_u32);

    let currents = Currents {
        i_ds_max: -0.0020613,
        i_ds_min: -0.0024066,
        i_gs: 7.79e-06,
    };
    defmt::debug!("{}", currents);

    let model = ThreeEquations::new(MODEL_PARAMS, currents);
    defmt::debug!("{}", MODEL_PARAMS);
    let algorithm: Adaptive<_, MeanRelative, 10> = Adaptive::new(model, ALG_PARAMS);
    defmt::debug!("{}", ALG_PARAMS);

    defmt::info!("Starting algorithm execution...");
    led.set_high();

    let res = algorithm.run();

    led.set_low();

    match res {
        Some((vars, err)) => {
            defmt::info!("Solution found: {}, error: {}", vars, err);
        }
        None => {
            defmt::warn!("No solution found");
        }
    }

    delay.delay_ms(1000_u32);

    loop {
        cortex_m::asm::wfi();
    }
}
