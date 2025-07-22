// Copyright 2025 xAI
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

use std::sync::Arc;
use tokio::time::{sleep, Duration};
use log;
use etcd_client::Client as EtcdClient;

use crate::client::Client;

pub async fn ping_task(client: Arc<Client>) {
    const MAX_PING_FAIL_COUNT: i32 = 3;
    const SUCCESS_PING_INTERVAL: u64 = 1000;
    const FAIL_PING_INTERVAL: u64 = 1000;
    let mut ping_fail_count = 0;

    let mut etcd_client = EtcdClient::connect(["http://127.0.0.1:2379"], None).await.unwrap();

    loop {
        if !client.ping_running.load(std::sync::atomic::Ordering::SeqCst) {
            break;
        }

        let ping_result = client.master_client.ping(client.client_id.clone()).await;
        match ping_result {
            Ok(status) => {
                ping_fail_count = 0;
                if status.client_status == 1 { // NEED_REMOUNT
                    let client_clone = Arc::clone(&client);
                    tokio::spawn(async move {
                        if let Err(e) = client_clone.remount_segments(&mut etcd_client).await {
                            log::error!("Failed to remount segments: {}", e);
                        }
                    });
                }
                sleep(Duration::from_millis(SUCCESS_PING_INTERVAL)).await;
            }
            Err(e) => {
                ping_fail_count += 1;
                if ping_fail_count >= MAX_PING_FAIL_COUNT {
                    log::error!("Failed to ping master {} times, reconnecting", ping_fail_count);
                    if let Ok(new_master) = client.get_new_master_address(&mut etcd_client).await {
                        if let Err(reconnect_err) = client.master_client.reconnect(new_master).await {
                            log::error!("Failed to reconnect to master: {}", reconnect_err);
                        } else {
                            ping_fail_count = 0;
                        }
                    }
                } else {
                    log::error!("Failed to ping master: {}", e);
                }
                sleep(Duration::from_millis(FAIL_PING_INTERVAL)).await;
            }
        }
    }
}