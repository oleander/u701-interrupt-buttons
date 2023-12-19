use hal::{gpio::PinDriver, prelude::Peripherals};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Mutex as StdMutex;
use svc::{sys, hal::gpio::*};
use lazy_static::lazy_static;
use critical_section::Mutex;
use std::cell::RefCell;
use esp_idf_svc as svc;
use esp_idf_svc::hal;
use sys::EspError;

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
    btn.set_interrupt_type(InterruptType::AnyEdge)?;

    // Default is pull up
    btn.set_pull(hal::gpio::Pull::Up)?;

    unsafe {
      // On click
      btn
        .subscribe(|| {
          critical_section::with(|cs| {
            let mut bbrn = $mutex.borrow_ref_mut(cs);
            let btn = bbrn.as_mut().unwrap();

            if btn.is_high() {
              EVENT.borrow_ref_mut(cs).replace(btn.pin());
            } else {
              EVENT.borrow_ref_mut(cs).replace(0);
            }

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

  loop {
    critical_section::with(|cs| {
      let curr = EVENT.borrow_ref_mut(cs).take();
      let prev = STATE.borrow_ref_mut(cs).take();

      match (curr, prev) {
        (Some(curr), Some(prev)) if curr == prev => {
          // Event has already been processed without a relase
        },

        // Button was released
        (Some(0), Some(id)) => {
          esp_println::println!("Button released: {:?}", id);
        },

        // Button was released but no previous state
        (Some(0), None) => {
          esp_println::println!("[BUG] Button released but no previous state");
        },

        // A new button was pressed
        (Some(id), _) => {
          esp_println::println!("Button {:?} pushed", id);
          keyboard.write(id.to_string().as_str());
          STATE.borrow_ref_mut(cs).replace(id);
        },

        (None, _) => {
          // No button pressed
        }
      }
    });

    hal::delay::FreeRtos::delay_ms(50);
  }
}
