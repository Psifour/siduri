use {
    anyhow::{Context, Error, anyhow, bail, ensure},
    arguments::Arguments,
    backend::Backend,
    clap::{Args, Parser},
    entry::Entry,
    gilgamesh::{Identity, Vault, Zeroizing},
    options::Options,
    serde::{Deserialize, Serialize},
    std::{
        env,
        io::{self, IsTerminal, Write},
        path::PathBuf,
        process,
    },
};

mod arguments;
mod backend;
mod clipboard;
mod entry;
mod options;
mod subcommand;
mod totp;

pub type Result<T = (), E = Error> = std::result::Result<T, E>;

/// Filter an untrusted string for terminal display: graphic ASCII and spaces
/// pass, everything else — control bytes, escape sequences, non-ASCII —
/// becomes `?`. Entry fields decrypt from an externally editable vault and
/// must not be able to smuggle escape sequences to the terminal.
fn sanitize(text: &str) -> String {
    text.chars()
        .map(|c| {
            if c.is_ascii_graphic() || c == ' ' {
                c
            } else {
                '?'
            }
        })
        .collect()
}

fn random<const N: usize>() -> Result<[u8; N]> {
    let mut bytes = [0; N];
    getrandom::fill(&mut bytes).map_err(|err| anyhow!("system entropy source failed: {err}"))?;
    Ok(bytes)
}

pub fn main() {
    let args = Arguments::parse();

    if let Err(err) = args.run() {
        eprintln!("error: {err}");

        for (i, cause) in err.chain().skip(1).enumerate() {
            if i == 0 {
                eprintln!();
                eprintln!("because:");
            }
            eprintln!("- {cause}");
        }

        if env::var_os("RUST_BACKTRACE")
            .map(|val| val == "1")
            .unwrap_or_default()
        {
            eprintln!();
            eprintln!("{}", err.backtrace());
        }

        process::exit(1);
    }
}
