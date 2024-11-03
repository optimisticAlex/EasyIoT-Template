//! # ESP32 Application code
//! Here goes everything that is specific to the Espressif ESP32 Chip.
//! 


#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

const HEAP_SIZE: usize = 1024 * 72; 

use esp_backtrace as _;
use esp_hal::{
    prelude::*, 
    rmt::{Rmt, TxChannelConfig, TxChannelCreator}, 
    gpio::Io,
};
//extern crate alloc;

use embassy_time::{Duration, Timer};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};

use static_cell::StaticCell;

macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.uninit().write(($val));
        x
    }};
}


mod web;
mod worker;
mod client_app; //generated at compile time by build.rs


#[esp_hal_embassy::main]
async fn main(spawner: embassy_executor::Spawner) {
    esp_println::logger::init_logger(log::LevelFilter::Info);
    log::info!("Initializing Hardware ;)");

    //ESP initializations
    esp_alloc::heap_allocator!(HEAP_SIZE);
    let peripherals = esp_hal::init({
        let mut config = esp_hal::Config::default();
        config.cpu_clock = CpuClock::max();
        config
    });
    let timg0 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG0);// timer0 -> embassy
    let timg1 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG1);// timer0 -> esp_wifi
    let wifi_init = esp_wifi::init(
        esp_wifi::EspWifiInitFor::Wifi,
        timg1.timer0,
        esp_hal::rng::Rng::new(peripherals.RNG),
        peripherals.RADIO_CLK,
    ).unwrap();
    let (wifi_interface, wifi_controller) = esp_wifi::wifi::new_with_mode(&wifi_init, peripherals.WIFI, esp_wifi::wifi::WifiStaDevice).unwrap();

    //ESP adressable LED initializations
    let rmt = Rmt::new(peripherals.RMT, 80.MHz()).unwrap();
    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);
    let led_channel = rmt.channel0.configure(
            io.pins.gpio13,
            TxChannelConfig {
                clk_divider: 1,
                ..TxChannelConfig::default()
            },
    ).unwrap();

    //Embassy initializations
    esp_hal_embassy::init(timg0.timer0);
    let wifi_config = embassy_net::Config::dhcpv4(Default::default());
    let seed = 5489345;
    let stack = &*mk_static!(
        embassy_net::Stack<esp_wifi::wifi::WifiDevice<'_, esp_wifi::wifi::WifiStaDevice>>,
        embassy_net::Stack::new(
            wifi_interface,
            wifi_config,
            mk_static!(embassy_net::StackResources<5>, embassy_net::StackResources::<5>::new()),
            seed
        )
    );
    log::info!("Embassy initialized!");



    //Task communication
    static LED_CTRL: StaticCell<Signal<CriticalSectionRawMutex, messages::Message>> = StaticCell::new();
    let ctrl_signal = &*LED_CTRL.init(Signal::new());
    
    log::info!("Spawning wifi and network tasks...");
    spawner.spawn(web::wifi_connection(wifi_controller)).ok();
    spawner.spawn(web::net_task(stack)).ok();
    web::wait_for_wifi_connection(stack).await;

    log::info!("Spawning {} HTTP servers with files:", web::HTTP_SERVER_POOL_SIZE);
    esp_println::println!("index.html: {} Bytes", client_app::INDEX_HTML.len());
    for file in client_app::CLIENT_APP_FILES.iter() {esp_println::println!("{}: {} Bytes", file.0, file.1.len());}
    for i in 0..web::HTTP_SERVER_POOL_SIZE {spawner.spawn(web::http_server(&stack, i)).ok();}

    log::info!("Spawning websocket and worker task");
    spawner.spawn(web::websocket_server(stack, ctrl_signal)).ok();
    spawner.spawn(worker::worker_task(ctrl_signal, led_channel)).ok();

    loop {
        Timer::after(Duration::from_millis(5000)).await;
    }
}