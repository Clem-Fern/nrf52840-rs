#![no_main]
#![no_std]

use core::panic::PanicInfo;

use embedded_hal::{delay::DelayNs, digital::OutputPin};
use nrf52840_hal::{gpio::{self, Level}, pac::Peripherals, Timer};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[cortex_m_rt::entry]
fn main() -> ! {
    let p = Peripherals::take().unwrap();
    let mut timer = Timer::new(p.TIMER0);
    let port0 = gpio::p0::Parts::new(p.P0);
    let port1 = gpio::p1::Parts::new(p.P1);

    let mut red_led = port0.p0_06.into_push_pull_output(Level::Low);
    let mut blue_led = port1.p1_11.into_push_pull_output(Level::Low);

    loop {
        red_led.set_high().unwrap();
        blue_led.set_high().unwrap();
        
        timer.delay_ms(1000);

        red_led.set_low().unwrap();
        blue_led.set_low().unwrap();

        timer.delay_ms(1000);
    }
}

