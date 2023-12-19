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
use lazy_static::*;

type Driver<'a> = PinDriver<'a, AnyInputPin, Input>;
type Button<'a> = ButtonBox<Driver<'a>>;

lazy_static::lazy_static! {
  static ref BUTTONS: Mutex<HashMap<i32, Button<'static>>> = Mutex::new(HashMap::new());
  static ref STATE: Mutex<Click> = Mutex::new(Click::Click(button::ID::A2));
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

#[derive(Debug, Copy, Clone)]
enum Click {
  Holding(button::ID, Duration),
  Held(button::ID, Duration),
  DoubleClick(button::ID),
  TripleClick(button::ID),
  Click(button::ID)
}

pub mod media {
  #[derive(Debug, Copy, Clone)]
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

#[derive(Debug, Copy, Clone)]
enum Packet {
  Command(media::Command),
  Shortcut(u8)
}

pub mod button {
  #[derive(Debug, Hash, PartialEq, Eq, Copy, Clone)]
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
        table.insert(button::ID::A3, media::Command::PrevTrack);
        table.insert(button::ID::A4, media::Command::PlayPause);
        table.insert(button::ID::B2, media::Command::VolumeUp);
        table.insert(button::ID::B3, media::Command::NextTrack);
        table.insert(button::ID::B4, media::Command::Eject);

        table
    };

    pub static ref META: HashMap<button::ID, HashMap<button::ID, u8>> = {
        let mut meta1 = HashMap::new();

        meta1.insert(button::ID::A2, 1); // '1' + 48 = 'a'
        meta1.insert(button::ID::A3, 2); // '1' + 49 = 'b'
        meta1.insert(button::ID::A4, 3); // '1' + 50 = 'c'
        meta1.insert(button::ID::B2, 4); // '1' + 51 = 'd'
        meta1.insert(button::ID::B3, 5); // '1' + 52 = 'e'
        meta1.insert(button::ID::B4, 6); // '1' + 53 = 'f'

        let mut meta2 = HashMap::new();
        meta2.insert(button::ID::A2, 6); // '1' + 0 = '1'
        meta2.insert(button::ID::A3, 7); // '1' + 1 = '2'
        meta2.insert(button::ID::A4, 8); // '1' + 2 = '3'
        meta2.insert(button::ID::B2, 9); // '1' + 3 = '4'
        meta2.insert(button::ID::B3, 10); // '1' + 4 = '5'
        meta2.insert(button::ID::B4, 11); // '1' + 5 = '6'

        let mut table = HashMap::new();
        table.insert(button::ID::M1, meta1);
        table.insert(button::ID::M2, meta2);
        table
    };
}

// Converts button events into click events
fn clicks() -> Vec<Click> {
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

// Converts click events into button events that can later be used to trigger commands
fn events() -> Vec<Packet> {
  let mut state = STATE.lock().unwrap();
  let mut events = Vec::new();
  use crate::button::ID::*;

  for event in clicks() {
    match (event, state.clone()) {
      (to @ Click::Click(M1 | M2), _) => {
        info!("Meta button clicked: {:?}", to);
      },

      (from @ Click::Click(bid), to @ Click::Click(mid @ (M1 | M2))) => {
        META.get(&mid).and_then(|meta| meta.get(&bid)).map(|shortcut| {
          info!("Clicked: {:?} + {:?}", from, to);
          events.push(Packet::Shortcut(*shortcut));
        });
      },

      (Click::Click(bid), _) => {
        EVENT.get(&bid).map(|cmd| {
          info!("Button clicked: {:?}", cmd);
          events.push(Packet::Command(*cmd));
        });
      },

      (to, from) => {
        warn!("Unhandled transition: {:?} -> {:?}", from, to);
      }
    }

    state.clone_from(&event);
  }

  events
}

fn main() -> Result<(), EspError> {
  sys::link_patches();
  esp_idf_svc::log::EspLogger::initialize_default();

  let peripherals = Peripherals::take().unwrap();

  setup_button!(peripherals.pins.gpio13);
  setup_button!(peripherals.pins.gpio12);
  setup_button!(peripherals.pins.gpio9);

  loop {
    for event in events() {
      info!("Event: {:?}", event);
    }
  }
}
