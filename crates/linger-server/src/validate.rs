//! Input validation with the PROTOCOL §2/§3 shapes. Kept dependency-free (no
//! regex crate for three character classes) and returning `ApiError` directly
//! so handlers read as straight-line code.

use linger_core::limits::{
    MAX_DISPLAY_NAME_CHARS, MAX_MESSAGE_CHARS, MAX_SIGN_FIELD_CHARS, MAX_SIGN_LINE_CHARS,
    MIN_PASSWORD_CHARS,
};
use linger_core::wire::{Fill, Sign, Style};

use crate::error::ApiError;

/// `[a-z0-9_]{2,24}` — lowercase on the way in is the caller's job; we reject,
/// not normalize, so people see exactly what their username is.
pub fn username(s: &str) -> Result<(), ApiError> {
    let ok = (2..=24).contains(&s.len())
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_');
    if ok {
        Ok(())
    } else {
        Err(ApiError::validation(
            "Usernames are 2–24 characters: lowercase letters, digits, underscore.",
        ))
    }
}

/// `[a-z0-9-]{1,32}`.
pub fn room_slug(s: &str) -> Result<(), ApiError> {
    let ok = (1..=32).contains(&s.len())
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-');
    if ok {
        Ok(())
    } else {
        Err(ApiError::validation(
            "Room slugs are 1–32 characters: lowercase letters, digits, hyphen.",
        ))
    }
}

pub fn display_name(s: &str) -> Result<(), ApiError> {
    let len = s.trim().chars().count();
    if (1..=MAX_DISPLAY_NAME_CHARS).contains(&len) {
        Ok(())
    } else {
        Err(ApiError::validation("Display names are 1–32 characters."))
    }
}

/// Minimum length only — composition rules are explicitly banned (PROTOCOL §2).
pub fn password(s: &str) -> Result<(), ApiError> {
    if s.chars().count() >= MIN_PASSWORD_CHARS {
        Ok(())
    } else {
        Err(ApiError::validation(
            "Passwords need at least 12 characters.",
        ))
    }
}

/// Trimmed message body, 1..=8000 chars (PROTOCOL §4).
pub fn message_body(s: &str) -> Result<String, ApiError> {
    let trimmed = s.trim();
    let len = trimmed.chars().count();
    if len == 0 {
        return Err(ApiError::validation("Say something."));
    }
    if len > MAX_MESSAGE_CHARS {
        return Err(ApiError::validation("That's too long for one message."));
    }
    Ok(trimmed.to_string())
}

/// The AGENTS.md hard rule: palette and font keys are validated server-side
/// against the closed sets in linger-core. Client-side validation alone is a
/// defect.
pub fn style(style: &Style) -> Result<(), ApiError> {
    if !linger_core::is_valid_font_key(&style.font_key) {
        return Err(ApiError::validation("That font isn't in the bundled set."));
    }
    if let Some(msg_font) = &style.msg_font_key {
        if !linger_core::is_valid_font_key(msg_font) {
            return Err(ApiError::validation(
                "That message font isn't in the bundled set.",
            ));
        }
    }
    if ![400u16, 500, 700].contains(&style.weight) {
        return Err(ApiError::validation("Weight must be 400, 500, or 700."));
    }
    let colors_ok = match &style.fill {
        Fill::Solid { color } => color.is_valid(),
        Fill::Gradient { from, to } => from.is_valid() && to.is_valid(),
    };
    if !colors_ok {
        return Err(ApiError::validation(
            "Colors are picked from the named palette.",
        ));
    }
    Ok(())
}

pub fn sign(sign: &Sign) -> Result<(), ApiError> {
    let cap = |field: &Option<String>, max: usize, what: &str| -> Result<(), ApiError> {
        match field {
            Some(v) if v.chars().count() > max => Err(ApiError::validation(format!(
                "{what} is capped at {max} characters."
            ))),
            _ => Ok(()),
        }
    };
    cap(&sign.line, MAX_SIGN_LINE_CHARS, "The sign line")?;
    cap(&sign.reading, MAX_SIGN_FIELD_CHARS, "Reading")?;
    cap(&sign.listening, MAX_SIGN_FIELD_CHARS, "Listening")?;
    cap(&sign.working_on, MAX_SIGN_FIELD_CHARS, "Working on")?;
    cap(&sign.away_message, MAX_SIGN_LINE_CHARS, "The away message")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use linger_core::wire::ColorKey;

    use super::*;

    #[test]
    fn username_shapes() {
        assert!(username("matt").is_ok());
        assert!(username("m_42").is_ok());
        assert!(username("m").is_err());
        assert!(username("Matt").is_err());
        assert!(username("matt guenther").is_err());
        assert!(username(&"x".repeat(25)).is_err());
    }

    #[test]
    fn style_rejects_off_palette_and_off_list_fonts() {
        let mut s = Style::default();
        assert!(style(&s).is_ok());
        s.font_key = "comic-sans".into();
        assert!(style(&s).is_err());
        s.font_key = "geist-sans".into();
        s.fill = Fill::Solid {
            color: ColorKey("#ff00ff".into()),
        };
        assert!(style(&s).is_err());
        s.fill = Fill::Gradient {
            from: ColorKey("teal".into()),
            to: ColorKey("nope".into()),
        };
        assert!(style(&s).is_err());
        s.fill = Fill::Gradient {
            from: ColorKey("teal".into()),
            to: ColorKey("violet".into()),
        };
        s.weight = 600;
        assert!(style(&s).is_err());
    }
}
