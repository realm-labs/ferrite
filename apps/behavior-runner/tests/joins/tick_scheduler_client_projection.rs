use ferrite_testkit::phase9::joins::{JoinOracle, run_tick_scheduler_client_projection};

#[test]
fn projection_flush_follows_authoritative_tick_commit() {
    let report = run_tick_scheduler_client_projection();
    assert_eq!(report.oracle, JoinOracle::TickThenFlush);
    assert_eq!(report.checkpoints, 2);
}
