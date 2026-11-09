//! Kaynak dalga biçimleri.

use std::f64::consts::TAU;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Waveform {
    Dc(f64),
    /// `offset + amplitude · sin(2π·f·t + phase)`; `phase` radyan.
    Sine {
        amplitude: f64,
        frequency: f64,
        offset: f64,
        phase: f64,
    },
    /// Kare dalga: periyodun `duty` (0–1) kısmında `high`, kalanında `low`.
    Pulse {
        low: f64,
        high: f64,
        frequency: f64,
        duty: f64,
    },
}

impl Waveform {
    /// `t` anındaki değer.
    pub fn value(&self, t: f64) -> f64 {
        match *self {
            Self::Dc(v) => v,
            Self::Sine {
                amplitude,
                frequency,
                offset,
                phase,
            } => offset + amplitude * (TAU * frequency * t + phase).sin(),
            Self::Pulse {
                low,
                high,
                frequency,
                duty,
            } => {
                if (t * frequency).fract() < duty {
                    high
                } else {
                    low
                }
            }
        }
    }

    /// DC analizlerde kullanılan değer (sinüste ofset).
    pub fn dc_value(&self) -> f64 {
        match *self {
            Self::Dc(v) => v,
            Self::Sine { offset, .. } => offset,
            Self::Pulse { low, .. } => low,
        }
    }

    /// Doğru örnekleme için önerilen en büyük zaman adımı.
    pub fn max_step(&self) -> Option<f64> {
        match *self {
            Self::Dc(_) => None,
            Self::Sine { frequency, .. } => Some(1.0 / (50.0 * frequency)),
            Self::Pulse { frequency, .. } => Some(1.0 / (100.0 * frequency)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sine_values() {
        let w = Waveform::Sine {
            amplitude: 2.0,
            frequency: 1.0,
            offset: 1.0,
            phase: 0.0,
        };
        assert!((w.value(0.0) - 1.0).abs() < 1e-12);
        assert!((w.value(0.25) - 3.0).abs() < 1e-12);
        assert!((w.value(0.75) + 1.0).abs() < 1e-12);
    }
}
