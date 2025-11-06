// Public modules
pub mod bgp_state;
pub mod mrt_processor;

// Internal modules
mod announcement;
mod mrt_records;
mod util;

// Re-export commonly used types for convenience
pub use bgp_state::{BgpState, ConnectionState};
pub use mrt_processor::{MrtProcessor, BgpPeer};
