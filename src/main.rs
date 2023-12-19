use esp_idf_svc as svc;
use svc::{sys, hal::gpio::InterruptType};
use hal::{gpio::PinDriver, prelude::Peripherals};
use esp_idf_svc::hal;
use sys::EspError;

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

  loop {
    hal::delay::FreeRtos::delay_ms(100);
  }
}

fn on_button_a_pushed() {
  esp_println::println!("GPIO interrupt (outside)");
}
