//! Where the client keeps its one long-lived secret.
//!
//! The refresh token is the only credential on this machine worth stealing, so
//! it lives in the OS keyring (Keychain, Credential Manager, Secret Service)
//! and never touches a file of ours.
//!
//! ARCHITECTURE §7.3 requires the no-wallet case to be handled explicitly: a
//! headless Linux box, or a session where KWallet/gnome-keyring is locked or
//! absent, must degrade to a clear "sign in again" prompt instead of a crash.
//! That is why nothing here returns `Err` — every outcome, including "this
//! computer has no keyring", is a value the frontend can render.

use keyring::Entry;
use serde::{Deserialize, Serialize};

/// Keyring service name. Matches the bundle identifier so the entry is
/// recognisable in Seahorse / KWalletManager / Keychain Access.
const SERVICE: &str = "com.linger.desktop";
const ACCOUNT: &str = "session";

/// What a cold start needs to get back to a signed-in state: which server, and
/// the refresh token to trade for a fresh access token.
///
/// The access token is deliberately not stored. It expires in 15 minutes, so
/// keeping it would buy almost nothing and would widen what a stolen keyring
/// entry is worth.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredSession {
    /// Origin of the server, e.g. `https://linger.example`. No trailing slash.
    pub base_url: String,
    pub refresh_token: String,
}

/// Result of reading the stored session. `Unavailable` is the no-wallet path
/// and is a normal, expected answer — not an error.
#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SessionLoad {
    Found { session: StoredSession },
    Empty,
    Unavailable { reason: String },
}

/// Result of writing or clearing the stored session.
#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SessionWrite {
    Done,
    Unavailable { reason: String },
}

/// Turn a keyring failure into something a person can act on. The underlying
/// messages are D-Bus and platform jargon, so they go in parentheses after a
/// plain sentence rather than being shown raw.
fn explain(err: &keyring::Error) -> String {
    format!("No usable keyring on this computer ({err}).")
}

fn entry(account: &str) -> Result<Entry, keyring::Error> {
    Entry::new(SERVICE, account)
}

pub fn load() -> SessionLoad {
    load_from(ACCOUNT)
}

pub fn save(session: &StoredSession) -> SessionWrite {
    save_to(ACCOUNT, session)
}

pub fn clear() -> SessionWrite {
    clear_from(ACCOUNT)
}

fn load_from(account: &str) -> SessionLoad {
    let entry = match entry(account) {
        Ok(entry) => entry,
        Err(err) => {
            return SessionLoad::Unavailable {
                reason: explain(&err),
            }
        }
    };
    match entry.get_password() {
        Ok(json) => match serde_json::from_str::<StoredSession>(&json) {
            Ok(session) => SessionLoad::Found { session },
            // A blob we can't parse is a leftover from an older build. Treat it
            // as "no session" so a format change costs a re-login, not a wedge.
            Err(_) => SessionLoad::Empty,
        },
        Err(keyring::Error::NoEntry) => SessionLoad::Empty,
        Err(err) => SessionLoad::Unavailable {
            reason: explain(&err),
        },
    }
}

fn save_to(account: &str, session: &StoredSession) -> SessionWrite {
    let entry = match entry(account) {
        Ok(entry) => entry,
        Err(err) => {
            return SessionWrite::Unavailable {
                reason: explain(&err),
            }
        }
    };
    let json = match serde_json::to_string(session) {
        Ok(json) => json,
        Err(err) => {
            return SessionWrite::Unavailable {
                reason: format!("Couldn't prepare the session for storage ({err})."),
            }
        }
    };
    match entry.set_password(&json) {
        Ok(()) => SessionWrite::Done,
        Err(err) => SessionWrite::Unavailable {
            reason: explain(&err),
        },
    }
}

fn clear_from(account: &str) -> SessionWrite {
    let entry = match entry(account) {
        Ok(entry) => entry,
        Err(err) => {
            return SessionWrite::Unavailable {
                reason: explain(&err),
            }
        }
    };
    match entry.delete_credential() {
        // Deleting nothing is a success: the caller wanted no stored session
        // and there is none.
        Ok(()) | Err(keyring::Error::NoEntry) => SessionWrite::Done,
        Err(err) => SessionWrite::Unavailable {
            reason: explain(&err),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The contract that matters on a machine with no wallet: reading answers
    /// with a variant instead of panicking or hanging. This test passes both on
    /// a desktop with a keyring and on a bare CI box without one — which is the
    /// whole point.
    #[test]
    fn load_answers_without_panicking() {
        match load() {
            SessionLoad::Found { .. } | SessionLoad::Empty => {}
            SessionLoad::Unavailable { reason } => assert!(!reason.is_empty()),
        }
    }

    #[test]
    fn stored_session_round_trips_as_json() {
        let session = StoredSession {
            base_url: "https://linger.example".into(),
            refresh_token: "abc".into(),
        };
        let json = serde_json::to_string(&session).expect("serialize");
        let back: StoredSession = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(session, back);
    }

    /// The real thing, against this machine's actual keyring. Ignored by
    /// default because it needs an unlocked wallet, which CI does not have —
    /// run it with `cargo test -- --ignored` on a desktop session. It writes to
    /// a separate account so it can never clobber a real sign-in.
    #[test]
    #[ignore = "needs an unlocked OS keyring"]
    fn round_trips_through_the_real_keyring() {
        const TEST_ACCOUNT: &str = "session-selftest";
        let session = StoredSession {
            base_url: "https://linger.test".into(),
            refresh_token: "not-a-real-token".into(),
        };

        assert!(matches!(
            save_to(TEST_ACCOUNT, &session),
            SessionWrite::Done
        ));
        match load_from(TEST_ACCOUNT) {
            SessionLoad::Found { session: got } => assert_eq!(got, session),
            other => panic!("expected the saved session back, got {other:?}"),
        }
        assert!(matches!(clear_from(TEST_ACCOUNT), SessionWrite::Done));
        assert!(matches!(load_from(TEST_ACCOUNT), SessionLoad::Empty));
        // Clearing twice is still success — sign-out must be idempotent.
        assert!(matches!(clear_from(TEST_ACCOUNT), SessionWrite::Done));
    }

    /// The frontend switches on `kind`, so the tag spelling is part of the
    /// contract with `client/src/lib/ipc.ts`.
    #[test]
    fn outcomes_are_tagged_in_snake_case() {
        let empty = serde_json::to_string(&SessionLoad::Empty).expect("serialize");
        assert_eq!(empty, r#"{"kind":"empty"}"#);
        let done = serde_json::to_string(&SessionWrite::Done).expect("serialize");
        assert_eq!(done, r#"{"kind":"done"}"#);
        let missing = serde_json::to_string(&SessionLoad::Unavailable {
            reason: "no wallet".into(),
        })
        .expect("serialize");
        assert_eq!(missing, r#"{"kind":"unavailable","reason":"no wallet"}"#);
    }
}
