//! MNA (Modified Nodal Analysis) tabanlı devre simülatörü.
//!
//! Bilinmeyenler: toprak hariç düğüm gerilimleri, ardından dal akımları
//! (gerilim kaynakları, bobinler, ampermetreler, op-amp ve lojik çıkışları).
//! Kondansatör ve bobinler geçici durumda trapez (veya geri Euler) yöntemiyle
//! eşdeğer devre modellerine dönüşür. Doğrusal olmayan elemanlar (diyot,
//! transistör, op-amp) Newton-Raphson ile çözülür.

use crate::lu::Lu;
use devrem_components::logic::{LOGIC_HIGH, LOGIC_OUTPUT_RESISTANCE, LOGIC_THRESHOLD};
use devrem_components::mosfet::limit_step;
use devrem_components::timer::{
    DISCHARGE_RESISTANCE, DIVIDER_RESISTANCE, OUTPUT_RESISTANCE, RESET_PULLUP, pins as tp,
};
use devrem_components::{
    Device, POT_MIN_RESISTANCE, SWITCH_R_OFF, SWITCH_R_ON, Waveform, opamp_output,
};
use devrem_core::{
    AcSettings, CircuitError, ComponentKind, DcSweepSettings, Document, Netlist, Point,
};
use num_complex::Complex64;
use serde::Serialize;

/// Her düğümden toprağa eklenen küçük iletkenlik (yüzen düğümleri önler).
const GMIN: f64 = 1e-12;
const MAX_NEWTON_ITERATIONS: usize = 200;
const RELTOL: f64 = 1e-6;
const VNTOL: f64 = 1e-7;
const ABSTOL: f64 = 1e-10;
/// Bundan büyük gerilim / akım çıkarsa devre hatalı kabul edilir.
const SANITY_LIMIT: f64 = 1e9;
/// DC çözümde lojik durumların oturması için en fazla yineleme.
const MAX_LOGIC_ITERATIONS: usize = 50;
/// AC analizinde en fazla frekans noktası.
const MAX_AC_POINTS: usize = 2000;

/// Zaman ayrıklaştırma yöntemi.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    /// Geri Euler: kararlı, ilk adımda ve anahtarlamadan sonra kullanılır.
    BackwardEuler,
    /// Trapez: ikinci dereceden doğru.
    Trapezoidal,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Mode {
    Dc,
    Transient { h: f64, method: Method },
}

/// Bir zaman anındaki sonuçlar.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Frame {
    pub time: f64,
    /// Düğüm gerilimleri; indeks 0 topraktır (0 V).
    pub node_voltages: Vec<f64>,
    /// Her bileşenin her pininden bileşene giren akım (`Topology::elements`
    /// sırası). İki uçlu bileşenlerde `[I, −I]`.
    pub pin_currents: Vec<Vec<f64>>,
    /// Her tel parçasında `a`'dan `b`'ye akan akım (`Topology::segments` sırası).
    pub segment_currents: Vec<f64>,
}

/// Ölçüm türü.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ProbeKind {
    Voltage,
    Current,
}

/// Ölçüm yapan bileşen (voltmetre, ampermetre, lojik prob).
#[derive(Debug, Clone, Serialize)]
pub struct Probe {
    pub id: String,
    pub kind: ProbeKind,
}

