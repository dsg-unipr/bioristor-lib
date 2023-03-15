#![no_main]
#![no_std]

use defmt_rtt as _; // global logger
use panic_probe as _; // panic handler

use stm32f7xx_hal::{pac, prelude::*};

use bioristor_lib::{
    algorithms::{AdaptiveSystem, AdaptiveParams, Algorithm},
    losses::MeanRelative,
    models::{Model, System},
    params::{Currents, Geometrics, ModelParams, Voltages},
    utils::FloatRange,
};
use profiler::{cycles_to_ms, Profiler};

const ALG_PARAMS: AdaptiveParams = AdaptiveParams {
    concentration_init: 1e-2,
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

const CORE_FREQ: u32 = 216_000_000;

#[cortex_m_rt::entry]
fn main() -> ! {
    // Retrieve device peripherals.
    let dp: pac::Peripherals = pac::Peripherals::take().unwrap();
    let cp: pac::CorePeripherals = pac::CorePeripherals::take().unwrap();

    let rcc = dp.RCC.constrain();
    let syst = cp.SYST;

    // Configure clocks.
    let clocks = rcc.cfgr.sysclk(CORE_FREQ.Hz()).freeze();

    defmt::info!("Bioristor application");

    // Setup LEDs.
    let gpiob = dp.GPIOB.split();
    let mut green_led = gpiob.pb0.into_push_pull_output();
    let mut blue_led = gpiob.pb7.into_push_pull_output();
    let mut red_led = gpiob.pb14.into_push_pull_output();
    blue_led.set_high();

    let mut delay = dp.TIM1.delay_us(&clocks);
    delay.delay_ms(1000_u32);

    let currents = Currents {
        i_ds_max: -0.0020613,
        i_ds_min: -0.0024066,
        i_gs: 7.79e-06,
    };
    defmt::debug!("{}", currents);

    // Setup model and algorithm.
    let model = System::new(MODEL_PARAMS, currents);
    defmt::debug!("{}", MODEL_PARAMS);
    let algorithm: AdaptiveSystem<_, MeanRelative, 10> = AdaptiveSystem::new(ALG_PARAMS, model);
    defmt::debug!("{}", ALG_PARAMS);

    blue_led.set_low();
    defmt::info!("Starting algorithm execution...");
    red_led.set_high();

    let profiler = Profiler::new(syst);

    // Run algorithm.
    let res = algorithm.run();

    let cycles = profiler.cycles();
    defmt::info!("Execution took {} ms", cycles_to_ms::<CORE_FREQ>(cycles));

    red_led.set_low();
    green_led.set_high();

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
