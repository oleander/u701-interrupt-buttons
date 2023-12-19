#![feature(lazy_cell)]

use std::cell::RefCell;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::collections::HashMap;
// use esp_idf_svc::hal;
// use esp_idf_svc::hal::gpio::{Input, Pin};

// use esp32c3_hal::IO;
// use esp32c3_hal::gpio::{InputSignal, Event};
// use critical_section::Mutex;

use critical_section::Mutex;
use esp32c3_hal::{
  clock::ClockControl, gpio::{Event, Gpio9, Input, PullDown, IO}, interrupt, peripherals::{self, Peripherals}, prelude::*, riscv, timer::TimerGroup, Delay, Rtc
};

static BUTTON_STATES: AtomicUsize = AtomicUsize::new(0);
static PROCESSED_STATES: AtomicUsize = AtomicUsize::new(0);

// use std::sync::Mutex;
use std::sync::LazyLock;

static BUTTONS_IDS: LazyLock<Mutex<HashMap<u8, usize>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

fn init_button(pin: u8) {
  // BUTTONS_IDS.lock().unwrap().insert(pin, 1 << pin);
}

fn is_button_pressed(pin: u8) -> bool {
  // Implement this function to return true if the button connected to `pin` is pressed
  false
}

// ISR to update BUTTON_STATES
fn button_isr(button_map: &HashMap<u8, usize>) {
  let mut current_state = 0;

  for (&pin, &mask) in button_map.iter() {
    if is_button_pressed(pin) {
      current_state |= mask;
    }
  }

  BUTTON_STATES.fetch_or(current_state, Ordering::SeqCst);
}

// #[interrupt]
// fn gpio2() {
//   button_isr(&*BUTTONS_IDS.lock().unwrap());
// }

static BUTTON: Mutex<RefCell<Option<Gpio9<Input<PullDown>>>>> = Mutex::new(RefCell::new(None));

fn main() {
  esp_idf_svc::sys::link_patches();
  esp_idf_svc::log::EspLogger::initialize_default();

  log::info!("Hello, world!");

  init_button(0); // Initialize button on pin 0
  init_button(1); // Initialize button on pin 1

  let peripherals = Peripherals::take();
  let system = peripherals.SYSTEM.split();
  let clocks = ClockControl::boot_defaults(system.clock_control).freeze();
  let mut rtc = Rtc::new(peripherals.RTC_CNTL);

  let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks);
  let mut wdt0 = timer_group0.wdt;
  let timer_group1 = TimerGroup::new(peripherals.TIMG1, &clocks);
  let mut wdt1 = timer_group1.wdt;

  rtc.swd.disable();
  rtc.rwdt.disable();
  wdt0.disable();
  wdt1.disable();

  // let io = IO::new(dp.GPIO, dp.IO_MUX);
  let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
  let mut button = io.pins.gpio9.into_pull_down_input();
  let mut led = io.pins.gpio5.into_push_pull_output();

  button.listen(Event::FallingEdge);

  critical_section::with(|cs| BUTTON.borrow_ref_mut(cs).replace(button));
  interrupt::enable(peripherals::Interrupt::GPIO, interrupt::Priority::Priority3).unwrap();

  unsafe {
    riscv::interrupt::enable();
  }

  let mut delay = Delay::new(&clocks);
  loop {
    led.toggle().unwrap();
    delay.delay_ms(500u32);
  }
}

#[interrupt]
fn GPIO() {
  critical_section::with(|cs| {
    esp_println::println!("GPIO interrupt");
    BUTTON.borrow_ref_mut(cs).as_mut().unwrap().clear_interrupt();
  });
}
