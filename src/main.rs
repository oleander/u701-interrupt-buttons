use esp_idf_svc::hal::gpio::{AnyInputPin, Input, InputPin, Pin};
use button_driver::{Button as ButtonBox, ButtonConfig};
use std::collections::HashMap;
use std::time::Duration;
use hal::prelude::Peripherals;
use esp_idf_svc::{hal, sys};
use hal::gpio::PinDriver;
use log::{info, warn};
use sys::EspError;
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

enum Click {
  Click(i32),
  DoubleClick(i32),
  TripleClick(i32),
  Holding(i32, Duration),
  Held(i32, Duration),
  Unknown(i32)
}

fn events() -> Vec<Click> {
  let mut events = Vec::new();

  for (pid, button) in BUTTONS.lock().unwrap().iter_mut() {
    button.tick();

    if button.is_clicked() {
      events.push(Click::Click(*pid));
    } else if button.is_double_clicked() {
      events.push(Click::DoubleClick(*pid));
    } else if button.is_triple_clicked() {
      events.push(Click::TripleClick(*pid));
    } else if let Some(dur) = button.current_holding_time() {
      events.push(Click::Holding(*pid, dur));
    } else if let Some(dur) = button.held_time() {
      events.push(Click::Held(*pid, dur));
    } else {
      events.push(Click::Unknown(*pid));
    }

    button.reset();
  }

  events
}

fn main() -> Result<(), EspError> {
  sys::link_patches();
  esp_idf_svc::log::EspLogger::initialize_default();

  let peripherals = Peripherals::take().unwrap();

  setup_button!(peripherals.pins.gpio13);
  setup_button!(peripherals.pins.gpio12);

  loop {
    for event in events() {
      match event {
        Click::Click(pid) => {
          info!("Button {} clicked", pid);
        },
        Click::DoubleClick(pid) => {
          info!("Button {} double clicked", pid);
        },
        Click::TripleClick(pid) => {
          info!("Button {} triple clicked", pid);
        },
        Click::Holding(pid, dur) => {
          info!("Button {} holding for {:?}ms", pid, dur);
        },
        Click::Held(pid, dur) => {
          info!("Button {} held for {:?}ms", pid, dur);
        },
        Click::Unknown(pid) => {
          warn!("Button {} unknown event", pid);
        }
      }
    }
  }
}
