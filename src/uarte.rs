use nrf52832_hal::{
    pac::UARTE0,
    uarte::{Baudrate, Pins},
};

pub struct Uarte {
    uarte: UARTE0,
}

impl Uarte {
    pub fn new(pins: Pins, uarte: UARTE0, baud_rate: Baudrate) -> Self {
        uarte.psel.rxd.write(|w| w.pin().variant(pins.rxd.pin()));
        uarte.psel.txd.write(|w| w.pin().variant(pins.txd.pin()));
        uarte.baudrate.write(|w| w.baudrate().variant(baud_rate));
        Self { uarte }
    }
}
