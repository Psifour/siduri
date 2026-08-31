use super::*;

#[derive(Parser)]
pub(crate) struct Ls {
    #[command(flatten)]
    options: Options,
}

impl Ls {
    pub(crate) fn run(self) -> Result {
        table(&entry::load(&mut self.options.backend()?)?);
        Ok(())
    }
}

/// Shared by `ls` and `search`: name, username, first url, tags — never the
/// password.
pub(crate) fn table(entries: &[(String, Entry)]) {
    if entries.is_empty() {
        eprintln!("no entries");
        return;
    }

    let width = |f: fn(&Entry) -> &str| {
        entries
            .iter()
            .map(|(_, entry)| sanitize(f(entry)).len())
            .max()
            .unwrap_or_default()
    };
    let name_width = width(|entry| &entry.name).max(4);
    let user_width = width(|entry| &entry.username).max(8);

    for (_, entry) in entries {
        println!(
            "{:name_width$}  {:user_width$}  {}{}",
            sanitize(&entry.name),
            sanitize(&entry.username),
            sanitize(entry.urls.first().map_or("", String::as_str)),
            if entry.tags.is_empty() {
                String::new()
            } else {
                format!("  [{}]", sanitize(&entry.tags.join(", ")))
            },
        );
    }
}
