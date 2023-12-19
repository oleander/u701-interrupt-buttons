use std::collections::HashMap;

use esp_idf_svc::hal::gpio::{AnyInputPin, Input, InputPin};
use button_driver::{Button, ButtonConfig};
use esp_idf_svc::hal::gpio::Pin;
use hal::prelude::Peripherals;
use esp_idf_svc::{hal, sys};
use esp_idf_svc::hal::gpio;
use hal::gpio::PinDriver;
use sys::EspError;
use log::{info, warn};

macro_rules! setup_button {
  ($pin:expr, $buttons:expr) => {{
    let pin_id = $pin.pin();

    info!("Setting up button on pin {}", pin_id);

    let config = ButtonConfig::default();
    let gpin = $pin.downgrade_input();
    let driver = PinDriver::input(gpin)?;
    let button = Button::new(driver, config);

    $buttons.insert(pin_id, button);
  }};
}

fn main() -> Result<(), EspError> {
  sys::link_patches();
  esp_idf_svc::log::EspLogger::initialize_default();

  let peripherals = Peripherals::take().unwrap();
  let mut buttons: HashMap<i32, _> = HashMap::new();

  setup_button!(peripherals.pins.gpio13, buttons);
  setup_button!(peripherals.pins.gpio12, buttons);

  loop {
    for (pid, button) in &mut buttons {
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

  Ok(())
}
