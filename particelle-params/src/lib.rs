//! `particelle-params` — Parameter signal graph, registry, and unit system.
//!
//! Every engine parameter is a `ParamSignal`. There are no raw f64 parameters
//! in the engine API; all values flow through this graph.

pub mod context;
pub mod registry;
pub mod signal;
pub mod unit;

pub use context::{FieldProvider, SignalContext};
pub use registry::{Domain, ParamDescriptor, ParamRegistry};
pub use signal::{MapFunc, ParamSignal};
pub use unit::Unit;
