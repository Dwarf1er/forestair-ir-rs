use esp_idf_hal::{
    gpio::OutputPin,
    peripheral::Peripheral,
    rmt::{
        config::{CarrierConfig, DutyPercent},
        PinState, RmtChannel, TxRmtConfig, TxRmtDriver,
    },
    units::Hertz,
};

use crate::ir_protocol::encode_ir;

pub struct IrTx<'d> {
    tx: TxRmtDriver<'d>,
}

impl<'d> IrTx<'d> {
    pub fn new<
        RMT: Peripheral<P = impl RmtChannel> + 'd,
        PIN: Peripheral<P = impl OutputPin> + 'd,
    >(
        rmt: RMT,
        pin: PIN,
    ) -> anyhow::Result<Self> {
        let carrier = CarrierConfig {
            frequency: Hertz(38000),
            carrier_level: PinState::High,
            duty_percent: DutyPercent::new(33).unwrap(),
        };

        let tx_config = TxRmtConfig {
            clock_divider: 80,
            carrier: Some(carrier),
            idle: Some(PinState::Low),
            ..Default::default()
        };

        let tx = TxRmtDriver::new(rmt, pin, &tx_config)?;

        Ok(Self { tx })
    }

    pub fn send_ir(&mut self, payload: u64) -> anyhow::Result<()> {
        let signal = encode_ir(payload);
        self.tx.start(signal)?;
        Ok(())
    }
}
