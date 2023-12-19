use button_driver::{Button, ButtonConfig};
use hal::prelude::Peripherals;
use hal::gpio::PinDriver;
use esp_idf_svc::sys;
use esp_idf_svc::hal;
use sys::EspError;
use log::info;

fn main() -> Result<(), EspError> {
  sys::link_patches();
  esp_idf_svc::log::EspLogger::initialize_default();

  let peripherals = Peripherals::take().unwrap();
  let pin = PinDriver::input(peripherals.pins.gpio9)?;

  let mut button = Button::new(pin, ButtonConfig::default());

  loop {
    button.tick();

    if button.is_clicked() {
      info!("Click");
    } else if button.is_double_clicked() {
      info!("Double click");
    } else if button.is_triple_clicked() {
      info!("Triple click");
    } else if let Some(dur) = button.current_holding_time() {
      info!("Held for {dur:?}");
    } else if let Some(dur) = button.held_time() {
      info!("Total holding time {dur:?}");
    }

    button.reset();
  }
}
