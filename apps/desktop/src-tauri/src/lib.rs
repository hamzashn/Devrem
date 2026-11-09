//! Devrem masaüstü uygulamasının Tauri katmanı.
//!
//! Arayüzle Rust çekirdeği arasındaki IPC komutları burada tanımlanır.

mod session;

use devrem_core::{CircuitError, Document, DocumentError};
use devrem_sim::{AcResult, DcSweepResult, Frame, Simulator, Topology};
use serde::Serialize;
use session::{Session, SimEvent};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;
use tauri::ipc::Channel;

/// Arayüze gönderilen uygulama bilgileri.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppInfo {
    app_version: &'static str,
    engine_version: &'static str,
    format_version: u32,
}

#[tauri::command]
fn app_info() -> AppInfo {
    AppInfo {
        app_version: env!("CARGO_PKG_VERSION"),
        engine_version: devrem_sim::VERSION,
        format_version: devrem_core::FORMAT_VERSION,
    }
}

// ---------------------------------------------------------------- Dosyalar

#[tauri::command]
fn open_document(path: PathBuf) -> Result<Document, DocumentError> {
    let text = std::fs::read_to_string(&path).map_err(|e| DocumentError::Io {
        message: e.to_string(),
    })?;
    Document::from_json(&text)
}

#[tauri::command]
fn save_document(path: PathBuf, document: Document) -> Result<(), DocumentError> {
    std::fs::write(&path, document.to_json()).map_err(|e| DocumentError::Io {
        message: e.to_string(),
    })
}

/// Belgenin SPICE netlist karşılığı.
#[tauri::command]
fn export_spice(document: Document) -> Result<String, CircuitError> {
    devrem_sim::spice::to_spice(&document)
}

/// Metin dosyası yazar (SPICE, SVG, CSV).
#[tauri::command]
fn write_text_file(path: PathBuf, contents: String) -> Result<(), DocumentError> {
    std::fs::write(&path, contents).map_err(|e| DocumentError::Io {
        message: e.to_string(),
    })
}

/// Base64 ile kodlanmış ikili dosya yazar (PNG).
#[tauri::command]
fn write_binary_file(path: PathBuf, base64: String) -> Result<(), DocumentError> {
    use base64::Engine;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(base64)
        .map_err(|e| DocumentError::Io {
            message: e.to_string(),
        })?;
    std::fs::write(&path, bytes).map_err(|e| DocumentError::Io {
        message: e.to_string(),
    })
}

// -------------------------------------------------------------- Simülasyon

#[derive(Default)]
struct SimState(Mutex<Option<Session>>);

impl SimState {
    fn replace(&self, next: Option<Session>) {
        let previous = {
            let mut guard = self.0.lock().expect("simülasyon kilidi");
            std::mem::replace(&mut *guard, next)
        };
        if let Some(session) = previous {
            session.stop();
        }
    }
}

/// Gerçek zamanlı simülasyonu başlatır (çalışan varsa durdurur).
#[tauri::command]
fn sim_start(
    state: State<'_, SimState>,
    document: Document,
    on_event: Channel<SimEvent>,
) -> Result<Topology, CircuitError> {
    state.replace(None);
    let sim = Simulator::new(&document)?;
    let topology = sim.topology();
    let session = Session::start(sim, document.simulation.time_scale, on_event);
    state.replace(Some(session));
    Ok(topology)
}

#[tauri::command]
fn sim_stop(state: State<'_, SimState>) {
    state.replace(None);
}

/// Etkileşimli bir bileşenin kontrolünü değiştirir (anahtar, buton, lojik
/// giriş: 0/1; potansiyometre: 0–1 konum). Devre durumu korunur.
#[tauri::command]
fn sim_set_control(state: State<'_, SimState>, id: String, value: f64) {
    if let Some(session) = &*state.0.lock().expect("simülasyon kilidi") {
        session.set_control(id, value);
    }
}

#[tauri::command]
fn sim_set_time_scale(state: State<'_, SimState>, scale: f64) {
    if let Some(session) = &*state.0.lock().expect("simülasyon kilidi") {
        session.set_time_scale(scale);
    }
}

#[derive(Serialize)]
struct DcResult {
    topology: Topology,
    frame: Frame,
}

/// DC çalışma noktasını hesaplar.
#[tauri::command]
fn sim_dc(document: Document) -> Result<DcResult, CircuitError> {
    let mut sim = Simulator::new(&document)?;
    let frame = sim.dc_operating_point()?;
    Ok(DcResult {
        topology: sim.topology(),
        frame,
    })
}

/// AC (frekans) analizi.
#[tauri::command]
async fn sim_ac(document: Document) -> Result<AcResult, CircuitError> {
    let mut sim = Simulator::new(&document)?;
    sim.ac_analysis(&document.simulation.ac)
}

/// DC tarama.
#[tauri::command]
async fn sim_dc_sweep(document: Document) -> Result<DcSweepResult, CircuitError> {
    let mut sim = Simulator::new(&document)?;
    sim.dc_sweep(&document.simulation.dc_sweep)
}

/// Uygulamayı başlatır.
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    tracing::info!("Devrem {} başlatılıyor", env!("CARGO_PKG_VERSION"));

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .manage(SimState::default())
        .invoke_handler(tauri::generate_handler![
            app_info,
            open_document,
            save_document,
            sim_start,
            sim_stop,
            sim_set_control,
            export_spice,
            write_text_file,
            write_binary_file,
            sim_set_time_scale,
            sim_dc,
            sim_ac,
            sim_dc_sweep,
        ])
        .run(tauri::generate_context!())
        .expect("Tauri uygulaması başlatılamadı");
}
