//! DD-68: on-demand read-only goal sources; no resident MCP daemon.
mod config;
mod http;
mod imports;
mod jsonc;
mod routing;
mod schema;
mod sources;
mod transport;
pub use config::{Binding, Server, Settings};
pub use imports::{
    Applied, Preview, Record, Reference, apply, context, discover, preview, references,
};
pub const MAX_RESPONSE: usize = 8 * 1024 * 1024;
pub const MAX_SOURCE: usize = 4 * 1024 * 1024;
pub const MAX_ARTIFACT: usize = 64 * 1024 * 1024;
pub use routing::{coordinator_profile, preview_with_coordinator};
pub use sources::Summary;
pub const MAX_CONTEXT: usize = 48 * 1024;
pub use imports::{load, store};
