use mikan::rpc::RpcTransaction;
use reqwest::Client;
use serde_json::{json, Value};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
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
    thread::sleep(Duration::from_secs(5));

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
        send_transactions(running).await;
    });

    // Wait for the child process to finish
    println!("Stopping nodes...");
    let _ = child.kill();
    let _ = child.wait();
}

async fn send_transactions(running: Arc<AtomicBool>) {
    let client = Client::new();
    let node_count = 3;

    // Create a vector of node RPC URLs
    let node_urls: Vec<String> = (0..node_count)
        .map(|i| format!("http://127.0.0.1:{}", 8545 + i))
        .collect();

    println!("Sending random transactions to {} nodes...", node_count);

    // Send transactions in a loop until Ctrl+C is pressed
    let mut handles = Vec::new();
    let num_workers = 10; // Number of parallel workers

    for worker_id in 0..num_workers {
        let client_clone = client.clone();
        let node_urls_clone = node_urls.clone();
        let running_clone = running.clone();

        let handle = tokio::spawn(async move {
            while running_clone.load(Ordering::SeqCst) {
                // Send a transaction to a random node
                let node_index = rand::random::<usize>() % node_count;
                let url = &node_urls_clone[node_index];

                // Create a random transaction
                let tx = RpcTransaction::random();

                // Send the transaction
                match send_transaction(&client_clone, url, tx).await {
                    Ok(hash) => println!(
                        "Worker {}: Sent transaction to node {}: {}",
                        worker_id, node_index, hash
                    ),
                    Err(e) => eprintln!(
                        "Worker {}: Failed to send transaction to node {}: {}",
                        worker_id, node_index, e
                    ),
                }

                // Small delay to prevent overwhelming the system
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        });

        handles.push(handle);
    }

    // Wait for all workers to complete
    for handle in handles {
        let _ = handle.await;
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
