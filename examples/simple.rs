use frieda::proof::Proof;
use mikan::blob::Blob;
use mikan::rpc::RpcTransaction;
use reqwest::Client;
use serde_json::{json, Value};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;

fn main() {
    // Start the nodes
    println!("Starting nodes with spawn.bash...");
    let mut child = Command::new("bash")
        .arg("spawn.bash")
        .arg("--nodes")
        .arg("3")
        .arg("--home")
        .arg("nodes")
        .spawn()
        .expect("Failed to start spawn.bash");

    // Wait for nodes to start
    println!("Waiting for nodes to start...");

    // Create a flag to control the transaction sending loop
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    // Set up Ctrl+C handler
    ctrlc::set_handler(move || {
        println!("Stopping transaction sender...");
        running_clone.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl+C handler");

    // Create a runtime for async operations
    let rt = Runtime::new().expect("Failed to create Tokio runtime");

    // Run the transaction sender in the runtime
    rt.block_on(async {
        run_workers(running).await;
    });

    // Wait for the child process to finish
    println!("Stopping nodes...");
    let _ = child.kill();
    let _ = child.wait();
}

async fn run_workers(running: Arc<AtomicBool>) {
    let client = Client::new();
    let node_count = 3;

    // Create a vector of node RPC URLs
    let node_urls: Vec<String> = (0..node_count)
        .map(|i| format!("http://127.0.0.1:{}", 8545 + i))
        .collect();

    println!("Starting transaction and sampling workers...");

    let mut handles = Vec::new();

    // Transaction workers
    let tx_workers = 6;
    for worker_id in 0..tx_workers {
        let client_clone = client.clone();
        let node_urls_clone = node_urls.clone();
        let running_clone = running.clone();

        let handle = tokio::spawn(async move {
            transaction_worker(worker_id, client_clone, node_urls_clone, running_clone).await;
        });

        handles.push(handle);
    }

    // Sampling workers
    let sampling_workers = 2;
    for worker_id in 0..sampling_workers {
        let client_clone = client.clone();
        let node_urls_clone = node_urls.clone();
        let running_clone = running.clone();

        let handle = tokio::spawn(async move {
            sampling_worker(worker_id, client_clone, node_urls_clone, running_clone).await;
        });

        handles.push(handle);
    }

    // Wait for all workers to complete
    for handle in handles {
        let _ = handle.await;
    }
}

async fn transaction_worker(
    worker_id: usize,
    client: Client,
    node_urls: Vec<String>,
    running: Arc<AtomicBool>,
) {
    let node_count = node_urls.len();
    println!("Transaction worker {} started", worker_id);

    while running.load(Ordering::SeqCst) {
        // Send a transaction to a random node
        let node_index = rand::random::<usize>() % node_count;
        let url = &node_urls[node_index];

        // Create a random transaction
        let tx = RpcTransaction::random();

        // Fire and forget - don't await the result
        let client_clone = client.clone();
        let url_clone = url.clone();
        tokio::spawn(async move {
            let _ = send_transaction(&client_clone, &url_clone, tx).await;
        });
    }
}

async fn sampling_worker(
    worker_id: usize,
    client: Client,
    node_urls: Vec<String>,
    running: Arc<AtomicBool>,
) {
    let node_count = node_urls.len();
    println!("Sampling worker {} started", worker_id);

    while running.load(Ordering::SeqCst) {
        // Select a random node to query
        let node_index = rand::random::<usize>() % node_count;
        let url = &node_urls[node_index];

        // Get current block number
        let nb = get_block_number(&client, url).await;

        // Sample a blob from a block if we have blocks available
        if nb > 0 {
            // Choose a random block within the available range
            let block_height = rand::random::<u64>() % nb + 1;
            let blob_index = rand::random::<usize>() % 4; // Assuming the first blob in the block

            // Randomly decide whether to provide a sampling seed
            let sampling_seed = if rand::random::<bool>() {
                Some(rand::random::<u64>())
            } else {
                None
            };

            // Call the sample_blob RPC method
            match sample_blob(&client, url, block_height, blob_index, sampling_seed).await {
                Some(proof) => {
                    let verified = frieda::api::verify(proof, sampling_seed);
                    println!(
                        "Sample Worker {}: Successfully({verified}) sampled blob {} from block {}",
                        worker_id, blob_index, block_height
                    );
                    let blob = get_blob(&client, url, block_height, blob_index).await;
                    if blob.is_some() {
                        println!("Successfully got Blob: {:?}", blob.unwrap().data().len());
                    } else {
                        println!("Failed to get Blob");
                    }
                }
                None => eprintln!(
                    "Sample Worker {}: No blob in block {}",
                    worker_id, block_height
                ),
            }
        }

        // Small delay between sampling attempts
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
}

async fn send_transaction(
    client: &Client,
    url: &str,
    tx: RpcTransaction,
) -> Result<String, Box<dyn std::error::Error>> {
    let response = client
        .post(url)
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "mikan_sendTransaction",
            "params": [tx],
            "id": 1
        }))
        .send()
        .await?;

    let result: Value = response.json().await?;

    // Extract the transaction hash from the response
    let hash = result["result"]
        .as_str()
        .ok_or("Missing result in response")?;

    Ok(hash.to_string())
}

async fn sample_blob(
    client: &Client,
    url: &str,
    block_height: u64,
    blob_index: usize,
    sampling_seed: Option<u64>,
) -> Option<Proof> {
    let params = if let Some(seed) = sampling_seed {
        json!([block_height, blob_index, seed])
    } else {
        json!([block_height, blob_index])
    };

    let response = client
        .post(url)
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "mikan_sampleBlob",
            "params": params,
            "id": 1
        }))
        .send()
        .await
        .ok()?;

    let result: Value = response.json().await.ok()?;

    if result.get("error").is_some() {
        return None;
    }

    // Parse the proof from the response
    let proof: Proof = serde_json::from_value(result["result"].clone()).ok()?;

    Some(proof)
}

async fn get_blob(
    client: &Client,
    url: &str,
    block_height: u64,
    blob_index: usize,
) -> Option<Blob> {
    let response = client
        .post(url)
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "mikan_getBlob",
            "params": [block_height, blob_index],
            "id": 1
        }))
        .send()
        .await
        .ok()?;

    let result: Value = response.json().await.ok()?;

    if result.get("error").is_some() {
        return None;
    }

    let blob: Blob = serde_json::from_value(result["result"].clone()).ok()?;

    Some(blob)
}

async fn get_block_number(client: &Client, url: &str) -> u64 {
    let response = client
        .post(url)
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "mikan_blockNumber",
            "params": [],
            "id": 1
        }))
        .send()
        .await;

    let result: u64 = match response {
        Ok(response) => {
            let json_value: Value = response.json().await.unwrap();
            json_value["result"].as_u64().unwrap_or_default()
        }
        Err(_) => 0,
    };

    result
}
