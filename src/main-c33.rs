#![allow(dead_code)]
#![allow(unused_variables)]
#![feature(lazy_cell)]

use std::sync::atomic::AtomicUsize;
use std::collections::HashMap;
use critical_section::Mutex;
use std::sync::LazyLock;
use std::cell::RefCell;
use esp32c3_hal::{
  clock::ClockControl, gpio::{Event, Gpio9, Input, PullDown, IO}, interrupt, peripherals::{self, Peripherals}, prelude::*, riscv, timer::TimerGroup, Delay, Rtc
};

static BUTTONS_IDS: LazyLock<Mutex<HashMap<u8, usize>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
static BUTTON: Mutex<RefCell<Option<Gpio9<Input<PullDown>>>>> = Mutex::new(RefCell::new(None));
static PROCESSED_STATES: AtomicUsize = AtomicUsize::new(0);
static BUTTON_STATES: AtomicUsize = AtomicUsize::new(0);

fn main() {
  esp_idf_svc::sys::link_patches();
  esp_idf_svc::log::EspLogger::initialize_default();

  log::info!("Hello, world!");

  log::info!("Fetching peripherals");
  let peripherals = Peripherals::take();

  log::info!("Setup system");
  let system = peripherals.SYSTEM.split();

  log::info!("Setup clocks");
  let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

  log::info!("Setup RTC");
  let mut rtc = Rtc::new(peripherals.RTC_CNTL);

  log::info!("Setup watchdog 1");
  let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks);
  let mut wdt0 = timer_group0.wdt;

  log::info!("Setup watchdog 2");
  let timer_group1 = TimerGroup::new(peripherals.TIMG1, &clocks);
  let mut wdt1 = timer_group1.wdt;

  log::info!("Disable watchdog swd");
  rtc.swd.disable();

  log::info!("Disable watchdog rwdt");
  rtc.rwdt.disable();

  log::info!("Disable watchdog wdt0");
  wdt0.disable();

  log::info!("Disable watchdog wdt1");
  wdt1.disable();

  log::info!("Setup IO");
  let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

  log::info!("Setup button");
  let mut button = io.pins.gpio9.into_pull_down_input();

  log::info!("Setup led");
  let mut led = io.pins.gpio3.into_push_pull_output();

  log::info!("Setup button interrupt");
  button.listen(Event::FallingEdge);

  log::info!("Setup button interrupt");
  critical_section::with(|cs| BUTTON.borrow_ref_mut(cs).replace(button));

  log::info!("Setup button interrupt");
  interrupt::enable(peripherals::Interrupt::GPIO, interrupt::Priority::Priority3).unwrap();

  log::info!("Setup button interrupt");
  unsafe {
    riscv::interrupt::enable();
  }

  log::info!("Setup delay");
  let mut delay = Delay::new(&clocks);

  log::info!("Entering loop");
  loop {
    log::info!("Toggle LED");
    led.toggle().unwrap();

    log::info!("Wait for 500ms");
    delay.delay_ms(500u32);
  }
}

#[interrupt]
fn GPIO() {
  esp_println::println!("GPIO interrupt (outside)");
  critical_section::with(|cs| {
    esp_println::println!("GPIO interrupt (inside)");
    BUTTON.borrow_ref_mut(cs).as_mut().unwrap().clear_interrupt();
  });
}
