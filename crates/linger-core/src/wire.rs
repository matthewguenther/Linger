//! REST wire types (PROTOCOL §§1–7). Timestamps are Unix milliseconds (i64).
//!
//! Everything here derives `ts_rs::TS` and is exported to `client/src/generated/`.
//! Optionality convention: protocol fields declared `T | null` are `Option<T>`.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::id::{AttachmentId, MessageId, RoomId, UploadId, UserId};

// NOTE on 64-bit integers: ts-rs maps i64/u64 to `bigint`, but JSON.parse hands
// the frontend plain numbers. Every 64-bit value on this wire (Unix ms, byte
// sizes, sequence numbers) fits in the 2^53 safe-integer range, so such fields
// carry `#[ts(type = "number")]` (or `"number | null"`). Keep doing this for
// new fields — a stray `bigint` in the bindings is a defect.

// ---------------------------------------------------------------------------
// Errors (PROTOCOL §1)
// ---------------------------------------------------------------------------

/// Stable machine-readable error identifiers the client switches on.
/// Additive within v1: new codes may be appended, never removed or renamed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[ts(export)]
pub enum ErrorCode {
    Unauthenticated,
    Forbidden,
    NotFound,
    RateLimited,
    ValidationFailed,
    InviteInvalid,
    InviteExpired,
    QuotaExceeded,
    FileTooLarge,
    UnsupportedMedia,
    Conflict,
    Internal,
}

/// Body of every non-2xx response: `{ "error": { code, message, retry_after_ms } }`.
/// `message` is human-readable and safe to display — never leak internals into it.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ErrorEnvelope {
    pub error: ErrorBody,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ErrorBody {
    pub code: ErrorCode,
    pub message: String,
    #[ts(type = "number | null")]
    pub retry_after_ms: Option<u64>,
}

// ---------------------------------------------------------------------------
// Styling (SPEC §4.5, PROTOCOL §5)
// ---------------------------------------------------------------------------

/// One of the 16 named palette keys in `crate::PALETTE`. The wire carries the
/// *name* (`"azure"`), never a color value — contrast safety is structural.
/// The server validates membership; client-side validation alone is a defect.
// Note: serde serializes single-field tuple structs as the inner value, so this
// travels as a bare string ("azure") with no attribute needed.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ColorKey(pub String);

impl ColorKey {
    /// Membership check against the canonical palette.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        crate::palette::is_valid_color_key(&self.0)
    }
}

