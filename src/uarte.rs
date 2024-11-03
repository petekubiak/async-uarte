use core::cell::RefCell;
use core::ops::DerefMut as _;

use critical_section::Mutex;
use nrf52832_hal::pac::{interrupt, CLOCK};
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

pub fn init(uarte: UARTE0, clock: &CLOCK, pins: Pins, baud_rate: Baudrate) {
    // Start the HF clock
    clock.tasks_hfclkstart.write(|w| unsafe { w.bits(1) });
    rprintln!("HF clock started");

    // Enable UARTE peripheral
    uarte.enable.write(|w| {
        w.enable()
            .variant(nrf52832_hal::pac::uarte0::enable::ENABLE_A::ENABLED)
    });

    // Set RX and TX pins
    uarte.psel.rxd.write(|w| {
        w.pin()
            .variant(pins.rxd.pin())
            .connect()
            .variant(nrf52832_hal::pac::uarte0::psel::rxd::CONNECT_A::CONNECTED)
    });
    uarte.psel.txd.write(|w| {
        w.pin()
            .variant(pins.txd.pin())
            .connect()
            .variant(nrf52832_hal::pac::uarte0::psel::txd::CONNECT_A::CONNECTED)
    });

    // Set baud rate
    uarte.baudrate.write(|w| w.baudrate().variant(baud_rate));

    // Initialise the ENDRX -> STARTRX shortcut
    uarte.shorts.write(|w| w.endrx_startrx().set_bit());

    // Enable interrupt for RXSTARTED, ERROR and ENDRX events
    uarte
        .intenset
        .write(|w| w.rxstarted().set_bit().error().set_bit().endrx().set_bit());

    // Set up UARTE DMA
    uarte.rxd.maxcnt.write(|w| w.maxcnt().variant(5));

    //Enable UARTE interrupt in NVIC
    unsafe { nrf52832_hal::pac::NVIC::unmask(nrf52832_hal::pac::Interrupt::UARTE0_UART0) };

    // Start UARTE and populate the static instance
    critical_section::with(|cs| {
        UARTE0_INSTANCE.replace(cs, Some(Uarte0::new(uarte)));
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
