//! Nullbox domain: pure types and rules. No IO, no async, no database.
//!
//! Everything here is safe to reuse anywhere (API, tests, a future Tauri app).
//! User-facing words never live here: the frontend owns all copy.

pub mod anon;
pub mod beast;
pub mod event;
pub mod status;
pub mod ticket;

pub use anon::{ANON_ANIMALS, AnonIdentity};
pub use beast::BeastMood;
pub use event::{DomainEvent, SystemEvent};
pub use status::{InvalidTransition, TicketStatus};
pub use ticket::{AuthorKind, Message, Severity, Ticket};
