use omc_core::config::OmcConfig;
use omc_server::server_client::OmcServerClient;
use omc_server::{AccountService, DaemonState, ModelService, TokenUsageService, start_server};
use omc_storage::create_backend;
use std::sync::Arc;
use tokio::sync::watch;

#[tokio::test]
async fn server_responds_to_health_on_windows() {
    let temp_dir = tempfile::tempdir().unwrap();
    let data_dir = temp_dir.path().to_string_lossy().to_string();

    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);

    let mut config = OmcConfig::default();
    config.daemon.bind_addr = Some("127.0.0.1".to_string());
    config.daemon.bind_port = Some(port);
    config.daemon.data_dir = Some(data_dir.clone());
    config.daemon.database_url = Some(format!("sqlite:{}/omc.db", data_dir));

    let db_url =
        omc_storage::database_url::DatabaseUrl::parse(&config.resolve_daemon().database_url);
    let backend = create_backend(&db_url).await.unwrap();

    let server_client = OmcServerClient::new();
    let account_service = Arc::new(AccountService::new(backend.clone(), server_client.clone()));
    let model_service = Arc::new(ModelService::new(
        backend.clone(),
        account_service.clone(),
        server_client.clone(),
    ));
    let token_usage_service = Arc::new(TokenUsageService::new(
        backend.clone(),
        account_service.clone(),
        server_client,
    ));

    let state = Arc::new(DaemonState::new(
        config,
        backend,
        account_service,
        model_service,
        token_usage_service,
    ));

    let (shutdown_tx, shutdown_rx) = watch::channel(());
    let server_handle = tokio::spawn(async move {
        start_server(state, shutdown_rx).await.unwrap();
    });

    let client = reqwest::Client::new();
    let mut ok = false;
    for _ in 0..20 {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        match client
            .get(format!("http://127.0.0.1:{}/health", port))
            .timeout(std::time::Duration::from_secs(1))
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => {
                ok = true;
                break;
            }
            _ => continue,
        }
    }

    let _ = shutdown_tx.send(());
    tokio::time::timeout(tokio::time::Duration::from_secs(5), server_handle)
        .await
        .expect("server did not shut down")
        .expect("server panicked");

    assert!(ok, "health endpoint did not respond");
}