/// A name fill: one palette color, or a gradient of any two. The gradient angle
/// is fixed at 92° and is deliberately not on the wire (SPEC §4.5).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "lowercase")]
#[ts(export)]
pub enum Fill {
    Solid { color: ColorKey },
    Gradient { from: ColorKey, to: ColorKey },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum NameEffect {
    None,
    Shimmer,
    Glow,
}

/// How a user renders their own display name (and optionally their message font).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Style {
    /// Must be a member of `crate::FONTS`; server-validated.
    pub font_key: String,
    #[ts(type = "400 | 500 | 700")]
    pub weight: u16,
    pub italic: bool,
    pub fill: Fill,
    pub effect: NameEffect,
    /// Optional message-body font override from the same curated set. This is the
    /// *only* message styling that exists (SPEC §4.5) — no colors, no sizes.
    pub msg_font_key: Option<String>,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            font_key: "geist-sans".into(),
            weight: 500,
            italic: false,
            fill: Fill::Solid {
                color: ColorKey("slate".into()),
            },
            effect: NameEffect::None,
            msg_font_key: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Users and signs (SPEC §4.6, PROTOCOL §5)
// ---------------------------------------------------------------------------

/// A user's sign: a small card, not a bio field. The away message supersedes
/// `line` when set.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Sign {
    pub line: Option<String>,
    pub reading: Option<String>,
    pub listening: Option<String>,
    pub working_on: Option<String>,
    pub image_key: Option<String>,
    pub away_message: Option<String>,
    #[ts(type = "number | null")]
    pub away_since: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct User {
    pub id: UserId,
    pub username: String,
    pub display_name: String,
    pub is_host: bool,
    pub style: Style,
    pub sign: Option<Sign>,
    /// Bundled sound key, or object key for a custom upload.
    pub entrance_sound: Option<String>,
    #[ts(type = "number | null")]
    pub last_seen_at: Option<i64>,
}

// ---------------------------------------------------------------------------
// Auth (PROTOCOL §2)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RegisterRequest {
    pub invite_code: String,
    pub username: String,
    pub display_name: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    /// Access-token lifetime in seconds.
    #[ts(type = "number")]
    pub expires_in: u64,
    pub user: User,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RefreshResponse {
    pub access_token: String,
    pub refresh_token: String,
    #[ts(type = "number")]
    pub expires_in: u64,
}

/// First-run setup (PROTOCOL §2.1): creates the host account and names the stoop.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SetupRequest {
    pub token: String,
    pub stoop_name: String,
    pub username: String,
    pub display_name: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SetupPreview {
    pub valid: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

/// `PATCH /me`: absent fields unchanged; `style`/`sign` replace whole objects;
/// `entrance_sound: ""` clears the sound (PROTOCOL §5).
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct UpdateMeRequest {
    pub display_name: Option<String>,
    pub style: Option<Style>,
    pub sign: Option<Sign>,
    pub entrance_sound: Option<String>,
}

/// Unauthenticated invite preview (`GET /auth/invite/:code`).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct InvitePreview {
    pub valid: bool,
    pub stoop_name: Option<String>,
    #[ts(type = "number | null")]
    pub expires_at: Option<i64>,
}

// ---------------------------------------------------------------------------
// The stoop and its rooms (PROTOCOL §3)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct StoopInfo {
    pub name: String,
    pub accent_key: Option<ColorKey>,
    pub icon_key: Option<String>,
    pub member_count: u32,
    #[ts(type = "number")]
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CreateRoomRequest {
    pub slug: String,
    pub name: String,
    pub topic: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct UpdateRoomRequest {
    pub name: Option<String>,
    pub topic: Option<String>,
    pub position: Option<i32>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct UpdateStoopRequest {
    pub name: Option<String>,
    pub accent_key: Option<ColorKey>,
    pub icon_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Room {
    pub id: RoomId,
    pub slug: String,
    pub name: String,
    pub topic: Option<String>,
    pub position: i32,
    #[ts(type = "number | null")]
    pub archived_at: Option<i64>,
    /// The client compares this to its read marker for the "left off here" line
    /// and the label-weight change. No count is ever computed server-side.
    pub last_message_id: Option<MessageId>,
}

// ---------------------------------------------------------------------------
// Messages (PROTOCOL §4)
// ---------------------------------------------------------------------------

/// One reaction key's accumulation on a message. `count` exists for accessibility
/// labels and hover — the client renders *weight*, never the number (SPEC §4.8).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ReactionGroup {
    pub key: String,
    pub count: u32,
    pub user_ids: Vec<UserId>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Message {
    pub id: MessageId,
    pub room_id: RoomId,
    pub author_id: UserId,
    /// Markdown source, unrendered. Empty string on tombstones.
    pub body: String,
    pub reply_to: Option<MessageId>,
    pub attachments: Vec<Attachment>,
    pub reactions: Vec<ReactionGroup>,
    #[ts(type = "number | null")]
    pub pinned_at: Option<i64>,
    #[ts(type = "number | null")]
    pub edited_at: Option<i64>,
    /// Tombstone marker: deleted messages are kept so reply chains survive.
    #[ts(type = "number | null")]
    pub deleted_at: Option<i64>,
    #[ts(type = "number")]
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CreateMessageRequest {
    pub body: String,
    pub reply_to: Option<MessageId>,
    pub attachment_ids: Option<Vec<AttachmentId>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct EditMessageRequest {
    pub body: String,
}

/// `PUT /rooms/:id/read` body. There is no unread-count endpoint and one must
/// never be added (SPEC §4.2).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct UpdateReadMarkerRequest {
    pub last_read_id: MessageId,
}

/// `GET /read` response: room id → last read message id.
pub type ReadMap = HashMap<RoomId, MessageId>;

// ---------------------------------------------------------------------------
// Uploads and the shelf (PROTOCOL §6)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Attachment {
    pub id: AttachmentId,
    pub filename: String,
    pub mime: String,
    #[ts(type = "number")]
    pub size_bytes: u64,
    /// Served from a separate origin (ARCHITECTURE §7) — uploads are hostile.
    pub url: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    #[ts(type = "number | null")]
    pub duration_ms: Option<u64>,
    pub blurhash: Option<String>,
    pub poster_url: Option<String>,
    #[ts(type = "number | null")]
    pub starred_at: Option<i64>,
    pub uploader_id: UserId,
    #[ts(type = "number")]
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CreateUploadRequest {
    pub filename: String,
    #[ts(type = "number")]
    pub size_bytes: u64,
    pub mime: String,
}

/// One pre-signed part of a multipart upload.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct UploadPart {
    pub number: u32,
    pub url: String,
}

/// Where and how the client PUTs bytes — directly to the object store, never
/// through the app server (ARCHITECTURE §8).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct UploadSlot {
    pub upload_id: UploadId,
    pub attachment_id: AttachmentId,
    pub method: String,
    pub url: String,
    pub headers: HashMap<String, String>,
    #[ts(type = "number")]
    pub part_size_bytes: u64,
    pub parts: Option<Vec<UploadPart>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CompletedPart {
    pub number: u32,
    pub etag: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CompleteUploadRequest {
    pub parts: Option<Vec<CompletedPart>>,
}

/// What the shelf grid renders: an attachment plus the moment it came from,
/// so every item links back to its message (SPEC §4.4).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ShelfItem {
    pub attachment: Attachment,
    pub message_id: Option<MessageId>,
    pub room_id: Option<RoomId>,
}

// ---------------------------------------------------------------------------
// Presence (SPEC §4.3, PROTOCOL §8) — shared by REST `ready` and the gateway
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum PresenceState {
    Sitting,
    Around,
    Idle,
    Away,
    Offline,
}

/// A resolved, registry-listed activity. There is no field for a window title
/// and one must never be added — the type system enforces the privacy rule.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ActivityInfo {
    pub registry_id: String,
    pub label: String,
    /// Registry `kind` (game | browser | creative | …); drives roster iconography.
    pub kind: String,
    #[ts(type = "number")]
    pub since: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PresenceEntry {
    pub user_id: UserId,
    pub state: PresenceState,
    pub room_id: Option<RoomId>,
    pub activity: Option<ActivityInfo>,
    pub away_message: Option<String>,
}

// ---------------------------------------------------------------------------
// Invites, notify rules, export (PROTOCOL §§5, 7)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Invite {
    pub code: String,
    pub created_by: UserId,
    #[ts(type = "number | null")]
    pub expires_at: Option<i64>,
    pub max_uses: Option<u32>,
    pub uses: u32,
    #[ts(type = "number | null")]
    pub revoked_at: Option<i64>,
    #[ts(type = "number")]
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CreateInviteRequest {
    pub expires_in_hours: Option<u32>,
    pub max_uses: Option<u32>,
}

/// "Always notify me when [person] posts" — per person, per room (`room_id: None`
/// means all rooms). The notification setting people actually want (SPEC §4.2).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct NotifyRule {
    pub target_user_id: UserId,
    pub room_id: Option<RoomId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum ExportState {
    Queued,
    Running,
    Complete,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ExportJob {
    pub job_id: String,
    pub state: ExportState,
    /// 0.0–1.0.
    pub progress: f32,
    pub url: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_envelope_matches_protocol_shape() {
        let env = ErrorEnvelope {
            error: ErrorBody {
                code: ErrorCode::InviteExpired,
                message: "That invite has expired.".into(),
                retry_after_ms: None,
            },
        };
        let json = serde_json::to_value(&env).unwrap();
        assert_eq!(json["error"]["code"], "INVITE_EXPIRED");
        assert!(json["error"]["retry_after_ms"].is_null());
    }

    #[test]
    fn fill_is_kind_tagged() {
        let solid: Fill = serde_json::from_str(r#"{"kind":"solid","color":"azure"}"#).unwrap();
        assert_eq!(
            solid,
            Fill::Solid {
                color: ColorKey("azure".into())
            }
        );

        let grad = Fill::Gradient {
            from: ColorKey("teal".into()),
            to: ColorKey("violet".into()),
        };
        let json = serde_json::to_value(&grad).unwrap();
        assert_eq!(json["kind"], "gradient");
        assert_eq!(json["from"], "teal");
    }

    #[test]
    fn default_style_is_valid_by_construction() {
        let style = Style::default();
        assert!(crate::is_valid_font_key(&style.font_key));
        if let Fill::Solid { color } = &style.fill {
            assert!(color.is_valid());
        } else {
            panic!("default fill must be solid");
        }
    }

    #[test]
    fn presence_states_serialize_lowercase() {
        assert_eq!(
            serde_json::to_string(&PresenceState::Sitting).unwrap(),
            "\"sitting\""
        );
        assert_eq!(
            serde_json::to_string(&PresenceState::Around).unwrap(),
            "\"around\""
        );
    }
}
