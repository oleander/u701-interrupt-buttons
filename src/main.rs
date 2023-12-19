use button_driver::{Button, ButtonConfig};
use esp_idf_svc::{sys, hal::gpio::InterruptType};
use std::time::Instant;
use esp_idf_svc::hal;
use hal::{gpio::PinDriver, prelude::Peripherals};
use sys::EspError;
use log::info;

fn main() -> Result<(), EspError> {
  sys::link_patches();
  esp_idf_svc::log::EspLogger::initialize_default();

  let peripherals = Peripherals::take().unwrap();
  // let pin = PinDriver::input(peripherals.pins.gpio12)?;
  let mut btn = PinDriver::input(peripherals.pins.gpio9)?;

  // let mut button = Button::new(pin, ButtonConfig::default());

  log::info!("Setup button interrupt");
  btn.set_interrupt_type(InterruptType::LowLevel)?;
  btn.set_pull(hal::gpio::Pull::Down)?;

  log::info!("Set subscribe");
  unsafe {
    btn.subscribe(on_button_a_pushed)?;
  }
  btn.enable_interrupt()?;

  loop {
    // button.tick();

    // if button.is_clicked() {
    //     info!("Click");
    // } else if button.is_double_clicked() {
    //     info!("Double click");
    // } else if button.is_triple_clicked() {
    //     info!("Triple click");
    // } else if let Some(dur) = button.current_holding_time() {
    //     info!("Held for {dur:?}");
    // } else if let Some(dur) = button.held_time() {
    //     info!("Total holding time {dur:?}");
    // }

    // button.reset();

    // delay
    hal::delay::FreeRtos::delay_ms(100);
  }
}

fn on_button_a_pushed() {
  info!("Button pushed");
}
