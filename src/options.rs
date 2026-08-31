use super::*;

/// Parse a secret-valued argument into zeroizing memory.
pub(crate) fn secret(value: &str) -> Result<Zeroizing<String>, std::convert::Infallible> {
    Ok(Zeroizing::new(value.into()))
}

/// No `Debug`: holds the unlock password and derivation passphrase.
#[derive(Args)]
pub(crate) struct Options {
    #[arg(
        long,
        env = "GILGAMESH_VAULT",
        help = "Vault file <PATH>. Defaults to the agent's vault when one is running, \
                else ./gilgamesh.vault."
    )]
    vault: Option<PathBuf>,
    // Hidden: argv leaks into shell history and /proc/*/cmdline (warned about
    // in `arguments`); scripts should prefer the environment variables.
    #[arg(
        long,
        env = "GILGAMESH_PASSWORD",
        hide = true,
        hide_env_values = true,
        value_parser = secret
    )]
    password: Option<Zeroizing<String>>,
    #[arg(
        long,
        env = "GILGAMESH_PASSPHRASE",
        hide = true,
        hide_env_values = true,
        value_parser = secret
    )]
    passphrase: Option<Zeroizing<String>>,
    #[arg(
        long,
        help = "Prompt for the derivation passphrase (immutable part of the identity)."
    )]
    ask_passphrase: bool,
    #[arg(
        long,
        env = "GILGAMESH_AGENT_SOCK",
        help = "Agent socket <PATH> (default: $XDG_RUNTIME_DIR/gilgamesh/agent.sock)."
    )]
    socket: Option<PathBuf>,
    #[arg(long, help = "Skip the agent and unlock the vault in-process.")]
    no_agent: bool,
}

impl Options {
    /// Prefer a running gilgamesh agent — no password prompt, no key material
    /// in this process beyond requested plaintext — and fall back to an
    /// in-process unlock. An explicit `--vault` that differs from the agent's
    /// also falls back, so the two can never silently diverge.
    pub(crate) fn backend(&self) -> Result<Backend> {
        #[cfg(unix)]
        if !self.no_agent
            && let Ok(mut client) = gilgamesh::agent::Client::connect(self.socket.as_deref())
            && let Ok(status) = client.status()
        {
            let matches = match &self.vault {
                None => true,
                Some(vault) => same_path(vault, &PathBuf::from(&status.vault)),
            };
            if matches {
                return Ok(Backend::Agent(client));
            }
            eprintln!(
                "note: the agent serves `{}`, not `{}`; unlocking directly",
                sanitize(&status.vault),
                self.vault().display()
            );
        }

        let path = self.vault();
        let vault = Vault::load(&path)?;

        let hardware = if vault.is_hardware_bound() {
            Some(gilgamesh::hardware::platform_sealer()?)
        } else {
            None
        };

        let password = match &self.password {
            Some(password) => password.clone(),
            None => Zeroizing::new(rpassword::prompt_password("unlock password: ")?),
        };
        let passphrase = match &self.passphrase {
            Some(passphrase) => Some(passphrase.clone()),
            None if self.ask_passphrase => Some(Zeroizing::new(rpassword::prompt_password(
                "derivation passphrase: ",
            )?)),
            None => None,
        }
        .filter(|passphrase| !passphrase.is_empty());

        let identity = vault.unlock(
            &password,
            passphrase.as_deref().map(String::as_str),
            hardware.as_deref(),
        )?;

        Ok(Backend::Direct { path, identity })
    }

    fn vault(&self) -> PathBuf {
        self.vault
            .clone()
            .unwrap_or_else(|| PathBuf::from("gilgamesh.vault"))
    }
}

#[cfg(unix)]
fn same_path(a: &std::path::Path, b: &std::path::Path) -> bool {
    let resolve =
        |p: &std::path::Path| std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf());
    resolve(a) == resolve(b)
}

pub(crate) fn prompt_confirmed(label: &str) -> Result<Zeroizing<String>> {
    let first = Zeroizing::new(rpassword::prompt_password(format!("{label}: "))?);
    let second = Zeroizing::new(rpassword::prompt_password(format!("confirm {label}: "))?);
    ensure!(*first == *second, "{label} entries do not match");
    Ok(first)
}

pub(crate) fn prompt_line(label: &str) -> Result<String> {
    eprint!("{label}");
    io::stderr().flush()?;
    let mut line = String::new();
    io::stdin().read_line(&mut line)?;
    Ok(line.trim().into())
}
