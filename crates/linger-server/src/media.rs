//! Turning arriving bytes into something safe to store (ARCHITECTURE §8 step 5).
//!
//! Three jobs, in order:
//!
//! 1. **Find out what the file actually is.** The type the client declared is a
//!    claim, not a fact. Magic bytes decide, and a file whose contents disagree
//!    with its label is refused.
//! 2. **Re-encode every image.** Decoding and re-encoding drops EXIF — which
//!    carries the GPS coordinates of the room the photo was taken in (SPEC
//!    §4.10) — and destroys polyglots, files that are a valid image and a valid
//!    something-else at the same time, in the same step. There is no toggle.
//! 3. **Describe it.** Dimensions, a blurhash to show while the real thing
//!    loads, a poster frame and duration for video. No transcoding in V1.
//!
//! Video work shells out to `ffmpeg`/`ffprobe`. They are optional: a server
//! without them stores videos perfectly well and simply has no poster frame.

use std::path::Path;
use std::process::Stdio;

use linger_core::media;

use crate::error::ApiError;

/// The biggest image this server will decode. Well past any camera, and short
/// of the memory a deliberately enormous one would ask for.
const MAX_IMAGE_BYTES: u64 = 64 * 1024 * 1024;
/// Decoded-pixel guards against a small file that claims enormous dimensions.
const MAX_IMAGE_DIMENSION: u32 = 16_384;
const MAX_DECODE_ALLOC: u64 = 512 * 1024 * 1024;
/// Blurhash is meant to be a smear of colour, so it is computed from a thumbnail.
const BLURHASH_MAX_EDGE: u32 = 64;
/// Where in a video to grab the poster frame. One second in, because frame zero
/// of a lot of video is black.
const POSTER_SECONDS: &str = "1";

/// What processing worked out about a file.
pub struct Processed {
    /// The real type, which may differ from what the client declared.
    pub mime: String,
    /// The filename, with its extension corrected if re-encoding changed format.
    pub filename: String,
    /// Size after re-encoding — what counts against the pool.
    pub size_bytes: u64,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub duration_ms: Option<u64>,
    pub blurhash: Option<String>,
    /// JPEG bytes of a generated video poster frame, if there is one.
    pub poster: Option<Vec<u8>>,
}

/// Inspect and clean up the staged file **in place**, so the caller can move it
/// straight to its permanent key afterwards.
pub async fn process(
    path: &Path,
    declared_mime: &str,
    filename: &str,
) -> Result<Processed, ApiError> {
    let mime = resolve_mime(path, declared_mime).await?;
    let filename = corrected_filename(filename, declared_mime, &mime);
    let mut out = Processed {
        mime: mime.clone(),
        filename,
        size_bytes: tokio::fs::metadata(path).await.map_err(io)?.len(),
        width: None,
        height: None,
        duration_ms: None,
        blurhash: None,
        poster: None,
    };

    match media::kind_of(&mime) {
        "image" => {
            if out.size_bytes > MAX_IMAGE_BYTES {
                return Err(ApiError::validation(
                    "That image is too big for this server to re-encode. \
                     Send it as a file, or shrink it first.",
                ));
            }
            let bytes = tokio::fs::read(path).await.map_err(io)?;
            let owned_mime = mime.clone();
            let clean = tokio::task::spawn_blocking(move || reencode_image(&bytes, &owned_mime))
                .await
                .map_err(|_| ApiError::internal())??;
            tokio::fs::write(path, &clean.bytes).await.map_err(io)?;
            out.size_bytes = clean.bytes.len() as u64;
            out.mime = clean.mime.clone();
            out.filename = corrected_filename(&out.filename, &mime, &clean.mime);
            out.width = Some(clean.width);
            out.height = Some(clean.height);
            out.blurhash = clean.blurhash;
        }
        "video" => {
            let probe = ffprobe(path).await;
            out.duration_ms = probe.as_ref().and_then(|p| p.duration_ms);
            out.width = probe.as_ref().and_then(|p| p.width);
            out.height = probe.as_ref().and_then(|p| p.height);
            if let Some(poster) = poster_frame(path).await {
                if let Ok(decoded) = decode_limited(&poster) {
                    let rgba = decoded.to_rgba8();
                    out.width = out.width.or(Some(rgba.width()));
                    out.height = out.height.or(Some(rgba.height()));
                    out.blurhash = blurhash_of(&rgba);
                }
                out.poster = Some(poster);
            }
        }
        "audio" => {
            out.duration_ms = ffprobe(path).await.and_then(|p| p.duration_ms);
        }
        _ => {}
    }
    Ok(out)
}

fn io(err: std::io::Error) -> ApiError {
    tracing::error!(error = %err, "upload processing io");
    ApiError::internal()
}

// ---------------------------------------------------------------------------
// What is this file, really
// ---------------------------------------------------------------------------

