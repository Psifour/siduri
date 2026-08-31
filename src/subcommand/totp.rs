use super::*;

#[derive(Parser)]
pub(crate) struct Totp {
    #[command(flatten)]
    options: Options,
    #[arg(help = "Entry name.")]
    entry: String,
}

impl Totp {
    pub(crate) fn run(self) -> Result {
        let (_, entry) = entry::find(&mut self.options.backend()?, &self.entry)?;
        let secret = entry
            .totp_secret
            .as_deref()
            .ok_or_else(|| anyhow!("`{}` has no TOTP secret", sanitize(&self.entry)))?;

        let (code, remaining) = crate::totp::code(secret)?;
        println!("{code}");
        eprintln!("valid for {remaining}s");
        Ok(())
    }
}
