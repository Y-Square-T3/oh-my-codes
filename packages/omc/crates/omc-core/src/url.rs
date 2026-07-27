use crate::error::{OmcError, Result};

pub fn normalize_server_url(input: &str) -> Result<String> {
    let parsed = url::Url::parse(input).map_err(|e| OmcError::Auth(format!("Invalid URL: {e}")))?;

    let mut normalized = String::new();
    normalized.push_str(parsed.scheme());
    normalized.push_str("://");
    normalized.push_str(
        parsed
            .host_str()
            .ok_or_else(|| OmcError::Auth("Missing host".into()))?,
    );
    if let Some(port) = parsed.port() {
        normalized.push(':');
        normalized.push_str(&port.to_string());
    }
    let path = parsed.path();
    if path != "/" {
        normalized.push_str(path.trim_end_matches('/'));
    }

    Ok(normalized)
}
