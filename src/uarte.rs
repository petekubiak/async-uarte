use core::sync::atomic::{AtomicBool, AtomicIsize};

use nrf52832_hal::{
    pac::UARTE0,
    uarte::{Baudrate, Pins},
};

#[derive(Default)]
struct Block {
    buffer: [u8; 5],
    filled: AtomicBool,
}

static UARTE_BUFFER: [Block; 3] = [
    Block {
        buffer: [0; 5],
        filled: AtomicBool::new(false),
    },
    Block {
        buffer: [0; 5],
        filled: AtomicBool::new(false),
    },
    Block {
        buffer: [0; 5],
        filled: AtomicBool::new(false),
    },
];

pub struct Uarte {
    uarte: UARTE0,
}

impl Uarte {
    pub fn new(pins: Pins, uarte: UARTE0, baud_rate: Baudrate) -> Self {
        // Enable UARTE peripheral
        uarte.enable.write(|w| {
            w.enable()
                .variant(nrf52832_hal::pac::uarte0::enable::ENABLE_A::ENABLED)
        });

        // Set RX and TX pins
        uarte.psel.rxd.write(|w| w.pin().variant(pins.rxd.pin()));
        uarte.psel.txd.write(|w| w.pin().variant(pins.txd.pin()));

        // Set baud rate
        uarte.baudrate.write(|w| w.baudrate().variant(baud_rate));

        // Initialise the ENDRX -> STARTRX shortcut
        uarte.shorts.write(|w| w.endrx_startrx().set_bit());

        // Enable interrupt for RXSTARTED, ERROR and ENDRX events
        uarte
            .intenset
            .write(|w| w.rxstarted().set_bit().error().set_bit().endrx().set_bit());
        Self { uarte }
    }

    fn update_rxd_buffer_location(&self) {
        static WRITE_OFFSET: AtomicIsize = AtomicIsize::new(0);
        todo!()
    }
}
