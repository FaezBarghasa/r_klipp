use actix_web::{test, web, App};
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, RwLock};

use host_server::api::{
    get_printer_info, get_server_files_list, get_server_info, post_gcode_script,
    query_printer_objects, AppState, MachineState,
};
use host_server::db::Database;

#[actix_web::test]
async fn test_moonraker_schema_conformance() {
    let db_path = "/tmp/r_klipp_test_moonraker_schema.db";
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
    });

    let app = test::init_service(
        App::new()
            .app_data(app_state)
            .route("/printer/info", web::get().to(get_printer_info))
            .route("/server/info", web::get().to(get_server_info))
            .route("/printer/objects/query", web::get().to(query_printer_objects))
            .route("/server/files/list", web::get().to(get_server_files_list)),
    )
    .await;

    // 1. /printer/info schema
    let req = test::TestRequest::get().uri("/printer/info").to_request();
    let resp = test::call_service(&app, req).await;
    let info: serde_json::Value = test::read_body_json(resp).await;

    assert!(info.get("result").is_some());
    let res = &info["result"];
    assert!(res.get("state").is_some());
    assert!(res.get("state_message").is_some());
    assert!(res.get("hostname").is_some());
    assert!(res.get("software_version").is_some());

    // 2. /server/info schema
    let req = test::TestRequest::get().uri("/server/info").to_request();
    let resp = test::call_service(&app, req).await;
    let srv_info: serde_json::Value = test::read_body_json(resp).await;

    let srv_res = &srv_info["result"];
    assert!(srv_res.get("klippy_state").is_some());
    assert!(srv_res.get("klippy_connected").is_some());
    assert!(srv_res.get("api_version").is_some());

    // 3. /printer/objects/query status keys
    let req = test::TestRequest::get().uri("/printer/objects/query").to_request();
    let resp = test::call_service(&app, req).await;
    let objects: serde_json::Value = test::read_body_json(resp).await;

    let status = &objects["result"]["status"];
    assert!(status.get("webhooks").is_some());
    assert!(status.get("extruder").is_some());
    assert!(status.get("heater_bed").is_some());
    assert!(status.get("toolhead").is_some());
    assert!(status.get("print_stats").is_some());

    let _ = std::fs::remove_dir_all(db_path);
}
