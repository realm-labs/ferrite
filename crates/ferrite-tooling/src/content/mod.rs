mod artifact;
mod classify;
mod importer;
mod model;
mod source;

use anyhow::{Context as _, Result, bail, ensure};
use ferrite_registry::minecraft_block::MinecraftBlockCatalog;
use std::ffi::OsStr;
use std::fs;
use std::io::Write;
use std::path::{Component, Path, PathBuf};

const DEFAULT_SOURCE: &str = "target/mc-reference/26.2";
const DEFAULT_BUNDLE: &str = "target/ferrite-content/26.2/content-bundle.json";

pub(crate) fn run(workspace: &Path, command: &str, arguments: &[String]) -> Result<()> {
    match command {
        "import" => import_command(workspace, parse_options(arguments, true)?),
        "verify" => verify_command(workspace, parse_options(arguments, false)?),
        _ => bail!("unknown content command {command:?}; expected import or verify"),
    }
}

pub(crate) fn verify(workspace: &Path) -> Result<()> {
    verify_command(workspace, ContentOptions::default())
}

fn import_command(workspace: &Path, options: ContentOptions) -> Result<()> {
    let source = resolve_input(
        workspace,
        options.source.as_deref().unwrap_or(DEFAULT_SOURCE),
    );
    ensure!(
        source.is_dir(),
        "content source directory does not exist: {}",
        source.display()
    );
    let output = resolve_output(
        workspace,
        options.bundle.as_deref().unwrap_or(DEFAULT_BUNDLE),
    )?;
    let imported = importer::import(workspace, &source)?;
    if let Some(lock) = importer::read_bundle_lock(workspace)? {
        importer::verify_lock(&imported, &lock)?;
    } else {
        println!("no {}; candidate lock follows:", importer::BUNDLE_LOCK);
        importer::print_candidate(&imported)?;
    }

    for artifact in imported.bundle.source_artifacts() {
        println!("verified {}", importer::artifact_summary(artifact));
    }
    write_validated_bundle(workspace, &output, &imported)?;
    println!(
        "content bundle imported: {} registries, {} entries, digest {}, output {}",
        imported.registries,
        imported.entries,
        imported.bundle.digest()?,
        output.display()
    );
    Ok(())
}

fn verify_command(workspace: &Path, options: ContentOptions) -> Result<()> {
    ensure!(
        options.source.is_none(),
        "--source is only valid for content import"
    );
    let bundle = resolve_output(
        workspace,
        options.bundle.as_deref().unwrap_or(DEFAULT_BUNDLE),
    )?;
    let lock = importer::read_bundle_lock(workspace)?
        .context("content bundle lock is missing; import once and commit the reviewed digest")?;
    let imported = importer::load(&bundle)?;
    importer::verify_lock(&imported, &lock)?;
    let block_registry = imported
        .bundle
        .registries()
        .find(|registry| registry.name().to_string() == "minecraft:block")
        .context("content bundle has no minecraft:block registry")?;
    let block_catalog = MinecraftBlockCatalog::from_registry(block_registry)
        .context("lower imported block-state catalog")?;
    let block_states = block_catalog
        .definitions()
        .map(|definition| u64::from(definition.schema().state_count()))
        .sum::<u64>();
    println!(
        "content bundle verified: {} registries, {} entries, digest {}; {} block definitions, \
         {block_states} canonical states",
        imported.registries,
        imported.entries,
        imported.bundle.digest()?,
        block_catalog.definitions().len(),
    );
    Ok(())
}

fn write_validated_bundle(
    workspace: &Path,
    output: &Path,
    imported: &importer::ImportedBundle,
) -> Result<()> {
    let parent = output.parent().context("bundle output has no parent")?;
    fs::create_dir_all(parent)
        .with_context(|| format!("create bundle directory {}", parent.display()))?;
    let part = output.with_extension("json.part");
    if part.exists() {
        fs::remove_file(&part)
            .with_context(|| format!("remove stale generated part {}", part.display()))?;
    }
    let bytes = serde_json::to_vec(&imported.bundle).context("encode content bundle")?;
    let mut file = fs::File::create(&part).with_context(|| format!("create {}", part.display()))?;
    file.write_all(&bytes)
        .with_context(|| format!("write {}", part.display()))?;
    file.sync_all()
        .with_context(|| format!("sync {}", part.display()))?;

    let candidate = importer::load(&part)?;
    ensure!(
        candidate.bundle.digest()? == imported.bundle.digest()?,
        "written content bundle failed digest round trip"
    );
    if let Some(lock) = importer::read_bundle_lock(workspace)? {
        importer::verify_lock(&candidate, &lock)?;
    }
    if output.exists() {
        fs::remove_file(output)
            .with_context(|| format!("replace generated bundle {}", output.display()))?;
    }
    fs::rename(&part, output)
        .with_context(|| format!("install generated bundle {}", output.display()))?;
    Ok(())
}

#[derive(Debug, Default)]
struct ContentOptions {
    source: Option<String>,
    bundle: Option<String>,
}

fn parse_options(arguments: &[String], allow_source: bool) -> Result<ContentOptions> {
    let mut options = ContentOptions::default();
    let mut index = 0;
    while index < arguments.len() {
        let option = &arguments[index];
        let value = arguments
            .get(index + 1)
            .with_context(|| format!("{option} requires a path"))?;
        match option.as_str() {
            "--source" if allow_source => set_once(&mut options.source, value, option)?,
            "--output" if allow_source => set_once(&mut options.bundle, value, option)?,
            "--bundle" if !allow_source => set_once(&mut options.bundle, value, option)?,
            _ => bail!("unknown or invalid content option {option:?}"),
        }
        index += 2;
    }
    Ok(options)
}

fn set_once(target: &mut Option<String>, value: &str, option: &str) -> Result<()> {
    ensure!(target.is_none(), "{option} may be specified only once");
    *target = Some(value.to_owned());
    Ok(())
}

fn resolve_input(workspace: &Path, value: &str) -> PathBuf {
    let path = Path::new(value);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        workspace.join(path)
    }
}

fn resolve_output(workspace: &Path, value: &str) -> Result<PathBuf> {
    let path = Path::new(value);
    ensure!(
        !path
            .components()
            .any(|component| matches!(component, Component::ParentDir)),
        "bundle output cannot contain parent traversal"
    );
    let output = if path.is_absolute() {
        path.to_path_buf()
    } else {
        workspace.join(path)
    };
    let root = workspace.join("target/ferrite-content");
    ensure!(
        output.starts_with(&root),
        "bundle output must remain inside {}",
        root.display()
    );
    ensure!(
        output.extension() == Some(OsStr::new("json")),
        "bundle output must use a .json extension"
    );
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_is_contained_in_the_generated_content_root() {
        let workspace = Path::new("C:/workspace");
        assert!(resolve_output(workspace, DEFAULT_BUNDLE).is_ok());
        assert!(resolve_output(workspace, "target/other/bundle.json").is_err());
        assert!(resolve_output(workspace, "target/ferrite-content/../escape.json").is_err());
    }

    #[test]
    fn command_options_are_single_use_and_command_specific() {
        assert!(
            parse_options(
                &[
                    "--source".to_owned(),
                    "cache".to_owned(),
                    "--output".to_owned(),
                    "target/ferrite-content/test.json".to_owned()
                ],
                true
            )
            .is_ok()
        );
        assert!(parse_options(&["--source".to_owned(), "cache".to_owned()], false).is_err());
    }
}
