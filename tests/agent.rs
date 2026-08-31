use {
    super::*,
    gilgamesh::agent::{Client, Config, Server},
    std::{os::unix::fs::DirBuilderExt, thread, time::Duration},
};

/// The full agent path: siduri finds the agent by socket, never sees the
/// vault password, and its writes land in the same vault file.
#[test]
fn siduri_uses_a_running_agent() {
    let dir = TempDir::new().unwrap();
    let vault = make_vault(&dir);

    let identity = Vault::load(&vault)
        .unwrap()
        .unlock(PASSWORD, None, None)
        .unwrap();

    // The agent refuses a socket directory that is not private (0700); a
    // tempdir is created with the default umask.
    let run = dir.path().join("run");
    std::fs::DirBuilder::new().mode(0o700).create(&run).unwrap();
    let socket = run.join("agent.sock");
    let server = Server::new(
        vault.clone(),
        identity,
        Config {
            socket: Some(socket.clone()),
            ..Config::default()
        },
    )
    .unwrap();
    thread::spawn(move || server.run().unwrap());
    for _ in 0..200 {
        if Client::connect(Some(&socket)).is_ok() {
            break;
        }
        thread::sleep(Duration::from_millis(25));
    }

    let output = Command::new(env!("CARGO_BIN_EXE_siduri"))
        .args(["add", "via-agent", "--username", "u"])
        .env("GILGAMESH_AGENT_SOCK", &socket)
        .env("GILGAMESH_VAULT", &vault)
        // No password: the agent is the whole point.
        .env_remove("GILGAMESH_PASSWORD")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map(|mut child| {
            child
                .stdin
                .as_mut()
                .unwrap()
                .write_all(b"agent-pw\n")
                .unwrap();
            child.wait_with_output().unwrap()
        })
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Direct mode sees the agent's write.
    assert_eq!(siduri(&vault, &["show", "via-agent"], b""), "agent-pw\n");
}
