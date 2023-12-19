use button_driver::{Button, ButtonConfig};
use esp_idf_svc::hal::gpio::AnyInputPin;
use esp_idf_svc::hal::gpio::Input;
use esp_idf_svc::hal::gpio::InputPin;
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
  let config = ButtonConfig::default();
  let mut buttons: Vec<button_driver::Button<PinDriver<AnyInputPin, Input>>> = Vec::new();

  let pin = peripherals.pins.gpio13;
  let dpin = pin.downgrade_input();
  let pin1 = PinDriver::input(dpin)?;
  buttons.push(Button::new(pin1.into(), config));
  // buttons.push(Button::new(PinDriver::input(peripherals.pins.gpio12)?, config));

  // loop {
  //   button.tick();

  //   if button.is_clicked() {
  //     info!("Click");
  //   } else if button.is_double_clicked() {
  //     info!("Double click");
  //   } else if button.is_triple_clicked() {
  //     info!("Triple click");
  //   } else if let Some(dur) = button.current_holding_time() {
  //     info!("Held for {dur:?}");
  //   } else if let Some(dur) = button.held_time() {
  //     info!("Total holding time {dur:?}");
  //   }

  //   button.reset();
  // }
  Ok(())
}
