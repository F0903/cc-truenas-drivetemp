use super::models::{DiskInfo, DiskQueryItem, DriveTemperature, RpcResponse};
use super::temperature::parse_temperature_result;
use super::transport::{WsStream, connect_websocket};
use crate::config::TrueNasConfig;
use anyhow::{Context, Result, anyhow, bail};
use futures_util::{SinkExt, StreamExt};
use log::debug;
use serde_json::{Value, json};
use std::collections::HashMap;
use tokio_tungstenite::tungstenite::Message;

pub struct TrueNasClient {
    config: TrueNasConfig,
    websocket: WsStream,
    next_id: u64,
}

impl TrueNasClient {
    pub async fn connect(config: TrueNasConfig) -> Result<Self> {
        let websocket = connect_websocket(&config).await?;
        let mut client = Self {
            config,
            websocket,
            next_id: 1,
        };
        client.authenticate().await?;
        Ok(client)
    }

    pub async fn query_disks(&mut self) -> Result<Vec<DiskInfo>> {
        let result = self
            .call(
                "disk.query",
                vec![
                    json!([]),
                    json!({
                        "extra": {"pools": true},
                        "select": ["name", "devname", "model", "serial", "pool", "identifier"],
                        "order_by": ["name"]
                    }),
                ],
            )
            .await?;

        let disks: Vec<DiskQueryItem> = serde_json::from_value(result)
            .context("disk.query returned an unexpected response shape")?;
        Ok(disks
            .into_iter()
            .filter(|disk| !disk.name().is_empty())
            .map(DiskInfo::from)
            .collect())
    }

    pub async fn query_temperatures(
        &mut self,
        names: &[String],
        metadata: &HashMap<String, DiskInfo>,
    ) -> Result<(Vec<DriveTemperature>, Vec<String>)> {
        let result = self
            .call("disk.temperatures", vec![json!(names), json!(false)])
            .await?;
        parse_temperature_result(&result, metadata)
    }

    async fn authenticate(&mut self) -> Result<()> {
        let api_key = self.config.resolve_api_key()?;

        if let Some(username) = self
            .config
            .username
            .clone()
            .filter(|value| !value.is_empty())
        {
            let login = self
                .call(
                    "auth.login_ex",
                    vec![json!({
                        "mechanism": "API_KEY_PLAIN",
                        "username": username,
                        "api_key": api_key,
                        "login_options": {"user_info": false}
                    })],
                )
                .await;

            match login {
                Ok(value)
                    if value.get("response_type").and_then(Value::as_str) == Some("SUCCESS") =>
                {
                    return Ok(());
                }
                Ok(value) => {
                    bail!("auth.login_ex returned {:?}", value.get("response_type"));
                }
                Err(err) => {
                    debug!(
                        "auth.login_ex failed, falling back to auth.login_with_api_key: {err:#}"
                    );
                }
            }
        }

        let result = self
            .call("auth.login_with_api_key", vec![json!(api_key)])
            .await?;
        if result.as_bool() == Some(true) {
            Ok(())
        } else {
            bail!("auth.login_with_api_key did not return true")
        }
    }

    async fn call(&mut self, method: &str, params: Vec<Value>) -> Result<Value> {
        let timeout = self.config.request_timeout();
        tokio::time::timeout(timeout, self.call_inner(method, params))
            .await
            .with_context(|| format!("TrueNAS method {method} timed out after {:?}", timeout))?
    }

    async fn call_inner(&mut self, method: &str, params: Vec<Value>) -> Result<Value> {
        let request_id = self.next_id;
        self.next_id += 1;
        let request = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": request_id
        });

        self.websocket
            .send(Message::Text(request.to_string().into()))
            .await
            .with_context(|| format!("failed to send TrueNAS method {method}"))?;

        while let Some(message) = self.websocket.next().await {
            let message =
                message.with_context(|| format!("failed to read TrueNAS method {method}"))?;
            let text = match message {
                Message::Text(text) => text,
                Message::Binary(bytes) => String::from_utf8(bytes.to_vec())
                    .context("TrueNAS returned non-UTF8 binary JSON-RPC payload")?
                    .into(),
                Message::Close(frame) => bail!("TrueNAS closed websocket: {frame:?}"),
                Message::Ping(payload) => {
                    self.websocket
                        .send(Message::Pong(payload))
                        .await
                        .context("failed to send TrueNAS websocket pong")?;
                    continue;
                }
                Message::Pong(_) | Message::Frame(_) => continue,
            };

            let response: RpcResponse = serde_json::from_str(&text)
                .with_context(|| format!("failed to parse TrueNAS response for {method}"))?;
            if response.id != Some(request_id) {
                continue;
            }
            if let Some(error) = response.error {
                bail!(
                    "TrueNAS JSON-RPC error for {method}: code={:?} message={}",
                    error.code,
                    error.message
                );
            }
            return Ok(response.result.unwrap_or(Value::Null));
        }

        Err(anyhow!(
            "TrueNAS websocket ended while waiting for {method}"
        ))
    }
}

