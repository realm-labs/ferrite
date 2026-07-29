use ferrite_registry::bundle::ContentBundle;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use toml::Value;

static IMPORTED_BUNDLE: OnceLock<Option<ContentBundle>> = OnceLock::new();

pub(crate) fn verify_category(kind: &str, count: usize, digest: &str) {
    let catalog_path = workspace().join("docs/reference/minecraft-java-26.2/catalog/catalog.toml");
    let catalog = fs::read_to_string(&catalog_path).unwrap();
    let document = toml::from_str::<Value>(&catalog).unwrap();
    let category = document
        .get("category")
        .and_then(Value::as_array)
        .unwrap()
        .iter()
        .find(|category| category.get("kind").and_then(Value::as_str) == Some(kind))
        .unwrap_or_else(|| panic!("catalog has no {kind} category"));
    assert_eq!(
        category.get("expected_count").and_then(Value::as_integer),
        Some(count as i64)
    );
    assert_eq!(
        category.get("ids_sha1").and_then(Value::as_str),
        Some(digest)
    );
    assert!(
        category
            .get("family")
            .and_then(Value::as_array)
            .is_some_and(|families| !families.is_empty())
    );

    if let Some(bundle) = imported_bundle() {
        verify_imported_registry(bundle, kind, count, digest);
    }
}

fn imported_bundle() -> Option<&'static ContentBundle> {
    IMPORTED_BUNDLE
        .get_or_init(|| {
            env::var_os("FERRITE_CONTENT_BUNDLE").map(|path| {
                let file = fs::File::open(PathBuf::from(path)).unwrap();
                serde_json::from_reader::<_, ContentBundle>(file).unwrap()
            })
        })
        .as_ref()
}

fn verify_imported_registry(bundle: &ContentBundle, kind: &str, count: usize, digest: &str) {
    let name = format!("minecraft:{kind}");
    let registry = bundle
        .registries()
        .find(|registry| registry.name().to_string() == name)
        .unwrap_or_else(|| panic!("content bundle has no {name} registry"));
    assert_eq!(registry.entries().len(), count);
    assert_eq!(registry.ids_sha1().as_str(), digest);
    assert!(registry.entries().all(|entry| {
        registry
            .families()
            .any(|family| family.name() == entry.family())
    }));
}

fn workspace() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap()
        .to_path_buf()
}
