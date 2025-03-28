// Copyright 2024 The Drasi Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::env;

use axum::async_trait;
use dapr::Client;

use crate::StateStore;

pub struct DaprStateStore {
    client: dapr::Client<dapr::client::TonicClient>,
    store_name: String,
}

impl DaprStateStore {
    pub async fn connect() -> Result<Self, dapr::error::Error> {
        match wait_for_dapr_start().await {
            Ok(_) => log::info!("Dapr is up and running"),
            Err(e) => {
                log::error!("Failed to connect to Dapr: {:?}", e);
                return Err(dapr::error::Error::TransportError);
            }
        };
        let client = {
            let mut attempt = 0;
            loop {
                match Client::<dapr::client::TonicClient>::connect("https://127.0.0.1".to_string()).await {
                    Ok(client) => break client,
                    Err(e) => {
                        attempt += 1;
                        if attempt >= 5 {
                            return Err(e);
                        }
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    }
                }
            }
        };

        Ok(DaprStateStore {
            client,
            store_name: match env::var("STATE_STORE_NAME") {
                Ok(val) => val,
                Err(_) => "drasi-state".to_string(),
            },
        })
    }
}

#[async_trait]
impl StateStore for DaprStateStore {
    async fn get(
        &self,
        id: &str,
    ) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error + Send + Sync>> {
        let mut dapr_client = self.client.clone();
        let response = dapr_client
            .get_state(self.store_name.as_str(), id, None)
            .await?;
        if response.data.len() == 0 {
            return Ok(None);
        }
        Ok(Some(response.data))
    }

    async fn put(
        &self,
        id: &str,
        value: Vec<u8>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut dapr_client = self.client.clone();
        dapr_client
            .save_state(self.store_name.as_str(), [(id, value)])
            .await?;
        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut dapr_client = self.client.clone();
        dapr_client
            .delete_state(self.store_name.as_str(), id, None)
            .await?;
        Ok(())
    }
}

async fn wait_for_dapr_start() -> Result<(), Box<dyn std::error::Error>> {
    let http_port: u16 = std::env::var("DAPR_HTTP_PORT")?.parse()?;
    let mut attempt = 0;
    loop {
        let url = format!("http://localhost:{}/v1.0/healthz/outbound", http_port);
        let response = reqwest::get(&url).await;

        match response {
            Ok(resp) => {
                if resp.status().is_success() {
                    log::info!("Dapr is up and running on port {}", http_port);
                    return Ok(());
                } else {
                    log::warn!("Dapr is not ready yet, status: {}", resp.status());
                }
            }
            Err(e) => {
                log::error!("Error connecting to Dapr: {:?}", e);
            }
        }

        attempt += 1;
        if attempt >= 10 {
            log::error!("Dapr did not start within the expected time frame.");
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "Dapr did not start within the expected time frame.",
            )));
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}