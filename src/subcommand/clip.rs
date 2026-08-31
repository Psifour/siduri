use super::*;

#[derive(Parser)]
pub(crate) struct Clip {
    #[command(flatten)]
    options: Options,
    #[arg(help = "Entry name.")]
    entry: String,
    #[arg(
        long,
        default_value_t = 30,
        value_name = "SECONDS",
        help = "Clear the clipboard after <SECONDS>; 0 disables the clear."
    )]
    seconds: u64,
    #[arg(long, help = "Copy the current TOTP code instead of the password.")]
    totp: bool,
}

impl Clip {
    pub(crate) fn run(self) -> Result {
        let (_, entry) = entry::find(&mut self.options.backend()?, &self.entry)?;

        let what = if self.totp {
            let secret = entry
                .totp_secret
                .as_deref()
                .ok_or_else(|| anyhow!("`{}` has no TOTP secret", sanitize(&self.entry)))?;
            clipboard::copy(&crate::totp::code(secret)?.0, self.seconds)?;
            "TOTP code"
        } else {
            clipboard::copy(&entry.password, self.seconds)?;
            "password"
        };

        if self.seconds == 0 {
            eprintln!("copied {what} for `{}`", sanitize(&self.entry));
        } else {
            eprintln!(
                "copied {what} for `{}`; clearing in {}s (clipboard-history managers may keep \
                 their own copy)",
                sanitize(&self.entry),
                self.seconds
            );
        }
        Ok(())
    }
}
