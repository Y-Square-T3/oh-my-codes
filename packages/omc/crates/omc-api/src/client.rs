use crate::types::*;
use omc_core::error::{OmcError, Result};
use serde::de::DeserializeOwned;

#[derive(Clone)]
pub struct OmcClient {
    endpoint: ClientEndpoint,
}

#[derive(Clone)]
enum ClientEndpoint {
    #[cfg(unix)]
    Unix(String),
    Http(String),
}

impl OmcClient {
    #[cfg(unix)]
    pub fn connect_unix(socket_path: &str) -> Self {
        Self {
            endpoint: ClientEndpoint::Unix(socket_path.to_string()),
        }
    }

    pub fn connect_http(base_url: &str) -> Self {
        Self {
            endpoint: ClientEndpoint::Http(base_url.to_string()),
        }
    }

    async fn request<T: DeserializeOwned>(
        &self,
        method: &str,
        path: &str,
        body: Option<&[u8]>,
    ) -> Result<T> {
        let body_bytes = body.unwrap_or(b"{}");
        self.request_raw(method, path, body_bytes, "application/json")
            .await
    }

    async fn request_raw<T: DeserializeOwned>(
        &self,
        method: &str,
        path: &str,
        body: &[u8],
        content_type: &str,
    ) -> Result<T> {
        let response_bytes = match &self.endpoint {
            #[cfg(unix)]
            ClientEndpoint::Unix(socket_path) => {
                use http::Uri;
                use hyper::body::Bytes;
                use hyper::header::{CONTENT_LENGTH, CONTENT_TYPE, HOST};
                use hyper_util::rt::TokioIo;

                let uri: Uri = path
                    .parse()
                    .map_err(|e| OmcError::Api(format!("Invalid URI: {e}")))?;

                let connect_timeout = std::time::Duration::from_secs(5);
                let stream = tokio::time::timeout(
                    connect_timeout,
                    tokio::net::UnixStream::connect(socket_path),
                )
                .await
                .map_err(|_| OmcError::Api("Daemon connection timed out".to_string()))?
                .map_err(|e| OmcError::Api(format!("Failed to connect: {e}")))?;
                let io = TokioIo::new(stream);

                let handshake_timeout = std::time::Duration::from_secs(5);
                let (mut sender, conn) = tokio::time::timeout(
                    handshake_timeout,
                    hyper::client::conn::http1::handshake(io),
                )
                .await
                .map_err(|_| OmcError::Api("Daemon handshake timed out".to_string()))?
                .map_err(|e| OmcError::Api(format!("Handshake failed: {e}")))?;

                tokio::spawn(async move {
                    if let Err(err) = conn.await {
                        tracing::error!("Connection error: {}", err);
                    }
                });

                let req = hyper::Request::builder()
                    .method(method)
                    .uri(uri)
                    .header(HOST, "localhost")
                    .header(CONTENT_TYPE, content_type)
                    .header(CONTENT_LENGTH, body.len())
                    .body(http_body_util::Full::new(Bytes::copy_from_slice(body)))
                    .map_err(|e| OmcError::Api(format!("Request build error: {e}")))?;

                let request_timeout = std::time::Duration::from_secs(30);
                let response = tokio::time::timeout(request_timeout, sender.send_request(req))
                    .await
                    .map_err(|_| OmcError::Api("Daemon request timed out".to_string()))?
                    .map_err(|e| OmcError::Api(format!("Request failed: {e}")))?;

                let status = response.status();
                let body = http_body_util::BodyExt::collect(response.into_body())
                    .await
                    .map_err(|e| OmcError::Api(format!("Body read error: {e}")))?
                    .to_bytes();

                if !status.is_success() {
                    let err_body = String::from_utf8_lossy(&body);
                    return Err(OmcError::Api(format!("HTTP {status}: {err_body}")));
                }

                body.to_vec()
            }
            ClientEndpoint::Http(base_url) => {
                let client = reqwest::Client::builder()
                    .connect_timeout(std::time::Duration::from_secs(10))
                    .timeout(std::time::Duration::from_secs(30))
                    .build()
                    .unwrap_or_else(|_| reqwest::Client::new());
                let url = format!("{base_url}{path}");

                let resp = match method {
                    "GET" => client.get(&url).send().await,
                    "POST" => {
                        client
                            .post(&url)
                            .header("Content-Type", content_type)
                            .body(body.to_vec())
                            .send()
                            .await
                    }
                    "DELETE" => client.delete(&url).send().await,
                    _ => return Err(OmcError::Api(format!("Unsupported method: {method}"))),
                }
                .map_err(|e| OmcError::Api(format!("Request failed: {e}")))?;

                let status = resp.status();
                let resp_body = resp
                    .bytes()
                    .await
                    .map_err(|e| OmcError::Api(format!("Body read error: {e}")))?;

                if !status.is_success() {
                    let err_body = String::from_utf8_lossy(&resp_body);
                    return Err(OmcError::Api(format!("HTTP {status}: {err_body}")));
                }

                resp_body.to_vec()
            }
        };

        serde_json::from_slice(&response_bytes)
            .map_err(|e| OmcError::Api(format!("Failed to parse response: {e}")))
    }

