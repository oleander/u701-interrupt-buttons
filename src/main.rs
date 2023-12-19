// #![no_std]
#![no_main]

use core::panic::PanicInfo;
use esp32_hal::{
  prelude::*, gpio::{Gpio13, PullUp, Input, Pin, Event}, clock::ClockControl, peripherals::Peripherals, Delay, IO
};

#[entry]
fn main() -> ! {
  sys::link_patches();
  svc::log::EspLogger::initialize_default();

  log::info!("Peripherals initialized");
  let dp = Peripherals::take();

  log::info!("System setup");
  let system = dp.SYSTEM.split();

  log::info!("Clock setup");
  let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

  log::info!("Delay setup");
  let mut delay = Delay::new(&clocks);

  log::info!("GPIO setup");
  let io = IO::new(dp.GPIO, dp.IO_MUX);

  log::info!("GPIO13 setup");
  let mut pin = io.pins.gpio13.into_pull_up_input();

  log::info!("GPIO13 interrupt setup");
  pin.listen(Event::LowLevel);

  log::info!("GPIO13 interrupt enable");
  pin.enable_input(true);

  log::info!("GPIO13 pull-up enable");
  pin.internal_pull_up(true);

  loop {
    log::info!("GPIO13 state: {}", pin.is_low().unwrap());
    delay.delay_ms(1000 as u32);
  }
}

#[interrupt]
fn GPIO() {
  log::info!("GPIO13 interrupt");
}
