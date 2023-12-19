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

static M1: Mutex<RefCell<Option<PinDriver<Gpio12, Input>>>> = Mutex::new(RefCell::new(None));
static M2: Mutex<RefCell<Option<PinDriver<Gpio13, Input>>>> = Mutex::new(RefCell::new(None));

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

    btn.set_interrupt_type(InterruptType::LowLevel)?;
    btn.set_pull(hal::gpio::Pull::Up)?;

    unsafe {
      btn
        .subscribe(|| {
          critical_section::with(|cs| {
            let mut bbrn = $mutex.borrow_ref_mut(cs);
            let btn = bbrn.as_mut().unwrap();
            btn.enable_interrupt().unwrap();
            STATE.borrow_ref_mut(cs).replace(btn.pin());
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
