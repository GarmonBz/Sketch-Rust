//! Leica DISTO BLE connection and measurement input -
//! `SketchApp.connectDisto` (sketch-app.js:5449).
//!
//! Native implementation of the leica-disto-ble skill's measured E7500i/D510
//! protocol: subscribe to Leica distance indications, float32 LE metres.
//! Standard stages keyboard input; Continuous uses the previous heading.
//! Native radio behavior remains unverified on hardware. Tests use no radio.

/// A device the scan found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Device {
    /// The platform peripheral id (stable within a scan).
    pub id: String,
    /// `local_name`, when the advertisement carried one.
    pub name: Option<String>,
}

impl Device {
    /// The chooser's row text: the advertised name, else Chrome's wording.
    pub fn display(&self) -> String {
        match &self.name {
            Some(n) if !n.trim().is_empty() => n.trim().to_string(),
            _ => {
                let tail: String = self
                    .id
                    .chars()
                    .rev()
                    .take(5)
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                    .collect();
                format!("Unknown or Unsupported Device ({tail})")
            }
        }
    }

    fn is_disto(&self) -> bool {
        self.name
            .as_deref()
            .is_some_and(|n| n.to_ascii_uppercase().contains("DISTO"))
    }
}

/// Named devices first (DISTO ones at the very top), the rest by name.
pub fn sort_devices(devices: &mut [Device]) {
    devices.sort_by(|a, b| {
        let key = |d: &Device| (!d.is_disto(), d.name.is_none(), d.display().to_lowercase());
        key(a).cmp(&key(b))
    });
}

