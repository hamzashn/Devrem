//! Gerçek zamanlı simülasyon oturumu.
//!
//! Simülasyon ayrı bir iş parçacığında çalışır ve sonuçları yaklaşık 30 kare
//! / saniye hızla bir Tauri kanalı üzerinden arayüze gönderir. Simülasyon
//! süresi, `time_scale` ile çarpılmış gerçek zamanla ilerler.

use devrem_core::CircuitError;
use devrem_sim::{Frame, Simulator};
use serde::Serialize;
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use tauri::ipc::Channel;

const FRAME_INTERVAL: Duration = Duration::from_millis(33);
/// Bir karede en az bu kadar alt adım atılır (doğruluk için).
const MIN_SUBSTEPS: f64 = 8.0;
/// Bir karede en fazla bu kadar alt adım atılır (performans için).
const MAX_SUBSTEPS: f64 = 400.0;
/// Osiloskop için bir karede gönderilen en fazla örnek sayısı.
const MAX_SAMPLES_PER_FRAME: usize = 64;

/// Osiloskop örnekleri: alt adımlardaki ölçüm aleti okumaları.
#[derive(Clone, Serialize)]
pub struct Samples {
    pub time: Vec<f64>,
    /// `values[p][i]`: `Topology::probes[p]` aletinin `time[i]` anındaki okuması.
    pub values: Vec<Vec<f64>>,
}

/// Arayüze gönderilen olaylar.
#[derive(Clone, Serialize)]
#[serde(tag = "event", content = "data", rename_all = "camelCase")]
pub enum SimEvent {
    Frame { frame: Frame, samples: Samples },
    Error(CircuitError),
}

enum Control {
    Stop,
    Adjust { id: String, value: f64 },
    SetTimeScale(f64),
}

pub struct Session {
    tx: Sender<Control>,
    handle: JoinHandle<()>,
}

impl Session {
    pub fn start(sim: Simulator, time_scale: f64, channel: Channel<SimEvent>) -> Self {
        let (tx, rx) = mpsc::channel();
        let handle = thread::Builder::new()
            .name("devrem-sim".into())
            .spawn(move || run(sim, time_scale, &rx, &channel))
            .expect("simülasyon iş parçacığı başlatılamadı");
        Self { tx, handle }
    }

    pub fn stop(self) {
        let _ = self.tx.send(Control::Stop);
        let _ = self.handle.join();
    }

    pub fn set_control(&self, id: String, value: f64) {
        let _ = self.tx.send(Control::Adjust { id, value });
    }

    pub fn set_time_scale(&self, scale: f64) {
        let _ = self.tx.send(Control::SetTimeScale(scale));
    }
}

fn run(
    mut sim: Simulator,
    mut time_scale: f64,
    rx: &Receiver<Control>,
    channel: &Channel<SimEvent>,
) {
    let probe_count = sim.topology().probes.len();
    let mut next = Instant::now();
    loop {
        loop {
            match rx.try_recv() {
                Ok(Control::Stop) | Err(TryRecvError::Disconnected) => return,
                Ok(Control::Adjust { id, value }) => {
                    sim.set_control(&id, value);
                }
                Ok(Control::SetTimeScale(s)) => time_scale = s,
                Err(TryRecvError::Empty) => break,
            }
        }

        let target = FRAME_INTERVAL.as_secs_f64() * time_scale;
        let max_h = sim.max_step().min(target / MIN_SUBSTEPS);
        let substeps = (target / max_h).ceil().clamp(1.0, MAX_SUBSTEPS);
        let h = target / substeps;
        let substeps = substeps as usize;
        let stride = substeps.div_ceil(MAX_SAMPLES_PER_FRAME);
        let mut samples = Samples {
            time: Vec::new(),
            values: vec![Vec::new(); probe_count],
        };
        for i in 0..substeps {
            if let Err(error) = sim.step(h) {
                tracing::warn!("simülasyon hatası: {error}");
                let _ = channel.send(SimEvent::Error(error));
                return;
            }
            if probe_count > 0 && (i + 1) % stride == 0 {
                samples.time.push(sim.time());
                for (p, v) in sim.probe_values().into_iter().enumerate() {
                    samples.values[p].push(v);
                }
            }
        }
        let event = SimEvent::Frame {
            frame: sim.frame(),
            samples,
        };
        if channel.send(event).is_err() {
            return; // pencere kapandı
        }

        next += FRAME_INTERVAL;
        let now = Instant::now();
        if next > now {
            thread::sleep(next - now);
        } else {
            next = now; // geride kaldıysak yetişmeye çalışma
        }
    }
}
