use super::*;

#[derive(Parser)]
pub(crate) struct Rm {
    #[command(flatten)]
    options: Options,
    #[arg(help = "Entry name.")]
    entry: String,
}

impl Rm {
    pub(crate) fn run(self) -> Result {
        let mut backend = self.options.backend()?;
        let (id, _) = entry::find(&mut backend, &self.entry)?;
        ensure!(
            backend.rm(&id)?,
            "entry `{}` vanished before it could be removed",
            sanitize(&self.entry)
        );
        eprintln!("removed `{}`", sanitize(&self.entry));
        Ok(())
    }
}