/// Arayüzün sonuçları şemaya eşlemesi için devre yapısı.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Topology {
    pub node_count: usize,
    pub elements: Vec<TopologyElement>,
    pub segments: Vec<TopologySegment>,
    pub probes: Vec<Probe>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TopologyElement {
    pub id: String,
    pub nodes: Vec<usize>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TopologySegment {
    pub wire_id: String,
    pub a: Point,
    pub b: Point,
    pub node: usize,
}

/// AC analizinde bir ölçüm aletinin frekans tepkisi.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AcProbe {
    pub id: String,
    pub kind: ProbeKind,
    /// `20·log10(|H|)`; H = ölçülen / kaynak genliği.
    pub magnitude_db: Vec<f64>,
    /// Faz (derece, sarılmamış).
    pub phase_deg: Vec<f64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AcResult {
    pub frequencies: Vec<f64>,
    /// Uyarı kaynağı (ilk AC gerilim kaynağı).
    pub reference: String,
    pub probes: Vec<AcProbe>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SweepProbe {
    pub id: String,
    pub kind: ProbeKind,
    pub data: Vec<f64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DcSweepResult {
    pub source: String,
    /// Taranan büyüklük gerilim mi (`true`) akım mı?
    pub voltage: bool,
    pub values: Vec<f64>,
    pub probes: Vec<SweepProbe>,
}

/// Kondansatör ve bobinlerin geçmiş durumu.
#[derive(Debug, Clone, Copy, Default)]
struct State {
    v: f64,
    i: f64,
}

struct System<T> {
    size: usize,
    a: Vec<T>,
    b: Vec<T>,
}

impl<T> System<T>
where
    T: Copy + Default + std::ops::AddAssign + std::ops::Neg<Output = T>,
{
    fn new(size: usize) -> Self {
        Self {
            size,
            a: vec![T::default(); size * size],
            b: vec![T::default(); size],
        }
    }

    fn add(&mut self, row: Option<usize>, col: Option<usize>, v: T) {
        if let (Some(r), Some(c)) = (row, col) {
            self.a[r * self.size + c] += v;
        }
    }

    fn rhs(&mut self, row: Option<usize>, v: T) {
        if let Some(r) = row {
            self.b[r] += v;
        }
    }

    fn conductance(&mut self, a: Option<usize>, b: Option<usize>, g: T) {
        self.add(a, a, g);
        self.add(b, b, g);
        self.add(a, b, -g);
        self.add(b, a, -g);
    }

    /// Eleman içinden `a`'dan `b`'ye akan sabit akım.
    fn current(&mut self, a: Option<usize>, b: Option<usize>, j: T) {
        self.rhs(a, -j);
        self.rhs(b, j);
    }

    /// `a`–`b` arasında `k` dal akımı bilinmeyeni olan eleman.
    fn branch(&mut self, a: Option<usize>, b: Option<usize>, k: usize, one: T) {
        self.add(a, Some(k), one);
        self.add(b, Some(k), -one);
        self.add(Some(k), a, one);
        self.add(Some(k), b, -one);
    }

    /// Bir terminalden cihaza giren akım: `Σ coeff·v + constant`.
    fn terminal_current(&mut self, row: Option<usize>, coeffs: &[(Option<usize>, T)], constant: T) {
        for &(col, g) in coeffs {
            self.add(row, col, g);
        }
        self.rhs(row, -constant);
    }
}

/// Tek bir devrenin simülasyon durumu.
pub struct Simulator {
    netlist: Netlist,
    ids: Vec<String>,
    wire_ids: Vec<String>,
    devices: Vec<Device>,
    /// Her elemanın pinlerinin matris indeksleri; `None` topraktır.
    terminals: Vec<Vec<Option<usize>>>,
    /// Her elemanın dal akımı bilinmeyenleri.
    branches: Vec<Vec<usize>>,
    probes: Vec<(usize, ProbeKind)>,
    node_unknowns: usize,
    size: usize,
    linear: bool,

    x: Vec<f64>,
    states: Vec<State>,
    /// Newton-Raphson sırasında sınırlanmış jonksiyon gerilimleri.
    junction: Vec<[f64; 2]>,
    /// Son kabul edilen çözümdeki jonksiyon gerilimleri.
    accepted_junction: Vec<[f64; 2]>,
    time: f64,
    eval_time: f64,
    next_method: Method,
    /// DC analizlerde AC kaynakların yalnızca DC bileşeni kullanılır.
    dc_bias_only: bool,
    /// Lojik başlangıç durumu oturtuldu mu?
    logic_primed: bool,
    lu_cache: Option<(Mode, Lu<f64>)>,
}

fn voltage(x: &[f64], node: Option<usize>) -> f64 {
    node.map_or(0.0, |i| x[i])
}

impl Simulator {
    pub fn new(doc: &Document) -> Result<Self, CircuitError> {
        let netlist = Netlist::build(doc)?;
        let node_unknowns = netlist.node_count - 1;
        let node_index = |n: usize| n.checked_sub(1);

        let mut ids = Vec::new();
        let mut devices = Vec::new();
        let mut terminals = Vec::new();
        let mut branches = Vec::new();
        let mut probes = Vec::new();
        let mut size = node_unknowns;

        for (e, element) in netlist.elements.iter().enumerate() {
            let component = &doc.components[element.component];
            let device = Device::from_component(component)?.expect("toprak netlist elemanı olamaz");
            let branch_count = match &device {
                Device::VoltageSource { .. }
                | Device::Inductor { .. }
                | Device::Ammeter
                | Device::OpAmp { .. }
                | Device::Timer(_) => 1,
                Device::Logic(logic) => logic.outputs.len(),
                _ => 0,
            };
            branches.push((0..branch_count).map(|i| size + i).collect());
            size += branch_count;
            match component.kind {
                ComponentKind::Voltmeter | ComponentKind::LogicProbe => {
                    probes.push((e, ProbeKind::Voltage));
                }
                ComponentKind::Ammeter => probes.push((e, ProbeKind::Current)),
                _ => {}
            }
            terminals.push(element.nodes.iter().map(|&n| node_index(n)).collect());
            ids.push(component.id.clone());
            devices.push(device);
        }

        let linear = !devices.iter().any(Device::is_nonlinear);
        let count = devices.len();
        Ok(Self {
            wire_ids: doc.wires.iter().map(|w| w.id.clone()).collect(),
            netlist,
            ids,
            devices,
            terminals,
            branches,
            probes,
            node_unknowns,
            size,
            linear,
            x: vec![0.0; size],
            states: vec![State::default(); count],
            junction: vec![[0.0; 2]; count],
            accepted_junction: vec![[0.0; 2]; count],
            time: 0.0,
            eval_time: 0.0,
            next_method: Method::BackwardEuler,
            dc_bias_only: false,
            logic_primed: false,
            lu_cache: None,
        })
    }

    pub fn time(&self) -> f64 {
        self.time
    }

    pub fn topology(&self) -> Topology {
        Topology {
            node_count: self.netlist.node_count,
            elements: self
                .netlist
                .elements
                .iter()
                .zip(&self.ids)
                .map(|(e, id)| TopologyElement {
                    id: id.clone(),
                    nodes: e.nodes.clone(),
                })
                .collect(),
            segments: self
                .netlist
                .segments
                .iter()
                .map(|s| TopologySegment {
                    wire_id: self.wire_ids[s.wire].clone(),
                    a: s.a,
                    b: s.b,
                    node: s.node,
                })
                .collect(),
            probes: self
                .probes
                .iter()
                .map(|&(e, kind)| Probe {
                    id: self.ids[e].clone(),
                    kind,
                })
                .collect(),
        }
    }

    /// Kaynakları doğru örneklemek için önerilen en büyük zaman adımı.
    pub fn max_step(&self) -> f64 {
        self.devices
            .iter()
            .filter_map(|d| match d {
                Device::VoltageSource { waveform } => waveform.max_step(),
                Device::Logic(logic) => logic.max_step(),
                _ => None,
            })
            .fold(f64::INFINITY, f64::min)
    }

    /// Anahtarı veya lojik girişi açar/kapatır. Bkz. [`Self::set_control`].
    pub fn set_toggle(&mut self, id: &str, on: bool) -> bool {
        self.set_control(id, if on { 1.0 } else { 0.0 })
    }

    /// Etkileşimli bir bileşenin kontrolünü değiştirir; devre durumu
    /// (kondansatör yükleri vb.) korunur. Anahtarlar, buton, iki yollu anahtar
    /// ve lojik giriş için `value > 0.5` açık/yüksek demektir; potansiyometrede
    /// `value` 0–1 arası konumdur. Bileşen bulunamazsa `false` döner.
    pub fn set_control(&mut self, id: &str, value: f64) -> bool {
        let Some(i) = self.ids.iter().position(|x| x == id) else {
            return false;
        };
        let on = value > 0.5;
        let changed_matrix = match &mut self.devices[i] {
            Device::Switch { closed } => {
                *closed = on;
                true
            }
            Device::Spdt { to_b } => {
                *to_b = on;
                true
            }
            Device::Potentiometer { position, .. } => {
                *position = value.clamp(0.0, 1.0);
                true
            }
            Device::Logic(logic) => {
                logic.high = on;
                false
            }
            _ => return false,
        };
        if changed_matrix {
            self.lu_cache = None;
            self.next_method = Method::BackwardEuler;
        }
        true
    }

    // ------------------------------------------------------------ Kurulum

    fn source_value(&self, waveform: &Waveform) -> f64 {
        if self.dc_bias_only {
            waveform.dc_value()
        } else {
            waveform.value(self.eval_time)
        }
    }

    fn assemble(&mut self, x: &[f64], mode: Mode, extra_gmin: f64) -> (System<f64>, bool) {
        let mut sys = System::new(self.size);
        let mut limited = false;

        for e in 0..self.devices.len() {
            let t = &self.terminals[e];
            let state = self.states[e];
            let v = |i: usize| voltage(x, t[i]);
            match &self.devices[e] {
                Device::Resistor { resistance } => sys.conductance(t[0], t[1], 1.0 / resistance),
                Device::Switch { closed } => {
                    let r = if *closed { SWITCH_R_ON } else { SWITCH_R_OFF };
                    sys.conductance(t[0], t[1], 1.0 / r);
                }
                Device::Voltmeter => {}
                Device::Capacitor { capacitance } => {
                    if let Mode::Transient { h, method } = mode {
                        let (geq, j) = match method {
                            Method::BackwardEuler => {
                                let g = capacitance / h;
                                (g, -g * state.v)
                            }
                            Method::Trapezoidal => {
                                let g = 2.0 * capacitance / h;
                                (g, -(g * state.v + state.i))
                            }
                        };
                        sys.conductance(t[0], t[1], geq);
                        sys.current(t[0], t[1], j);
                    }
                }
                Device::Inductor { inductance } => {
                    let k = self.branches[e][0];
                    sys.branch(t[0], t[1], k, 1.0);
                    if let Mode::Transient { h, method } = mode {
                        let (req, rhs) = match method {
                            Method::BackwardEuler => {
                                let r = inductance / h;
                                (r, -r * state.i)
                            }
                            Method::Trapezoidal => {
                                let r = 2.0 * inductance / h;
                                (r, -r * state.i - state.v)
                            }
                        };
                        sys.add(Some(k), Some(k), -req);
                        sys.rhs(Some(k), rhs);
                    }
                }
                Device::VoltageSource { waveform } => {
                    let k = self.branches[e][0];
                    sys.branch(t[0], t[1], k, 1.0);
                    sys.rhs(Some(k), self.source_value(waveform));
                }
                Device::Ammeter => sys.branch(t[0], t[1], self.branches[e][0], 1.0),
                Device::CurrentSource { current } => sys.current(t[0], t[1], -current),
                Device::Diode { model } => {
                    let v_new = v(0) - v(1);
                    let vd = model.limit(v_new, self.junction[e][0]);
                    limited |= (vd - v_new).abs() > 1e-9;
                    self.junction[e][0] = vd;
                    let (id, gd) = model.evaluate(vd);
                    sys.conductance(t[0], t[1], gd + GMIN);
                    sys.current(t[0], t[1], id - gd * vd);
                }
                Device::Bjt { model } => {
                    let (c, b, em) = (t[0], t[1], t[2]);
                    let vbe_new = v(1) - v(2);
                    let vbc_new = v(1) - v(0);
                    let vbe = model.limit(vbe_new, self.junction[e][0]);
                    let vbc = model.limit(vbc_new, self.junction[e][1]);
                    limited |= (vbe - vbe_new).abs() > 1e-9 || (vbc - vbc_new).abs() > 1e-9;
                    self.junction[e] = [vbe, vbc];
                    let ev = model.evaluate(vbe, vbc);
                    let ic0 = ev.ic - ev.dic_dvbe * vbe - ev.dic_dvbc * vbc;
                    let ib0 = ev.ib - ev.dib_dvbe * vbe - ev.dib_dvbc * vbc;
                    let row_c = [
                        (b, ev.dic_dvbe + ev.dic_dvbc),
                        (em, -ev.dic_dvbe),
                        (c, -ev.dic_dvbc),
                    ];
                    let row_b = [
                        (b, ev.dib_dvbe + ev.dib_dvbc),
                        (em, -ev.dib_dvbe),
                        (c, -ev.dib_dvbc),
                    ];
                    let row_e: Vec<_> = row_c
                        .iter()
                        .zip(&row_b)
                        .map(|(&(n, gc), &(_, gb))| (n, -(gc + gb)))
                        .collect();
                    sys.terminal_current(c, &row_c, ic0);
                    sys.terminal_current(b, &row_b, ib0);
                    sys.terminal_current(em, &row_e, -(ic0 + ib0));
                    sys.conductance(b, em, GMIN);
                    sys.conductance(b, c, GMIN);
                }
                Device::Mosfet { model } => {
                    let (d, g, s) = (t[0], t[1], t[2]);
                    let vgs_new = v(1) - v(2);
                    let vds_new = v(0) - v(2);
                    let vgs = limit_step(vgs_new, self.junction[e][0], 1.0);
                    let vds = limit_step(vds_new, self.junction[e][1], 2.0);
                    limited |= vgs != vgs_new || vds != vds_new;
                    self.junction[e] = [vgs, vds];
                    let ev = model.evaluate(vgs, vds);
                    let i0 = ev.id - ev.gm * vgs - ev.gds * vds;
                    let row_d = [(g, ev.gm), (d, ev.gds), (s, -(ev.gm + ev.gds))];
                    let row_s = [(g, -ev.gm), (d, -ev.gds), (s, ev.gm + ev.gds)];
                    sys.terminal_current(d, &row_d, i0);
                    sys.terminal_current(s, &row_s, -i0);
                    sys.conductance(d, s, GMIN);
                }
                Device::OpAmp { gain, vsat } => {
                    let (minus, plus, out) = (t[0], t[1], t[2]);
                    let k = self.branches[e][0];
                    let vd = v(1) - v(0);
                    let (f, slope) = opamp_output(*gain, *vsat, vd);
                    sys.add(out, Some(k), 1.0);
                    sys.add(Some(k), out, 1.0);
                    sys.add(Some(k), plus, -slope);
                    sys.add(Some(k), minus, slope);
                    sys.rhs(Some(k), f - slope * vd);
                }
                Device::Logic(logic) => {
                    for (j, &pin) in logic.outputs.iter().enumerate() {
                        let k = self.branches[e][j];
                        sys.branch(t[pin], None, k, 1.0);
                        sys.add(Some(k), Some(k), -LOGIC_OUTPUT_RESISTANCE);
                        let level = if logic.levels[j] { LOGIC_HIGH } else { 0.0 };
                        sys.rhs(Some(k), level);
                    }
                }
                Device::Potentiometer {
                    resistance,
                    position,
                } => {
                    let (g_aw, g_wb) = pot_conductances(*resistance, *position);
                    sys.conductance(t[0], t[2], g_aw);
                    sys.conductance(t[2], t[1], g_wb);
                }
                Device::Spdt { to_b } => {
                    let (ga, gb) = spdt_conductances(*to_b);
                    sys.conductance(t[0], t[1], ga);
                    sys.conductance(t[0], t[2], gb);
                }
                Device::Timer(timer) => {
                    let k = self.branches[e][0];
                    stamp_timer_linear(&mut sys, t, k, timer.q, 1.0);
                    sys.rhs(Some(k), if timer.q { timer.level } else { 0.0 });
                }
            }
        }

        for i in 0..self.node_unknowns {
            sys.a[i * self.size + i] += GMIN + extra_gmin;
        }
        (sys, limited)
    }

    fn solve_system(
        &mut self,
        sys: System<f64>,
        mode: Mode,
        cacheable: bool,
    ) -> Result<Vec<f64>, CircuitError> {
        let System { a, mut b, .. } = sys;
        if cacheable {
            match &self.lu_cache {
                Some((m, lu)) if *m == mode => lu.solve(&mut b),
                _ => {
                    let lu = Lu::factor(a, self.size)?;
                    lu.solve(&mut b);
                    self.lu_cache = Some((mode, lu));
                }
            }
        } else {
            Lu::factor(a, self.size)?.solve(&mut b);
        }
        let sane = b.iter().all(|v| v.is_finite() && v.abs() < SANITY_LIMIT);
        if sane {
            Ok(b)
        } else {
            Err(CircuitError::Singular)
        }
    }

    fn newton(&mut self, mode: Mode, extra_gmin: f64) -> Result<Vec<f64>, CircuitError> {
        let mut x = self.x.clone();
        self.junction.clone_from(&self.accepted_junction);
        let cacheable = self.linear && extra_gmin == 0.0;
        for _ in 0..MAX_NEWTON_ITERATIONS {
            let (sys, limited) = self.assemble(&x, mode, extra_gmin);
            let x_new = self.solve_system(sys, mode, cacheable)?;
            if self.linear {
                return Ok(x_new);
            }
            let converged = !limited
                && x.iter().zip(&x_new).enumerate().all(|(i, (old, new))| {
                    let abs = if i < self.node_unknowns {
                        VNTOL
                    } else {
                        ABSTOL
                    };
                    (new - old).abs() <= RELTOL * new.abs().max(old.abs()) + abs
                });
            x = x_new;
            if converged {
                return Ok(x);
            }
        }
        Err(CircuitError::NoConvergence {
            time: self.eval_time,
        })
    }

    /// Yakınsamayan devrelerde iletkenliği kademeli azaltarak çözüme yaklaşır.
    fn gmin_stepping(&mut self) -> Result<Vec<f64>, CircuitError> {
        let mut gmin = 1e-2;
        while gmin > 1e-13 {
            self.x = self.newton(Mode::Dc, gmin)?;
            self.accepted_junction.clone_from(&self.junction);
            gmin /= 10.0;
        }
        self.newton(Mode::Dc, 0.0)
    }

    fn solve_dc(&mut self) -> Result<(), CircuitError> {
        let x = match self.newton(Mode::Dc, 0.0) {
            Ok(x) => x,
            Err(CircuitError::NoConvergence { .. }) => self.gmin_stepping()?,
            Err(e) => return Err(e),
        };
        self.x = x;
        self.accept(Mode::Dc);
        Ok(())
    }

    /// Lojik bileşenlerin çıkışlarını güncel çözüme göre günceller.
    fn update_logic(&mut self, time: f64) -> bool {
        self.update_logic_with(time, false)
    }

    fn update_logic_with(&mut self, time: f64, prime: bool) -> bool {
        let mut changed = false;
        for e in 0..self.devices.len() {
            if let Device::Timer(timer) = &mut self.devices[e] {
                let (x, t) = (&self.x, &self.terminals[e]);
                changed |= timer.update(|pin| voltage(x, t[pin]));
                continue;
            }
            let Device::Logic(logic) = &mut self.devices[e] else {
                continue;
            };
            let inputs: Vec<bool> = logic
                .inputs
                .iter()
                .map(|&pin| voltage(&self.x, self.terminals[e][pin]) > LOGIC_THRESHOLD)
                .collect();
            changed |= if prime {
                logic.prime(&inputs, time)
            } else {
                logic.update(&inputs, time)
            };
        }
        changed
    }

    /// Simülasyon başlamadan önce lojik çıkışların tutarlı bir başlangıç
    /// durumuna oturmasını sağlar. Flip-flop'lar bu sırada saat kenarı görmez;
    /// böylece açılışta sahte tetiklenme olmaz. Bulunan çözüm ilk adımın
    /// başlangıç tahmini olur; kondansatör/bobin durumları değişmez.
    fn prime_logic(&mut self) -> Result<(), CircuitError> {
        if self.logic_primed {
            return Ok(());
        }
        self.logic_primed = true;
        if !self
            .devices
            .iter()
            .any(|d| matches!(d, Device::Logic(_) | Device::Timer(_)))
        {
            return Ok(());
        }
        self.eval_time = self.time;
        self.update_logic_with(self.time, true);
        for _ in 0..MAX_LOGIC_ITERATIONS {
            self.x = self.newton(Mode::Dc, 0.0)?;
            self.accepted_junction.clone_from(&self.junction);
            if !self.update_logic_with(self.time, true) {
                break;
            }
        }
        self.lu_cache = None;
        Ok(())
    }

    /// DC çalışma noktasını hesaplar (kondansatörler açık devre, bobinler kısa
    /// devre). Sonuç geçici durum analizinin başlangıç koşulu olur.
    pub fn dc_operating_point(&mut self) -> Result<Frame, CircuitError> {
        self.prime_logic()?;
        self.eval_time = self.time;
        self.update_logic(self.time);
        for _ in 0..MAX_LOGIC_ITERATIONS {
            self.solve_dc()?;
            if !self.update_logic(self.time) {
                break;
            }
        }
        self.next_method = Method::BackwardEuler;
        Ok(self.frame())
    }

    /// Geçici durum analizini `h` saniye ilerletir.
    pub fn step(&mut self, h: f64) -> Result<(), CircuitError> {
        self.prime_logic()?;
        self.eval_time = self.time + h;
        let logic_changed = self.update_logic(self.eval_time);
        let method = if logic_changed {
            Method::BackwardEuler
        } else {
            self.next_method
        };
        let x = match self.newton(Mode::Transient { h, method }, 0.0) {
            Ok(x) => x,
            Err(CircuitError::NoConvergence { .. }) if method == Method::Trapezoidal => self
                .newton(
                    Mode::Transient {
                        h,
                        method: Method::BackwardEuler,
                    },
                    0.0,
                )?,
            Err(e) => return Err(e),
        };
        self.x = x;
        self.accept(Mode::Transient { h, method });
        self.time += h;
        self.next_method = Method::Trapezoidal;
        Ok(())
    }

    /// Kabul edilen çözüme göre elemanların geçmiş durumunu günceller.
    fn accept(&mut self, mode: Mode) {
        for e in 0..self.devices.len() {
            let t = &self.terminals[e];
            let v = |i: usize| voltage(&self.x, t[i]);
            match self.devices[e] {
                Device::Capacitor { capacitance } => {
                    let vc = v(0) - v(1);
                    let prev = self.states[e];
                    let i = match mode {
                        Mode::Dc => 0.0,
                        Mode::Transient {
                            h,
                            method: Method::BackwardEuler,
                        } => capacitance / h * (vc - prev.v),
                        Mode::Transient {
                            h,
                            method: Method::Trapezoidal,
                        } => 2.0 * capacitance / h * (vc - prev.v) - prev.i,
                    };
                    self.states[e] = State { v: vc, i };
                }
                Device::Inductor { .. } => {
                    self.states[e] = State {
                        v: v(0) - v(1),
                        i: self.x[self.branches[e][0]],
                    };
                }
                Device::Diode { .. } => self.accepted_junction[e] = [v(0) - v(1), 0.0],
                Device::Bjt { .. } => self.accepted_junction[e] = [v(1) - v(2), v(1) - v(0)],
                Device::Mosfet { .. } => self.accepted_junction[e] = [v(1) - v(2), v(0) - v(2)],
                _ => {}
            }
        }
    }

    // ------------------------------------------------------------ Sonuçlar

    /// `e` elemanının her pininden giren akım.
    fn pin_currents(&self, e: usize) -> Vec<f64> {
        let t = &self.terminals[e];
        let v = |i: usize| voltage(&self.x, t[i]);
        let two = |i: f64| vec![i, -i];
        match &self.devices[e] {
            Device::Resistor { resistance } => two((v(0) - v(1)) / resistance),
            Device::Switch { closed } => {
                two((v(0) - v(1)) / if *closed { SWITCH_R_ON } else { SWITCH_R_OFF })
            }
            Device::Capacitor { .. } => two(self.states[e].i),
            Device::Inductor { .. } | Device::VoltageSource { .. } | Device::Ammeter => {
                two(self.x[self.branches[e][0]])
            }
            Device::CurrentSource { current } => two(-current),
            Device::Diode { model } => {
                let vd = v(0) - v(1);
                two(model.evaluate(vd).0 + GMIN * vd)
            }
            Device::Voltmeter => vec![0.0, 0.0],
            Device::Bjt { model } => {
                let ev = model.evaluate(v(1) - v(2), v(1) - v(0));
                vec![ev.ic, ev.ib, -(ev.ic + ev.ib)]
            }
            Device::Mosfet { model } => {
                let id = model.evaluate(v(1) - v(2), v(0) - v(2)).id;
                vec![id, 0.0, -id]
            }
            Device::OpAmp { .. } => vec![0.0, 0.0, self.x[self.branches[e][0]]],
            Device::Logic(logic) => {
                let mut currents = vec![0.0; t.len()];
                for (j, &pin) in logic.outputs.iter().enumerate() {
                    currents[pin] = self.x[self.branches[e][j]];
                }
                currents
            }
            Device::Potentiometer {
                resistance,
                position,
            } => {
                let (g_aw, g_wb) = pot_conductances(*resistance, *position);
                let i_aw = g_aw * (v(0) - v(2));
                let i_wb = g_wb * (v(2) - v(1));
                vec![i_aw, -i_wb, i_wb - i_aw]
            }
            Device::Spdt { to_b } => {
                let (ga, gb) = spdt_conductances(*to_b);
                let ia = ga * (v(0) - v(1));
                let ib = gb * (v(0) - v(2));
                vec![ia + ib, -ia, -ib]
            }
            Device::Timer(timer) => {
                let mut c = vec![0.0; 8];
                let mut path = |a: usize, b: usize, i: f64| {
                    c[a] += i;
                    c[b] -= i;
                };
                let divider = DIVIDER_RESISTANCE;
                path(tp::VCC, tp::CTRL, (v(tp::VCC) - v(tp::CTRL)) / divider);
                path(
                    tp::CTRL,
                    tp::GND,
                    (v(tp::CTRL) - v(tp::GND)) / (2.0 * divider),
                );
                path(
                    tp::RESET,
                    tp::VCC,
                    (v(tp::RESET) - v(tp::VCC)) / RESET_PULLUP,
                );
                path(tp::OUT, tp::GND, self.x[self.branches[e][0]]);
                let g_dis = discharge_conductance(timer.q);
                path(tp::DIS, tp::GND, g_dis * (v(tp::DIS) - v(tp::GND)));
                c
            }
        }
    }

    /// Güncel sonuçlar.
    pub fn frame(&self) -> Frame {
        let mut node_voltages = vec![0.0; self.netlist.node_count];
        node_voltages[1..].copy_from_slice(&self.x[..self.node_unknowns]);
        let pin_currents: Vec<Vec<f64>> = (0..self.devices.len())
            .map(|e| self.pin_currents(e))
            .collect();
        let segment_currents = self.netlist.segment_currents(&pin_currents);
        Frame {
            time: self.time,
            node_voltages,
            pin_currents,
            segment_currents,
        }
    }

    /// Ölçüm aletlerinin güncel okumaları (`Topology::probes` sırası).
    pub fn probe_values(&self) -> Vec<f64> {
        self.probes
            .iter()
            .map(|&(e, kind)| {
                let t = &self.terminals[e];
                match kind {
                    ProbeKind::Current => self.x[self.branches[e][0]],
                    ProbeKind::Voltage => {
                        let v1 = t.get(1).map_or(0.0, |&n| voltage(&self.x, n));
                        voltage(&self.x, t[0]) - v1
                    }
                }
            })
            .collect()
    }

    // ------------------------------------------------------------ AC analizi

    /// Küçük sinyal frekans analizi. Devre DC çalışma noktasında doğrusallaştırılır;
    /// uyarı, ilk AC gerilim kaynağının genliği ve fazıdır.
    pub fn ac_analysis(&mut self, settings: &AcSettings) -> Result<AcResult, CircuitError> {
        if self.probes.is_empty() {
            return Err(CircuitError::NoProbes);
        }
        let (reference, stimulus) = self
            .devices
            .iter()
            .enumerate()
            .find_map(|(e, d)| match d {
                Device::VoltageSource {
                    waveform:
                        Waveform::Sine {
                            amplitude, phase, ..
                        },
                } if *amplitude != 0.0 => Some((e, Complex64::from_polar(*amplitude, *phase))),
                _ => None,
            })
            .ok_or(CircuitError::NoAcSource)?;
        let frequencies = log_space(settings)?;

        self.dc_bias_only = true;
        let op = self.dc_operating_point();
        self.dc_bias_only = false;
        op?;

        let mut values: Vec<Vec<Complex64>> = vec![Vec::new(); self.probes.len()];
        for &f in &frequencies {
            let x = self.solve_ac(std::f64::consts::TAU * f, reference)?;
            for (p, &(e, kind)) in self.probes.iter().enumerate() {
                let t = &self.terminals[e];
                let node = |n: Option<usize>| n.map_or(Complex64::default(), |i| x[i]);
                let value = match kind {
                    ProbeKind::Current => x[self.branches[e][0]],
                    ProbeKind::Voltage => {
                        node(t[0]) - t.get(1).map_or(Complex64::default(), |&n| node(n))
                    }
                };
                values[p].push(value / stimulus);
            }
        }

        let probes = self
            .probes
            .iter()
            .zip(values)
            .map(|(&(e, kind), h)| AcProbe {
                id: self.ids[e].clone(),
                kind,
                magnitude_db: h
                    .iter()
                    .map(|z| 20.0 * z.norm().max(1e-15).log10())
                    .collect(),
                phase_deg: unwrap_degrees(h.iter().map(|z| z.arg().to_degrees())),
            })
            .collect();
        Ok(AcResult {
            frequencies,
            reference: self.ids[reference].clone(),
            probes,
        })
    }

    fn solve_ac(&self, omega: f64, reference: usize) -> Result<Vec<Complex64>, CircuitError> {
        let mut sys: System<Complex64> = System::new(self.size);
        let one = Complex64::new(1.0, 0.0);
        let re = |v: f64| Complex64::new(v, 0.0);
        let x = &self.x;

        for e in 0..self.devices.len() {
            let t = &self.terminals[e];
            let v = |i: usize| voltage(x, t[i]);
            match &self.devices[e] {
                Device::Resistor { resistance } => {
                    sys.conductance(t[0], t[1], re(1.0 / resistance))
                }
                Device::Switch { closed } => {
                    let r = if *closed { SWITCH_R_ON } else { SWITCH_R_OFF };
                    sys.conductance(t[0], t[1], re(1.0 / r));
                }
                Device::Voltmeter | Device::CurrentSource { .. } => {}
                Device::Capacitor { capacitance } => {
                    sys.conductance(t[0], t[1], Complex64::new(0.0, omega * capacitance));
                }
                Device::Inductor { inductance } => {
                    let k = self.branches[e][0];
                    sys.branch(t[0], t[1], k, one);
                    sys.add(Some(k), Some(k), Complex64::new(0.0, -omega * inductance));
                }
                Device::VoltageSource { waveform } => {
                    let k = self.branches[e][0];
                    sys.branch(t[0], t[1], k, one);
                    if e == reference
                        && let Waveform::Sine {
                            amplitude, phase, ..
                        } = waveform
                    {
                        sys.rhs(Some(k), Complex64::from_polar(*amplitude, *phase));
                    }
                }
                Device::Ammeter => sys.branch(t[0], t[1], self.branches[e][0], one),
                Device::Diode { model } => {
                    let (_, gd) = model.evaluate(v(0) - v(1));
                    sys.conductance(t[0], t[1], re(gd + GMIN));
                }
                Device::Bjt { model } => {
                    let (c, b, em) = (t[0], t[1], t[2]);
                    let ev = model.evaluate(v(1) - v(2), v(1) - v(0));
                    let row_c = [
                        (b, re(ev.dic_dvbe + ev.dic_dvbc)),
                        (em, re(-ev.dic_dvbe)),
                        (c, re(-ev.dic_dvbc)),
                    ];
                    let row_b = [
                        (b, re(ev.dib_dvbe + ev.dib_dvbc)),
                        (em, re(-ev.dib_dvbe)),
                        (c, re(-ev.dib_dvbc)),
                    ];
                    let row_e: Vec<_> = row_c
                        .iter()
                        .zip(&row_b)
                        .map(|(&(n, gc), &(_, gb))| (n, -(gc + gb)))
                        .collect();
                    sys.terminal_current(c, &row_c, Complex64::default());
                    sys.terminal_current(b, &row_b, Complex64::default());
                    sys.terminal_current(em, &row_e, Complex64::default());
                    sys.conductance(b, em, re(GMIN));
                    sys.conductance(b, c, re(GMIN));
                }
                Device::Mosfet { model } => {
                    let (d, g, s) = (t[0], t[1], t[2]);
                    let ev = model.evaluate(v(1) - v(2), v(0) - v(2));
                    let row_d = [(g, re(ev.gm)), (d, re(ev.gds)), (s, re(-(ev.gm + ev.gds)))];
                    let row_s = [(g, re(-ev.gm)), (d, re(-ev.gds)), (s, re(ev.gm + ev.gds))];
                    sys.terminal_current(d, &row_d, Complex64::default());
                    sys.terminal_current(s, &row_s, Complex64::default());
                    sys.conductance(d, s, re(GMIN));
                }
                Device::OpAmp { gain, vsat } => {
                    let (minus, plus, out) = (t[0], t[1], t[2]);
                    let k = self.branches[e][0];
                    let (_, slope) = opamp_output(*gain, *vsat, v(1) - v(0));
                    sys.add(out, Some(k), one);
                    sys.add(Some(k), out, one);
                    sys.add(Some(k), plus, re(-slope));
                    sys.add(Some(k), minus, re(slope));
                }
                Device::Logic(logic) => {
                    for (j, &pin) in logic.outputs.iter().enumerate() {
                        let k = self.branches[e][j];
                        sys.branch(t[pin], None, k, one);
                        sys.add(Some(k), Some(k), re(-LOGIC_OUTPUT_RESISTANCE));
                    }
                }
                Device::Potentiometer {
                    resistance,
                    position,
                } => {
                    let (g_aw, g_wb) = pot_conductances(*resistance, *position);
                    sys.conductance(t[0], t[2], re(g_aw));
                    sys.conductance(t[2], t[1], re(g_wb));
                }
                Device::Spdt { to_b } => {
                    let (ga, gb) = spdt_conductances(*to_b);
                    sys.conductance(t[0], t[1], re(ga));
                    sys.conductance(t[0], t[2], re(gb));
                }
                Device::Timer(timer) => {
                    stamp_timer_linear(&mut sys, t, self.branches[e][0], timer.q, one);
                }
            }
        }
        for i in 0..self.node_unknowns {
            sys.a[i * self.size + i] += re(GMIN);
        }
        let System { a, mut b, .. } = sys;
        Lu::factor(a, self.size)?.solve(&mut b);
        if b.iter().all(|z| z.re.is_finite() && z.im.is_finite()) {
            Ok(b)
        } else {
            Err(CircuitError::Singular)
        }
    }

    // ------------------------------------------------------------ DC tarama

    /// Bir DC kaynağın değerini tarayarak ölçüm aletlerinin okumalarını verir.
    pub fn dc_sweep(&mut self, settings: &DcSweepSettings) -> Result<DcSweepResult, CircuitError> {
        if self.probes.is_empty() {
            return Err(CircuitError::NoProbes);
        }
        let invalid = || CircuitError::InvalidSweepSource {
            id: settings.source.clone(),
        };
        let e = self
            .ids
            .iter()
            .position(|id| *id == settings.source)
            .ok_or_else(invalid)?;
        let original = self.devices[e].clone();
        let voltage_source = match original {
            Device::VoltageSource {
                waveform: Waveform::Dc(_),
            } => true,
            Device::CurrentSource { .. } => false,
            _ => return Err(invalid()),
        };
        if settings.steps == 0
            || settings.steps > 10_000
            || !settings.start.is_finite()
            || !settings.stop.is_finite()
        {
            return Err(CircuitError::InvalidSettings);
        }

        let n = settings.steps as usize;
        let mut values = Vec::with_capacity(n + 1);
        let mut data: Vec<Vec<f64>> = vec![Vec::with_capacity(n + 1); self.probes.len()];
        self.dc_bias_only = true;
        let mut result = Ok(());
        for i in 0..=n {
            let value = settings.start + (settings.stop - settings.start) * i as f64 / n as f64;
            self.devices[e] = if voltage_source {
                Device::VoltageSource {
                    waveform: Waveform::Dc(value),
                }
            } else {
                Device::CurrentSource { current: value }
            };
            self.lu_cache = None;
            if let Err(err) = self.dc_operating_point() {
                result = Err(err);
                break;
            }
            values.push(value);
            for (p, v) in self.probe_values().into_iter().enumerate() {
                data[p].push(v);
            }
        }
        self.dc_bias_only = false;
        self.devices[e] = original;
        self.lu_cache = None;
        result?;

        Ok(DcSweepResult {
            source: settings.source.clone(),
            voltage: voltage_source,
            values,
            probes: self
                .probes
                .iter()
                .zip(data)
                .map(|(&(e, kind), data)| SweepProbe {
                    id: self.ids[e].clone(),
                    kind,
                    data,
                })
                .collect(),
        })
    }
}

fn pot_conductances(resistance: f64, position: f64) -> (f64, f64) {
    let r_aw = (resistance * position).max(POT_MIN_RESISTANCE);
    let r_wb = (resistance * (1.0 - position)).max(POT_MIN_RESISTANCE);
    (1.0 / r_aw, 1.0 / r_wb)
}

fn spdt_conductances(to_b: bool) -> (f64, f64) {
    let (on, off) = (1.0 / SWITCH_R_ON, 1.0 / SWITCH_R_OFF);
    if to_b { (off, on) } else { (on, off) }
}

fn discharge_conductance(q: bool) -> f64 {
    if q { GMIN } else { 1.0 / DISCHARGE_RESISTANCE }
}

/// 555'in doğrusal kısmı: iç bölücü, RESET çekme direnci, çıkış dalı ve
/// deşarj transistörü. Çıkış seviyesi (sağ taraf) çağıran tarafından eklenir.
fn stamp_timer_linear<T>(sys: &mut System<T>, t: &[Option<usize>], k: usize, q: bool, one: T)
where
    T: Copy
        + Default
        + std::ops::AddAssign
        + std::ops::Neg<Output = T>
        + std::ops::Mul<f64, Output = T>,
{
    sys.conductance(t[tp::VCC], t[tp::CTRL], one * (1.0 / DIVIDER_RESISTANCE));
    sys.conductance(t[tp::CTRL], t[tp::GND], one * (0.5 / DIVIDER_RESISTANCE));
    sys.conductance(t[tp::RESET], t[tp::VCC], one * (1.0 / RESET_PULLUP));
    sys.branch(t[tp::OUT], t[tp::GND], k, one);
    sys.add(Some(k), Some(k), -(one * OUTPUT_RESISTANCE));
    sys.conductance(t[tp::DIS], t[tp::GND], one * discharge_conductance(q));
}

/// Logaritmik aralıklı frekanslar.
fn log_space(s: &AcSettings) -> Result<Vec<f64>, CircuitError> {
    if !(s.start > 0.0 && s.stop > s.start && s.stop.is_finite() && s.points_per_decade > 0) {
        return Err(CircuitError::InvalidSettings);
    }
    let decades = (s.stop / s.start).log10();
    let count = ((decades * f64::from(s.points_per_decade)).ceil() as usize + 1).min(MAX_AC_POINTS);
    Ok((0..count)
        .map(|i| {
            if i + 1 == count {
                s.stop
            } else {
                s.start * 10f64.powf(decades * i as f64 / (count - 1) as f64)
            }
        })
        .collect())
}

/// Faz atlamalarını (±360°) gidererek sürekli faz eğrisi üretir.
fn unwrap_degrees(phases: impl Iterator<Item = f64>) -> Vec<f64> {
    let mut out: Vec<f64> = Vec::new();
    let mut offset = 0.0;
    for p in phases {
        if let Some(&last) = out.last() {
            let mut value = p + offset;
            while value - last > 180.0 {
                offset -= 360.0;
                value -= 360.0;
            }
            while value - last < -180.0 {
                offset += 360.0;
                value += 360.0;
            }
            out.push(value);
        } else {
            out.push(p);
        }
    }
    out
}
