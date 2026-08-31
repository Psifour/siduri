use super::*;

#[test]
fn add_show_search_edit_rm_roundtrip() {
    let dir = TempDir::new().unwrap();
    let vault = make_vault(&dir);

    siduri(
        &vault,
        &[
            "add",
            "github",
            "--username",
            "adam",
            "--url",
            "https://github.com",
            "--tag",
            "dev",
        ],
        b"hunter22 hunter22\n",
    );

    // Piped `show` is the script interface: the password, nothing else.
    assert_eq!(
        siduri(&vault, &["show", "github"], b""),
        "hunter22 hunter22\n"
    );

    let ls = siduri(&vault, &["ls"], b"");
    assert!(ls.contains("github") && ls.contains("adam") && ls.contains("[dev]"));
    assert!(!ls.contains("hunter22"), "ls must never print passwords");

    assert!(siduri(&vault, &["search", "GITHUB.COM"], b"").contains("github"));
    assert!(!siduri(&vault, &["search", "nomatch"], b"").contains("github"));

    siduri(
        &vault,
        &["edit", "github", "--username", "other", "--add-tag", "work"],
        b"",
    );
    let ls = siduri(&vault, &["ls"], b"");
    assert!(ls.contains("other") && ls.contains("work"));

    // Same name twice is refused.
    let dup = run(&vault, &["add", "github"], b"pw\n");
    assert!(!dup.status.success());
    assert!(String::from_utf8_lossy(&dup.stderr).contains("already exists"));

    siduri(&vault, &["rm", "github"], b"");
    let gone = run(&vault, &["show", "github"], b"");
    assert!(!gone.status.success());

    // Record ids in the vault are opaque hex, not entry names: the plaintext
    // id list must not leak the account list.
    siduri(&vault, &["add", "secretsite", "--username", "u"], b"pw\n");
    let raw = std::fs::read_to_string(&vault).unwrap();
    assert!(!raw.contains("secretsite"));
}

#[test]
fn totp_codes_are_computed_from_the_stored_secret() {
    let dir = TempDir::new().unwrap();
    let vault = make_vault(&dir);

    siduri(
        &vault,
        &["add", "mfa", &format!("--totp-secret={TOTP_SECRET}")],
        b"pw\n",
    );
    let code = siduri(&vault, &["totp", "mfa"], b"");
    let code = code.trim();
    assert_eq!(code.len(), 6, "{code}");
    assert!(code.chars().all(|c| c.is_ascii_digit()), "{code}");

    // A garbage secret is rejected at `add`, not at first use.
    let bad = run(&vault, &["add", "bad", "--totp-secret=!!!"], b"pw\n");
    assert!(!bad.status.success());
}

#[test]
fn generated_passwords_have_the_advertised_shape() {
    let dir = TempDir::new().unwrap();
    let vault = make_vault(&dir);

    let words = siduri(&vault, &["gen"], b"");
    assert_eq!(words.trim().split('-').count(), 6);

    let chars = siduri(&vault, &["gen", "--length", "20"], b"");
    assert_eq!(chars.trim().chars().count(), 20);

    let word_list = gilgamesh::bip39::Language::English.word_list();
    assert!(
        words
            .trim()
            .split('-')
            .all(|word| word_list.contains(&word))
    );

    // `add --generate` prints the password once and stores the same value.
    let generated = siduri(&vault, &["add", "gen-site", "--generate"], b"");
    assert_eq!(
        siduri(&vault, &["show", "gen-site"], b""),
        generated,
        "stored password must match the printed one"
    );
}
