#[test]
fn test_workspace_crates_initialization() {
    let cas = nexusedge_cas::CasEngine::new(".cache");
    assert_eq!(cas.storage_dir().to_str().unwrap(), ".cache");

    let log = nexusedge_crdt::CrdtLog::new();
    assert_eq!(log.records().len(), 0);

    let vec = nexusedge_telemetry::TelemetryCollector::poll_resource_vector();
    assert_eq!(vec.thermal_status, "Normal");
}
