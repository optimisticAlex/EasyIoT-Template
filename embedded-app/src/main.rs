#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl, 
    peripherals::Peripherals, 
    prelude::*, 
    timer::TimerGroup, 
    embassy, 
    rmt::{Rmt, TxChannelConfig, TxChannelCreator}, 
    gpio::IO
};

use embassy_time::{Duration, Timer};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};

use static_cell::{make_static, StaticCell};

mod web;
mod worker;

#[main]
async fn main(spawner: embassy_executor::Spawner) -> ! {


    esp_println::logger::init_logger(log::LevelFilter::Info);

    //ESP initializations
    log::info!("Initializing ESP32 HAL...");
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::max(system.clock_control).freeze();
    let timg0 = TimerGroup::new_async(peripherals.TIMG0, &clocks); //-> embassy
    let timg1 = TimerGroup::new(peripherals.TIMG1, &clocks, None); // timer0 -> esp_wifi
    let wifi_init = esp_wifi::initialize(
        esp_wifi::EspWifiInitFor::Wifi,
        timg1.timer0,
        esp_hal::rng::Rng::new(peripherals.RNG),
        system.radio_clock_control,
        &clocks,
    ).unwrap();
    let (wifi_interface, wifi_controller) = esp_wifi::wifi::new_with_mode(
        &wifi_init, 
        peripherals.WIFI, 
        esp_wifi::wifi::WifiStaDevice
    ).unwrap();
    
    //ESP adressable LED initializations
    let rmt = Rmt::new(peripherals.RMT, 80.MHz(), &clocks, None).unwrap();
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let led_channel = rmt.channel0.configure(
            io.pins.gpio13.into_push_pull_output(),
            TxChannelConfig {
                clk_divider: 1,
                ..TxChannelConfig::default()
            },
    ).unwrap();

    //Embassy initializations
    log::info!("Initializing embassy...");
    embassy::init(&clocks, timg0);
    let wifi_config = embassy_net::Config::dhcpv4(Default::default());
    let seed = 5489345;
    let stack = &*make_static!(embassy_net::Stack::new(
        wifi_interface,
        wifi_config,
        make_static!(embassy_net::StackResources::<3>::new()),
        seed
    ));
    
    log::info!("Spawning wifi and network tasks...");
    spawner.spawn(web::wifi_connection(wifi_controller)).ok();
    spawner.spawn(web::net_task(stack)).ok();
    web::wait_for_wifi_connection(stack).await;

    log::info!("Spawning other tasks...");
    //spawner.spawn(web::run_http_server(&stack)).ok();
    //spawner.spawn(web::websocket_echo_server(stack)).ok();

    static LED_CTRL: StaticCell<Signal<CriticalSectionRawMutex, messages::Message>> = StaticCell::new();
    let ctrl_signal = &*LED_CTRL.init(Signal::new());
    spawner.spawn(web::websocket_server(stack, ctrl_signal)).ok();
    spawner.spawn(worker::worker_task(ctrl_signal, led_channel)).ok();

    loop{Timer::after(Duration::from_millis(100)).await;}
}


