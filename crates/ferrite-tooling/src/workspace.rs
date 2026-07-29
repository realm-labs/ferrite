use anyhow::{Context as _, Result, bail};
use std::env;
use std::path::{Path, PathBuf};

const MARKER: &str = ".ferrite/cache-policy.toml";

pub(crate) fn discover() -> Result<PathBuf> {
    discover_from(&env::current_dir().context("read current directory")?)
}

fn discover_from(start: &Path) -> Result<PathBuf> {
    let mut current = start.to_path_buf();
    loop {
        if current.join("Cargo.toml").is_file() && current.join(MARKER).is_file() {
            return current
                .canonicalize()
                .with_context(|| format!("resolve workspace {}", current.display()));
        }
        if !current.pop() {
            bail!("run inside the Ferrite workspace; Cargo.toml and {MARKER} were not found");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn discovers_workspace_from_a_descendant() {
        let directory = tempdir().unwrap();
        let nested = directory.path().join("a/b");
        fs::create_dir_all(directory.path().join(".ferrite")).unwrap();
        fs::create_dir_all(&nested).unwrap();
        fs::write(directory.path().join("Cargo.toml"), "[workspace]\n").unwrap();
        fs::write(directory.path().join(MARKER), "schema_version = 1\n").unwrap();

        assert_eq!(
            discover_from(&nested).unwrap(),
            directory.path().canonicalize().unwrap()
        );
    }
}
