use std::thread;
use std::time::Duration;

use esp_idf_hal::prelude::Peripherals;

mod ir_protocol;
mod ir_tx;

use ir_protocol::{pack_ir_payload, AcMode, IrData, Temperature};
use ir_tx::IrTx;

use self::ir_protocol::FanMode;

fn main() {
    // It is necessary to call this function once. Otherwise, some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let rmt_channel = peripherals.rmt.channel0;
    let pin = peripherals.pins.gpio14;

    let mut ir_tx = IrTx::new(rmt_channel, pin).unwrap();

    let data = IrData {
        ac_mode: AcMode::Ventilation,
        on_off: true,
        fan_mode: FanMode::Low,
        swing: false,
        temperature: Temperature::T24,
    };

    let payload = pack_ir_payload(data);

    loop {
        ir_tx.send_ir(payload).unwrap();
        thread::sleep(Duration::from_secs(3));
    }
}
