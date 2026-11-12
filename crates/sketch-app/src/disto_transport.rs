//! Native Leica GATT transport. Tests substitute a radio-free transport.
use super::{Device, Failure, RadioEvent};
use btleplug::api::{
    Central, CharPropFlags, Characteristic, Manager as _, Peripheral as _, ScanFilter,
    ValueNotification,
};
use btleplug::platform::{Adapter, Manager, Peripheral};
use iced::futures::{
    channel::{mpsc, oneshot},
    FutureExt, Stream, StreamExt,
};
use sketch_core::disto::{parse_meters, LEICA_DISTANCE, LEICA_SERVICE};
use std::{
    pin::Pin,
    sync::{
        atomic::{AtomicU64, Ordering},
        OnceLock,
    },
    time::Duration,
};

enum Cmd {
    Scan(bool, oneshot::Sender<Result<Vec<Device>, Failure>>),
    Connect(String, u64, oneshot::Sender<Result<String, Failure>>),
    Disconnect,
    Listen(mpsc::UnboundedSender<(u64, RadioEvent)>),
}

static SESSION: AtomicU64 = AtomicU64::new(0);

struct Link {
    peripheral: Peripheral,
    characteristic: Characteristic,
    values: Pin<Box<dyn Stream<Item = ValueNotification> + Send>>,
    session: u64,
}

impl Link {
    async fn close(self) {
        let _ = tokio::time::timeout(
            Duration::from_secs(2),
            self.peripheral.unsubscribe(&self.characteristic),
        )
        .await;
        let _ = tokio::time::timeout(Duration::from_secs(2), self.peripheral.disconnect()).await;
    }
}

fn worker() -> &'static mpsc::UnboundedSender<Cmd> {
    static TX: OnceLock<mpsc::UnboundedSender<Cmd>> = OnceLock::new();
    TX.get_or_init(|| {
        let (tx, rx) = mpsc::unbounded();
        std::thread::spawn(move || {
            if let Ok(rt) = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                rt.block_on(run(rx));
            }
        });
        tx
    })
}

enum Wake {
    Command(Option<Cmd>),
    Value(Option<ValueNotification>),
    Check,
}

async fn run(mut commands: mpsc::UnboundedReceiver<Cmd>) {
    let mut adapter = None;
    let mut known = Vec::new();
    let mut link: Option<Link> = None;
    let mut events: Option<mpsc::UnboundedSender<(u64, RadioEvent)>> = None;
    loop {
        let wake = if let Some(active) = &mut link {
            iced::futures::select! {
                cmd = commands.next().fuse() => Wake::Command(cmd),
                value = active.values.next().fuse() => Wake::Value(value),
                _ = tokio::time::sleep(Duration::from_secs(1)).fuse() => Wake::Check,
            }
        } else {
            Wake::Command(commands.next().await)
        };
        match wake {
            Wake::Command(None) => break,
            Wake::Command(Some(Cmd::Listen(sender))) => events = Some(sender),
            Wake::Command(Some(Cmd::Disconnect)) => {
                if let Some(active) = link.take() {
                    active.close().await;
                }
            }
            Wake::Command(Some(Cmd::Scan(manual, reply))) => {
                let _ = reply.send(scan(&mut adapter, &mut known, manual).await);
            }
            Wake::Command(Some(Cmd::Connect(id, session, reply))) => {
                if let Some(active) = link.take() {
                    active.close().await;
                }
                let result = connect(&id, session, &known).await;
                let result = match result {
                    Ok((name, active)) => {
                        link = Some(active);
                        Ok(name)
                    }
                    Err(error) => Err(error),
                };
                let _ = reply.send(result);
            }
            Wake::Value(Some(value)) => {
                let active = link.as_ref().expect("value requires a link");
                if SESSION.load(Ordering::SeqCst) == active.session
                    && value.uuid == active.characteristic.uuid
                {
                    if let (Some(meters), Some(sender)) = (parse_meters(&value.value), &events) {
                        let _ = sender
                            .unbounded_send((active.session, RadioEvent::Measurement(meters)));
                    }
                }
            }
            Wake::Value(None) | Wake::Check => {
                let active = link.as_ref().expect("check requires a link");
                let connected =
                    tokio::time::timeout(Duration::from_secs(2), active.peripheral.is_connected())
                        .await;
                if matches!(wake, Wake::Value(None)) || !matches!(connected, Ok(Ok(true))) {
                    let active = link.take().expect("checked link");
                    if let Some(sender) = &events {
                        let _ = sender.unbounded_send((active.session, RadioEvent::Disconnected));
                    }
                    active.close().await;
                }
            }
        }
    }
    if let Some(active) = link {
        active.close().await;
    }
}

