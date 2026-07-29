use crate::verification::verify_cached_artifacts;
use crate::*;

pub(crate) fn fetch(context: &Context, version: &str) -> Result<()> {
    ensure!(
        version == context.lock.version,
        "only locked version {} is accepted",
        context.lock.version
    );
    fs::create_dir_all(&context.cache)?;
    let client = Client::builder()
        .user_agent("Ferrite mc-reference/0.1")
        .build()?;
    let manifest_bytes = get(&client, &context.lock.manifest_url)?;
    let manifest: Manifest = serde_json::from_slice(&manifest_bytes)?;
    let metadata_is_current =
        manifest_metadata_is_current(&manifest, version, &context.lock.metadata)?;
    write_verified(
        &context.cache.join("version_manifest_v2.json"),
        &manifest_bytes,
        None,
        None,
    )?;
    if !metadata_is_current {
        eprintln!(
            "warning: the live manifest now points {version} at revised launcher metadata; \
             fetching the SHA-1-locked metadata instead"
        );
    }

    let metadata_bytes = get(&client, &context.lock.metadata.url)?;
    write_verified(
        &context.cache.join("version.json"),
        &metadata_bytes,
        Some(&context.lock.metadata.sha1),
        None,
    )?;
    let metadata: VersionMetadata = serde_json::from_slice(&metadata_bytes)?;
    for (name, locked) in [
        ("client", &context.lock.client),
        ("server", &context.lock.server),
    ] {
        let declared = metadata
            .downloads
            .get(name)
            .with_context(|| format!("metadata has no {name} download"))?;
        ensure!(
            declared.url == locked.url && declared.sha1 == locked.sha1,
            "{name} metadata differs from lock"
        );
        ensure!(
            locked.size == Some(declared.size),
            "{name} size differs from lock"
        );
        download_file(&client, locked, &context.cache.join(format!("{name}.jar")))?;
    }
    println!(
        "fetched and verified Minecraft Java {} (data pack {}, resource pack {})",
        context.lock.version, context.lock.data_pack, context.lock.resource_pack
    );
    Ok(())
}

pub(crate) fn manifest_metadata_is_current(
    manifest: &Manifest,
    version: &str,
    locked: &Artifact,
) -> Result<bool> {
    let entry = manifest
        .versions
        .iter()
        .find(|entry| entry.id == version)
        .with_context(|| format!("{version} is absent from the official manifest"))?;
    Ok(entry.url == locked.url && entry.sha1 == locked.sha1)
}

pub(crate) fn get(client: &Client, url: &str) -> Result<Vec<u8>> {
    let response = client.get(url).send()?.error_for_status()?;
    Ok(response.bytes()?.to_vec())
}

fn download_file(client: &Client, artifact: &Artifact, destination: &Path) -> Result<()> {
    if destination.is_file() && verify_file(destination, &artifact.sha1, artifact.size).is_ok() {
        return Ok(());
    }
    let part = destination.with_extension("jar.part");
    let mut response = client.get(&artifact.url).send()?.error_for_status()?;
    let mut output = File::create(&part)?;
    io::copy(&mut response, &mut output)?;
    output.flush()?;
    verify_file(&part, &artifact.sha1, artifact.size)?;
    fs::rename(part, destination)?;
    Ok(())
}

fn write_verified(path: &Path, bytes: &[u8], sha1: Option<&str>, size: Option<u64>) -> Result<()> {
    if let Some(expected) = sha1 {
        ensure!(
            sha1_bytes(bytes) == expected,
            "SHA-1 mismatch for {}",
            path.display()
        );
    }
    if let Some(expected) = size {
        ensure!(
            bytes.len() as u64 == expected,
            "size mismatch for {}",
            path.display()
        );
    }
    fs::write(path, bytes)?;
    Ok(())
}

pub(crate) fn verify_file(
    path: &Path,
    expected_sha1: &str,
    expected_size: Option<u64>,
) -> Result<()> {
    let file = File::open(path).with_context(|| format!("missing {}", path.display()))?;
    if let Some(expected) = expected_size {
        ensure!(
            file.metadata()?.len() == expected,
            "size mismatch for {}",
            path.display()
        );
    }
    ensure!(
        file_sha1(file)? == expected_sha1,
        "SHA-1 mismatch for {}",
        path.display()
    );
    Ok(())
}

fn file_sha1(file: File) -> Result<String> {
    let mut reader = BufReader::new(file);
    let mut hasher = Sha1::new();
    io::copy(&mut reader, &mut hasher)?;
    Ok(hex::encode(hasher.finalize()))
}

pub(crate) fn path_sha1(path: &Path) -> Result<String> {
    file_sha1(File::open(path).with_context(|| format!("missing {}", path.display()))?)
}

pub(crate) fn reports(context: &Context) -> Result<()> {
    verify_cached_artifacts(context)?;
    let output = context.cache.join("generated");
    fs::create_dir_all(&output)?;
    let java = env::var("MC_REF_JAVA").unwrap_or_else(|_| "java".into());
    check_java_major(&java, context.lock.java_major)?;
    let status = ProcessCommand::new(java)
        .current_dir(&context.cache)
        .arg("-DbundlerMainClass=net.minecraft.data.Main")
        .arg("-jar")
        .arg(context.cache.join("server.jar"))
        .arg("--reports")
        .arg("--output")
        .arg(&output)
        .status()?;
    ensure!(
        status.success(),
        "official report generator exited with {status}"
    );
    ensure!(
        output.join("reports/blocks.json").is_file(),
        "report generation did not produce blocks.json"
    );
    extract_server(context)?;
    println!("reports generated in {}", output.display());
    Ok(())
}

fn check_java_major(java: &str, expected: u32) -> Result<()> {
    let output = ProcessCommand::new(java).arg("-version").output()?;
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let regex = Regex::new(r#"version "(\d+)"#)?;
    let actual: u32 = regex
        .captures(&combined)
        .context("cannot parse java -version")?[1]
        .parse()?;
    ensure!(
        actual == expected,
        "Java {expected} required, found Java {actual}; set MC_REF_JAVA"
    );
    Ok(())
}

pub(crate) fn extract_server(context: &Context) -> Result<PathBuf> {
    let destination = context
        .cache
        .join(format!("server-{}.jar", context.lock.version));
    if destination.is_file() {
        return Ok(destination);
    }
    let input = File::open(context.cache.join("server.jar"))?;
    let mut archive = ZipArchive::new(input)?;
    let suffix = format!("/server-{}.jar", context.lock.version);
    let index = (0..archive.len())
        .find(|index| {
            archive
                .by_index(*index)
                .map(|file| file.name().ends_with(&suffix))
                .unwrap_or(false)
        })
        .context("bundled server jar not found")?;
    let mut member = archive.by_index(index)?;
    let mut output = File::create(&destination)?;
    io::copy(&mut member, &mut output)?;
    Ok(destination)
}
