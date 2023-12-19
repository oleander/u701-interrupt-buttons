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

#[derive(Debug)]
enum Click {
  Holding(button::ID, Duration),
  Held(button::ID, Duration),
  DoubleClick(button::ID),
  TripleClick(button::ID),
  Click(button::ID)
}

use lazy_static::*;

pub mod media {
  pub enum Command {
    VolumeDown,
    NextTrack,
    PrevTrack,
    PlayPause,
    VolumeUp,
    Eject
  }

  impl Command {
    pub fn to_command(&self) -> [u8; 2] {
      match self {
        Command::VolumeDown => [64, 0],
        Command::NextTrack => [1, 0],
        Command::PrevTrack => [2, 0],
        Command::PlayPause => [8, 0],
        Command::VolumeUp => [32, 0],
        Command::Eject => [16, 0]
      }
    }
  }
}

pub mod button {
  #[derive(Debug, Hash, PartialEq, Eq)]
  pub enum ID {
    M1 = 0x04, // Corresponds to BUTTON_1: Red (Meta)
    A2 = 0x50, // Corresponds to BUTTON_2: Black (Volume down)
    A3 = 0x51, // Corresponds to BUTTON_3: Blue (Prev track)
    A4 = 0x52, // Corresponds to BUTTON_4: Black (Play/Pause)
    M2 = 0x29, // Corresponds to BUTTON_5: Red (Meta)
    B2 = 0x4F, // Corresponds to BUTTON_6: Black (Volume up)
    B3 = 0x05, // Corresponds to BUTTON_7: Blue (Next track)
    B4 = 0x28  // Corresponds to BUTTON_8: Black (Toggle AC)
  }

  impl From<ID> for i32 {
    fn from(id: ID) -> Self {
      id as Self
    }
  }

  impl From<&i32> for ID {
    fn from(id: &i32) -> Self {
      match id {
        0x04 => ID::M1,
        0x50 => ID::A2,
        0x51 => ID::A3,
        0x52 => ID::A4,
        0x29 => ID::M2,
        0x4F => ID::B2,
        0x05 => ID::B3,
        0x28 => ID::B4,
        _ => panic!("Unknown button ID: {}", id)
      }
    }
  }
}

lazy_static! {
    pub static ref EVENT: HashMap<button::ID, media::Command> = {
        let mut table = HashMap::new();

        table.insert(button::ID::A2, media::Command::VolumeDown);
        // table.insert(Button::A3 as u8, Command::PrevTrack.to_command());
        // table.insert(Button::A4 as u8, Command::PlayPause.to_command());
        // table.insert(Button::B2 as u8, Command::VolumeUp.to_command());
        // table.insert(Button::B3 as u8, Command::NextTrack.to_command());
        // table.insert(Button::B4 as u8, Command::Eject.to_command());

        table
    };

    pub static ref META: HashMap<button::ID, HashMap<button::ID, u8>> = {
        let mut meta1 = HashMap::new();

        meta1.insert(button::ID::A2, 1); // '1' + 48 = 'a'
        // meta1.insert(Button::A3 as u8, 1); // '1' + 49 = 'b'
        // meta1.insert(Button::A4 as u8, 2); // '1' + 50 = 'c'
        // meta1.insert(Button::B2 as u8, 3); // '1' + 51 = 'd'
        // meta1.insert(Button::B3 as u8, 4); // '1' + 52 = 'e'
        // meta1.insert(Button::B4 as u8, 5); // '1' + 53 = 'f'

        let mut meta2 = HashMap::new();
        meta2.insert(button::ID::A2, 6); // '1' + 0 = '1'
        // meta2.insert(Button::A3 as u8, 7); // '1' + 1 = '2'
        // meta2.insert(Button::A4 as u8, 8); // '1' + 2 = '3'
        // meta2.insert(Button::B2 as u8, 9); // '1' + 3 = '4'
        // meta2.insert(Button::B3 as u8, 10); // '1' + 4 = '5'
        // meta2.insert(Button::B4 as u8, 11); // '1' + 5 = '6'
//
        let mut table = HashMap::new();
        table.insert(button::ID::M1, meta1);
        table.insert(button::ID::M2, meta2);
        table
    };
}

fn events() -> Vec<Click> {
  let mut events = Vec::new();

  for (pid, button) in BUTTONS.lock().unwrap().iter_mut() {
    button.tick();

    if button.is_clicked() {
      events.push(Click::Click(pid.into()));
    } else if button.is_double_clicked() {
      events.push(Click::DoubleClick(pid.into()));
    } else if button.is_triple_clicked() {
      events.push(Click::TripleClick(pid.into()));
    } else if let Some(dur) = button.current_holding_time() {
      events.push(Click::Holding(pid.into(), dur));
    } else if let Some(dur) = button.held_time() {
      events.push(Click::Held(pid.into(), dur));
    }

    button.reset();
  }

  events
}

fn main() -> Result<(), EspError> {
  sys::link_patches();
  esp_idf_svc::log::EspLogger::initialize_default();

  let peripherals = Peripherals::take().unwrap();
  let mut state = Click::Click(button::ID::A2);

  setup_button!(peripherals.pins.gpio13);
  setup_button!(peripherals.pins.gpio12);

  loop {
    for event in events() {
      match event {
        Click::Click(ref pid) => {
          info!("Button {:?} clicked", pid);
        },
        Click::DoubleClick(ref pid) => {
          info!("Button {:?} double clicked", pid);
        },
        Click::TripleClick(ref pid) => {
          info!("Button {:?} triple clicked", pid);
        },
        Click::Holding(ref pid, dur) => {
          info!("Button {:?} holding for {:?}ms", pid, dur);
        },
        Click::Held(ref pid, dur) => {
          info!("Button {:?} held for {:?}ms", pid, dur);
        }
      }

      state = event;
    }
  }
}
