use std::cell::RefCell;

use critical_section::Mutex;
use esp_idf_svc as svc;
use svc::{
  sys, hal::gpio::{InterruptType, Gpio12, Input}
};
use hal::{gpio::PinDriver, prelude::Peripherals};
use esp_idf_svc::hal;
use sys::EspError;

static BUTTON: Mutex<RefCell<Option<PinDriver<Gpio12, Input>>>> = Mutex::new(RefCell::new(None));

fn main() -> Result<(), EspError> {
  sys::link_patches();
  svc::log::EspLogger::initialize_default();

  let peripherals = Peripherals::take().unwrap();
  let mut btn = PinDriver::input(peripherals.pins.gpio12)?;

  log::info!("Setup button interrupt");
  btn.set_interrupt_type(InterruptType::NegEdge)?;
  btn.set_pull(hal::gpio::Pull::Up)?;

  log::info!("Set subscribe");
  unsafe {
    btn.subscribe(on_button_a_pushed)?;
  }
  btn.enable_interrupt()?;

  critical_section::with(|cs| BUTTON.borrow_ref_mut(cs).replace(btn));

  loop {
    hal::delay::FreeRtos::delay_ms(100);
  }
}

fn on_button_a_pushed() {
  esp_println::println!("GPIO interrupt (outside)");
  critical_section::with(|cs| {
    BUTTON.borrow_ref_mut(cs).as_mut().unwrap().enable_interrupt().unwrap();
  });
}
