//! Where the client keeps its long-lived secrets.
//!
//! Each server the person is signed into has its own refresh token. Those
//! tokens are the only credentials on this machine worth stealing, so they
//! live in the OS keyring (Keychain, Credential Manager, Secret Service) and
//! never touch a file of ours.
//!
//! They share one keyring account: a small JSON list. Signing out of one
//! server rewrites the list without changing the other tokens. A single blob
//! is one write, so two saves cannot interleave into a torn list.
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

/// What a cold start needs to get back to a signed-in state on one server:
/// which server, and the refresh token to trade for a fresh access token.
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

/// The list as it sits in the keyring. `active` is the server the window was
/// last looking at, so a restart lands in the same place.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredSessionList {
    pub sessions: Vec<StoredSession>,
    pub active: Option<String>,
}

/// Result of reading the stored sessions. `Unavailable` is the no-wallet path
/// and is a normal, expected answer — not an error.
#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SessionLoad {
    Found {
        sessions: Vec<StoredSession>,
        active: Option<String>,
    },
    Empty,
    Unavailable {
        reason: String,
    },
}

/// Result of writing or clearing stored sessions.
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
    match read_list(ACCOUNT) {
        ListRead::Unavailable(reason) => SessionLoad::Unavailable { reason },
        ListRead::Missing => SessionLoad::Empty,
        ListRead::List(list) if list.sessions.is_empty() => SessionLoad::Empty,
        ListRead::List(list) => SessionLoad::Found {
            sessions: list.sessions,
            active: list.active,
        },
    }
}

/// Insert or replace one server's tokens, and make it the active server.
pub fn save(session: &StoredSession) -> SessionWrite {
    let mut list = match read_list(ACCOUNT) {
        ListRead::Unavailable(reason) => return SessionWrite::Unavailable { reason },
        ListRead::Missing => StoredSessionList {
            sessions: Vec::new(),
            active: None,
        },
        ListRead::List(list) => list,
    };
    upsert(&mut list, session);
    list.active = Some(session.base_url.clone());
    write_list(ACCOUNT, &list)
}

/// Forget one server, or all of them when `base_url` is `None`.
///
/// Signing out of one must not drop the others: we rewrite the list with that
/// one row gone, and leave every remaining token as it was.
pub fn clear(base_url: Option<&str>) -> SessionWrite {
    let Some(url) = base_url else {
        return clear_from(ACCOUNT);
    };
    let mut list = match read_list(ACCOUNT) {
        ListRead::Unavailable(reason) => return SessionWrite::Unavailable { reason },
        ListRead::Missing => return SessionWrite::Done,
        ListRead::List(list) => list,
    };
    list.sessions.retain(|held| held.base_url != url);
    if list.active.as_deref() == Some(url) {
        list.active = list.sessions.first().map(|held| held.base_url.clone());
    }
    if list.sessions.is_empty() {
        return clear_from(ACCOUNT);
    }
    write_list(ACCOUNT, &list)
}

/// Remember which server the window is looking at, without touching tokens.
pub fn set_active(base_url: &str) -> SessionWrite {
    let mut list = match read_list(ACCOUNT) {
        ListRead::Unavailable(reason) => return SessionWrite::Unavailable { reason },
        ListRead::Missing => return SessionWrite::Done,
        ListRead::List(list) => list,
    };
    if !list.sessions.iter().any(|held| held.base_url == base_url) {
        return SessionWrite::Done;
    }
    list.active = Some(base_url.to_string());
    write_list(ACCOUNT, &list)
}

fn upsert(list: &mut StoredSessionList, session: &StoredSession) {
    if let Some(held) = list
        .sessions
        .iter_mut()
        .find(|held| held.base_url == session.base_url)
    {
        *held = session.clone();
        return;
    }
    list.sessions.push(session.clone());
}

#[derive(Debug)]
enum ListRead {
    List(StoredSessionList),
    Missing,
    Unavailable(String),
}

/// Read the blob and accept both the current list shape and the single-session
/// shape from before T-412, so a restart after this build does not demand a
/// re-login.
fn parse_blob(json: &str) -> Option<StoredSessionList> {
    if let Ok(list) = serde_json::from_str::<StoredSessionList>(json) {
        return Some(list);
    }
    let one = serde_json::from_str::<StoredSession>(json).ok()?;
    Some(StoredSessionList {
        active: Some(one.base_url.clone()),
        sessions: vec![one],
    })
}