fn failed(error: impl std::fmt::Display) -> Failure {
    Failure::Failed(error.to_string())
}

async fn scan(
    slot: &mut Option<Adapter>,
    known: &mut Vec<Peripheral>,
    manual: bool,
) -> Result<Vec<Device>, Failure> {
    let central = if let Some(central) = slot {
        central.clone()
    } else {
        let manager = Manager::new().await.map_err(|_| Failure::Unavailable)?;
        let central = manager
            .adapters()
            .await
            .map_err(failed)?
            .into_iter()
            .next()
            .ok_or(Failure::Unavailable)?;
        *slot = Some(central.clone());
        central
    };
    central
        .start_scan(ScanFilter::default())
        .await
        .map_err(failed)?;
    tokio::time::sleep(Duration::from_secs(8)).await;
    let _ = central.stop_scan().await;
    let found = central.peripherals().await.map_err(failed)?;
    let mut out = Vec::new();
    for peripheral in &found {
        if let Some(props) = peripheral.properties().await.map_err(failed)? {
            let leica = props
                .services
                .iter()
                .any(|id| id.to_string() == LEICA_SERVICE);
            let device = Device {
                id: peripheral.id().to_string(),
                name: props.local_name,
            };
            if manual || leica || device.is_disto() {
                out.push(device);
            }
        }
    }
    *known = found;
    Ok(out)
}

async fn connect(id: &str, session: u64, known: &[Peripheral]) -> Result<(String, Link), Failure> {
    let peripheral = known
        .iter()
        .find(|p| p.id().to_string() == id)
        .ok_or_else(|| failed("device is no longer visible; rescan"))?
        .clone();
    let setup = async {
        peripheral.connect().await.map_err(failed)?;
        peripheral.discover_services().await.map_err(failed)?;
        let characteristic = peripheral
            .characteristics()
            .into_iter()
            .find(|c| {
                c.service_uuid.to_string() == LEICA_SERVICE && c.uuid.to_string() == LEICA_DISTANCE
            })
            .ok_or_else(|| {
                failed(
                    "Leica distance service was not found; this device may not support DISTO BLE",
                )
            })?;
        if !characteristic
            .properties
            .intersects(CharPropFlags::INDICATE | CharPropFlags::NOTIFY)
        {
            return Err(failed(
                "Leica distance characteristic cannot indicate or notify",
            ));
        }
        let values = peripheral.notifications().await.map_err(failed)?;
        peripheral
            .subscribe(&characteristic)
            .await
            .map_err(failed)?;
        let name = peripheral
            .properties()
            .await
            .map_err(failed)?
            .and_then(|p| p.local_name)
            .unwrap_or_else(|| "DISTO".into());
        Ok((
            name,
            Link {
                peripheral: peripheral.clone(),
                characteristic,
                values,
                session,
            },
        ))
    };
    let cancelled = async {
        loop {
            if SESSION.load(Ordering::SeqCst) != session {
                break;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    };
    let result = iced::futures::select! {
        result = tokio::time::timeout(Duration::from_secs(90), setup).fuse() =>
            result.unwrap_or_else(|_| Err(failed("connection timed out; move the DISTO closer or check the Bluetooth adapter"))),
        _ = cancelled.fuse() => Err(failed("connection cancelled")),
    };
    if result.is_err() {
        let _ = tokio::time::timeout(Duration::from_secs(2), peripheral.disconnect()).await;
    }
    result
}

pub async fn scan_devices(manual: bool) -> Result<Vec<Device>, Failure> {
    let (tx, rx) = oneshot::channel();
    worker()
        .unbounded_send(Cmd::Scan(manual, tx))
        .map_err(|_| Failure::Unavailable)?;
    rx.await.unwrap_or(Err(Failure::Unavailable))
}

pub async fn connect_device(id: String, session: u64) -> Result<String, Failure> {
    // A cancelled task must never re-arm an obsolete connection.
    if SESSION.load(Ordering::SeqCst) != session {
        return Err(failed("connection cancelled"));
    }
    let (tx, rx) = oneshot::channel();
    worker()
        .unbounded_send(Cmd::Connect(id, session, tx))
        .map_err(|_| Failure::Unavailable)?;
    rx.await
        .unwrap_or_else(|_| Err(failed("Bluetooth worker stopped")))
}

pub fn set_session(session: u64) {
    SESSION.store(session, Ordering::SeqCst);
}

pub fn disconnect_device() {
    let _ = worker().unbounded_send(Cmd::Disconnect);
}

pub fn event_stream() -> impl Stream<Item = super::DistoMsg> {
    let (tx, rx) = mpsc::unbounded();
    let _ = worker().unbounded_send(Cmd::Listen(tx));
    rx.map(|(session, event)| super::DistoMsg::Radio(session, event))
}
