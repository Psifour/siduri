use super::*;

#[derive(Parser)]
pub(crate) struct Search {
    #[command(flatten)]
    options: Options,
    #[arg(help = "Query matched against name, urls, username, tags, and notes.")]
    query: String,
}

impl Search {
    pub(crate) fn run(self) -> Result {
        let query = self.query.to_lowercase();
        let matches: Vec<_> = entry::load(&mut self.options.backend()?)?
            .into_iter()
            .filter(|(_, entry)| {
                let mut haystacks = Vec::with_capacity(4 + entry.urls.len() + entry.tags.len());
                haystacks.push(entry.name.as_str());
                haystacks.push(entry.username.as_str());
                haystacks.extend(entry.urls.iter().map(String::as_str));
                haystacks.extend(entry.tags.iter().map(String::as_str));
                haystacks.extend(entry.notes.as_deref());
                haystacks
                    .iter()
                    .any(|haystack| haystack.to_lowercase().contains(&query))
            })
            .collect();
        ls::table(&matches);
        Ok(())
    }
}
