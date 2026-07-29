use crate::artifact::{extract_server, path_sha1};
use crate::verification::verify_cached_artifacts;
use crate::*;

pub(crate) fn symbols(context: &Context) -> Result<()> {
    verify_cached_artifacts(context)?;
    let server = extract_server(context)?;
    let client = context.cache.join("client.jar");
    let javap = env::var("MC_REF_JAVAP").unwrap_or_else(|_| "javap".into());
    let symbol_regex = Regex::new(
        r"`(?P<class>net\.minecraft\.[A-Za-z0-9_.$]+)#(?P<member>[A-Za-z0-9_$<>]+)(?P<params>\([^`]*\))?`",
    )?;
    let mut symbols = BTreeSet::new();
    for file in markdown_files(&context.reference) {
        let text = fs::read_to_string(&file)?;
        for captures in symbol_regex.captures_iter(&text) {
            symbols.insert((
                captures["class"].to_string(),
                captures["member"].to_string(),
                captures.name("params").map(|m| m.as_str().to_string()),
            ));
        }
    }
    ensure!(
        !symbols.is_empty(),
        "no source symbols found in documentation"
    );
    let classes = symbols
        .iter()
        .map(|(class, _, _)| class.clone())
        .collect::<BTreeSet<_>>();
    let javap_identity = javap_identity(&javap)?;
    let server_cache =
        symbol_cache_directory(context, &path_sha1(&server)?, &javap, &javap_identity);
    let client_cache =
        symbol_cache_directory(context, &context.lock.client.sha1, &javap, &javap_identity);
    let mut cache = BTreeMap::<String, String>::new();
    let mut missing_server = Vec::new();
    let mut missing_client = Vec::new();
    let mut cache_hits = 0;
    for class in &classes {
        let (directory, missing) = if class.starts_with("net.minecraft.client.") {
            (&client_cache, &mut missing_client)
        } else {
            (&server_cache, &mut missing_server)
        };
        if let Some(output) = read_symbol_cache(directory, class)? {
            cache.insert(class.clone(), output);
            cache_hits += 1;
        } else {
            missing.push(class.clone());
        }
    }
    let mut javap_batches = 0;
    javap_batches +=
        populate_symbol_cache(&javap, &server, &server_cache, &missing_server, &mut cache)?;
    javap_batches +=
        populate_symbol_cache(&javap, &client, &client_cache, &missing_client, &mut cache)?;
    for (class, member, params) in &symbols {
        let output = cache
            .get(class)
            .with_context(|| format!("missing javap output for {class}"))?;
        ensure!(
            output.contains(member),
            "symbol not found: {class}#{member}"
        );
        if let Some(params) = params {
            ensure!(
                descriptor_matches(output, member, params),
                "method overload not found: {class}#{member}{params}"
            );
        }
    }
    println!(
        "symbols verified: {} locators across {} classes ({} cache hits, {} misses, {} javap batches)",
        symbols.len(),
        cache.len(),
        cache_hits,
        classes.len() - cache_hits,
        javap_batches
    );
    Ok(())
}

fn javap_identity(javap: &str) -> Result<String> {
    let output = ProcessCommand::new(javap)
        .arg("-version")
        .output()
        .with_context(|| format!("could not execute {javap} -version"))?;
    ensure!(
        output.status.success(),
        "{javap} -version failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let identity = format!(
        "{}{}",
        String::from_utf8(output.stdout)?,
        String::from_utf8(output.stderr)?
    );
    let identity = identity.trim();
    ensure!(
        !identity.is_empty(),
        "{javap} -version returned no identity"
    );
    Ok(identity.to_string())
}

pub(crate) fn symbol_cache_directory(
    context: &Context,
    jar_sha1: &str,
    javap: &str,
    javap_identity: &str,
) -> PathBuf {
    let tool_key = sha1_bytes(
        format!("{SYMBOL_CACHE_VERSION}\0{javap}\0{javap_identity}\0-sysinfo\0-p\0-s").as_bytes(),
    );
    context
        .cache
        .join("symbol-cache")
        .join(SYMBOL_CACHE_VERSION)
        .join(jar_sha1)
        .join(tool_key)
}

pub(crate) fn symbol_cache_file(directory: &Path, class: &str) -> PathBuf {
    directory.join(format!("{}.txt", sha1_bytes(class.as_bytes())))
}

