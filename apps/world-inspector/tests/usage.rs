use std::process::Command;

#[test]
fn missing_arguments_report_the_stable_usage_contract() {
    let output = Command::new(env!("CARGO_BIN_EXE_world-inspector"))
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("<store-directory> <world-id-hex> <dimension> <region-x> <region-z>"));
}
