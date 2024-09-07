use fugit::{Duration, Instant};
use nrf52832_hal as hal;

use hal::pac::RTC0;
use hal::Rtc;

type TickInstant = Instant<u64, 1, 32768>;
type TickDuration = Duration<u64, 1, 32768>;

pub struct Timer<'a> {
    deadline: TickInstant,
    ticker: &'a Ticker,
}

impl<'a> Timer<'a> {
    pub fn new(duration: TickDuration, ticker: &'a Ticker) -> Self {
        Self {
            deadline: ticker.now() + duration,
            ticker,
        }
    }

    pub fn elapsed(&self) -> bool {
        self.ticker.now() >= self.deadline
    }
}

pub struct Ticker {
    rtc: Rtc<RTC0>,
}

impl Ticker {
    pub fn new(rtc0: RTC0) -> Self {
        let rtc = Rtc::new(rtc0, 0).unwrap();
        rtc.enable_counter();
        Self { rtc }
    }

    pub fn now(&self) -> TickInstant {
        TickInstant::from_ticks(self.rtc.get_counter() as u64)
    }
}
