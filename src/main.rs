use std::cell::RefCell;

use critical_section::Mutex;
use std::sync::Mutex as StdMutex;
use esp_idf_svc as svc;
use svc::{sys, hal::gpio::*};
use hal::{gpio::PinDriver, prelude::Peripherals};
use std::sync::mpsc::{channel, Receiver, Sender};

use esp_idf_svc::hal;
use lazy_static::lazy_static;

use sys::EspError;

static M1: Mutex<RefCell<Option<PinDriver<Gpio12, Input>>>> = Mutex::new(RefCell::new(None));
static M2: Mutex<RefCell<Option<PinDriver<Gpio13, Input>>>> = Mutex::new(RefCell::new(None));

// Use an atomic u8 to store the state of the button
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

    log::info!("Setup button interrupt");
    btn.set_interrupt_type(InterruptType::LowLevel)?;
    btn.set_pull(hal::gpio::Pull::Up)?;
    let pin = btn.pin();

    log::info!("Set subscribe");
    unsafe {
      btn
        .subscribe(move || {
          // esp_println::println!("Button pressed: {:?}", pin);
          critical_section::with(|cs| {
            STATE.borrow_ref_mut(cs).replace(pin);
            $mutex.borrow_ref_mut(cs).as_mut().unwrap().enable_interrupt().unwrap();
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

  setup_button_interrupt!(M1, peripherals.pins.gpio12);
  setup_button_interrupt!(M2, peripherals.pins.gpio13);

  loop {
    critical_section::with(|cs| {
      if let Some(pin) = STATE.borrow_ref_mut(cs).take() {
        esp_println::println!("Button pressed: {:?}", pin);
      }
    });

    hal::delay::FreeRtos::delay_ms(30);
  }
}
