use core::{
    cell::Cell,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
};

use critical_section::Mutex;
use nrf52832_hal::{
    pac::UARTE0,
    uarte::{Baudrate, Pins},
};

const BUFFER_SLOTS_COUNT: usize = 3;

#[derive(Default)]
struct Block {
    buffer: [u8; 5],
    filled: AtomicBool,
}

struct Uarte0 {
    inner: UARTE0,
    buffer: [Block; BUFFER_SLOTS_COUNT],
}

static UARTE0_INSTANCE: Mutex<Cell<Option<Uarte0>>> = Mutex::new(Cell::new(None));

impl Uarte0 {
    pub fn init(pins: Pins, peripheral: UARTE0, baud_rate: Baudrate) {
        // Populate the static instance
        critical_section::with(|cs| {
            UARTE0_INSTANCE.borrow(cs).set(Some(Self {
                inner: peripheral,
                buffer: core::array::from_fn(|_| Block::default()),
            }));
        });

        // Enable UARTE peripheral
        peripheral.enable.write(|w| {
            w.enable()
                .variant(nrf52832_hal::pac::uarte0::enable::ENABLE_A::ENABLED)
        });

        // Set RX and TX pins
        peripheral
            .psel
            .rxd
            .write(|w| w.pin().variant(pins.rxd.pin()));
        peripheral
            .psel
            .txd
            .write(|w| w.pin().variant(pins.txd.pin()));

        // Set baud rate
        peripheral
            .baudrate
            .write(|w| w.baudrate().variant(baud_rate));

        // Initialise the ENDRX -> STARTRX shortcut
        peripheral.shorts.write(|w| w.endrx_startrx().set_bit());

        // Enable interrupt for RXSTARTED, ERROR and ENDRX events
        peripheral
            .intenset
            .write(|w| w.rxstarted().set_bit().error().set_bit().endrx().set_bit());

        // Set up UARTE DMA
        uarte.inner.rxd.maxcnt.write(|w| w.maxcnt().variant(5));
        uarte.update_rxd_buffer_location();
    }

    fn update_rxd_buffer_location(&self) {
        static WRITE_OFFSET: AtomicUsize = AtomicUsize::new(0);
        let new_offset = (WRITE_OFFSET.load(Ordering::Acquire) + 1) % BUFFER_SLOTS_COUNT;
        self.inner.rxd.ptr.write(|w| {
            w.ptr()
                .variant(self.buffer[new_offset].buffer.as_ptr() as u32)
        });
        WRITE_OFFSET.store(new_offset, Ordering::Release);
    }
}
