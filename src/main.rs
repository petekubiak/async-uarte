#![no_std]
#![no_main]

use embedded_hal::{delay::DelayNs, digital::OutputPin};
use nrf52832_hal::{
    self as hal,
    uarte::{Baudrate, Pins},
};
use panic_rtt_target as _;
use rtt_target::rprintln;

mod time;
mod trace;
mod uarte;

#[cortex_m_rt::entry]
fn main() -> ! {
    rtt_target::rtt_init_print!();
    let peripherals = hal::pac::Peripherals::take().unwrap();

    let uarte0 = peripherals.UARTE0;
    let gpio_p0 = hal::gpio::p0::Parts::new(peripherals.P0);
    let uarte_pins = Pins {
        rxd: gpio_p0.p0_03.into_floating_input().into(),
        txd: gpio_p0
            .p0_04
            .into_push_pull_output(nrf52832_hal::gpio::Level::High)
            .into(),
        cts: None,
        rts: None,
    };
    uarte::init(uarte0, uarte_pins, Baudrate::BAUD9600);
    rprintln!("UARTE initialised");

    let mut timer = hal::Timer::new(peripherals.TIMER0);
    let mut led = gpio_p0.p0_17.into_push_pull_output(hal::gpio::Level::High);

    loop {
        timer.delay_ms(500);
        rprintln!("Setting pin low");
        led.set_low().unwrap();
        timer.delay_ms(500);
        rprintln!("Setting pin high");
        led.set_high().unwrap();
    }
}
