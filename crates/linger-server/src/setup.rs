//! First-run host setup (ARCHITECTURE §9, PROTOCOL §2.1). On boot with zero
//! users the server holds a one-time token and prints a setup URL to stdout.
//! The token dies on use or restart; no env-var bootstrap credentials.

use std::sync::Mutex;

use rand::RngCore;

pub struct SetupState {
    token: Mutex<Option<String>>,
}

impl SetupState {
    /// Armed with a fresh token when the server has no users yet.
    #[must_use]
    pub fn new(needs_setup: bool) -> Self {
        let token = needs_setup.then(|| {
            let mut bytes = [0u8; 16];
            rand::rngs::OsRng.fill_bytes(&mut bytes);
            hex::encode(bytes)
        });
        Self {
            token: Mutex::new(token),
        }
    }

    /// The current token, for printing the setup URL (and for tests).
    #[must_use]
    pub fn peek(&self) -> Option<String> {
        self.token.lock().ok().and_then(|t| t.clone())
    }

    #[must_use]
    pub fn matches(&self, candidate: &str) -> bool {
        self.peek().is_some_and(|t| t == candidate)
    }

    /// One-shot consume: true exactly once, for the matching token.
    #[must_use]
    pub fn consume(&self, candidate: &str) -> bool {
        let Ok(mut slot) = self.token.lock() else {
            return false;
        };
        if slot.as_deref() == Some(candidate) {
            *slot = None;
            true
        } else {
            false
        }
    }
}
