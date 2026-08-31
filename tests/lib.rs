use {
    gilgamesh::{Vault, seal::Argon2Params},
    std::{
        io::Write,
        path::{Path, PathBuf},
        process::{Command, Output, Stdio},
    },
    tempfile::TempDir,
};

#[cfg(unix)]
mod agent;
mod entries;

const PASSWORD: &str = "correct horse battery staple";

/// RFC 6238's ASCII test key `12345678901234567890`, base32.
const TOTP_SECRET: &str = "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ";

fn make_vault(dir: &TempDir) -> PathBuf {
    let path = dir.path().join("test.vault");
    let (vault, _) = Vault::create(
        gilgamesh::Seed::from_bytes([0x42; 32]),
        None,
        PASSWORD,
        Argon2Params::default(),
    )
    .unwrap();
    vault.store(&path).unwrap();
    path
}

fn run(vault: &Path, args: &[&str], stdin: &[u8]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_siduri"))
        .arg(args[0])
        // `gen` takes no vault options; everything else must not wander off
        // to a real agent on the developer's machine.
        .args(if args[0] == "gen" { &[][..] } else { &["--no-agent"] })
        .args(&args[1..])
        .env("GILGAMESH_VAULT", vault)
        .env("GILGAMESH_PASSWORD", PASSWORD)
        .env_remove("GILGAMESH_PASSPHRASE")
        .env_remove("GILGAMESH_AGENT_SOCK")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    child.stdin.as_mut().unwrap().write_all(stdin).unwrap();

    child.wait_with_output().unwrap()
}

fn siduri(vault: &Path, args: &[&str], stdin: &[u8]) -> String {
    let output = run(vault, args, stdin);
    assert!(
        output.status.success(),
        "`siduri {}` failed:\n{}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr),
    );
    String::from_utf8(output.stdout).unwrap()
}