pub(crate) fn read_symbol_cache(directory: &Path, class: &str) -> Result<Option<String>> {
    let path = symbol_cache_file(directory, class);
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error).with_context(|| format!("cannot read {}", path.display())),
    };
    let Some((header, output)) = text.split_once('\n') else {
        return Ok(None);
    };
    let fields = header.split('\t').collect::<Vec<_>>();
    if fields.as_slice()
        != [
            SYMBOL_CACHE_HEADER,
            class,
            sha1_bytes(output.as_bytes()).as_str(),
        ]
    {
        return Ok(None);
    }
    Ok(Some(output.to_string()))
}

pub(crate) fn write_symbol_cache(directory: &Path, class: &str, output: &str) -> Result<()> {
    fs::create_dir_all(directory)?;
    let path = symbol_cache_file(directory, class);
    let temporary = path.with_extension(format!("tmp-{}", process::id()));
    fs::write(
        &temporary,
        format!(
            "{SYMBOL_CACHE_HEADER}\t{class}\t{}\n{output}",
            sha1_bytes(output.as_bytes())
        ),
    )?;
    if path.is_file() {
        fs::remove_file(&path)?;
    }
    fs::rename(&temporary, &path)?;
    Ok(())
}

fn populate_symbol_cache(
    javap: &str,
    jar: &Path,
    directory: &Path,
    classes: &[String],
    cache: &mut BTreeMap<String, String>,
) -> Result<usize> {
    let mut batches = 0;
    for batch in classes.chunks(JAVAP_BATCH_SIZE) {
        let output = ProcessCommand::new(javap)
            .args(["-sysinfo", "-p", "-s", "-classpath"])
            .arg(jar)
            .args(batch)
            .output()
            .with_context(|| format!("could not execute {javap} for {}", jar.display()))?;
        ensure!(
            output.status.success(),
            "javap could not resolve a class batch from {}: {}",
            jar.display(),
            String::from_utf8_lossy(&output.stderr)
        );
        let text = String::from_utf8_lossy(&output.stdout);
        let parsed = parse_javap_batch(&text, batch)?;
        for (class, output) in parsed {
            write_symbol_cache(directory, &class, &output)?;
            cache.insert(class, output);
        }
        batches += 1;
    }
    Ok(batches)
}

pub(crate) fn parse_javap_batch(
    output: &str,
    classes: &[String],
) -> Result<BTreeMap<String, String>> {
    let mut starts = output
        .match_indices("Classfile ")
        .filter_map(|(index, _)| {
            (index == 0 || output.as_bytes().get(index.wrapping_sub(1)) == Some(&b'\n'))
                .then_some(index)
        })
        .collect::<Vec<_>>();
    ensure!(
        starts.len() == classes.len(),
        "javap returned {} class sections for {} requested classes",
        starts.len(),
        classes.len()
    );
    starts.push(output.len());
    let mut parsed = BTreeMap::new();
    for bounds in starts.windows(2) {
        let section = &output[bounds[0]..bounds[1]];
        let marker = section.lines().next().unwrap_or_default();
        let class = classes
            .iter()
            .find(|class| marker.ends_with(&format!("!/{}.class", class.replace('.', "/"))))
            .with_context(|| format!("cannot identify javap class section {marker}"))?;
        ensure!(
            parsed.insert(class.clone(), section.to_string()).is_none(),
            "javap returned duplicate class section for {class}"
        );
    }
    ensure!(
        parsed.len() == classes.len(),
        "javap output omitted one or more requested classes"
    );
    Ok(parsed)
}

pub(crate) fn descriptor_matches(output: &str, member: &str, parameters: &str) -> bool {
    let expected: String = parameters
        .trim_matches(['(', ')'])
        .split(',')
        .filter(|value| !value.trim().is_empty())
        .map(java_type_descriptor)
        .collect();
    let lines: Vec<_> = output.lines().collect();
    lines.windows(2).any(|pair| {
        pair[0].contains(&format!(" {member}("))
            && pair[1]
                .trim()
                .strip_prefix("descriptor: (")
                .and_then(|value| value.split_once(')'))
                .is_some_and(|(actual, _)| actual == expected)
    })
}

fn java_type_descriptor(value: &str) -> String {
    let value = value.trim();
    if let Some(component) = value.strip_suffix("[]") {
        return format!("[{}", java_type_descriptor(component));
    }
    match value {
        "boolean" => "Z".into(),
        "byte" => "B".into(),
        "char" => "C".into(),
        "short" => "S".into(),
        "int" => "I".into(),
        "long" => "J".into(),
        "float" => "F".into(),
        "double" => "D".into(),
        _ => format!("L{};", value.replace('.', "/")),
    }
}
