#![no_main]
#![no_std]

use defmt_rtt as _; // global logger
use panic_probe as _; // panic handler

use stm32l4xx_hal::{delay::Delay, pac, prelude::*};

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

const CORE_FREQ: u32 = 80_000_000;

#[cortex_m_rt::entry]
fn main() -> ! {
    // Retrieve device and core peripherals.
    let dp: pac::Peripherals = pac::Peripherals::take().unwrap();
    let cp: pac::CorePeripherals = pac::CorePeripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    let mut pwr = dp.PWR.constrain(&mut rcc.apb1r1);
    let mut syst = cp.SYST;

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

    // Setup delay.
    let mut delay = Delay::new(syst, clocks);
    for _ in 0..5 {
        led.set_high();
        delay.delay_ms(500_u32);
        led.set_low();
        delay.delay_ms(500_u32);
    }
    syst = delay.free();

    defmt::info!("Starting algorithm execution...");
    let profiler = Profiler::new(syst);

    // Run algorithm.
    let res = algorithm.run();

    let cycles = profiler.cycles();
    defmt::info!(
        "Execution took {} CPU cycles, {} us",
        cycles,
        cycles_to_us::<CORE_FREQ>(cycles)
    );
    syst = profiler.free();

    match res {
        Some((vars, err)) => {
            defmt::info!("Solution found: {}, error: {}", vars, err);
        }
        None => {
            defmt::warn!("No solution found");
        }
    }

    let mut delay = Delay::new(syst, clocks);
    delay.delay_ms(1000_u32);

    loop {
        cortex_m::asm::wfi();
    }
}
