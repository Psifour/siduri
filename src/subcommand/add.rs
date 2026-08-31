use super::*;

#[derive(Parser)]
pub(crate) struct Add {
    #[command(flatten)]
    options: Options,
    #[arg(help = "Entry name (unique).")]
    entry: String,
    #[arg(long, value_name = "URL", help = "Associated URL (repeatable).")]
    url: Vec<String>,
    #[arg(long, help = "Username / login identifier.")]
    username: Option<String>,
    #[arg(long, value_name = "TAG", help = "Tag (repeatable).")]
    tag: Vec<String>,
    #[arg(long, help = "Free-form notes.")]
    notes: Option<String>,
    // Hidden: secrets on argv leak (warned about in `arguments`).
    #[arg(long, hide = true, value_parser = options::secret)]
    totp_secret: Option<Zeroizing<String>>,
    #[arg(
        long,
        visible_alias = "gen",
        help = "Generate the password (printed once to stdout)."
    )]
    generate: bool,
    #[arg(
        long,
        value_name = "N",
        help = "Words for --generate (default 6, 11 bits each)."
    )]
    words: Option<usize>,
    #[arg(
        long,
        value_name = "N",
        conflicts_with = "words",
        help = "Generate <N> random characters instead of words."
    )]
    length: Option<usize>,
}

impl Add {
    pub(crate) fn run(self) -> Result {
        // Reject a bad TOTP secret before it is stored, not at first use.
        if let Some(secret) = &self.totp_secret {
            crate::totp::code(secret)?;
        }

        let mut backend = self.options.backend()?;
        ensure!(
            !entry::load(&mut backend)?
                .iter()
                .any(|(_, entry)| entry.name == self.entry),
            "an entry named `{}` already exists",
            sanitize(&self.entry)
        );

        let password = if self.generate {
            let password = generate::password(self.words, self.length, false)?;
            println!("{}", *password);
            password
        } else if !io::stdin().is_terminal() {
            let mut line = String::new();
            io::stdin().read_line(&mut line)?;
            Zeroizing::new(line.trim_end_matches(['\r', '\n']).into())
        } else {
            options::prompt_confirmed("password")?
        };
        ensure!(!password.is_empty(), "refusing an empty password");

        let username = match self.username {
            Some(username) => username,
            None if io::stdin().is_terminal() && !self.generate => {
                options::prompt_line("username (optional): ")?
            }
            None => String::new(),
        };

        let now = entry::now();
        let entry = Entry {
            name: self.entry.clone(),
            urls: self.url,
            username,
            password: password.to_string(),
            totp_secret: self.totp_secret.map(|secret| secret.to_string()),
            notes: self.notes,
            tags: self.tag,
            created: now.clone(),
            modified: now,
        };

        entry::save(&mut backend, &entry::new_id()?, &entry)?;
        eprintln!("added `{}`", sanitize(&self.entry));
        Ok(())
    }
}
