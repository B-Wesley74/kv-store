// --- IMPORTS ---
// TcpStream: lets us CONNECT to a server as a client (server used TcpListener to ACCEPT
// connections)
use tokio::net::TcpStream;
// Same async read/write tools the server uses to send/receive bytes over a connection
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
// Instant: the "stopwatch" for timing how long the benchmark takes
use std::time::Instant;

// --- One full round trip (connect, send one command, read one reply) ---

// Takes an OWNED String (not &str) because this function will be run inside
// tokio::spawn - independent, concurrent tasks need to own their data,
// since Rust can't guarantee a borrowed reference would still be valid
// by the time the spawned task actually runs.
async fn send_command(command: String) {
    // Connect to the running server, just like a real client would.
    let stream = TcpStream::connect("127.0.0.1:7878").await.unwrap();
    
    // Same split-into-reader/writer pattern as the server's handle_connection.
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    // Send the command as raw bytes (e.g. "SET key0 value0\n" as bytes).
    writer.write_all(command.as_bytes()).await.unwrap();

    // Read back one line of response (e.g. "OK\n"). Don't check its
    // content here, for this benchmark we only care that the round trip
    // completed, not what the server said.
    let mut response = String::new();
    reader.read_line(&mut response).await.unwrap();

    // Function ends here, 'stream' goes out of scope, close the connection.
}

// --- THE BENCHMARK ---
#[tokio::main]
async fn main() {
    let total_requests = 1000;

    // Start the stopwatch, BEFORE any work is spawned.
    let start = Instant::now();

    // Collect a "ticket" (JoinHandle) for each spawned task here,
    // so we can later wait for every single one to actually finish.
    let mut handles = Vec::new();

    for i in 0..total_requests {
        // Build a unique command per iteration, e.g. "SET key0 value0\n", "SET key1 value1\n", ...
        // (unique keys so we're not just hammering ONE key/ONE shard the whole time)
        let command = format!("SET key{} value{}\n", i, i);

        // Kick off send_command as an independent task. This returns
        // immediately with a handle, it does not wait for send_command
        // to finish. The actual connecting/sending/reading happens
        // in the background, possibly interleaved with the other 999 tasks.
        let handle = tokio::spawn(send_command(command));
        
        // Save the ticket so we can wait on it later.
        handles.push(handle);
    }
    // <-- at this point, all 1000 tasks have been HANDED OFF, but likely
    // very few (if any) have finished running yet.

    // Now actually wait for every single task to truly complete.
    // Only after ALL 1000 of these .await calls resolve do we know
    // every command has been send and responded to.
    for handle in handles {
        handle.await.unwrap();
    }

    // Stop the stopwatch, NOW it's accurate, since we know all work is done.
    let duration = start.elapsed();

    // Calculate how many operations-per-second 
    let ops_per_sec = total_requests as f64 / duration.as_secs_f64();

    println!("Completed {} requests in {:.2?}", total_requests, duration);
    println!("Throughput: {:.2} ops/sec", ops_per_sec);
}