fn read_list(account: &str) -> ListRead {
    let entry = match entry(account) {
        Ok(entry) => entry,
        Err(err) => return ListRead::Unavailable(explain(&err)),
    };
    match entry.get_password() {
        Ok(json) => match parse_blob(&json) {
            Some(list) => ListRead::List(list),
            // A blob we can't parse is a leftover from an older build. Treat
            // it as "no session" so a format change costs a re-login, not a
            // wedge.
            None => ListRead::Missing,
        },
        Err(keyring::Error::NoEntry) => ListRead::Missing,
        Err(err) => ListRead::Unavailable(explain(&err)),
    }
}

fn write_list(account: &str, list: &StoredSessionList) -> SessionWrite {
    let json = match serde_json::to_string(list) {
        Ok(json) => json,
        Err(err) => {
            return SessionWrite::Unavailable {
                reason: format!("Couldn't prepare the session for storage ({err})."),
            }
        }
    };
    write_json(account, &json)
}

fn write_json(account: &str, json: &str) -> SessionWrite {
    let entry = match entry(account) {
        Ok(entry) => entry,
        Err(err) => {
            return SessionWrite::Unavailable {
                reason: explain(&err),
            }
        }
    };
    match entry.set_password(json) {
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

    #[test]
    fn list_round_trips_as_json() {
        let list = StoredSessionList {
            sessions: vec![
                StoredSession {
                    base_url: "https://home.example".into(),
                    refresh_token: "aaa".into(),
                },
                StoredSession {
                    base_url: "https://work.example".into(),
                    refresh_token: "bbb".into(),
                },
            ],
            active: Some("https://work.example".into()),
        };
        let json = serde_json::to_string(&list).expect("serialize");
        let back: StoredSessionList = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(list, back);
    }

    #[test]
    fn old_single_session_blob_becomes_a_one_item_list() {
        let old = StoredSession {
            base_url: "https://linger.example".into(),
            refresh_token: "abc".into(),
        };
        let json = serde_json::to_string(&old).expect("serialize");
        let list = parse_blob(&json).expect("migrate");
        assert_eq!(list.sessions, vec![old.clone()]);
        assert_eq!(list.active.as_deref(), Some("https://linger.example"));
    }

    #[test]
    fn upsert_replaces_the_matching_server_and_leaves_the_other() {
        let mut list = StoredSessionList {
            sessions: vec![
                StoredSession {
                    base_url: "https://home.example".into(),
                    refresh_token: "old-home".into(),
                },
                StoredSession {
                    base_url: "https://work.example".into(),
                    refresh_token: "work".into(),
                },
            ],
            active: Some("https://home.example".into()),
        };
        upsert(
            &mut list,
            &StoredSession {
                base_url: "https://home.example".into(),
                refresh_token: "new-home".into(),
            },
        );
        assert_eq!(list.sessions.len(), 2);
        assert_eq!(list.sessions[0].refresh_token, "new-home");
        assert_eq!(list.sessions[1].refresh_token, "work");
    }

    /// The real thing, against this machine's actual keyring. Ignored by
    /// default because it needs an unlocked wallet, which CI does not have —
    /// run it with `cargo test -- --ignored` on a desktop session. It writes to
    /// a separate account so it can never clobber a real sign-in.
    #[test]
    #[ignore = "needs an unlocked OS keyring"]
    fn round_trips_through_the_real_keyring() {
        const TEST_ACCOUNT: &str = "session-selftest";
        let home = StoredSession {
            base_url: "https://home.test".into(),
            refresh_token: "not-a-real-token".into(),
        };
        let work = StoredSession {
            base_url: "https://work.test".into(),
            refresh_token: "also-not-real".into(),
        };

        let list = StoredSessionList {
            sessions: vec![home.clone(), work.clone()],
            active: Some(home.base_url.clone()),
        };
        assert!(matches!(write_list(TEST_ACCOUNT, &list), SessionWrite::Done));
        match read_list(TEST_ACCOUNT) {
            ListRead::List(got) => assert_eq!(got, list),
            other => panic!("expected the saved list back, got {other:?}"),
        }
        assert!(matches!(clear_from(TEST_ACCOUNT), SessionWrite::Done));
        assert!(matches!(read_list(TEST_ACCOUNT), ListRead::Missing));
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
        let found = serde_json::to_string(&SessionLoad::Found {
            sessions: vec![StoredSession {
                base_url: "https://x".into(),
                refresh_token: "t".into(),
            }],
            active: Some("https://x".into()),
        })
        .expect("serialize");
        assert!(found.contains(r#""kind":"found""#));
        assert!(found.contains(r#""sessions""#));
        assert!(found.contains(r#""active":"https://x""#));
    }
}
