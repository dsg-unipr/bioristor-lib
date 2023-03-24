#![no_main]
#![no_std]

use defmt_rtt as _; // global logger
use panic_probe as _; // panic handler

use stm32f7xx_hal::{pac, prelude::*};

use bioristor_lib::{
    algorithms::{Algorithm, NewtonEquation, NewtonParams},
    losses::Absolute,
    models::{Equation, Model},
    params::{Currents, ModelParams, ModulationParams, StemResistanceInvParams, Voltages},
};
use profiler::{cycles_to_us, Profiler};

const ALG_PARAMS: NewtonParams = NewtonParams {
    concentration_init: 1e-2,
    grad_tolerance: 1e-9,
    max_iterations: 10,
    tolerance: 1e-15,
};

const MODEL_PARAMS: ModelParams = ModelParams {
    mod_params: ModulationParams(0.0, -0.01463, -0.32),
    r_dry: 15.8,
    res_params: StemResistanceInvParams(1.35e-6, 2.73e-4),
    voltages: Voltages {
        v_ds: -0.05,
        v_gs: 0.5,
    },
};

const CORE_FREQ: u32 = 216_000_000;

#[cortex_m_rt::entry]
fn main() -> ! {
    // Retrieve device and core peripherals.
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
        i_ds_on: -0.0028583,
        i_ds_off: -0.0031083,
        i_gs_on: 7.775_862e-7,
    };
    defmt::debug!("{}", currents);

    // Setup model and algorithm.
    let model = Equation::new(MODEL_PARAMS, currents);
    defmt::debug!("{}", MODEL_PARAMS);
    let algorithm: NewtonEquation<_, Absolute> = NewtonEquation::new(ALG_PARAMS, model);
    defmt::debug!("{}", ALG_PARAMS);

    blue_led.set_low();
    defmt::info!("Starting algorithm execution...");
    red_led.set_high();

    let profiler = Profiler::new(syst);

    // Run algorithm.
    let res = algorithm.run();

    let cycles = profiler.cycles();
    defmt::info!(
        "Execution took {} CPU cycles, {} us",
        cycles,
        cycles_to_us::<CORE_FREQ>(cycles)
    );

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
