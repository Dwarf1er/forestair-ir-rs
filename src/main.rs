use std::sync::{Arc, Mutex};

use esp_idf_hal::prelude::Peripherals;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::{BlockingWifi, EspWifi};
use esp_wifi_provisioning::Provisioner;

mod ac_server;
mod ir_protocol;
mod ir_tx;

use ac_server::AcServer;
use ir_tx::IrTx;

fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let sysloop = EspSystemEventLoop::take().unwrap();
    let nvs = EspDefaultNvsPartition::take().unwrap();

    let wifi_driver = BlockingWifi::wrap(
        EspWifi::new(peripherals.modem, sysloop.clone(), Some(nvs.clone())).unwrap(),
        sysloop,
    )
    .unwrap();

    let _wifi = Provisioner::new(wifi_driver, nvs)
        .ap_ssid("ForestAir-Setup")
        .provision()
        .unwrap();

    log::info!("WiFi ready, starting AC server");

    let ir_tx = Arc::new(Mutex::new(
        IrTx::new(peripherals.rmt.channel0, peripherals.pins.gpio14).unwrap(),
    ));

    let _server = AcServer::new(ir_tx).unwrap();

    loop {
        std::thread::sleep(std::time::Duration::from_secs(60));
    }
}
