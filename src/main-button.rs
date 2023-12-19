use esp_idf_svc::hal;
use esp_idf_svc::sys;

use hal::{gpio::InterruptType, prelude::Peripherals};
use critical_section::Mutex;
use std::cell::RefCell;

static LED: Mutex<RefCell<Option<hal::gpio::PinDriver<'static, hal::gpio::Gpio3, hal::gpio::Output>>>> =
  Mutex::new(RefCell::new(None));
static BUTTON: Mutex<RefCell<Option<hal::gpio::PinDriver<'static, hal::gpio::Gpio19, hal::gpio::Input>>>> =
  Mutex::new(RefCell::new(None));

fn main() -> anyhow::Result<()> {
  sys::link_patches();
  esp_idf_svc::log::EspLogger::initialize_default();

  log::info!("Setup peripherals");
  let peripherals = Peripherals::take().unwrap();

  log::info!("Setup button and led");
  let mut button = hal::gpio::PinDriver::input(peripherals.pins.gpio19)?;

  log::info!("Setup led");
  let mut led = hal::gpio::PinDriver::output(peripherals.pins.gpio3)?;

  log::info!("Setup button interrupt");
  led.set_high()?;

  log::info!("Setup button interrupt");
  button.set_interrupt_type(InterruptType::AnyEdge)?;

  log::info!("Set subscribe");
  unsafe {
    button.subscribe(on_button_a_pushed)?;
  }

  log::info!("Set LED and BUTTON");
  critical_section::with(|cs| LED.borrow_ref_mut(cs).replace(led));
  critical_section::with(|cs| BUTTON.borrow_ref_mut(cs).replace(button));

  log::info!("Start loop");
  loop {
    hal::delay::FreeRtos::delay_ms(1000);
    // led.set_high()?;
    // // we are sleeping here to make sure the watchdog isn't triggered
    // hal::delay::FreeRtos::delay_ms(1000);

    // led.set_low()?;
  }
}

fn on_button_a_pushed() {
  esp_println::println!("button a pushed");
  critical_section::with(|cs| {
    BUTTON.borrow_ref_mut(cs).as_mut().unwrap().enable_interrupt().unwrap();
    LED.borrow_ref_mut(cs).as_mut().unwrap().toggle().unwrap();
  })
}
