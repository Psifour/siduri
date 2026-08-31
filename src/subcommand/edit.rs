use super::*;

#[derive(Parser)]
pub(crate) struct Edit {
    #[command(flatten)]
    options: Options,
    #[arg(help = "Entry name.")]
    entry: String,
    #[arg(long, help = "Rename the entry.")]
    rename: Option<String>,
    #[arg(long, help = "Replace the username.")]
    username: Option<String>,
    #[arg(long, help = "Replace the notes (empty clears them).")]
    notes: Option<String>,
    #[arg(long, value_name = "URL", help = "Add a URL (repeatable).")]
    add_url: Vec<String>,
    #[arg(long, value_name = "URL", help = "Remove a URL (repeatable).")]
    rm_url: Vec<String>,
    #[arg(long, value_name = "TAG", help = "Add a tag (repeatable).")]
    add_tag: Vec<String>,
    #[arg(long, value_name = "TAG", help = "Remove a tag (repeatable).")]
    rm_tag: Vec<String>,
    // Hidden: secrets on argv leak (warned about in `arguments`).
    #[arg(long, hide = true, value_parser = options::secret)]
    totp_secret: Option<Zeroizing<String>>,
    #[arg(long, help = "Remove the TOTP secret.")]
    rm_totp: bool,
    #[arg(long, help = "Prompt for a new password.")]
    ask_password: bool,
    #[arg(
        long,
        visible_alias = "gen",
        help = "Generate a new password (printed once to stdout)."
    )]
    generate: bool,
    #[arg(long, value_name = "N", help = "Words for --generate (default 6).")]
    words: Option<usize>,
    #[arg(
        long,
        value_name = "N",
        conflicts_with = "words",
        help = "Generate <N> random characters instead of words."
    )]
    length: Option<usize>,
}

impl Edit {
    pub(crate) fn run(self) -> Result {
        let changing = self.rename.is_some()
            || self.username.is_some()
            || self.notes.is_some()
            || !self.add_url.is_empty()
            || !self.rm_url.is_empty()
            || !self.add_tag.is_empty()
            || !self.rm_tag.is_empty()
            || self.totp_secret.is_some()
            || self.rm_totp
            || self.ask_password
            || self.generate;
        ensure!(
            changing,
            "nothing to change; see `siduri edit --help` for the field flags"
        );

        if let Some(secret) = &self.totp_secret {
            crate::totp::code(secret)?;
        }

        let mut backend = self.options.backend()?;
        let (id, mut entry) = entry::find(&mut backend, &self.entry)?;

        if let Some(name) = self.rename {
            ensure!(
                !entry::load(&mut backend)?
                    .iter()
                    .any(|(other, entry)| entry.name == name && *other != id),
                "an entry named `{}` already exists",
                sanitize(&name)
            );
            entry.name = name;
        }
        if let Some(username) = self.username {
            entry.username = username;
        }
        if let Some(notes) = self.notes {
            entry.notes = (!notes.is_empty()).then_some(notes);
        }
        entry.urls.extend(self.add_url);
        entry.urls.retain(|url| !self.rm_url.contains(url));
        entry.urls.dedup();
        entry.tags.extend(self.add_tag);
        entry.tags.retain(|tag| !self.rm_tag.contains(tag));
        entry.tags.dedup();
        if self.rm_totp {
            entry.totp_secret = None;
        }
        if let Some(secret) = self.totp_secret {
            entry.totp_secret = Some(secret.to_string());
        }
        if self.generate {
            let password = generate::password(self.words, self.length, false)?;
            println!("{}", *password);
            entry.password = password.to_string();
        } else if self.ask_password {
            entry.password = options::prompt_confirmed("new password")?.to_string();
        }

        entry.modified = entry::now();
        entry::save(&mut backend, &id, &entry)?;
        eprintln!("updated `{}`", sanitize(&entry.name));
        Ok(())
    }
}