    pub async fn health(&self) -> Result<HealthResponse> {
        self.request("GET", "/health", None).await
    }

    pub async fn config(&self) -> Result<ConfigResponse> {
        self.request("GET", "/config", None).await
    }

    pub async fn config_path(&self) -> Result<ConfigPathResponse> {
        self.request("GET", "/config/path", None).await
    }

    pub async fn create_channel(&self, name: &str) -> Result<CreateChannelResponse> {
        let body = serde_json::to_vec(&CreateChannelRequest {
            name: name.to_string(),
        })
        .map_err(|e| OmcError::Api(format!("Serialize error: {e}")))?;
        self.request("POST", "/channels", Some(&body)).await
    }

    pub async fn list_channels(&self) -> Result<ChannelsResponse> {
        self.request("GET", "/channels", None).await
    }

    pub async fn send_message(
        &self,
        channel_id: &str,
        content: &str,
    ) -> Result<SendMessageResponse> {
        let body = serde_json::to_vec(&SendMessageRequest {
            content: content.to_string(),
        })
        .map_err(|e| OmcError::Api(format!("Serialize error: {e}")))?;
        self.request(
            "POST",
            &format!("/channels/{channel_id}/messages"),
            Some(&body),
        )
        .await
    }

    pub async fn get_messages(
        &self,
        channel_id: &str,
        limit: Option<usize>,
        before: Option<&str>,
    ) -> Result<MessagesResponse> {
        let mut query = Vec::new();
        if let Some(l) = limit {
            query.push(format!("limit={l}"));
        }
        if let Some(b) = before {
            query.push(format!("before={b}"));
        }
        let qs = if query.is_empty() {
            String::new()
        } else {
            format!("?{}", query.join("&"))
        };
        self.request("GET", &format!("/channels/{channel_id}/messages{qs}"), None)
            .await
    }

    pub async fn account_login(&self, url: &str) -> Result<LoginResponse> {
        let body = serde_json::to_vec(&LoginRequest {
            url: url.to_string(),
        })
        .map_err(|e| OmcError::Api(format!("Serialize error: {e}")))?;
        self.request("POST", "/account/login", Some(&body)).await
    }

    pub async fn account_poll(&self, req: &PollRequest) -> Result<PollResponse> {
        let body =
            serde_json::to_vec(req).map_err(|e| OmcError::Api(format!("Serialize error: {e}")))?;
        self.request("POST", "/account/poll", Some(&body)).await
    }

    pub async fn account_active(&self) -> Result<ActiveResponse> {
        self.request("GET", "/account/active", None).await
    }

