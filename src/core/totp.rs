use data_encoding::BASE32_NOPAD;
use hmac::{digest::KeyInit, Hmac, Mac};
use image::DynamicImage;
use sha1::Sha1;
use sha2::{Sha256, Sha512};
use std::path::Path;
use thiserror::Error;
use url::Url;
use zeroize::Zeroizing;

#[derive(Debug, Error)]
pub enum TotpError {
    #[error("invalid otpauth URI: {0}")]
    InvalidUri(String),
    #[error("missing or invalid secret")]
    InvalidSecret,
    #[error("unsupported otpauth parameters: {0}")]
    Unsupported(String),
    #[error("image decode failed: {0}")]
    Image(String),
    #[error("no QR code found in image")]
    QrNotFound,
    #[error("QR code does not contain an otpauth:// URI")]
    NotOtpauth,
    #[error("capture failed: {0}")]
    Capture(String),
    #[error("no capture target selected")]
    NoCaptureTarget,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TotpAlgorithm {
    Sha1,
    Sha256,
    Sha512,
}

impl TotpAlgorithm {
    fn parse(s: &str) -> Result<Self, TotpError> {
        match s.to_ascii_uppercase().as_str() {
            "SHA1" => Ok(Self::Sha1),
            "SHA256" => Ok(Self::Sha256),
            "SHA512" => Ok(Self::Sha512),
            other => Err(TotpError::Unsupported(format!("algorithm {other}"))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OtpAuth {
    pub issuer: String,
    pub account_name: String,
    pub secret_base32: Zeroizing<String>,
    pub algorithm: TotpAlgorithm,
    pub digits: u32,
    pub period: u64,
}

impl OtpAuth {
    pub fn normalized_secret(&self) -> String {
        self.secret_base32
            .chars()
            .filter(|c| !c.is_whitespace() && *c != '=' && *c != '-')
            .collect::<String>()
            .to_uppercase()
    }
}

/// Parse an `otpauth://totp/...` URI and extract TOTP parameters.
pub fn parse_otpauth_uri(raw: &str) -> Result<OtpAuth, TotpError> {
    let uri = Url::parse(raw.trim()).map_err(|e| TotpError::InvalidUri(e.to_string()))?;
    if uri.scheme() != "otpauth" || uri.host_str() != Some("totp") {
        return Err(TotpError::InvalidUri(
            "expected otpauth://totp URI".to_string(),
        ));
    }

    let label = uri
        .path()
        .trim_start_matches('/')
        .replace("%3A", ":")
        .replace("%3a", ":");
    let decoded_label = percent_decode_component(&label);
    let (label_issuer, account_name) = if let Some((i, a)) = decoded_label.split_once(':') {
        (i.trim().to_string(), a.trim().to_string())
    } else {
        (String::new(), decoded_label.trim().to_string())
    };

    let mut secret: Option<String> = None;
    let mut issuer = label_issuer;
    let mut algorithm = TotpAlgorithm::Sha1;
    let mut digits: u32 = 6;
    let mut period: u64 = 30;

    for (k, v) in uri.query_pairs() {
        match k.to_ascii_lowercase().as_str() {
            "secret" => secret = Some(v.to_string()),
            "issuer" => {
                if !v.trim().is_empty() {
                    issuer = v.trim().to_string();
                }
            }
            "algorithm" => algorithm = TotpAlgorithm::parse(&v)?,
            "digits" => {
                digits = v
                    .parse::<u32>()
                    .map_err(|_| TotpError::Unsupported("digits".into()))?;
                if digits != 6 && digits != 8 {
                    return Err(TotpError::Unsupported("digits must be 6 or 8".into()));
                }
            }
            "period" => {
                period = v
                    .parse::<u64>()
                    .map_err(|_| TotpError::Unsupported("period".into()))?;
                if period == 0 {
                    return Err(TotpError::Unsupported("period must be > 0".into()));
                }
            }
            _ => {}
        }
    }

    let secret = secret.ok_or(TotpError::InvalidSecret)?;
    let normalized: String = secret
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '=' && *c != '-')
        .collect::<String>()
        .to_uppercase();
    if normalized.is_empty() {
        return Err(TotpError::InvalidSecret);
    }
    // Validate Base32 (RFC4648, no padding after normalization)
    BASE32_NOPAD
        .decode(normalized.as_bytes())
        .map_err(|_| TotpError::InvalidSecret)?;

    Ok(OtpAuth {
        issuer,
        account_name,
        secret_base32: Zeroizing::new(normalized),
        algorithm,
        digits,
        period,
    })
}

fn percent_decode_component(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(h), Some(l)) = (hex(bytes[i + 1]), hex(bytes[i + 2])) {
                out.push((h << 4 | l) as char);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    out
}

fn hex(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

/// Decode a QR code from image bytes and extract an `otpauth://` TOTP URI.
pub fn decode_otpauth_from_image_bytes(bytes: &[u8]) -> Result<OtpAuth, TotpError> {
    let img = image::load_from_memory(bytes).map_err(|e| TotpError::Image(e.to_string()))?;
    decode_otpauth_from_dynamic_image(&img)
}

/// Decode a QR code from a user-selected image file and extract an `otpauth://` TOTP URI.
pub fn decode_otpauth_from_image_path(path: impl AsRef<Path>) -> Result<OtpAuth, TotpError> {
    let img = image::open(path.as_ref()).map_err(|e| TotpError::Image(e.to_string()))?;
    decode_otpauth_from_dynamic_image(&img)
}

fn decode_otpauth_from_dynamic_image(img: &DynamicImage) -> Result<OtpAuth, TotpError> {
    let gray = img.to_luma8();
    let mut prepared = rqrr::PreparedImage::prepare(gray);
    let grids = prepared.detect_grids();
    let mut last_err: Option<TotpError> = None;
    for grid in grids {
        let mut out = Vec::new();
        match grid.decode_to(&mut out) {
            Ok(_) => match String::from_utf8(out) {
                Ok(text) => {
                    let trimmed = text.trim();
                    if trimmed.to_ascii_lowercase().starts_with("otpauth://") {
                        return parse_otpauth_uri(trimmed);
                    } else {
                        last_err = Some(TotpError::NotOtpauth);
                    }
                }
                Err(_) => last_err = Some(TotpError::NotOtpauth),
            },
            Err(_) => last_err = Some(TotpError::QrNotFound),
        }
    }
    Err(last_err.unwrap_or(TotpError::QrNotFound))
}

/// Generate a TOTP code for the given Unix timestamp (seconds).
pub fn generate_totp(otp: &OtpAuth, unix_seconds: u64) -> Result<String, TotpError> {
    let secret_bytes = BASE32_NOPAD
        .decode(otp.normalized_secret().as_bytes())
        .map_err(|_| TotpError::InvalidSecret)?;
    let counter = unix_seconds / otp.period;
    let msg = counter.to_be_bytes();

    let code_int: u32 = match otp.algorithm {
        TotpAlgorithm::Sha1 => dynamic_truncate(&hmac_bytes::<Hmac<Sha1>>(&secret_bytes, &msg)),
        TotpAlgorithm::Sha256 => dynamic_truncate(&hmac_bytes::<Hmac<Sha256>>(&secret_bytes, &msg)),
        TotpAlgorithm::Sha512 => dynamic_truncate(&hmac_bytes::<Hmac<Sha512>>(&secret_bytes, &msg)),
    };
    let modulus = 10u32.pow(otp.digits);
    Ok(format!(
        "{:0width$}",
        code_int % modulus,
        width = otp.digits as usize
    ))
}

fn hmac_bytes<M: Mac + KeyInit>(key: &[u8], msg: &[u8]) -> Vec<u8> {
    let mut mac = <M as Mac>::new_from_slice(key).expect("HMAC accepts any key length");
    mac.update(msg);
    mac.finalize().into_bytes().to_vec()
}

fn dynamic_truncate(hmac_result: &[u8]) -> u32 {
    let offset = (hmac_result[hmac_result.len() - 1] & 0x0f) as usize;
    ((u32::from(hmac_result[offset]) & 0x7f) << 24)
        | ((u32::from(hmac_result[offset + 1]) & 0xff) << 16)
        | ((u32::from(hmac_result[offset + 2]) & 0xff) << 8)
        | (u32::from(hmac_result[offset + 3]) & 0xff)
}

pub fn seconds_remaining(period: u64, unix_seconds: u64) -> u64 {
    if period == 0 {
        return 0;
    }
    period - (unix_seconds % period)
}

#[cfg(any(target_os = "windows", target_os = "linux"))]
pub mod desktop_capture {
    use super::{decode_otpauth_from_dynamic_image, OtpAuth, TotpError};
    use image::DynamicImage;

    #[derive(Debug, Clone)]
    pub enum CaptureTarget {
        Monitor {
            id: u32,
            name: String,
        },
        Window {
            id: u32,
            title: String,
            app_name: String,
        },
    }

    #[derive(Debug, Clone, Copy)]
    pub struct CaptureRegion {
        pub x: u32,
        pub y: u32,
        pub width: u32,
        pub height: u32,
    }

    #[derive(Debug, Clone)]
    pub struct CaptureSelection {
        pub target: CaptureTarget,
        /// Optional region within the captured target image. `None` captures/scans the full target.
        pub region: Option<CaptureRegion>,
    }

    pub fn list_monitors() -> Result<Vec<CaptureTarget>, TotpError> {
        let monitors = xcap::Monitor::all().map_err(|e| TotpError::Capture(e.to_string()))?;
        let mut out = Vec::new();
        for m in monitors {
            let id = m.id().map_err(|e| TotpError::Capture(e.to_string()))?;
            let name = m.name().unwrap_or_else(|_| format!("Monitor {id}"));
            out.push(CaptureTarget::Monitor { id, name });
        }
        Ok(out)
    }

    pub fn list_windows() -> Result<Vec<CaptureTarget>, TotpError> {
        let windows = xcap::Window::all().map_err(|e| TotpError::Capture(e.to_string()))?;
        let mut out = Vec::new();
        for w in windows {
            let minimized = w.is_minimized().unwrap_or(false);
            if minimized {
                continue;
            }
            let id = match w.id() {
                Ok(id) => id,
                Err(_) => continue,
            };
            let title = w.title().unwrap_or_default();
            let app_name = w.app_name().unwrap_or_default();
            out.push(CaptureTarget::Window {
                id,
                title,
                app_name,
            });
        }
        Ok(out)
    }

    /// Capture a selected desktop window/monitor (optionally cropped to a region) and scan it for an `otpauth://` QR code.
    /// This enables QR/TOTP import without a camera on Windows/Linux.
    pub fn scan_capture_selection(selection: &CaptureSelection) -> Result<OtpAuth, TotpError> {
        let mut img = capture_target_image(&selection.target)?;
        if let Some(r) = selection.region {
            img = crop_region(img, r)?;
        }
        decode_otpauth_from_dynamic_image(&img)
    }

    fn capture_target_image(target: &CaptureTarget) -> Result<DynamicImage, TotpError> {
        match target {
            CaptureTarget::Monitor { id, .. } => {
                let monitor = xcap::Monitor::all()
                    .map_err(|e| TotpError::Capture(e.to_string()))?
                    .into_iter()
                    .find(|m| m.id().ok().as_ref() == Some(id))
                    .ok_or_else(|| TotpError::Capture("monitor not found".into()))?;
                let image = monitor
                    .capture_image()
                    .map_err(|e| TotpError::Capture(e.to_string()))?;
                to_dynamic_image(image)
            }
            CaptureTarget::Window { id, .. } => {
                let window = xcap::Window::all()
                    .map_err(|e| TotpError::Capture(e.to_string()))?
                    .into_iter()
                    .find(|w| w.id().ok().as_ref() == Some(id))
                    .ok_or_else(|| TotpError::Capture("window not found".into()))?;
                let image = window
                    .capture_image()
                    .map_err(|e| TotpError::Capture(e.to_string()))?;
                to_dynamic_image(image)
            }
        }
    }

    fn to_dynamic_image<I>(image: I) -> Result<DynamicImage, TotpError>
    where
        I: Into<image::RgbaImage>,
    {
        Ok(DynamicImage::ImageRgba8(image.into()))
    }

    fn crop_region(img: DynamicImage, r: CaptureRegion) -> Result<DynamicImage, TotpError> {
        if r.width == 0 || r.height == 0 {
            return Err(TotpError::Capture("invalid capture region".into()));
        }
        let (w, h) = (img.width(), img.height());
        if r.x >= w || r.y >= h {
            return Err(TotpError::Capture("capture region out of bounds".into()));
        }
        let cw = r.width.min(w.saturating_sub(r.x));
        let ch = r.height.min(h.saturating_sub(r.y));
        Ok(img.crop_imm(r.x, r.y, cw, ch))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_otpauth_uri_extracts_fields() {
        let uri = "otpauth://totp/Example:alice@example.com?secret=JBSWY3DPEHPK3PXP&issuer=Example&algorithm=SHA1&digits=6&period=30";
        let otp = parse_otpauth_uri(uri).expect("parse");
        assert_eq!(otp.issuer, "Example");
        assert_eq!(otp.account_name, "alice@example.com");
        assert_eq!(&*otp.secret_base32, "JBSWY3DPEHPK3PXP");
        assert_eq!(otp.digits, 6);
        assert_eq!(otp.period, 30);
    }

    #[test]
    fn parse_otpauth_uri_accepts_lowercase_and_padding() {
        let uri = "otpauth://totp/Issuer:user?secret=jbswy3dpehpk3pxp====&issuer=Issuer";
        let otp = parse_otpauth_uri(uri).expect("parse");
        assert_eq!(&*otp.secret_base32, "JBSWY3DPEHPK3PXP");
    }

    #[test]
    fn parse_otpauth_uri_rejects_non_otpauth() {
        assert!(parse_otpauth_uri("https://example.com").is_err());
    }

    #[test]
    fn rfc6238_sha1_vector_6_digits() {
        // RFC 6238 test secret: 12345678901234567890 in ASCII => GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ
        let otp = OtpAuth {
            issuer: "Test".into(),
            account_name: "test".into(),
            secret_base32: Zeroizing::new("GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ".into()),
            algorithm: TotpAlgorithm::Sha1,
            digits: 8, // RFC table shows 8-digit truncated values for these timestamps
            period: 30,
        };
        // Known RFC 6238 values (8 digits): 59->94287082, 1111111109->07081804, 1234567890->89005924
        assert_eq!(generate_totp(&otp, 59).unwrap(), "94287082");
        assert_eq!(generate_totp(&otp, 1_111_111_109).unwrap(), "07081804");
        assert_eq!(generate_totp(&otp, 1_234_567_890).unwrap(), "89005924");
    }

    #[test]
    fn invalid_image_bytes_fail_without_camera() {
        let err = decode_otpauth_from_image_bytes(b"not an image").unwrap_err();
        assert!(matches!(err, TotpError::Image(_)));
    }
}