/// Decide the stored type from the declared one and the actual bytes.
///
/// Formats with magic bytes have to match the category they were declared as —
/// that is what catches a file renamed to `.png` to get past the allowlist.
/// Formats with no magic bytes at all (a text file, a project file) can only be
/// taken at their word, so they are allowed through as generic files and served
/// as downloads, where being wrong about them costs nothing.
async fn resolve_mime(path: &Path, declared: &str) -> Result<String, ApiError> {
    let head = read_head(path).await?;
    let declared_kind = media::kind_of(declared);
    let sniffed = infer::get(&head).map(|t| media::canonical_mime(t.mime_type()).to_string());

    match sniffed {
        Some(sniffed) => {
            if !media::is_allowed_mime(&sniffed) {
                return Err(ApiError::unsupported_media(
                    "That isn't a kind of file this server takes.",
                ));
            }
            if media::kind_of(&sniffed) != declared_kind {
                return Err(ApiError::unsupported_media(
                    "That file isn't what it says it is.",
                ));
            }
            Ok(sniffed)
        }
        None if declared_kind == "file" => Ok(media::canonical_mime(declared).to_string()),
        None => Err(ApiError::unsupported_media(
            "That file isn't what it says it is.",
        )),
    }
}

async fn read_head(path: &Path) -> Result<Vec<u8>, ApiError> {
    use tokio::io::AsyncReadExt;
    let mut file = tokio::fs::File::open(path).await.map_err(io)?;
    let mut head = vec![0u8; 4096];
    let read = file.read(&mut head).await.map_err(io)?;
    head.truncate(read);
    Ok(head)
}

/// Keep the extension honest when re-encoding changed the format.
fn corrected_filename(filename: &str, was: &str, now: &str) -> String {
    if media::canonical_mime(was) == media::canonical_mime(now) {
        return filename.to_string();
    }
    let Some(ext) = media::extension_for(now) else {
        return filename.to_string();
    };
    let stem = filename.rsplit_once('.').map_or(filename, |(stem, _)| stem);
    format!("{stem}.{ext}")
}

// ---------------------------------------------------------------------------
// Images
// ---------------------------------------------------------------------------

struct CleanImage {
    bytes: Vec<u8>,
    mime: String,
    width: u32,
    height: u32,
    blurhash: Option<String>,
}

fn decode_limited(bytes: &[u8]) -> Result<image::DynamicImage, ApiError> {
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(MAX_IMAGE_DIMENSION);
    limits.max_image_height = Some(MAX_IMAGE_DIMENSION);
    limits.max_alloc = Some(MAX_DECODE_ALLOC);

    let mut reader = image::ImageReader::new(std::io::Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|_| ApiError::unsupported_media("That image can't be read."))?;
    reader.limits(limits);
    reader
        .decode()
        .map_err(|_| ApiError::unsupported_media("That image can't be read."))
}

/// Re-encode an image so nothing of the original file survives except pixels.
///
/// GIFs go through frame by frame so an animation stays animated. WebP has no
/// encoder in the `image` crate, so a WebP comes back as a PNG — same picture,
/// honest extension, and still no metadata.
fn reencode_image(bytes: &[u8], mime: &str) -> Result<CleanImage, ApiError> {
    use image::codecs::gif::{GifDecoder, GifEncoder, Repeat};
    use image::{AnimationDecoder, ImageEncoder};

    if mime == "image/gif" {
        let frames = GifDecoder::new(std::io::Cursor::new(bytes))
            .and_then(|d| d.into_frames().collect_frames())
            .map_err(|_| ApiError::unsupported_media("That image can't be read."))?;
        let first = frames
            .first()
            .ok_or_else(|| ApiError::unsupported_media("That image has no frames."))?
            .buffer()
            .clone();
        let (width, height) = (first.width(), first.height());
        let blurhash = blurhash_of(&first);

        let mut out = Vec::new();
        {
            let mut encoder = GifEncoder::new(&mut out);
            encoder
                .set_repeat(Repeat::Infinite)
                .and_then(|()| encoder.encode_frames(frames))
                .map_err(|_| ApiError::internal())?;
        }
        return Ok(CleanImage {
            bytes: out,
            mime: "image/gif".to_string(),
            width,
            height,
            blurhash,
        });
    }

    let decoded = decode_limited(bytes)?;
    let rgba = decoded.to_rgba8();
    let (width, height) = (rgba.width(), rgba.height());
    let blurhash = blurhash_of(&rgba);
    let mut out = Vec::new();

    let mime = if mime == "image/jpeg" {
        let rgb = decoded.to_rgb8();
        image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, 88)
            .write_image(rgb.as_raw(), width, height, image::ExtendedColorType::Rgb8)
            .map_err(|_| ApiError::internal())?;
        "image/jpeg"
    } else {
        // PNG for everything else, WebP included: lossless, and the one format
        // the crate can both read and write for these inputs.
        image::codecs::png::PngEncoder::new(&mut out)
            .write_image(
                rgba.as_raw(),
                width,
                height,
                image::ExtendedColorType::Rgba8,
            )
            .map_err(|_| ApiError::internal())?;
        "image/png"
    };

    Ok(CleanImage {
        bytes: out,
        mime: mime.to_string(),
        width,
        height,
        blurhash,
    })
}

