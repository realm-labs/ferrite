use std::process::Command;

#[test]
fn cluster_command_verifies_process_isolated_playable_equivalence() {
    let output = Command::new(env!("CARGO_BIN_EXE_ferrite-cluster"))
        .arg("verify-playable")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("playable equivalence verified:"));
    assert!(stdout.contains("processes=3"));
}
