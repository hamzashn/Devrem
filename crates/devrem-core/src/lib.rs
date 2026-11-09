//! Devrem çekirdeği: devre modeli, netlist üretimi ve `.dvrm` dosya formatı.

pub mod builder;
pub mod document;
pub mod error;
pub mod geometry;
pub mod library;
pub mod netlist;
pub mod units;

pub use builder::DocBuilder;
pub use document::{
    AcSettings, Component, DcSweepSettings, Document, DocumentError, DocumentMeta, FORMAT_VERSION,
    SimulationSettings, Wire,
};
pub use error::CircuitError;
pub use geometry::Point;
pub use library::ComponentKind;
pub use netlist::{NetElement, Netlist, Segment};
pub use units::parse_value;
