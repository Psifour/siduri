use super::*;

#[derive(Parser)]
pub(crate) struct Show {
    #[command(flatten)]
    options: Options,
    #[arg(help = "Entry name.")]
    entry: String,
    #[arg(long, help = "Include the password (and TOTP secret) in the output.")]
    reveal: bool,
}

impl Show {
    pub(crate) fn run(self) -> Result {
        let (_, entry) = entry::find(&mut self.options.backend()?, &self.entry)?;

        // Piped output is the script interface: the password, nothing else.
        if !io::stdout().is_terminal() {
            println!("{}", entry.password);
            return Ok(());
        }

        println!("name:     {}", sanitize(&entry.name));
        if !entry.username.is_empty() {
            println!("username: {}", sanitize(&entry.username));
        }
        for url in &entry.urls {
            println!("url:      {}", sanitize(url));
        }
        if !entry.tags.is_empty() {
            println!("tags:     {}", sanitize(&entry.tags.join(", ")));
        }
        if let Some(notes) = &entry.notes {
            println!("notes:    {}", sanitize(notes));
        }
        println!(
            "totp:     {}",
            if entry.totp_secret.is_some() {
                "yes (`siduri totp`)"
            } else {
                "no"
            }
        );
        println!("created:  {}", sanitize(&entry.created));
        println!("modified: {}", sanitize(&entry.modified));

        if self.reveal {
            println!("password: {}", sanitize(&entry.password));
            if let Some(secret) = &entry.totp_secret {
                println!("totp-secret: {}", sanitize(secret));
            }
        } else {
            println!("password: <hidden; --reveal, or pipe for the raw value>");
        }
        Ok(())
    }
}
