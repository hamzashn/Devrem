//! Shockley diyot modeli ve SPICE tarzı gerilim sınırlama.

/// Oda sıcaklığında (300 K) termal gerilim (V).
pub const THERMAL_VOLTAGE: f64 = 0.025_852;
/// Zener'in kırılma geriliminde akıttığı akım (veri sayfası test akımı), A.
const BREAKDOWN_CURRENT: f64 = 5e-3;

/// Diyot modeli: `I = Is · (exp(V / (n·Vt)) − 1)`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DiodeModel {
    /// Doyma akımı (A).
    pub saturation_current: f64,
    /// İdealite faktörü.
    pub emission: f64,
    /// Ters kırılma gerilimi (zener), V. `None`: kırılma yok.
    pub breakdown: Option<f64>,
}

impl DiodeModel {
    /// 1N4148 benzeri küçük sinyal silisyum diyot.
    pub const SILICON: Self = Self {
        saturation_current: 2.52e-9,
        emission: 1.752,
        breakdown: None,
    };

    /// Verilen kırılma gerilimine sahip zener diyot.
    pub fn zener(breakdown: f64) -> Self {
        Self {
            saturation_current: 1e-12,
            emission: 1.0,
            breakdown: Some(breakdown),
        }
    }

    /// 10 mA'de verilen ileri yön gerilimine sahip bir LED modeli.
    pub fn led(color: &str) -> Option<Self> {
        let forward_voltage = match color {
            "red" => 1.8,
            "yellow" => 2.0,
            "green" => 2.1,
            "blue" | "white" => 3.0,
            _ => return None,
        };
        Some(Self::with_forward_voltage(forward_voltage))
    }

    /// 10 mA'de verilen ileri yön gerilimine sahip LED benzeri diyot (n = 2).
    pub fn with_forward_voltage(forward_voltage: f64) -> Self {
        let emission = 2.0;
        let saturation_current = 0.01 / (forward_voltage / (emission * THERMAL_VOLTAGE)).exp();
        Self {
            saturation_current,
            emission,
            breakdown: None,
        }
    }

    fn nvt(&self) -> f64 {
        self.emission * THERMAL_VOLTAGE
    }

    /// Verilen gerilimde akım ve küçük sinyal iletkenliği `(I, dI/dV)`.
    pub fn evaluate(&self, v: f64) -> (f64, f64) {
        let nvt = self.nvt();
        let e = (v / nvt).exp();
        let mut current = self.saturation_current * (e - 1.0);
        let mut conductance = self.saturation_current * e / nvt;
        if let Some(bv) = self.breakdown {
            // Ters yönde kırılma akımı: v = −bv iken 5 mA, ötesinde üstel artar.
            let r = (-(v + bv) / nvt).exp();
            current -= BREAKDOWN_CURRENT * r;
            conductance += BREAKDOWN_CURRENT * r / nvt;
        }
        (current, conductance)
    }

    /// Kritik gerilim: bunun üzerinde adım sınırlama devreye girer.
    fn critical_voltage(&self, saturation_current: f64) -> f64 {
        let nvt = self.nvt();
        nvt * (nvt / (std::f64::consts::SQRT_2 * saturation_current)).ln()
    }

    /// Newton-Raphson adımlarında üstel taşmayı önlemek için yeni gerilimi
    /// sınırlar (SPICE `pnjlim`).
    pub fn limit(&self, v_new: f64, v_old: f64) -> f64 {
        if let Some(bv) = self.breakdown
            && v_new < 0.0
        {
            // Kırılma bölgesi ileri yönün aynası gibi sınırlanır.
            let mirrored = self.limit_forward(-(v_new + bv), -(v_old + bv), BREAKDOWN_CURRENT);
            return -mirrored - bv;
        }
        self.limit_forward(v_new, v_old, self.saturation_current)
    }

    fn limit_forward(&self, v_new: f64, v_old: f64, saturation_current: f64) -> f64 {
        let nvt = self.nvt();
        let v_crit = self.critical_voltage(saturation_current);
        if v_new > v_crit && (v_new - v_old).abs() > 2.0 * nvt {
            if v_old > 0.0 {
                let arg = 1.0 + (v_new - v_old) / nvt;
                if arg > 0.0 {
                    v_old + nvt * arg.ln()
                } else {
                    v_crit
                }
            } else {
                nvt * (v_new / nvt).ln()
            }
        } else {
            v_new
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn led_forward_voltage_at_10ma() {
        for (color, vf) in [("red", 1.8), ("green", 2.1), ("blue", 3.0)] {
            let model = DiodeModel::led(color).unwrap();
            let (i, _) = model.evaluate(vf);
            assert!((i - 0.01).abs() < 1e-6, "{color}: {i}");
        }
        assert!(DiodeModel::led("purple").is_none());
    }

    #[test]
    fn zener_conducts_in_reverse() {
        let z = DiodeModel::zener(5.1);
        assert!(z.evaluate(-3.0).0.abs() < 1e-9, "kırılmadan önce akım yok");
        let (i, _) = z.evaluate(-5.1);
        assert!((i + 5e-3).abs() < 1e-6, "kırılma geriliminde 5 mA: {i}");
    }

    #[test]
    fn limiting_keeps_steps_small() {
        let m = DiodeModel::SILICON;
        let limited = m.limit(5.0, 0.6);
        assert!(limited < 1.0, "{limited}");
        assert_eq!(m.limit(0.1, 0.0), 0.1);
    }
}
