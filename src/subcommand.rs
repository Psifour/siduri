use super::*;

pub mod add;
pub mod clip;
pub mod edit;
pub mod generate;
pub mod ls;
pub mod rm;
pub mod search;
pub mod show;
pub mod totp;

#[derive(Parser)]
pub(crate) enum Subcommand {
    #[command(about = "Add a login entry")]
    Add(add::Add),
    #[command(about = "Show an entry (password hidden unless --reveal)")]
    Show(show::Show),
    #[command(about = "Edit an entry")]
    Edit(edit::Edit),
    #[command(about = "Remove an entry")]
    Rm(rm::Rm),
    #[command(about = "List entries")]
    Ls(ls::Ls),
    #[command(about = "Search entries by name, url, username, or tag")]
    Search(search::Search),
    #[command(about = "Copy an entry's password to the clipboard with a timed clear")]
    Clip(clip::Clip),
    #[command(about = "Print the current TOTP code for an entry")]
    Totp(totp::Totp),
    #[command(name = "gen", about = "Generate a password (charclass or diceware)")]
    Gen(generate::Generate),
}

impl Subcommand {
    pub(crate) fn run(self) -> Result {
        match self {
            Self::Add(add) => add.run(),
            Self::Show(show) => show.run(),
            Self::Edit(edit) => edit.run(),
            Self::Rm(rm) => rm.run(),
            Self::Ls(ls) => ls.run(),
            Self::Search(search) => search.run(),
            Self::Clip(clip) => clip.run(),
            Self::Totp(totp) => totp.run(),
            Self::Gen(generate) => generate.run(),
        }
    }
}
