//! Numeric limits shared by client and server, so validation logic never drifts.
//! Sources: SPEC §4 and ARCHITECTURE §7 (rate limits).

/// 500 MB per file (SPEC §4.10).
pub const MAX_FILE_BYTES: u64 = 500 * 1024 * 1024;
/// 50 GB default per-server pool; host-configurable (SPEC §4.10).
pub const DEFAULT_POOL_BYTES: u64 = 50 * 1024 * 1024 * 1024;
/// Files over this use multipart upload, which is what makes uploads resumable.
pub const MULTIPART_THRESHOLD_BYTES: u64 = 8 * 1024 * 1024;
/// Non-starred, non-pinned files expire after this many days by default.
pub const DEFAULT_FILE_EXPIRY_DAYS: u32 = 365;

/// Custom entrance sounds: max 2 seconds, max 200 KB, reject (never truncate).
pub const MAX_ENTRANCE_SOUND_BYTES: u64 = 200 * 1024;
pub const MAX_ENTRANCE_SOUND_MS: u32 = 2_000;
/// A given user's entrance sound plays at most once per 5 min per listener.
pub const ENTRANCE_SOUND_COOLDOWN_MS: u64 = 5 * 60 * 1000;

/// User-status fields (SPEC §4.6).
pub const MAX_STATUS_LINE_CHARS: usize = 240;
pub const MAX_STATUS_FIELD_CHARS: usize = 80;
pub const MAX_STATUS_IMAGE_BYTES: u64 = 512 * 1024;

/// Message body cap, chars after trim (PROTOCOL §4).
pub const MAX_MESSAGE_CHARS: usize = 8_000;

/// Access tokens are short-lived JWTs; refresh tokens rotate (ARCHITECTURE §7).
pub const ACCESS_TOKEN_TTL_SECS: u64 = 15 * 60;
pub const REFRESH_TOKEN_TTL_DAYS: i64 = 30;

/// Identity fields (PROTOCOL §2).
pub const USERNAME_PATTERN: &str = "^[a-z0-9_]{2,24}$";
pub const ROOM_SLUG_PATTERN: &str = "^[a-z0-9-]{1,32}$";
pub const MAX_DISPLAY_NAME_CHARS: usize = 32;
pub const MIN_PASSWORD_CHARS: usize = 12;
pub const INVITE_CODE_CHARS: usize = 12;

/// Gateway (PROTOCOL §8).
pub const HEARTBEAT_INTERVAL_MS: u64 = 30_000;
pub const RESUME_BUFFER_FRAMES: usize = 500;
pub const RESUME_WINDOW_MS: u64 = 120_000;

/// Rate limits (ARCHITECTURE §7). Format: (events, per_seconds).
pub const RATE_LOGIN_PER_IP: (u32, u64) = (5, 60);
pub const RATE_MESSAGE_SEND: (u32, u64) = (10, 10);
pub const RATE_UPLOAD_SLOTS: (u32, u64) = (20, 3_600);
pub const RATE_INVITE_CREATE: (u32, u64) = (10, 86_400);
pub const RATE_KNOCK_PER_TARGET: (u32, u64) = (3, 3_600);
pub const RATE_EXPORT: (u32, u64) = (1, 3_600);
pub const RATE_TYPING_PER_ROOM: (u32, u64) = (1, 4);
/// Read-marker updates are debounced client-side to once per 5s per room.
pub const READ_MARKER_DEBOUNCE_MS: u64 = 5_000;
