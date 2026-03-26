use std::sync::{Arc, Mutex};

use esp_idf_hal::io::Write;
use esp_idf_svc::{
    http::server::{Configuration as HttpConfig, EspHttpServer},
    mdns::EspMdns,
    sys::EspError,
};

use crate::ir_protocol::{AcMode, FanMode, IrData, Temperature, pack_ir_payload};
use crate::ir_tx::{IrTx, IrTxError};

/// Minified control UI, bundled at compile time by `build.rs`.
static INDEX_HTML: &str = include_str!(concat!(env!("OUT_DIR"), "/ac.min.html"));
static MANIFEST: &str = include_str!(concat!(env!("OUT_DIR"), "/manifest.json"));
static ICON: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/icon.png"));

#[derive(Debug)]
pub enum AcServerError {
    Mdns(EspError),
    Http(Box<dyn std::error::Error>),
    IrTx(IrTxError),
}

impl std::fmt::Display for AcServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AcServerError::Mdns(e) => write!(f, "mDNS error: {e}"),
            AcServerError::Http(e) => write!(f, "HTTP server error: {e}"),
            AcServerError::IrTx(e) => write!(f, "IR TX error: {e}"),
        }
    }
}

impl std::error::Error for AcServerError {}

impl From<IrTxError> for AcServerError {
    fn from(e: IrTxError) -> Self {
        AcServerError::IrTx(e)
    }
}

/// The AC state as last commanded by the user.
///
/// Serialised to/from JSON manually — avoids pulling in serde/serde_json.
/// All fields use their numeric discriminant so the web UI doesn't need to
/// know Rust enum names.
#[derive(Clone)]
pub struct AcState {
    /// `AcMode` discriminant (0 = Auto, 1 = Cool, 2 = Dehumidify, 3 = Vent, 4 = Heat).
    pub mode: u8,
    /// `true` = unit on, `false` = unit off.
    pub on: bool,
    /// `FanMode` discriminant (0 = Auto, 1 = Low, 2 = Medium, 3 = High).
    pub fan: u8,
    /// `true` = swing enabled.
    pub swing: bool,
    /// Temperature in °C, clamped to 16–30.
    pub temp: u8,
}

impl Default for AcState {
    fn default() -> Self {
        Self {
            mode: AcMode::Auto as u8,
            on: false,
            fan: FanMode::Auto as u8,
            swing: false,
            temp: 24,
        }
    }
}

impl AcState {
    /// Converts the state into an [`IrData`] value ready for [`pack_ir_payload`].
    ///
    /// Returns `None` if any discriminant is out of range.
    fn to_ir_data(&self) -> Option<IrData> {
        let ac_mode = match self.mode {
            0 => AcMode::Auto,
            1 => AcMode::Cool,
            2 => AcMode::Dehumidify,
            3 => AcMode::Ventilation,
            4 => AcMode::Heat,
            _ => return None,
        };
        let fan_mode = match self.fan {
            0 => FanMode::Auto,
            1 => FanMode::Low,
            2 => FanMode::Medium,
            3 => FanMode::High,
            _ => return None,
        };
        let temperature = match self.temp {
            16 => Temperature::T16,
            17 => Temperature::T17,
            18 => Temperature::T18,
            19 => Temperature::T19,
            20 => Temperature::T20,
            21 => Temperature::T21,
            22 => Temperature::T22,
            23 => Temperature::T23,
            24 => Temperature::T24,
            25 => Temperature::T25,
            26 => Temperature::T26,
            27 => Temperature::T27,
            28 => Temperature::T28,
            29 => Temperature::T29,
            30 => Temperature::T30,
            _ => return None,
        };
        Some(IrData {
            ac_mode,
            on_off: self.on,
            fan_mode,
            swing: self.swing,
            temperature,
        })
    }

    /// Deserialise from a flat JSON object.
    ///
    /// Expects exactly: `{"mode":<u8>,"on":<bool>,"fan":<u8>,"swing":<bool>,"temp":<u8>}`
    /// Field order does not matter. Returns `None` on any parse error.
    fn from_json(s: &str) -> Option<Self> {
        fn find_field<'a>(json: &'a str, key: &str) -> Option<&'a str> {
            let needle = format!("\"{}\"", key);
            let after_key = json.find(needle.as_str())? + needle.len();
            let after_colon = json[after_key..].find(':')? + after_key + 1;
            let value_str = json[after_colon..].trim_start();
            let end = value_str
                .find(|c| c == ',' || c == '}')
                .unwrap_or(value_str.len());
            Some(value_str[..end].trim())
        }

        fn parse_bool(s: &str) -> Option<bool> {
            match s {
                "true" => Some(true),
                "false" => Some(false),
                _ => None,
            }
        }

        Some(AcState {
            mode: find_field(s, "mode")?.parse().ok()?,
            on: parse_bool(find_field(s, "on")?)?,
            fan: find_field(s, "fan")?.parse().ok()?,
            swing: parse_bool(find_field(s, "swing")?)?,
            temp: find_field(s, "temp")?.parse().ok()?,
        })
    }
}

