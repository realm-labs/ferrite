use anyhow::{Context as _, Result, bail};
use std::fmt::{self, Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};

const MAX_HANDWRITTEN_LINES: usize = 1_200;
const SKIPPED_DIRECTORIES: &[&str] = &[".git", "target", "generated", "vendor"];

pub(crate) fn verify(workspace: &Path) -> Result<()> {
    let mut sources = Vec::new();
    collect_rust_sources(workspace, workspace, &mut sources)?;
    sources.sort();

    let mut violations = Vec::new();
    for path in &sources {
        audit_file(workspace, path, &mut violations)?;
    }
    if !violations.is_empty() {
        let report = violations
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n");
        bail!("source policy violations:\n{report}");
    }
    println!(
        "source policy verified: {} handwritten Rust files, maximum {} physical lines",
        sources.len(),
        MAX_HANDWRITTEN_LINES
    );
    Ok(())
}

fn collect_rust_sources(root: &Path, directory: &Path, output: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(directory)
        .with_context(|| format!("read source directory {}", directory.display()))?
    {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let path = entry.path();
        if file_type.is_symlink() {
            continue;
        }
        if file_type.is_dir() {
            if should_skip(root, &path) {
                continue;
            }
            collect_rust_sources(root, &path, output)?;
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            output.push(path);
        }
    }
    Ok(())
}

fn should_skip(root: &Path, path: &Path) -> bool {
    path.strip_prefix(root)
        .ok()
        .into_iter()
        .flat_map(Path::components)
        .any(|component| {
            let value = component.as_os_str().to_string_lossy();
            SKIPPED_DIRECTORIES.contains(&value.as_ref())
        })
}

fn audit_file(root: &Path, path: &Path, violations: &mut Vec<Violation>) -> Result<()> {
    let source =
        fs::read_to_string(path).with_context(|| format!("read Rust source {}", path.display()))?;
    let relative = path.strip_prefix(root).unwrap_or(path).to_path_buf();
    let line_count = source.lines().count();
    if line_count > MAX_HANDWRITTEN_LINES {
        violations.push(Violation::new(
            relative.clone(),
            None,
            format!("contains {line_count} physical lines; maximum is {MAX_HANDWRITTEN_LINES}"),
        ));
    }

    let forbidden = [
        (
            concat!("super::", "super"),
            "uses a deep parent-relative path",
        ),
        (
            concat!("pub", " use "),
            "uses a public re-export without a reviewed facade exception",
        ),
        (
            concat!("allow(clippy::", "all"),
            "broadly disables all Clippy lints",
        ),
        (
            concat!("expect(clippy::", "all"),
            "broadly expects all Clippy lints",
        ),
        (
            concat!("cfg_attr(clippy", ","),
            "changes compilation to bypass Clippy",
        ),
        (
            concat!("cfg(clippy", ")"),
            "changes compilation to bypass Clippy",
        ),
    ];
    for (index, line) in source.lines().enumerate() {
        for (needle, message) in forbidden {
            if line.contains(needle) {
                violations.push(Violation::new(relative.clone(), Some(index + 1), message));
            }
        }
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Violation {
    path: PathBuf,
    line: Option<usize>,
    message: String,
}

impl Violation {
    fn new(path: PathBuf, line: Option<usize>, message: impl Into<String>) -> Self {
        Self {
            path,
            line,
            message: message.into(),
        }
    }
}

impl Display for Violation {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self.line {
            Some(line) => write!(
                formatter,
                "{}:{line}: {}",
                self.path.display(),
                self.message
            ),
            None => write!(formatter, "{}: {}", self.path.display(), self.message),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn reports_size_paths_reexports_and_clippy_bypasses() {
        let directory = tempdir().unwrap();
        let source = [
            format!("{} {} thing;", "pub", "use"),
            format!("use {}::item;", ["super", "super"].join("::")),
            format!("#![{}(clippy::all)]", "allow"),
        ]
        .join("\n");
        let path = directory.path().join("bad.rs");
        fs::write(&path, source).unwrap();
        let mut violations = Vec::new();
        audit_file(directory.path(), &path, &mut violations).unwrap();
        assert_eq!(violations.len(), 3);
        assert_eq!(violations[0].line, Some(1));
    }

    #[test]
    fn source_discovery_skips_build_and_generated_directories() {
        let directory = tempdir().unwrap();
        for name in ["src", "target", "generated", "vendor"] {
            fs::create_dir(directory.path().join(name)).unwrap();
            fs::write(directory.path().join(name).join("file.rs"), "fn item() {}").unwrap();
        }
        let mut sources = Vec::new();
        collect_rust_sources(directory.path(), directory.path(), &mut sources).unwrap();
        assert_eq!(sources, vec![directory.path().join("src/file.rs")]);
    }
}
