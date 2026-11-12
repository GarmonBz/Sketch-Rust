//! Leica DISTO BLE connection and measurement input -
//! `SketchApp.connectDisto` (sketch-app.js:5449).
//!
//! Native implementation of the leica-disto-ble skill's measured E7500i/D510
//! protocol: subscribe to Leica distance indications, float32 LE metres.
//! Standard stages keyboard input; Continuous uses the previous heading.
//! Native radio behavior remains unverified on hardware. Tests use no radio.

use iced::widget::{button, column, container, opaque, row, scrollable, text, Space};
use iced::{Alignment, Background, Border, Color, Element, Length};

use crate::theme::*;
use crate::Message;

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
                        Effect::Toast("DISTO disconnected.".into())
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
                        Effect::Toast(format!("DISTO connection was not completed: {reason}"))
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

// ---------------------------------------------------------------------------
// The chooser card
// ---------------------------------------------------------------------------

fn chip(label: &'static str, msg: Option<DistoMsg>, primary: bool) -> Element<'static, Message> {
    let enabled = msg.is_some();
    let ink = if !enabled {
        muted()
    } else if primary {
        Color::WHITE
    } else {
        text_main()
    };
    let b = button(container(text(label).font(SEGOE).size(12.0).color(ink)).padding([5, 14]))
        .padding(0)
        .style(move |_, status| button::Style {
            background: Some(Background::Color(if !enabled {
                bg_ribbon()
            } else if primary {
                accent()
            } else if matches!(status, button::Status::Hovered) {
                bg_hover()
            } else {
                bg_ribbon()
            })),
            border: Border {
                color: border(),
                width: 1.0,
                radius: 2.0.into(),
            },
            text_color: ink,
            ..button::Style::default()
        });
    match msg {
        Some(m) => b.on_press(Message::Disto(m)).into(),
        None => b.into(),
    }
}

/// The chooser (the browser's `requestDevice` prompt), or None when closed.
pub fn layer(d: &Disto) -> Option<Element<'_, Message>> {
    if !d.open() {
        return None;
    }
    let status = match d.phase {
        Phase::Scanning => "Scanning for Bluetooth devices\u{2026}".to_string(),
        Phase::Connecting => "Connecting\u{2026}".to_string(),
        _ if d.devices.is_empty() => "No Bluetooth devices were found.".to_string(),
        _ => format!(
            "{} device{} found",
            d.devices.len(),
            if d.devices.len() == 1 { "" } else { "s" }
        ),
    };
    let mut list = column![].spacing(0);
    for (i, device) in d.devices.iter().enumerate() {
        let picked = d.selected == Some(i);
        list = list.push(
            button(
                container(
                    text(device.display())
                        .font(SEGOE)
                        .size(12.0)
                        .color(if picked { Color::WHITE } else { text_main() }),
                )
                .padding([6, 8])
                .width(Length::Fill),
            )
            .padding(0)
            .width(Length::Fill)
            .on_press(Message::Disto(DistoMsg::Pick(i)))
            .style(move |_, status| button::Style {
                background: Some(Background::Color(if picked {
                    accent()
                } else if matches!(status, button::Status::Hovered) {
                    bg_hover()
                } else {
                    Color::TRANSPARENT
                })),
                ..button::Style::default()
            }),
        );
    }
    let list = container(scrollable(list).height(Length::Fill))
        .height(220)
        .width(Length::Fill)
        .padding(5)
        .style(|_| container::Style {
            background: Some(Background::Color(bg_app())),
            border: Border {
                color: border(),
                width: 1.0,
                radius: 0.0.into(),
            },
            ..container::Style::default()
        });
    let busy = matches!(d.phase, Phase::Scanning | Phase::Connecting);
    let can_choose = d.phase == Phase::Picking && d.selected.is_some();
    let actions = row![
        Space::new().width(Length::Fill),
        chip("Cancel", Some(DistoMsg::Cancel), false),
        chip("Rescan", (!busy).then_some(DistoMsg::Rescan), false),
        chip("Connect", can_choose.then_some(DistoMsg::Choose), true),
    ]
    .spacing(8);
    let card = container(
        column![
            text("Select a Bluetooth device")
                .font(SEGOE_SEMIBOLD)
                .size(13.0)
                .color(text_main()),
            text(status).font(SEGOE).size(11.0).color(muted()),
            list,
            actions,
        ]
        .spacing(10),
    )
    .padding(12)
    .width(420)
    .style(|_| container::Style {
        background: Some(Background::Color(bg_panel())),
        border: Border {
            color: border(),
            width: 1.0,
            radius: 5.0.into(),
        },
        ..container::Style::default()
    });
    let backdrop = container(opaque(card))
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Alignment::Center)
        .align_y(Alignment::Center)
        .style(|_| container::Style {
            background: Some(Color::from_rgba(0.0, 0.0, 0.0, 0.46).into()),
            ..container::Style::default()
        });
    Some(opaque(backdrop))
}

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
            Effect::Toast("DISTO disconnected.".into())
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
            Effect::Toast("DISTO connection was not completed: x".into())
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