/// Owns the HTTP server and mDNS handle for the AC control interface.
///
/// Drop this value to shut down the server and withdraw the mDNS advertisement.
pub struct AcServer {
    // These fields are never read — they exist solely to keep the resources
    // alive for as long as `AcServer` itself is alive.
    _http: EspHttpServer<'static>,
    _mdns: EspMdns,
}

impl AcServer {
    /// Starts the HTTP server and registers the `ac.local` mDNS record.
    ///
    /// `ir_tx` is wrapped in an `Arc<Mutex<…>>` so the HTTP handler closure
    /// can send IR commands without blocking the main thread.
    pub fn new(ir_tx: Arc<Mutex<IrTx<'static>>>) -> Result<Self, AcServerError> {
        let state: Arc<Mutex<AcState>> = Arc::new(Mutex::new(AcState::default()));

        let mut mdns = EspMdns::take().map_err(AcServerError::Mdns)?;
        mdns.set_hostname("ac").map_err(AcServerError::Mdns)?;
        mdns.set_instance_name("ForestAir AC Controller")
            .map_err(AcServerError::Mdns)?;
        mdns.add_service(None, "_http", "_tcp", 80, &[])
            .map_err(AcServerError::Mdns)?;

        let server_config = HttpConfig {
            http_port: 80,
            ..Default::default()
        };
        let mut server =
            EspHttpServer::new(&server_config).map_err(|e| AcServerError::Http(e.into()))?;

        server
            .fn_handler("/", esp_idf_svc::http::Method::Get, |req| {
                req.into_ok_response()?
                    .write_all(INDEX_HTML.as_bytes())
                    .map(|_| ())
            })
            .map_err(|e| AcServerError::Http(e.into()))?;

        let state_r = Arc::clone(&state);
        server
            .fn_handler("/state", esp_idf_svc::http::Method::Get, move |req| {
                let s = state_r.lock().unwrap();
                let json = state_to_json(&s);
                req.into_response(200, None, &[("Content-Type", "application/json")])?
                    .write_all(json.as_bytes())
                    .map(|_| ())
            })
            .map_err(|e| AcServerError::Http(e.into()))?;

        let state_w = Arc::clone(&state);
        server
            .fn_handler(
                "/command",
                esp_idf_svc::http::Method::Post,
                move |mut req| {
                    let mut body = [0u8; 256];
                    let mut total = 0usize;
                    loop {
                        let n = req.read(&mut body[total..])?;
                        if n == 0 {
                            break;
                        }
                        total += n;
                        if total >= body.len() {
                            break;
                        }
                    }

                    let body_str = match std::str::from_utf8(&body[..total]) {
                        Ok(s) => s,
                        Err(_) => {
                            req.into_response(400, Some("Bad Request"), &[])?
                                .write_all(b"")
                                .map(|_| ())?;
                            return Ok(());
                        }
                    };

                    let new_state = match AcState::from_json(body_str) {
                        Some(s) => s,
                        None => {
                            req.into_response(400, Some("Bad Request"), &[])?
                                .write_all(b"")
                                .map(|_| ())?;
                            return Ok(());
                        }
                    };

                    let ir_data = match new_state.to_ir_data() {
                        Some(d) => d,
                        None => {
                            req.into_response(422, Some("Unprocessable Entity"), &[])?
                                .write_all(b"")
                                .map(|_| ())?;
                            return Ok(());
                        }
                    };

                    {
                        let mut tx = ir_tx.lock().unwrap();
                        let payload = pack_ir_payload(ir_data);
                        if let Err(e) = tx.send_ir(payload) {
                            log::error!("IR send failed: {e}");
                            req.into_response(500, Some("IR Error"), &[])?
                                .write_all(b"")
                                .map(|_| ())?;
                            return Ok(());
                        }
                    }

                    *state_w.lock().unwrap() = new_state.clone();
                    let json = state_to_json(&new_state);
                    req.into_response(200, None, &[("Content-Type", "application/json")])?
                        .write_all(json.as_bytes())
                        .map(|_| ())
                },
            )
            .map_err(|e| AcServerError::Http(e.into()))?;

        log::info!("AC server running | browse to http://ac.local");

        Ok(Self {
            _http: server,
            _mdns: mdns,
        })
    }
}

/// Serialises [`AcState`] to a JSON string.
fn state_to_json(s: &AcState) -> String {
    format!(
        r#"{{"mode":{},"on":{},"fan":{},"swing":{},"temp":{}}}"#,
        s.mode, s.on, s.fan, s.swing, s.temp
    )
}
