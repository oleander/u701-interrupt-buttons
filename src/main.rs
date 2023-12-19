use esp_idf_svc::hal::gpio::{InputPin, Pin};
use button_driver::{Button as ButtonBox, ButtonConfig};
use std::collections::HashMap;
use hal::prelude::Peripherals;
use esp_idf_svc::{hal, sys};
use hal::gpio::PinDriver;
use log::{info, warn};
use sys::EspError;

use esp_idf_svc::hal::gpio::Input;
use esp_idf_svc::hal::gpio::AnyInputPin;
use std::sync::Mutex;

type Driver<'a> = PinDriver<'a, AnyInputPin, Input>;
type Button<'a> = ButtonBox<Driver<'a>>;

lazy_static::lazy_static! {
  static ref BUTTONS: Mutex<HashMap<i32, Button<'static>>> = Mutex::new(HashMap::new());
}

macro_rules! setup_button {
  ($pin:expr) => {{
    let pin_id = $pin.pin();

    info!("Setting up button on pin {}", pin_id);

    let config = ButtonConfig::default();
    let gpin = $pin.downgrade_input();
    let driver = PinDriver::input(gpin)?;
    let button = ButtonBox::new(driver, config);

    BUTTONS.lock().unwrap().insert(pin_id, button);
  }};
}

fn main() -> Result<(), EspError> {
  sys::link_patches();
  esp_idf_svc::log::EspLogger::initialize_default();

  let peripherals = Peripherals::take().unwrap();

  setup_button!(peripherals.pins.gpio13);
  setup_button!(peripherals.pins.gpio12);

  loop {
    for (pid, button) in BUTTONS.lock().unwrap().iter_mut() {
      button.tick();

      if button.is_clicked() {
        info!("[{}] Click", pid);
      } else if button.is_double_clicked() {
        info!("[{}] Double click", pid);
      } else if button.is_triple_clicked() {
        info!("[{}] Triple click", pid);
      } else if let Some(dur) = button.current_holding_time() {
        info!("[{}] Holding time {:?}", pid, dur);
      } else if let Some(dur) = button.held_time() {
        info!("[{}] Held time {:?}", pid, dur);
      } else {
        warn!("[{}] Unknown state", pid);
      }

      button.reset();
    }
  }
}
