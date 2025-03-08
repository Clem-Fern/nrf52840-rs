#![no_main]
#![no_std]

use core::panic::PanicInfo;
use rtt_target::{rprintln, rtt_init_print};

use embassy_executor::Spawner;
use embassy_nrf::gpio::{AnyPin, Level, Output, OutputDrive, Pin};
use embassy_time::Timer;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rprintln!("{}", info);
    loop {}
}

#[embassy_executor::task]
async fn blink(pin: AnyPin) {
    let mut led = Output::new(pin, Level::Low, OutputDrive::Standard);
    loop {
        // Timekeeping is globally available, no need to mess with hardware timers.
        led.set_high();
        Timer::after_millis(150).await;
        led.set_low();
        Timer::after_millis(150).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    rtt_init_print!();
    rprintln!("Hello, world!");
    let p = embassy_nrf::init(Default::default());
    let mut led = Output::new(p.P1_11, Level::Low, OutputDrive::Standard);

    spawner.spawn(blink(p.P0_06.degrade())).unwrap();

    rprintln!("LED get ready.");
    loop {
        rprintln!("LED set high.");
        led.set_high();
        Timer::after_millis(300).await;
        rprintln!("LED set low.");
        led.set_low();
        Timer::after_millis(300).await;
    }
}
