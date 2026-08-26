use actix_web::{test, web, App};
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, RwLock};

use host_server::api::{
    get_printer_info, get_server_files_list, get_server_info, post_gcode_script,
    query_printer_objects, AppState, MachineState,
};
use host_server::bridge::HostToMcu;
use host_server::db::Database;

#[actix_web::test]
async fn test_printer_info_endpoint() {
    let db_path = "/tmp/r_klipp_test_api_info.db";
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
        file_manager: Arc::new(host_server::components::FileManager::new("/tmp/test_gcodes", "/tmp/test_config")),
        job_queue: Arc::new(host_server::components::JobQueue::new()),
        data_store: Arc::new(host_server::components::DataStore::new(600.0)),
        machine_mgr: Arc::new(host_server::components::MachineManager::new()),
        power_mgr: Arc::new(host_server::components::PowerManager::new()),
        update_mgr: Arc::new(host_server::components::UpdateManager::new()),
        spoolman: Arc::new(host_server::components::SpoolmanClient::new()),
    });

    let app = test::init_service(
        App::new()
            .app_data(app_state)
            .route("/printer/info", web::get().to(get_printer_info))
            .route("/server/info", web::get().to(get_server_info)),
    )
    .await;

    let req = test::TestRequest::get().uri("/printer/info").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["result"]["state"], "ready");
    assert_eq!(body["result"]["software_version"], env!("CARGO_PKG_VERSION"));

    let req_server = test::TestRequest::get().uri("/server/info").to_request();
    let resp_server = test::call_service(&app, req_server).await;
    assert!(resp_server.status().is_success());

    let body_server: serde_json::Value = test::read_body_json(resp_server).await;
    assert_eq!(body_server["result"]["klippy_state"], "ready");
    assert_eq!(body_server["result"]["klippy_connected"], true);

    let _ = std::fs::remove_dir_all(db_path);
}

#[actix_web::test]
async fn test_objects_query_and_gcode_script() {
    let db_path = "/tmp/r_klipp_test_api_objects.db";
    let _ = std::fs::remove_dir_all(db_path);

    let database = Database::new(db_path).await.expect("DB init");
    database.init_schema().await.expect("DB schema");

    let (telemetry_tx, _) = broadcast::channel(128);
    let (mcu_tx, mut mcu_rx) = mpsc::channel(128);
    let machine_state = Arc::new(RwLock::new(MachineState::default()));

    let app_state = web::Data::new(AppState {
        db: Arc::new(database),
        telemetry_broadcaster: telemetry_tx,
        mcu_cmd_sender: mcu_tx,
        machine_state,
        file_manager: Arc::new(host_server::components::FileManager::new("/tmp/test_gcodes", "/tmp/test_config")),
        job_queue: Arc::new(host_server::components::JobQueue::new()),
        data_store: Arc::new(host_server::components::DataStore::new(600.0)),
        machine_mgr: Arc::new(host_server::components::MachineManager::new()),
        power_mgr: Arc::new(host_server::components::PowerManager::new()),
        update_mgr: Arc::new(host_server::components::UpdateManager::new()),
        spoolman: Arc::new(host_server::components::SpoolmanClient::new()),
    });

    let app = test::init_service(
        App::new()
            .app_data(app_state)
            .route("/printer/objects/query", web::get().to(query_printer_objects))
            .route("/printer/gcode/script", web::post().to(post_gcode_script))
            .route("/server/files/list", web::get().to(get_server_files_list)),
    )
    .await;

    // 1. Query objects
    let req_query = test::TestRequest::get().uri("/printer/objects/query").to_request();
    let resp_query = test::call_service(&app, req_query).await;
    assert!(resp_query.status().is_success());

    let body_query: serde_json::Value = test::read_body_json(resp_query).await;
    assert!(body_query["result"]["status"]["extruder"]["temperature"].is_number());
    assert!(body_query["result"]["status"]["heater_bed"]["temperature"].is_number());

    // 2. Post G-Code script
    let gcode_payload = serde_json::json!({
        "script": "G28\nM104 S210\n"
    });

    let req_post = test::TestRequest::post()
        .uri("/printer/gcode/script")
        .set_json(&gcode_payload)
        .to_request();
    let resp_post = test::call_service(&app, req_post).await;
    assert!(resp_post.status().is_success());

    // Verify commands forwarded to mcu channel
    let cmd1 = mcu_rx.recv().await.expect("Received G28");
    assert_eq!(cmd1, HostToMcu::GCode("G28".to_string()));

    let cmd2 = mcu_rx.recv().await.expect("Received M104");
    assert_eq!(cmd2, HostToMcu::GCode("M104 S210".to_string()));

    let _ = std::fs::remove_dir_all(db_path);
}
