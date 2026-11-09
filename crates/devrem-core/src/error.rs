//! Devre ve simülasyon hataları.
//!
//! Hatalar arayüze `{"code": "...", ...}` biçiminde gönderilir; arayüz
//! `code` alanına göre mesajı kullanıcının dilinde gösterir.

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize, thiserror::Error)]
#[serde(tag = "code", rename_all = "camelCase")]
pub enum CircuitError {
    #[error("devre boş")]
    EmptyCircuit,

    #[error("devrede toprak (GND) yok")]
    NoGround,

    #[error("{component} bileşeninin {prop} değeri geçersiz: {value:?}")]
    #[serde(rename_all = "camelCase")]
    InvalidValue {
        component: String,
        prop: String,
        value: String,
    },

    #[error("{component} bileşeninin {prop} değeri izin verilen aralıkta değil")]
    #[serde(rename_all = "camelCase")]
    OutOfRange { component: String, prop: String },

    /// Matris tekil: kaynak döngüsü, kısa devre veya açık uçlu akım kaynağı.
    #[error("devre çözülemedi (kısa devre, kaynak döngüsü veya açık uçlu akım kaynağı)")]
    Singular,

    #[error("çözüm yakınsamadı (t = {time} s)")]
    NoConvergence { time: f64 },

    /// Analiz sonucunu okuyacak voltmetre / ampermetre yok.
    #[error("devrede ölçüm aleti yok")]
    NoProbes,

    /// AC analizi için uyarı kaynağı (AC gerilim kaynağı) yok.
    #[error("devrede AC kaynak yok")]
    NoAcSource,

    /// DC taramada seçilen kaynak bulunamadı veya DC kaynak değil.
    #[error("taranacak kaynak geçersiz: {id:?}")]
    InvalidSweepSource { id: String },

    /// Analiz ayarları geçersiz (ör. başlangıç ≥ bitiş).
    #[error("analiz ayarları geçersiz")]
    InvalidSettings,
}