    pub async fn account_list(&self) -> Result<ListResponse> {
        self.request("GET", "/account/list", None).await
    }

    pub async fn account_switch(&self, account_id: &str, workspace_id: &str) -> Result<()> {
        let body = serde_json::to_vec(&SwitchRequest {
            account_id: account_id.to_string(),
            workspace_id: workspace_id.to_string(),
        })
        .map_err(|e| OmcError::Api(format!("Serialize error: {e}")))?;
        self.request::<serde_json::Value>("POST", "/account/switch", Some(&body))
            .await?;
        Ok(())
    }

    pub async fn account_remove(&self, account_id: &str) -> Result<()> {
        let body = serde_json::to_vec(&RemoveRequest {
            account_id: account_id.to_string(),
        })
        .map_err(|e| OmcError::Api(format!("Serialize error: {e}")))?;
        self.request::<serde_json::Value>("POST", "/account/remove", Some(&body))
            .await?;
        Ok(())
    }

    pub async fn account_refresh_token(&self, email: Option<&str>) -> Result<RefreshTokenResponse> {
        let body = serde_json::to_vec(&RefreshTokenRequest {
            email: email.map(|s| s.to_string()),
        })
        .map_err(|e| OmcError::Api(format!("Serialize error: {e}")))?;
        self.request("POST", "/account/refresh-token", Some(&body))
            .await
    }

    pub async fn account_workspaces(&self, account_id: &str) -> Result<WorkspacesResponse> {
        self.request(
            "GET",
            &format!("/account/workspaces?account_id={account_id}"),
            None,
        )
        .await
    }

    pub async fn models_list(&self, provider: Option<&str>) -> Result<ModelsListResponse> {
        let qs = provider
            .map(|p| format!("?provider={p}"))
            .unwrap_or_default();
        self.request("GET", &format!("/models{qs}"), None).await
    }

    pub async fn models_sync(&self) -> Result<ModelsSyncResponse> {
        self.request("POST", "/models/sync", None).await
    }

    pub async fn token_usage_record(&self, req: &TokenUsageRecordRequest) -> Result<()> {
        let body =
            serde_json::to_vec(req).map_err(|e| OmcError::Api(format!("Serialize error: {e}")))?;
        self.request::<serde_json::Value>("POST", "/token-usage", Some(&body))
            .await?;
        Ok(())
    }

    pub async fn token_usage_status(&self) -> Result<TokenUsageStatusResponse> {
        self.request("GET", "/token-usage/status", None).await
    }

    pub async fn token_usage_push(&self) -> Result<TokenUsagePushResponse> {
        self.request("POST", "/token-usage/push", None).await
    }

    pub async fn token_usage_list(
        &self,
        limit: Option<usize>,
        offset: Option<usize>,
        pushed: Option<bool>,
    ) -> Result<TokenUsageListResponse> {
        let mut query = Vec::new();
        if let Some(l) = limit {
            query.push(format!("limit={l}"));
        }
        if let Some(o) = offset {
            query.push(format!("offset={o}"));
        }
        if let Some(p) = pushed {
            query.push(format!("pushed={p}"));
        }
        let qs = if query.is_empty() {
            String::new()
        } else {
            format!("?{}", query.join("&"))
        };
        self.request("GET", &format!("/token-usage/list{qs}"), None)
            .await
    }

    pub async fn token_usage_summary(
        &self,
        days: Option<i64>,
    ) -> Result<TokenUsageSummaryResponse> {
        let qs = days.map(|d| format!("?days={d}")).unwrap_or_default();
        self.request("GET", &format!("/token-usage/summary{qs}"), None)
            .await
    }

    pub async fn token_usage_overview(
        &self,
        days: Option<i64>,
    ) -> Result<omc_core::token_usage::TokenUsageOverview> {
        let qs = days.map(|d| format!("?days={d}")).unwrap_or_default();
        self.request("GET", &format!("/token-usage/overview{qs}"), None)
            .await
    }
}
