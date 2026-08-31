use super::*;

/// Where records live: a running gilgamesh agent (preferred), or a directly
/// unlocked vault. Both expose the same record operations; direct mode
/// mirrors the agent's discipline — reload the file for every operation,
/// verify it belongs to the held identity, and serialize mutations on the
/// vault's advisory lock.
pub(crate) enum Backend {
    #[cfg(unix)]
    Agent(gilgamesh::agent::Client),
    Direct {
        path: PathBuf,
        identity: Identity,
    },
}

impl Backend {
    /// `(id, record_type)` of every record in the vault.
    pub(crate) fn list(&mut self) -> Result<Vec<(String, String)>> {
        match self {
            #[cfg(unix)]
            Self::Agent(client) => Ok(client
                .vault_list()?
                .0
                .into_iter()
                .map(|record| (record.id, record.record_type))
                .collect()),
            Self::Direct { path, identity } => {
                let vault = Vault::load(path)?;
                vault.verify_identity(identity)?;
                Ok(vault
                    .records()
                    .iter()
                    .map(|record| (record.id().into(), record.record_type().into()))
                    .collect())
            }
        }
    }

    pub(crate) fn get(&mut self, id: &str) -> Result<Zeroizing<Vec<u8>>> {
        match self {
            #[cfg(unix)]
            Self::Agent(client) => client.vault_get(id),
            Self::Direct { path, identity } => {
                let vault = Vault::load(path)?;
                vault.verify_identity(identity)?;
                vault.get(identity, id)
            }
        }
    }

    pub(crate) fn put(&mut self, id: &str, record_type: &str, plaintext: &[u8]) -> Result {
        match self {
            #[cfg(unix)]
            Self::Agent(client) => client.vault_put(id, record_type, plaintext),
            Self::Direct { path, identity } => {
                let _lock = Vault::lock_file(path)?;
                let mut vault = Vault::load(path)?;
                vault.verify_identity(identity)?;
                vault.put(identity, id, record_type, plaintext)?;
                vault.store(path)
            }
        }
    }

    pub(crate) fn rm(&mut self, id: &str) -> Result<bool> {
        match self {
            #[cfg(unix)]
            Self::Agent(client) => client.vault_rm(id),
            Self::Direct { path, identity } => {
                let _lock = Vault::lock_file(path)?;
                let mut vault = Vault::load(path)?;
                vault.verify_identity(identity)?;
                let removed = vault.remove(identity, id);
                if removed {
                    vault.store(path)?;
                }
                Ok(removed)
            }
        }
    }
}