#[cfg(test)]
mod manual_tests {
    use super::*;
    use std::env;

    #[tokio::test]
    #[ignore = "requires a live TrueNAS instance and .env credentials"]
    async fn manual_query_truenas_temperatures_from_dotenv() -> Result<()> {
        dotenvy::dotenv().ok();

        let config = truenas_config_from_env()?;
        let explicit_disks = env_csv("TRUENAS_DISKS");

        println!("Connecting to {}", config.url);
        let mut client = TrueNasClient::connect(config).await?;

        let metadata = if explicit_disks.is_empty() {
            let disks = client
                .query_disks()
                .await
                .context("failed to discover disks through disk.query")?;
            println!("Discovered {} disk(s)", disks.len());
            disks
                .into_iter()
                .map(|disk| (disk.name.clone(), disk))
                .collect::<HashMap<_, _>>()
        } else {
            println!("Using explicit disk list: {}", explicit_disks.join(", "));
            explicit_disks
                .iter()
                .map(|name| (name.clone(), DiskInfo::from_name(name)))
                .collect::<HashMap<_, _>>()
        };

        let mut disk_names = if explicit_disks.is_empty() {
            metadata.keys().cloned().collect::<Vec<_>>()
        } else {
            explicit_disks
        };
        disk_names.sort();

        if disk_names.is_empty() {
            bail!("no disks selected; set TRUENAS_DISKS or grant DISK_READ for disk.query");
        }

        let (temperatures, missing) = client
            .query_temperatures(&disk_names, &metadata)
            .await
            .context("failed to query disk.temperatures")?;

        println!("Temperatures:");
        for temp in &temperatures {
            println!("  {} = {:.1} C", temp.name, temp.celsius);
        }

        if !missing.is_empty() {
            println!("Missing temperatures: {}", missing.join(", "));
        }

        if temperatures.is_empty() {
            bail!("TrueNAS returned no usable temperatures for selected disks");
        }

        Ok(())
    }

    fn truenas_config_from_env() -> Result<TrueNasConfig> {
        Ok(TrueNasConfig {
            url: required_env("TRUENAS_URL")?,
            username: optional_env("TRUENAS_USERNAME"),
            api_key: optional_env("TRUENAS_API_KEY"),
            api_key_file: optional_env("TRUENAS_API_KEY_FILE").map(Into::into),
            api_key_env: Some("TRUENAS_API_KEY".to_string()),
            verify_tls: optional_env("TRUENAS_VERIFY_TLS")
                .as_deref()
                .map(parse_bool)
                .transpose()?
                .unwrap_or(true),
            timeout_seconds: optional_env("TRUENAS_TIMEOUT_SECONDS")
                .as_deref()
                .map(str::parse::<u64>)
                .transpose()
                .context("TRUENAS_TIMEOUT_SECONDS must be a positive integer")?
                .unwrap_or(20),
        })
    }

    fn required_env(key: &str) -> Result<String> {
        optional_env(key).ok_or_else(|| anyhow!("{key} is required in .env"))
    }

    fn optional_env(key: &str) -> Option<String> {
        env::var(key)
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
    }

    fn env_csv(key: &str) -> Vec<String> {
        optional_env(key)
            .map(|value| {
                value
                    .split(',')
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(ToOwned::to_owned)
                    .collect()
            })
            .unwrap_or_default()
    }

    fn parse_bool(value: &str) -> Result<bool> {
        match value.trim().to_ascii_lowercase().as_str() {
            "true" | "1" | "yes" | "y" => Ok(true),
            "false" | "0" | "no" | "n" => Ok(false),
            _ => bail!("expected boolean value, got {value:?}"),
        }
    }
}
