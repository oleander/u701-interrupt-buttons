#![no_std]
#![no_main]

use core::panic::PanicInfo;
use esp32_hal::{
  clock_control::{sleep, ClockControl, XTAL_FREQUENCY_AUTO}, dport::Split, dprintln, prelude::*, target, timer::Timer
};
use esp32_hal::analog::config::{Adc1Config, Attenuation};
use esp32_hal::analog::adc::ADC;
use esp32_hal::target::EFUSE;
use esp32_hal::efuse::Efuse;

#[entry]
fn main() -> ! {
  let dp = target::Peripherals::take().expect("failed to acquire peripherals");
  let (_, dport_clock_control) = dp.DPORT.split();

  let clock_control = ClockControl::new(dp.RTCCNTL, dp.APB_CTRL, dport_clock_control, XTAL_FREQUENCY_AUTO).unwrap();

  // disable RTC watchdog
  let (clock_control_config, mut watchdog) = clock_control.freeze().unwrap();
  watchdog.disable();

  // disable MST watchdogs
  let (.., mut watchdog0) = Timer::new(dp.TIMG0, clock_control_config);
  let (.., mut watchdog1) = Timer::new(dp.TIMG1, clock_control_config);
  watchdog0.disable();
  watchdog1.disable();

  let gpios = dp.GPIO.split();
  let mut pin = gpios.gpio36.into_analog();
  let mut adc_config = Adc1Config::new();
  adc_config.enable_pin(&pin, Attenuation::Attenuation11dB);

  let analog = dp.SENS.split();
  let mut adc = ADC::adc1(analog.adc1, adc_config).unwrap();

  loop {
    let raw: u16 = nb::block!(adc.read(&mut pin)).unwrap();
    let reading = convert_to_volts(raw);
    convert_to_fahrenheit(reading);
    sleep(1.s());
  }
}

fn convert_to_fahrenheit(reading: f32) {
  let celsius = reading / 0.01;
  let fahrenheit = celsius * 1.8 + 32.0;
  dprintln!("fahrenheit: {}, celsius: {}", fahrenheit, celsius);
}

fn convert_to_volts(value: u16) -> f32 {
  // for some reason, the adc reading in the heltec = 4095 - value.
  let reading = 4095 - value;
  4.96 * reading as f32 / 4095.0
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
  dprintln!("PANIC: {:?}", info);
  loop {}
}
