use ferrite_server_runtime::config::{SERVER_CONFIG_SCHEMA, ServerConfig};
use std::fs;
use std::process::Command;

#[test]
fn cli_migrates_schema_one_to_a_new_schema_two_file_without_overwrite() {
    let temporary = tempfile::tempdir().unwrap();
    let config =
        ServerConfig::development_node(1, 1, 31_000, &temporary.path().join("state")).unwrap();
    let mut legacy: toml::Value = config.to_toml().unwrap().parse().unwrap();
    let table = legacy.as_table_mut().unwrap();
    table.insert("schema_version".to_owned(), toml::Value::Integer(1));
    table.remove("world");

    let input = temporary.path().join("server-v1.toml");
    let output = temporary.path().join("server-v2.toml");
    fs::write(&input, toml::to_string(&legacy).unwrap()).unwrap();
    let binary = env!("CARGO_BIN_EXE_ferrite-server");

    assert!(
        Command::new(binary)
            .args(["--config", input.to_str().unwrap(), "--migrate-config"])
            .arg(&output)
            .status()
            .unwrap()
            .success()
    );
    let migrated = ServerConfig::load(&output).unwrap();
    assert_eq!(migrated.config().schema_version, SERVER_CONFIG_SCHEMA);
    assert_eq!(migrated.world_id().get(), 1);

    assert!(
        !Command::new(binary)
            .args(["--config", input.to_str().unwrap(), "--migrate-config"])
            .arg(&output)
            .status()
            .unwrap()
            .success()
    );
}