fn blurhash_of(source: &image::RgbaImage) -> Option<String> {
    let (width, height) = (source.width(), source.height());
    if width == 0 || height == 0 {
        return None;
    }
    let scale = f64::from(BLURHASH_MAX_EDGE) / f64::from(width.max(height));
    let small = if scale < 1.0 {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let (w, h) = (
            ((f64::from(width) * scale).round() as u32).max(1),
            ((f64::from(height) * scale).round() as u32).max(1),
        );
        image::imageops::thumbnail(source, w, h)
    } else {
        source.clone()
    };
    blurhash::encode(4, 3, small.width(), small.height(), small.as_raw()).ok()
}

// ---------------------------------------------------------------------------
// Video and audio: ffmpeg, if the host has it
// ---------------------------------------------------------------------------

struct Probe {
    duration_ms: Option<u64>,
    width: Option<u32>,
    height: Option<u32>,
}

async fn ffprobe(path: &Path) -> Option<Probe> {
    let output = tokio::process::Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-print_format",
            "json",
            "-show_format",
            "-show_streams",
        ])
        .arg(path)
        .stdin(Stdio::null())
        .output()
        .await
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).ok()?;

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let duration_ms = json["format"]["duration"]
        .as_str()
        .and_then(|d| d.parse::<f64>().ok())
        .filter(|d| d.is_finite() && *d >= 0.0)
        .map(|d| (d * 1000.0).round() as u64);

    let video = json["streams"]
        .as_array()
        .and_then(|streams| {
            streams
                .iter()
                .find(|s| s["codec_type"].as_str() == Some("video"))
        })
        .cloned();

    #[allow(clippy::cast_possible_truncation)]
    let dimension = |value: &serde_json::Value| value.as_u64().map(|v| v as u32).filter(|v| *v > 0);

    Some(Probe {
        duration_ms,
        width: video.as_ref().and_then(|v| dimension(&v["width"])),
        height: video.as_ref().and_then(|v| dimension(&v["height"])),
    })
}

/// One frame, as JPEG. `None` whenever ffmpeg isn't installed or the file has
/// no frame to give — a missing poster is not a failed upload.
async fn poster_frame(path: &Path) -> Option<Vec<u8>> {
    for seek in [POSTER_SECONDS, "0"] {
        let temp = tempfile::Builder::new()
            .suffix(".jpg")
            .tempfile()
            .ok()?
            .into_temp_path();
        let status = tokio::process::Command::new("ffmpeg")
            .args(["-v", "error", "-y", "-ss", seek, "-i"])
            .arg(path)
            .args(["-frames:v", "1", "-f", "image2"])
            .arg(&temp)
            .stdin(Stdio::null())
            .status()
            .await;
        let Ok(status) = status else { return None };
        if status.success() {
            if let Ok(bytes) = tokio::fs::read(&temp).await {
                if !bytes.is_empty() {
                    return Some(bytes);
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn png_bytes(width: u32, height: u32) -> Vec<u8> {
        let mut image = image::RgbaImage::new(width, height);
        for (x, y, pixel) in image.enumerate_pixels_mut() {
            *pixel = image::Rgba([(x % 256) as u8, (y % 256) as u8, 128, 255]);
        }
        let mut out = Vec::new();
        image::DynamicImage::ImageRgba8(image)
            .write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
            .unwrap();
        out
    }

    #[test]
    fn re_encoding_keeps_the_picture_and_produces_a_blurhash() {
        let clean = reencode_image(&png_bytes(40, 30), "image/png").unwrap();
        assert_eq!((clean.width, clean.height), (40, 30));
        assert_eq!(clean.mime, "image/png");
        assert!(clean.blurhash.is_some());
        assert!(decode_limited(&clean.bytes).is_ok());
    }

    #[test]
    fn a_webp_comes_back_as_a_png_with_a_matching_name() {
        assert_eq!(
            corrected_filename("holiday.webp", "image/webp", "image/png"),
            "holiday.png"
        );
        assert_eq!(
            corrected_filename("holiday.jpg", "image/jpeg", "image/jpeg"),
            "holiday.jpg"
        );
    }

    #[tokio::test]
    async fn a_file_that_isnt_what_it_says_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("staged");

        tokio::fs::write(&path, png_bytes(4, 4)).await.unwrap();
        assert_eq!(resolve_mime(&path, "image/png").await.unwrap(), "image/png");
        // A real PNG declared as a video is still a lie.
        assert!(resolve_mime(&path, "video/mp4").await.is_err());

        // A zip wearing a .png name.
        tokio::fs::write(&path, b"PK\x03\x04zipzipzip")
            .await
            .unwrap();
        assert!(resolve_mime(&path, "image/png").await.is_err());

        // Plain text has no magic bytes, so it is taken at its word.
        tokio::fs::write(&path, b"just some notes\n").await.unwrap();
        assert_eq!(
            resolve_mime(&path, "text/plain").await.unwrap(),
            "text/plain"
        );
        assert!(resolve_mime(&path, "image/jpeg").await.is_err());
    }
}
