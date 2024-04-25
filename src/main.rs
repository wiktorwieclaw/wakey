#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy_executor::Spawner;
use embassy_net::{tcp::TcpSocket, Ipv4Address, Ipv4Cidr, StaticConfigV4};
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl,
    embassy,
    gpio::{Gpio2, Output, PushPull, IO},
    peripherals::Peripherals,
    prelude::*,
    rng::Rng,
    timer::TimerGroup,
};
use esp_wifi::{
    wifi::{WifiApDevice, WifiDevice},
    EspWifiInitFor,
};
use static_cell::make_static;

type Led = Gpio2<Output<PushPull>>;
type NetworkStack = embassy_net::Stack<WifiDevice<'static, WifiApDevice>>;

#[embassy_executor::task]
async fn blink(mut led: Led) {
    loop {
        led.toggle();
        Timer::after(Duration::from_secs(1)).await;
    }
}

#[embassy_executor::task]
async fn run_network_stack(stack: &'static NetworkStack) {
    stack.run().await
}

#[main]
async fn main(spawner: Spawner) {
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    let timer_group_0 = TimerGroup::new_async(peripherals.TIMG0, &clocks);
    embassy::init(&clocks, timer_group_0);

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let led = io.pins.gpio2.into_push_pull_output();
    spawner.spawn(blink(led)).unwrap();

    let timer_group_1 = TimerGroup::new(peripherals.TIMG1, &clocks, None);
    let wifi_initalization = esp_wifi::initialize(
        EspWifiInitFor::Wifi,
        timer_group_1.timer0,
        Rng::new(peripherals.RNG),
        system.radio_clock_control,
        &clocks,
    )
    .unwrap();
    let (wifi_device, wifi_controller) =
        esp_wifi::wifi::new_with_mode(&wifi_initalization, peripherals.WIFI, WifiApDevice).unwrap();
    let network_config = embassy_net::Config::ipv4_static(StaticConfigV4 {
        address: Ipv4Cidr::new(Ipv4Address::new(192, 168, 2, 1), 24),
        gateway: Some(Ipv4Address::from_bytes(&[192, 168, 2, 1])),
        dns_servers: Default::default(),
    });
    let random_seed = 1234; // TODO: generate random seed
    let network_stack = make_static!(embassy_net::Stack::new(
        wifi_device,
        network_config,
        make_static!(embassy_net::StackResources::<3>::new()),
        random_seed
    ));
    spawner.spawn(run_network_stack(network_stack)).unwrap();
    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];
    let mut socket = TcpSocket::new(network_stack, &mut rx_buffer, &mut tx_buffer);
    socket.set_timeout(Some(Duration::from_secs(10)));

    // TODO
    // examples:
    // * https://github.com/esp-rs/esp-wifi/blob/main/esp-wifi/examples/static_ip.rs
    // * https://github.com/esp-rs/esp-wifi/blob/main/esp-wifi/examples/embassy_bench.rs
    // * https://github.com/embassy-rs/embassy/blob/main/examples/std/src/bin/net.rs
}
