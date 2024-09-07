#![no_std]
#![no_main]

use embedded_hal::{delay::DelayNs, digital::OutputPin};
use nrf52832_hal as hal;
use panic_rtt_target as _;
use rtt_target::rprintln;

mod time;
mod trace;
mod uarte;

#[cortex_m_rt::entry]
fn main() -> ! {
    rtt_target::rtt_init_print!();
    rprintln!("Blinky!");
    let peripherals = hal::pac::Peripherals::take().unwrap();
    let mut timer = hal::Timer::new(peripherals.TIMER0);
    let mut led = hal::gpio::p0::Parts::new(peripherals.P0)
        .p0_17
        .into_push_pull_output(hal::gpio::Level::High);

    loop {
        timer.delay_ms(500);
        rprintln!("Setting pin low");
        led.set_low().unwrap();
        timer.delay_ms(500);
        rprintln!("Setting pin high");
        led.set_high().unwrap();
    }
}
