#![no_std]
#![no_main]

use core::mem;
use core::panic::PanicInfo;

extern crate alloc;
use alloc::vec::Vec;

use embassy_nrf::interrupt::Priority;
use embassy_nrf::twim::{self, Twim};
use embassy_nrf::{bind_interrupts, peripherals};
use embassy_time::{Delay, Timer};
use embedded_alloc::LlffHeap as Heap;

use embassy_executor::Spawner;
use embassy_nrf::gpio::{AnyPin, Level, Output, OutputDrive, Pin};
use nrf_softdevice::ble::advertisement_builder::{
    AdvertisementDataType, Flag, LegacyAdvertisementBuilder, LegacyAdvertisementPayload,
    ServiceList, ServiceUuid16,
};
use nrf_softdevice::ble::peripheral;
use nrf_softdevice::{raw, Softdevice};
use rtt_target::{rprintln, rtt_init_print};
use shtcx::{Measurement, PowerMode};

#[global_allocator]
static HEAP: Heap = Heap::empty();

bind_interrupts!(struct Irqs {
    TWISPI0 => twim::InterruptHandler<peripherals::TWISPI0>;
});

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rprintln!("Panic: {}", info);
    loop {}
}

#[embassy_executor::task]
async fn softdevice_task(sd: &'static Softdevice) -> ! {
    sd.run().await
}

#[embassy_executor::task]
async fn alive(pin: AnyPin) {
    rprintln!("alive task");
    let mut led = Output::new(pin, Level::Low, OutputDrive::Standard);
    loop {
        rprintln!("alive blink");
        led.set_low();
        Timer::after_millis(200).await;
        led.set_high();
        Timer::after_secs(4).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Initialize the allocator BEFORE you use it
    {
        use core::mem::MaybeUninit;
        const HEAP_SIZE: usize = 1024;
        static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe { HEAP.init(&raw mut HEAP_MEM as usize, HEAP_SIZE) }
    }
    rtt_init_print!();
    rprintln!("Hello, world!");

    let mut config = embassy_nrf::config::Config::default();
    config.time_interrupt_priority = Priority::P2;
    let p = embassy_nrf::init(config);
    let led = Output::new(p.P1_11, Level::Low, OutputDrive::Standard).unwrap();

    let config = nrf_softdevice::Config {
        clock: Some(raw::nrf_clock_lf_cfg_t {
            source: raw::NRF_CLOCK_LF_SRC_RC as u8,
            rc_ctiv: 16,
            rc_temp_ctiv: 2,
            accuracy: raw::NRF_CLOCK_LF_ACCURACY_500_PPM as u8,
        }),
        conn_gap: Some(raw::ble_gap_conn_cfg_t {
            conn_count: 6,
            event_length: 24,
        }),
        conn_gatt: None,
        gatts_attr_tab_size: None,
        gap_role_count: Some(raw::ble_gap_cfg_role_count_t {
            adv_set_count: 1,
            periph_role_count: 0,
            central_role_count: 0,
            central_sec_count: 0,
            _bitfield_1: raw::ble_gap_cfg_role_count_t::new_bitfield_1(0),
        }),
        gap_device_name: None,
        ..Default::default()
    };

    let sd = Softdevice::enable(&config);
    spawner.spawn(softdevice_task(sd)).unwrap();

    rprintln!("Initializing TWI...");
    let config = twim::Config::default();
    let twi = Twim::new(p.TWISPI0, Irqs, p.P0_24, p.P0_25, config);
    let mut sht = shtcx::shtc3(twi);
    let mut delay = Delay;

    loop {
        led.set_low();
        let combined = sht.measure(PowerMode::NormalMode, &mut delay).unwrap();
        rprintln!(
            "Combined: {} °C / {} %RH",
            combined.temperature.as_degrees_celsius(),
            combined.humidity.as_percent()
        );
        led.set_high();
        advertise_temp(sd, &combined).await;
        // no sleep needed as advertise_temp take 10 sec to run.
    }
}

async fn advertise_temp(sd: &Softdevice, measurement: &Measurement) {
    let temp = measurement.temperature.as_degrees_celsius() as u8;
    rprintln!("Advertissing {}", temp);

    let config = peripheral::Config {
        timeout: Some(1000),
        ..Default::default()
    };
    let mut data = Vec::new();
    let mut service = ServiceUuid16::from_u16(0x1809)
        .to_u16()
        .to_le_bytes()
        .to_vec();
    data.append(&mut service);
    data.push(temp.to_le_bytes()[0]);

    let adv_data: LegacyAdvertisementPayload = LegacyAdvertisementBuilder::new()
        .flags(&[Flag::GeneralDiscovery, Flag::LE_Only])
        .services_16(ServiceList::Complete, &[ServiceUuid16::HEALTH_THERMOMETER]) // if there were a lot of these there may not be room for the full name
        .short_name("NRF-Hello")
        .raw(AdvertisementDataType::SERVICE_DATA_16, &data)
        .build();

    let adv = peripheral::NonconnectableAdvertisement::NonscannableUndirected {
        adv_data: &adv_data,
    };
    let _ = peripheral::advertise(sd, adv, &config).await;
}
