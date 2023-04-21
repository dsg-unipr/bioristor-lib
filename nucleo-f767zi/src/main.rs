#![no_main]
#![no_std]

use defmt_rtt as _; // global logger
use panic_probe as _; // panic handler

use stm32f7xx_hal::{pac, prelude::*};

use bioristor_lib::{
    algorithms::{Adaptive2Equation, Adaptive2Params, Algorithm},
    losses::Absolute,
    models::{Equation, Model},
    params::{Currents, ModelParams, ModulationParams, StemResistanceInvParams, Voltages},
    utils::FloatRange,
};
use profiler::{cycles_to_us, Profiler};

const ALG_PARAMS: Adaptive2Params = Adaptive2Params {
    concentration_range: FloatRange::new(1e-4, 1e-1, 1_000),
    max_iterations: 10,
    reduction_factor: 0.2,
    resistance_range: FloatRange::new(10.0, 100.0, 100),
    saturation_range: FloatRange::new(0.0, 1.0, 100),
    tolerance: 1e-15,
};
//const ALG_PARAMS: BruteForceParams = BruteForceParams {
//    concentration_range: FloatRange::new(1e-4, 1e-1, 100_000),
//    resistance_range: FloatRange::new(10.0, 100.0, 100),
//    saturation_range: FloatRange::new(0.0, 1.0, 100),
//};
//const ALG_PARAMS: GradientDescentParams = GradientDescentParams {
//    concentration_init: 1e-2,
//    grad_tolerance: 1e-9,
//    learning_rate_init: 0.1,
//    max_iterations: 10,
//    tolerance: 1e-15,
//};
//const ALG_PARAMS: NewtonParams = NewtonParams {
//    concentration_init: 1e-2,
//    grad_tolerance: 1e-9,
//    max_iterations: 10,
//    tolerance: 1e-15,
//};
//const ALG_PARAMS: () = ();

const MODEL_PARAMS: ModelParams = ModelParams {
    mod_params: ModulationParams(0.0, -0.01463, -0.32),
    r_dry: 38.2,
    res_params: StemResistanceInvParams(1.35e-6, 2.73e-4),
    voltages: Voltages {
        v_ds: -0.05,
        v_gs: 0.5,
    },
};

const CORE_FREQ: u32 = 216_000_000;

#[cortex_m_rt::entry]
fn main() -> ! {
    // Retrieve core and device peripherals.
    let cp: pac::CorePeripherals = pac::CorePeripherals::take().unwrap();
    let dp: pac::Peripherals = pac::Peripherals::take().unwrap();

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

    let currents = core::hint::black_box(Currents {
        i_ds_on: -0.0026829,
        i_ds_off: -0.0030365,
        i_gs_on: 1.169828e-6,
    });
    defmt::debug!("{}", currents);

    let mut delay = dp.TIM1.delay_us(&clocks);
    delay.delay_ms(1000_u32);

    blue_led.set_low();
    defmt::info!("Starting algorithm execution...");
    red_led.set_high();

    // Setup model and algorithm.
    let model = Equation::new(MODEL_PARAMS, currents);
    defmt::debug!("{}", MODEL_PARAMS);

    let algorithm: Adaptive2Equation<_, Absolute, 10> = Adaptive2Equation::new(ALG_PARAMS, model);
    defmt::debug!("{}", ALG_PARAMS);

    let profiler = Profiler::new(syst);

    // Run algorithm.
    let res = algorithm.run();

    let cycles = profiler.cycles();

    match res {
        Some((variables, error)) => {
            defmt::info!("Solution found: {}, error: {}", variables, error);
        }
        None => {
            defmt::warn!("No solution found");
        }
    }

    red_led.set_low();
    green_led.set_high();

    defmt::info!(
        "Execution took {} CPU cycles, {} us",
        cycles,
        cycles_to_us::<CORE_FREQ>(cycles)
    );

    delay.delay_ms(1000_u32);

    loop {
        cortex_m::asm::wfi();
    }
}
