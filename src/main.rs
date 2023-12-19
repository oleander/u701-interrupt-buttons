
use std::sync::mpsc::{channel, Receiver, Sender};
use svc::hal::task::watchdog::TWDTDriver;
use hal::task::watchdog::TWDTConfig;
use std::sync::Mutex as StdMutex;
use hal::prelude::Peripherals;
use lazy_static::lazy_static;
use critical_section::Mutex;
use hal::gpio::PinDriver;
use std::time::Duration;
use svc::hal::cpu::Core;
use std::cell::RefCell;
use esp_idf_svc as svc;
use svc::hal::gpio::*;
use esp_idf_svc::hal;
use sys::EspError;
use svc::sys;

mod keyboard;
use keyboard::Keyboard;

static M1: Mutex<RefCell<Option<PinDriver<Gpio12, Input>>>> = Mutex::new(RefCell::new(None));
static M2: Mutex<RefCell<Option<PinDriver<Gpio13, Input>>>> = Mutex::new(RefCell::new(None));
// 6 more buttons will be added

static EVENT: Mutex<RefCell<Option<i32>>> = Mutex::new(RefCell::new(None));
static STATE: Mutex<RefCell<Option<i32>>> = Mutex::new(RefCell::new(None));

lazy_static! {
  static ref CHANNEL: (Mutex<Sender<i32>>, StdMutex<Receiver<i32>>) = {
    let (send, recv) = channel();
    let recv = StdMutex::new(recv);
    let send = Mutex::new(send);
    (send, recv)
  };
}

macro_rules! setup_button_interrupt {
  ($mutex:ident, $pin:expr) => {
    let mut btn = PinDriver::input($pin)?;

    // Trigger when button is pushed
    btn.set_interrupt_type(InterruptType::LowLevel)?;

    // Default is pull up
    btn.set_pull(hal::gpio::Pull::Up)?;

    unsafe {
      // On click
      btn
        .subscribe(|| {
          critical_section::with(|cs| {
            let mut bbrn = $mutex.borrow_ref_mut(cs);
            let btn = bbrn.as_mut().unwrap();
            EVENT.borrow_ref_mut(cs).replace(btn.pin());
            btn.enable_interrupt().unwrap();
          });
        })
        .unwrap();
    }

    btn.enable_interrupt()?;
    critical_section::with(|cs| $mutex.borrow_ref_mut(cs).replace(btn));
  };
}

fn main() -> Result<(), EspError> {
  sys::link_patches();
  svc::log::EspLogger::initialize_default();

  let peripherals = Peripherals::take().unwrap();

  let mut keyboard = Keyboard::new();

  setup_button_interrupt!(M1, peripherals.pins.gpio12);
  setup_button_interrupt!(M2, peripherals.pins.gpio13);

  let config = TWDTConfig {
    duration: Duration::from_secs(10),
    panic_on_trigger: false,
    subscribed_idle_tasks: Core::Core0.into()
  };

  let mut driver = TWDTDriver::new(peripherals.twdt, &config)?;

  let mut watchdog = driver.watch_current_task()?;

  loop {
    critical_section::with(|cs| {
      let curr = EVENT.borrow_ref_mut(cs).take();
      let prev = STATE.borrow_ref_mut(cs).take();

      match (curr, prev) {
        (Some(curr), Some(prev)) if curr == prev => {
          // Event has already been processed without a relase
        },

        // A new button was pressed
        (Some(id), _) => {
          esp_println::println!("Button {:?} pushed", id);
          if keyboard.connected() {
            keyboard.write(id.to_string().as_str());
          }
          STATE.borrow_ref_mut(cs).replace(id);
        },

        (None, _) => {
          // No button pressed
        }
      }
    });

    watchdog.feed().unwrap();
    hal::delay::FreeRtos::delay_ms(50);
  }
}
