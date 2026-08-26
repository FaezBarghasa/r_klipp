use actix_web::{test, web, App};
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, RwLock};

use host_server::api::{
    get_printer_info, get_server_files_list, get_server_info, post_gcode_script,
    post_jsonrpc, query_printer_objects, AppState, MachineState,
};
use host_server::db::Database;
use host_server::components::{FileManager, MetadataParser};
use host_server::config::parse_case_sensitive_ini;

#[actix_web::test]
async fn test_moonraker_jsonrpc_endpoint() {
    let db_path = "/tmp/r_klipp_test_moonraker_jsonrpc.db";
    let _ = std::fs::remove_dir_all(db_path);

    let database = Database::new(db_path).await.expect("DB init");
    database.init_schema().await.expect("DB schema");

    let (telemetry_tx, _) = broadcast::channel(128);
    let (mcu_tx, _) = mpsc::channel(128);
    let machine_state = Arc::new(RwLock::new(MachineState::default()));

    let app_state = web::Data::new(AppState {
        db: Arc::new(database),
        telemetry_broadcaster: telemetry_tx,
        mcu_cmd_sender: mcu_tx,
        machine_state,
        file_manager: Arc::new(FileManager::new("/tmp/test_gcodes", "/tmp/test_config")),
        job_queue: Arc::new(host_server::components::JobQueue::new()),
        data_store: Arc::new(host_server::components::DataStore::new(600.0)),
    });

    let app = test::init_service(
        App::new()
            .app_data(app_state)
            .route("/server/jsonrpc", web::post().to(post_jsonrpc)),
    )
    .await;

    // 1. Test server.info via JSON-RPC
    let req = test::TestRequest::post()
        .uri("/server/jsonrpc")
        .set_json(serde_json::json!({
            "jsonrpc": "2.0",
            "method": "server.info",
            "id": 42
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    let json_resp: serde_json::Value = test::read_body_json(resp).await;

    assert_eq!(json_resp["jsonrpc"], "2.0");
    assert_eq!(json_resp["id"], 42);
    assert_eq!(json_resp["result"]["klippy_state"], "ready");
    assert_eq!(json_resp["result"]["klippy_connected"], true);

    // 2. Test printer.objects.query via JSON-RPC
    let req = test::TestRequest::post()
        .uri("/server/jsonrpc")
        .set_json(serde_json::json!({
            "jsonrpc": "2.0",
            "method": "printer.objects.query",
            "id": 43
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    let json_resp: serde_json::Value = test::read_body_json(resp).await;

    assert_eq!(json_resp["id"], 43);
    assert!(json_resp["result"]["status"]["toolhead"].is_object());
    assert!(json_resp["result"]["status"]["extruder"].is_object());

    // 3. Test unknown method error
    let req = test::TestRequest::post()
        .uri("/server/jsonrpc")
        .set_json(serde_json::json!({
            "jsonrpc": "2.0",
            "method": "nonexistent.method",
            "id": 44
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    let json_resp: serde_json::Value = test::read_body_json(resp).await;

    assert_eq!(json_resp["error"]["code"], -32601);

    let _ = std::fs::remove_dir_all(db_path);
}

#[tokio::test]
async fn test_file_manager_and_ini_parity() {
    let conf = r#"
[server]
host: 127.0.0.1
port: 7125

[file_manager]
config_path: ~/printer_data/config
"#;
    let parsed = parse_case_sensitive_ini(conf).expect("parse ini");
    assert_eq!(parsed["server"]["host"], "127.0.0.1");
    assert_eq!(parsed["file_manager"]["config_path"], "~/printer_data/config");

    let fm = FileManager::new("/tmp/test_gcodes", "/tmp/test_config");
    let safe_path = fm.sanitize_path(std::path::Path::new("/tmp/test_gcodes"), "test.gcode").unwrap();
    assert_eq!(safe_path, std::path::PathBuf::from("/tmp/test_gcodes/test.gcode"));

    // Traversal test
    assert!(fm.sanitize_path(std::path::Path::new("/tmp/test_gcodes"), "../../../etc/passwd").is_err());
}