/// Why a scan or connect failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Failure {
    /// No Bluetooth adapter / the radio is unavailable (`!navigator.bluetooth`).
    Unavailable,
    /// Anything else (`Bluetooth connection was not completed.`).
    Failed(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Phase {
    #[default]
    Idle,
    /// The chooser is open and the scan is running.
    Scanning,
    /// The scan finished; the user picks.
    Picking,
    /// A pick is connecting.
    Connecting,
}

#[derive(Debug, Clone)]
pub enum RadioEvent {
    Measurement(f64),
    Disconnected,
}

#[derive(Debug, Clone)]
pub enum DistoMsg {
    /// Tools > Connect, the status-bar Bluetooth button, the palette.
    Connect,
    Scanned(Result<Vec<Device>, Failure>),
    Pick(usize),
    Rescan,
    Cancel,
    Choose,
    Connected(Result<String, Failure>),
    ScannedFor(u64, Result<Vec<Device>, Failure>),
    ConnectedFor(u64, Result<String, Failure>),
    Radio(u64, RadioEvent),
}

/// What the app should do after a message.
#[derive(Debug, PartialEq)]
pub enum Effect {
    None,
    Toast(String),
    /// Start a scan on the worker.
    Scan,
    /// Connect to this device on the worker.
    Connect(Device),
    Disconnect,
    Measurement(f64),
}

#[derive(Debug, Default)]
pub struct Disto {
    pub phase: Phase,
    pub devices: Vec<Device>,
    pub selected: Option<usize>,
    /// `#disto-status`'s label once a device was chosen.
    pub connected: Option<String>,
    pub session: u64,
}

impl Disto {
    /// The chooser is on screen.
    pub fn open(&self) -> bool {
        self.phase != Phase::Idle
    }

    /// `#disto-status` text: the device name, else `Disconnected`.
    #[cfg(test)]
    pub fn chip_label(&self) -> Option<&str> {
        self.connected.as_deref()
    }

    pub fn update(&mut self, msg: DistoMsg) -> Effect {
        match msg {
            DistoMsg::ScannedFor(session, result) => {
                if session != self.session {
                    return Effect::None;
                }
                self.update(DistoMsg::Scanned(result))
            }
            DistoMsg::ConnectedFor(session, result) => {
                if session != self.session {
                    return Effect::None;
                }
                self.update(DistoMsg::Connected(result))
            }
            DistoMsg::Radio(session, event) => {
                if session != self.session || self.connected.is_none() {
                    return Effect::None;
                }
                match event {
                    RadioEvent::Measurement(meters) if meters.is_finite() && meters > 0.0 => {
                        Effect::Measurement(meters)
                    }
                    RadioEvent::Measurement(_) => Effect::None,
                    RadioEvent::Disconnected => {
                        self.connected = None;
                        Effect::Toast("Laser disconnected.".into())
                    }
                }
            }
            DistoMsg::Connect if self.connected.is_some() => {
                self.session += 1;
                self.connected = None;
                self.reset();
                Effect::Disconnect
            }
            DistoMsg::Connect | DistoMsg::Rescan => {
                self.session += 1;
                // `requestDevice`: a new chooser over a fresh scan.
                self.phase = Phase::Scanning;
                self.devices.clear();
                self.selected = None;
                Effect::Scan
            }
            DistoMsg::Scanned(result) => {
                if self.phase != Phase::Scanning {
                    return Effect::None;
                }
                match result {
                    Ok(mut devices) => {
                        sort_devices(&mut devices);
                        self.devices = devices;
                        self.phase = Phase::Picking;
                        Effect::None
                    }
                    Err(failure) => {
                        self.reset();
                        match failure {
                            Failure::Unavailable => Effect::Toast(
                                "Bluetooth device access is not available on this computer."
                                    .to_string(),
                            ),
                            Failure::Failed(_) => {
                                Effect::Toast("Bluetooth connection was not completed.".to_string())
                            }
                        }
                    }
                }
            }
            DistoMsg::Pick(i) => {
                if self.phase == Phase::Picking && i < self.devices.len() {
                    self.selected = Some(i);
                }
                Effect::None
            }
            DistoMsg::Cancel => {
                // `NotFoundError`: silent.
                let connecting = self.phase == Phase::Connecting;
                self.session += 1;
                self.reset();
                if connecting {
                    Effect::Disconnect
                } else {
                    Effect::None
                }
            }
            DistoMsg::Choose => {
                if self.phase != Phase::Picking {
                    return Effect::None;
                }
                match self.selected.and_then(|i| self.devices.get(i)).cloned() {
                    Some(device) => {
                        self.phase = Phase::Connecting;
                        Effect::Connect(device)
                    }
                    None => Effect::None,
                }
            }
            DistoMsg::Connected(result) => {
                if self.phase != Phase::Connecting {
                    return Effect::None;
                }
                self.reset();
                match result {
                    Ok(name) => {
                        let label = if name.trim().is_empty() {
                            "Connected".to_string()
                        } else {
                            name
                        };
                        // `status.textContent = device.name || 'Connected'`,
                        // `Connected to ${device.name || 'Bluetooth device'}.`
                        let shown = if label == "Connected" {
                            "Bluetooth device".to_string()
                        } else {
                            label.clone()
                        };
                        self.connected = Some(label);
                        Effect::Toast(format!("Connected to {shown}."))
                    }
                    Err(Failure::Unavailable) => Effect::Toast(
                        "Bluetooth device access is not available on this computer.".into(),
                    ),
                    Err(Failure::Failed(reason)) => {
                        Effect::Toast(format!("Laser connection was not completed: {reason}"))
                    }
                }
            }
        }
    }

    fn reset(&mut self) {
        self.phase = Phase::Idle;
        self.devices.clear();
        self.selected = None;
    }
}

// ---------------------------------------------------------------------------
// Transport (btleplug on a worker thread)
// ---------------------------------------------------------------------------

#[cfg(not(test))]
#[path = "disto_transport.rs"]
mod transport;

#[cfg(test)]
mod transport {
    use super::{Device, DistoMsg, Failure};
    pub async fn scan_devices(_manual: bool) -> Result<Vec<Device>, Failure> {
        std::future::pending().await
    }
    pub async fn connect_device(_id: String, _session: u64) -> Result<String, Failure> {
        std::future::pending().await
    }
    pub fn set_session(_session: u64) {}
    pub fn disconnect_device() {}
    pub fn event_stream() -> impl iced::futures::Stream<Item = DistoMsg> {
        iced::futures::stream::pending()
    }
}

pub use transport::{connect_device, disconnect_device, event_stream, scan_devices, set_session};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_changes_reject_late_scans_connections_and_measurements() {
        let mut d = Disto::default();
        d.update(DistoMsg::Connect);
        let old = d.session;
        d.update(DistoMsg::Cancel);
        d.update(DistoMsg::Connect);
        assert_eq!(
            d.update(DistoMsg::ScannedFor(old, Ok(vec![dev("x", Some("DISTO"))]))),
            Effect::None
        );
        assert_eq!(d.phase, Phase::Scanning);
        d.update(DistoMsg::Scanned(Ok(vec![dev("x", Some("DISTO"))])));
        d.update(DistoMsg::Pick(0));
        d.update(DistoMsg::Choose);
        assert_eq!(
            d.update(DistoMsg::ConnectedFor(old, Ok("DISTO".into()))),
            Effect::None
        );
        assert_eq!(d.phase, Phase::Connecting);
        let session = d.session;
        d.update(DistoMsg::ConnectedFor(session, Ok("DISTO".into())));
        assert_eq!(
            d.update(DistoMsg::Radio(session, RadioEvent::Measurement(1.7185))),
            Effect::Measurement(1.7185)
        );
        assert_eq!(
            d.update(DistoMsg::Radio(old, RadioEvent::Measurement(1.0))),
            Effect::None
        );
        assert_eq!(
            d.update(DistoMsg::Radio(session, RadioEvent::Measurement(f64::NAN))),
            Effect::None
        );
        assert_eq!(d.update(DistoMsg::Connect), Effect::Disconnect);
        assert!(d.chip_label().is_none());
        assert_eq!(
            d.update(DistoMsg::Radio(session, RadioEvent::Measurement(1.0))),
            Effect::None
        );
    }

    #[test]
    fn cancel_connect_and_hardware_drop_clear_the_measurement_path() {
        let mut d = Disto::default();
        d.update(DistoMsg::Connect);
        d.update(DistoMsg::Scanned(Ok(vec![dev("x", Some("DISTO"))])));
        d.update(DistoMsg::Pick(0));
        d.update(DistoMsg::Choose);
        assert_eq!(d.update(DistoMsg::Cancel), Effect::Disconnect);
        assert_eq!(d.phase, Phase::Idle);
        d.connected = Some("DISTO".into());
        let session = d.session;
        assert_eq!(
            d.update(DistoMsg::Radio(session, RadioEvent::Disconnected)),
            Effect::Toast("Laser disconnected.".into())
        );
        assert!(d.chip_label().is_none());
        assert_eq!(
            d.update(DistoMsg::Radio(session, RadioEvent::Measurement(1.0))),
            Effect::None
        );
    }

    fn dev(id: &str, name: Option<&str>) -> Device {
        Device {
            id: id.into(),
            name: name.map(str::to_string),
        }
    }

    #[test]
    fn connect_opens_a_scan_and_picking_waits_for_the_results() {
        let mut d = Disto::default();
        assert_eq!(d.update(DistoMsg::Connect), Effect::Scan);
        assert_eq!(d.phase, Phase::Scanning);
        assert!(d.open());
        assert_eq!(
            d.update(DistoMsg::Scanned(Ok(vec![dev("a", Some("DISTO X4"))]))),
            Effect::None
        );
        assert_eq!(d.phase, Phase::Picking);
        // A Choose with nothing picked does nothing.
        assert_eq!(d.update(DistoMsg::Choose), Effect::None);
        d.update(DistoMsg::Pick(0));
        assert_eq!(
            d.update(DistoMsg::Choose),
            Effect::Connect(dev("a", Some("DISTO X4")))
        );
        assert_eq!(
            d.update(DistoMsg::Connected(Ok("DISTO X4".into()))),
            Effect::Toast("Connected to DISTO X4.".into())
        );
        assert_eq!(d.chip_label(), Some("DISTO X4"));
        assert!(!d.open());
    }

    #[test]
    fn a_cancelled_chooser_is_silent_and_keeps_the_chip() {
        let mut d = Disto {
            connected: Some("DISTO X4".into()),
            ..Disto::default()
        };
        d.phase = Phase::Picking;
        assert_eq!(d.update(DistoMsg::Cancel), Effect::None);
        assert!(!d.open());
        assert_eq!(
            d.chip_label(),
            Some("DISTO X4"),
            "NotFoundError changes nothing"
        );
    }

    #[test]
    fn failures_toast_the_originals_two_messages() {
        let mut d = Disto::default();
        d.update(DistoMsg::Connect);
        assert_eq!(
            d.update(DistoMsg::Scanned(Err(Failure::Unavailable))),
            Effect::Toast("Bluetooth device access is not available on this computer.".into())
        );
        assert!(!d.open());
        d.update(DistoMsg::Connect);
        assert_eq!(
            d.update(DistoMsg::Scanned(Err(Failure::Failed("boom".into())))),
            Effect::Toast("Bluetooth connection was not completed.".into())
        );
        // A failed connection reports the transport's reason.
        d.update(DistoMsg::Connect);
        d.update(DistoMsg::Scanned(Ok(vec![dev("a", None)])));
        d.update(DistoMsg::Pick(0));
        d.update(DistoMsg::Choose);
        assert_eq!(
            d.update(DistoMsg::Connected(Err(Failure::Failed("x".into())))),
            Effect::Toast("Laser connection was not completed: x".into())
        );
        assert_eq!(d.chip_label(), None);
    }

    #[test]
    fn an_unnamed_device_connects_as_a_bluetooth_device() {
        let mut d = Disto::default();
        d.update(DistoMsg::Connect);
        d.update(DistoMsg::Scanned(Ok(vec![dev("x", None)])));
        d.update(DistoMsg::Pick(0));
        d.update(DistoMsg::Choose);
        assert_eq!(
            d.update(DistoMsg::Connected(Ok(String::new()))),
            Effect::Toast("Connected to Bluetooth device.".into())
        );
        assert_eq!(
            d.chip_label(),
            Some("Connected"),
            "`device.name || 'Connected'`"
        );
    }

    #[test]
    fn stale_results_are_ignored() {
        let mut d = Disto::default();
        assert_eq!(d.update(DistoMsg::Scanned(Ok(vec![]))), Effect::None);
        d.update(DistoMsg::Connect);
        d.update(DistoMsg::Cancel);
        assert_eq!(
            d.update(DistoMsg::Scanned(Ok(vec![dev("a", None)]))),
            Effect::None,
            "a scan that lands after Cancel"
        );
        assert!(d.devices.is_empty());
        assert_eq!(d.update(DistoMsg::Connected(Ok("x".into()))), Effect::None);
    }

    #[test]
    fn devices_sort_disto_first_then_named_then_unnamed() {
        let mut v = vec![
            dev("1", None),
            dev("2", Some("Headset")),
            dev("3", Some("DISTO D2")),
            dev("4", Some("Beacon")),
        ];
        sort_devices(&mut v);
        let order: Vec<&str> = v.iter().map(|d| d.id.as_str()).collect();
        assert_eq!(order, vec!["3", "4", "2", "1"]);
    }

    #[test]
    fn display_names_the_unnamed_like_chrome() {
        assert_eq!(
            dev("AA:BB:CC:DD:EE:FF", None).display(),
            "Unknown or Unsupported Device (EE:FF)"
        );
        assert_eq!(dev("x", Some("  Laser ")).display(), "Laser");
    }
}
