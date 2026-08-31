use super::*;

/// Frozen record type: schema changes ship as `login.v2`, never as silent
/// field changes. Dotted, not slashed: vault record types cannot contain
/// `/`, which the record AD framing reserves.
pub(crate) const LOGIN_TYPE: &str = "login.v1";

/// The decrypted payload of a `login.v1` record. Record ids are random hex —
/// never the site name, which would leak the account list through the
/// vault's plaintext ids — so every human-readable field lives in here, in
/// ciphertext.
#[derive(Deserialize, Serialize, zeroize::Zeroize, zeroize::ZeroizeOnDrop)]
#[serde(deny_unknown_fields)]
pub(crate) struct Entry {
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) urls: Vec<String>,
    #[serde(default)]
    pub(crate) username: String,
    pub(crate) password: String,
    #[serde(default)]
    pub(crate) totp_secret: Option<String>,
    #[serde(default)]
    pub(crate) notes: Option<String>,
    #[serde(default)]
    pub(crate) tags: Vec<String>,
    pub(crate) created: String,
    pub(crate) modified: String,
}

/// Decrypt every `login.v1` record. A record that fails to parse is reported
/// and skipped, not fatal: one corrupt or future-versioned record must not
/// lock the rest of the store.
pub(crate) fn load(backend: &mut Backend) -> Result<Vec<(String, Entry)>> {
    let mut entries = Vec::new();
    for (id, record_type) in backend.list()? {
        if record_type != LOGIN_TYPE {
            continue;
        }
        let plaintext = backend.get(&id)?;
        match serde_json::from_slice::<Entry>(&plaintext) {
            Ok(entry) => entries.push((id, entry)),
            Err(err) => eprintln!(
                "warning: skipping record `{}`: not a parsable {LOGIN_TYPE} entry ({err})",
                sanitize(&id)
            ),
        }
    }
    entries.sort_by(|(_, a), (_, b)| a.name.cmp(&b.name));
    Ok(entries)
}

pub(crate) fn find(backend: &mut Backend, name: &str) -> Result<(String, Entry)> {
    load(backend)?
        .into_iter()
        .find(|(_, entry)| entry.name == name)
        .ok_or_else(|| anyhow!("no entry named `{}` (try `siduri search`)", sanitize(name)))
}

pub(crate) fn save(backend: &mut Backend, id: &str, entry: &Entry) -> Result {
    let plaintext = Zeroizing::new(serde_json::to_vec(entry)?);
    backend.put(id, LOGIN_TYPE, &plaintext)
}

/// 128-bit random record id, hex.
pub(crate) fn new_id() -> Result<String> {
    Ok(random::<16>()?.iter().map(|b| format!("{b:02x}")).collect())
}

/// Current UTC time as RFC 3339, from civil-from-days — no calendar
/// dependency for two timestamp fields.
pub(crate) fn now() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or_default();

    let (h, min, s) = (secs / 3600 % 24, secs / 60 % 60, secs % 60);

    let z = (secs / 86400) as i64 + 719_468;
    let era = z / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);

    format!("{y:04}-{m:02}-{d:02}T{h:02}:{min:02}:{s:02}Z")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timestamps_are_rfc3339_utc() {
        let now = now();
        assert_eq!(now.len(), 20, "{now}");
        assert!(now.starts_with("20"), "{now}");
        assert!(now.ends_with('Z'), "{now}");
        assert_eq!(now.as_bytes()[10], b'T');
    }

    #[test]
    fn entries_roundtrip_json() {
        let entry = Entry {
            name: "example".into(),
            urls: vec!["https://example.com".into()],
            username: "user".into(),
            password: "hunter22".into(),
            totp_secret: None,
            notes: Some("note".into()),
            tags: vec!["work".into()],
            created: now(),
            modified: now(),
        };
        let parsed: Entry = serde_json::from_slice(&serde_json::to_vec(&entry).unwrap()).unwrap();
        assert_eq!(parsed.name, "example");
        assert_eq!(parsed.password, "hunter22");
    }
}
