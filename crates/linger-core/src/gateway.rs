//! Gateway frames (PROTOCOL §8). JSON over WSS at `/api/v1/gateway`.
//!
//! Frames are `{ op, d, s? }`. `s` is a monotonically increasing sequence number,
//! present on server→client frames only — it is what makes resume replay possible.
//! Clients must ignore unknown `op` values and unknown fields (PROTOCOL §9), which
//! is why `ServerEvent`/`ClientEvent` may gain variants within v1 but never lose any.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::id::{MessageId, RoomId, UserId};
use crate::wire::{Message, PresenceEntry, PresenceState, Room, User};

// ---------------------------------------------------------------------------
// Client → server
// ---------------------------------------------------------------------------

/// A client frame is just `{ op, d }` — no sequence number in this direction.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "op", content = "d", rename_all = "snake_case")]
#[ts(export)]
pub enum ClientFrame {
    Identify {
        token: String,
        /// e.g. "linger-desktop/0.1.0"
        client: String,
    },
    Resume {
        session_id: String,
        token: String,
        /// Last sequence number the client saw.
        #[ts(type = "number")]
        s: u64,
    },
    Heartbeat {
        /// Last sequence number seen, for the server's replay bookkeeping.
        #[ts(type = "number | null")]
        s: Option<u64>,
    },
    /// `activity` is a resolved registry id or `None`. **Never a window title.**
    #[serde(rename = "presence.update")]
    PresenceUpdate {
        state: PresenceState,
        activity: Option<String>,
        away_message: Option<String>,
    },
    /// Fired when the client focuses a room. `room_id: None` means the user
    /// left the room (unfocused, backgrounded, or idle).
    #[serde(rename = "room.focus")]
    RoomFocus { room_id: Option<RoomId> },
    #[serde(rename = "typing.start")]
    TypingStart { room_id: RoomId },
}

// ---------------------------------------------------------------------------
// Server → client
// ---------------------------------------------------------------------------

/// Payload of `ready`: everything a client needs to render without further fetches.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ReadyData {
    pub session_id: String,
    pub user: User,
    pub users: Vec<User>,
    pub rooms: Vec<Room>,
    pub presence: Vec<PresenceEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "op", content = "d", rename_all = "snake_case")]
#[ts(export)]
pub enum ServerEvent {
    Hello {
        #[ts(type = "number")]
        heartbeat_interval_ms: u64,
    },
    Ready(ReadyData),
    Resumed {
        /// Number of frames replayed after the resume.
        #[ts(type = "number")]
        replayed: u64,
    },
    InvalidSession {
        reason: String,
    },
    HeartbeatAck,
    #[serde(rename = "message.create")]
    MessageCreate(Message),
    #[serde(rename = "message.update")]
    MessageUpdate(Message),
    #[serde(rename = "message.delete")]
    MessageDelete {
        id: MessageId,
        room_id: RoomId,
    },
    /// `count` is present for accessibility labels; the client renders weight.
    #[serde(rename = "reaction.update")]
    ReactionUpdate {
        message_id: MessageId,
        key: String,
        count: u32,
        user_ids: Vec<UserId>,
    },
    #[serde(rename = "presence.update")]
    PresenceUpdate(PresenceEntry),
    #[serde(rename = "room.occupancy")]
    RoomOccupancy {
        room_id: RoomId,
        user_ids: Vec<UserId>,
    },
    /// Sent only to clients in that room; the receiver applies its own
    /// mute rules and quiet hours before playing anything (SPEC §4.1).
    #[serde(rename = "room.enter")]
    RoomEnter {
        room_id: RoomId,
        user_id: UserId,
        entrance_sound: Option<String>,
    },
    #[serde(rename = "room.leave")]
    RoomLeave {
        room_id: RoomId,
        user_id: UserId,
    },
    /// Display name, style, or status changed.
    #[serde(rename = "user.update")]
    UserUpdate(User),
    #[serde(rename = "room.create")]
    RoomCreate(Room),
    #[serde(rename = "room.update")]
    RoomUpdate(Room),
    Typing {
        room_id: RoomId,
        user_id: UserId,
    },
    /// V2 (SPEC §4.9); defined now because op values are additive within v1.
    Knock {
        from_user_id: UserId,
    },
}

/// A server frame: an event plus its sequence number. `hello`, `heartbeat_ack`,
/// and pre-`ready` traffic carry no `s`.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ServerFrame {
    #[serde(flatten)]
    #[ts(flatten)]
    pub event: ServerEvent,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional, type = "number")]
    pub s: Option<u64>,
}

impl ServerFrame {
    /// A frame that participates in resume replay (has a sequence number).
    #[must_use]
    pub fn sequenced(event: ServerEvent, s: u64) -> Self {
        Self { event, s: Some(s) }
    }

    /// A control frame outside the replay stream.
    #[must_use]
    pub fn control(event: ServerEvent) -> Self {
        Self { event, s: None }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_heartbeat_matches_protocol() {
        let f = ClientFrame::Heartbeat { s: Some(41) };
        let json = serde_json::to_value(&f).unwrap();
        assert_eq!(json["op"], "heartbeat");
        assert_eq!(json["d"]["s"], 41);
    }

    #[test]
    fn dotted_op_names_survive_round_trip() {
        let f = ClientFrame::RoomFocus { room_id: None };
        let json = serde_json::to_string(&f).unwrap();
        assert!(json.contains(r#""op":"room.focus""#));
        let back: ClientFrame = serde_json::from_str(&json).unwrap();
        assert!(matches!(back, ClientFrame::RoomFocus { room_id: None }));
    }

    #[test]
    fn server_frame_flattens_op_d_s() {
        let f = ServerFrame::sequenced(
            ServerEvent::RoomLeave {
                room_id: RoomId::new(),
                user_id: UserId::new(),
            },
            7,
        );
        let json = serde_json::to_value(&f).unwrap();
        assert_eq!(json["op"], "room.leave");
        assert_eq!(json["s"], 7);
        assert!(json["d"]["room_id"].is_string());

        let hello = ServerFrame::control(ServerEvent::Hello {
            heartbeat_interval_ms: 30_000,
        });
        let json = serde_json::to_value(&hello).unwrap();
        assert_eq!(json["op"], "hello");
        assert!(
            json.get("s").is_none(),
            "control frames must omit s entirely"
        );
    }

    #[test]
    fn server_frame_deserializes_from_wire_shape() {
        let raw = r#"{"op":"typing","d":{"room_id":"018f6f4a7b2c7d3e9f0a1b2c3d4e5f60","user_id":"018f6f4a7b2c7d3e9f0a1b2c3d4e5f61"},"s":12}"#;
        let f: ServerFrame = serde_json::from_str(raw).unwrap();
        assert_eq!(f.s, Some(12));
        assert!(matches!(f.event, ServerEvent::Typing { .. }));
    }
}
