use esp_idf_hal::rmt::{FixedLengthSignal, PinState, Pulse, PulseTicks};

pub struct IrData {
    pub ac_mode: AcMode,
    pub on_off: bool,
    pub fan_mode: FanMode,
    pub swing: bool,
    pub temperature: Temperature,
}

#[allow(dead_code)]
#[repr(u8)]
pub enum Temperature {
    T16 = 0,
    T17,
    T18,
    T19,
    T20,
    T21,
    T22,
    T23,
    T24,
    T25,
    T26,
    T27,
    T28,
    T29,
    T30,
}

#[allow(dead_code)]
#[repr(u8)]
pub enum FanMode {
    Auto = 0,
    Low,
    Medium,
    High,
}

#[allow(dead_code)]
#[repr(u8)]
pub enum AcMode {
    Auto = 0,
    Cool,
    Dehumidify,
    Ventilation,
    Heat,
}

pub const HEADER_MARK_US: u16 = 9000;
pub const HEADER_SPACE_US: u16 = 4450;
pub const BIT_MARK_US: u16 = 650;
pub const ONE_SPACE_US: u16 = 1650;
pub const ZERO_SPACE_US: u16 = 550;

pub fn pack_ir_payload(data: IrData) -> u64 {
    let mut payload = 0x250000000;
    payload |= data.ac_mode as u64;
    payload |= (data.on_off as u64) << 3;
    payload |= (data.fan_mode as u64) << 4;
    payload |= (data.swing as u64) << 7;
    payload |= (data.temperature as u64) << 8;
    payload
}

pub fn encode_ir(payload: u64) -> FixedLengthSignal<37> {
    let mut signal = FixedLengthSignal::<37>::new();

    signal
        .set(
            0,
            &(
                Pulse::new(PinState::High, PulseTicks::new(HEADER_MARK_US).unwrap()),
                Pulse::new(PinState::Low, PulseTicks::new(HEADER_SPACE_US).unwrap()),
            ),
        )
        .unwrap();

    for i in 0..35 {
        let bit_is_one = (payload >> i) & 1 != 0;

        signal
            .set(
                1 + i,
                &(
                    Pulse::new(PinState::High, PulseTicks::new(BIT_MARK_US).unwrap()),
                    Pulse::new(
                        PinState::Low,
                        PulseTicks::new(if bit_is_one {
                            ONE_SPACE_US
                        } else {
                            ZERO_SPACE_US
                        })
                        .unwrap(),
                    ),
                ),
            )
            .unwrap();
    }

    signal
        .set(
            36,
            &(
                Pulse::new(PinState::High, PulseTicks::new(BIT_MARK_US).unwrap()),
                Pulse::new(PinState::Low, PulseTicks::new(0).unwrap()),
            ),
        )
        .unwrap();

    signal
}
