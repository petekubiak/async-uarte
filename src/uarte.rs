use core::cell::RefCell;
use core::ops::DerefMut as _;

use critical_section::Mutex;
use nrf52832_hal::pac::interrupt;
use nrf52832_hal::{
    pac::UARTE0,
    uarte::{Baudrate, Pins},
};
use rtt_target::{rprint, rprintln};

const BUFFER_SLOTS_COUNT: usize = 3;

#[derive(Default)]
struct Block {
    buffer: [u8; 5],
    filled: bool,
}

impl Block {
    fn as_str(&self) -> Result<&str, &str> {
        if let Ok(string) = core::str::from_utf8(&self.buffer) {
            Ok(string)
        } else {
            Err("Block contains non-ascii characters!")
        }
    }
}

struct Uarte0 {
    inner: UARTE0,
    buffer: [Block; BUFFER_SLOTS_COUNT],
    write_offset: usize,
    read_offset: usize,
}

impl Uarte0 {
    pub fn new(peripheral: UARTE0) -> Self {
        Self {
            inner: peripheral,
            buffer: core::array::from_fn(|_| Block::default()),
            write_offset: 0,
            read_offset: 0,
        }
    }
    fn update_rxd_buffer_location(&mut self) {
        self.write_offset = (self.write_offset + 1) % BUFFER_SLOTS_COUNT;
        assert!(
            !self.buffer[self.write_offset].filled,
            "UARTE buffer overflow!"
        );
        self.inner.rxd.ptr.write(|w| {
            w.ptr()
                .variant(self.buffer[self.write_offset].buffer.as_ptr() as u32)
        });
    }
    fn next(&mut self) -> Result<&str, &str> {
        let stringify_result = self.buffer[self.read_offset].as_str();
        self.read_offset = (self.read_offset + 1) % BUFFER_SLOTS_COUNT;
        stringify_result
    }
}

pub fn init(peripheral: UARTE0, pins: Pins, baud_rate: Baudrate) {
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
    peripheral.rxd.maxcnt.write(|w| w.maxcnt().variant(5));

    //Enable UARTE interrupt in NVIC
    unsafe { nrf52832_hal::pac::NVIC::unmask(nrf52832_hal::pac::Interrupt::UARTE0_UART0) };

    // Start UARTE and populate the static instance
    critical_section::with(|cs| {
        UARTE0_INSTANCE.replace(cs, Some(Uarte0::new(peripheral)));
        if let Some(instance) = UARTE0_INSTANCE.borrow_ref_mut(cs).deref_mut() {
            instance.inner.rxd.ptr.write(|w| {
                w.ptr()
                    .variant(instance.buffer[instance.write_offset].buffer.as_ptr() as u32)
            });
            instance.inner.tasks_startrx.write(|w| unsafe { w.bits(1) });
        }
    });
}

static UARTE0_INSTANCE: Mutex<RefCell<Option<Uarte0>>> = Mutex::new(RefCell::new(None));

#[interrupt]
fn UARTE0_UART0() {
    critical_section::with(|cs| {
        if let Some(instance) = UARTE0_INSTANCE.borrow_ref_mut(cs).deref_mut() {
            if instance.inner.events_endrx.read().bits() == 1 {
                match instance.next() {
                    Ok(chars) => rprint!("{}", chars),
                    Err(error) => rprintln!("\nError: {}", error),
                }
                instance.inner.events_endrx.reset();
            } else if instance.inner.events_rxstarted.read().bits() == 1 {
                instance.update_rxd_buffer_location();
                instance.inner.events_rxstarted.reset();
            } else if instance.inner.events_error.read().bits() == 1 {
                instance.inner.tasks_flushrx.write(|w| unsafe { w.bits(1) });
                instance.inner.events_error.reset();
            }
        }
    })
}
