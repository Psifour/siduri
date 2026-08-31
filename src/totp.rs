use {
    super::*,
    hmac::{Hmac, Mac},
    sha1::Sha1,
};

/// TOTP defaults per RFC 6238: HMAC-SHA1, 30-second steps, 6 digits — what
/// effectively every provisioning QR uses. Deviating parameters would ship
/// as extra `login/v2` fields, not guesses.
const STEP: u64 = 30;
const DIGITS: u32 = 6;

/// Current code and the seconds until it rotates.
pub(crate) fn code(secret: &str) -> Result<(String, u64)> {
    let key = base32_decode(secret)?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or_default();
    Ok((at(&key, now)?, STEP - now % STEP))
}

fn at(key: &[u8], time: u64) -> Result<String> {
    let counter = time / STEP;

    let mut mac =
        <Hmac<Sha1> as Mac>::new_from_slice(key).map_err(|_| anyhow!("empty TOTP secret"))?;
    mac.update(&counter.to_be_bytes());
    let digest = mac.finalize().into_bytes();

    let offset = (digest[19] & 0x0f) as usize;
    let binary =
        u32::from_be_bytes(digest[offset..offset + 4].try_into().expect("4 bytes")) & 0x7fff_ffff;

    Ok(format!(
        "{:0width$}",
        binary % 10u32.pow(DIGITS),
        width = DIGITS as usize
    ))
}

/// RFC 4648 base32, as used in otpauth secrets: case-insensitive, spaces and
/// padding ignored.
fn base32_decode(secret: &str) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    let mut buffer: u64 = 0;
    let mut bits = 0;

    for c in secret.chars() {
        if c == ' ' || c == '=' || c == '-' {
            continue;
        }
        let value = match c.to_ascii_uppercase() {
            c @ 'A'..='Z' => c as u64 - 'A' as u64,
            c @ '2'..='7' => c as u64 - '2' as u64 + 26,
            _ => bail!(
                "invalid base32 character `{}` in TOTP secret",
                sanitize(&c.to_string())
            ),
        };
        buffer = buffer << 5 | value;
        bits += 5;
        if bits >= 8 {
            bits -= 8;
            bytes.push((buffer >> bits) as u8);
        }
    }

    ensure!(!bytes.is_empty(), "empty TOTP secret");
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base32_decodes_rfc4648_vectors() {
        assert_eq!(base32_decode("MZXW6YTBOI======").unwrap(), b"foobar");
        assert_eq!(base32_decode("mzxw6ytboi").unwrap(), b"foobar");
        assert_eq!(base32_decode("MZXW 6YTB OI").unwrap(), b"foobar");
        assert!(base32_decode("1!").is_err());
        assert!(base32_decode("").is_err());
    }

    #[test]
    fn totp_matches_rfc6238_vectors() {
        // RFC 6238 SHA-1 vectors, truncated from 8 to our 6 digits.
        let key = b"12345678901234567890";
        assert_eq!(at(key, 59).unwrap(), "287082");
        assert_eq!(at(key, 1_111_111_109).unwrap(), "081804");
        assert_eq!(at(key, 1_234_567_890).unwrap(), "005924");
        assert_eq!(at(key, 20_000_000_000).unwrap(), "353130");
    }
}
