use esp_idf_hal::{
    gpio::OutputPin,
    peripheral::Peripheral,
    rmt::{
        PinState, RmtChannel, TxRmtConfig, TxRmtDriver,
        config::{CarrierConfig, DutyPercent},
    },
    sys::EspError,
    units::Hertz,
};

use crate::ir_protocol::encode_ir;

#[derive(Debug)]
pub enum IrTxError {
    Init(EspError),
    Send(EspError),
}

impl std::fmt::Display for IrTxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IrTxError::Init(e) => write!(f, "IR TX init failed: {e}"),
            IrTxError::Send(e) => write!(f, "IR TX send failed: {e}"),
        }
    }
}

impl std::error::Error for IrTxError {}

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
    ) -> Result<Self, IrTxError> {
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

        let tx = TxRmtDriver::new(rmt, pin, &tx_config).map_err(IrTxError::Init)?;

        Ok(Self { tx })
    }

    pub fn send_ir(&mut self, payload: u64) -> Result<(), IrTxError> {
        let signal = encode_ir(payload);
        self.tx.start(signal).map_err(IrTxError::Send)
    }
}
