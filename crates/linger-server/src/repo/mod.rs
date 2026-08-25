//! Row → wire-type assembly. Plain async functions over the pools; if Postgres
//! ever happens (ARCHITECTURE §2 says ~never), this module is the seam.

pub mod attachments;
pub mod messages;
pub mod rooms;
pub mod users;
