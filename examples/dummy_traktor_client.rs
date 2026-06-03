//! Dummy Traktor client for manually testing the connection indicator
//! (issue #184).
//!
//! A real Traktor client setup holds open a `/cover` WebSocket (that is what
//! the cover-loader does), and that connection is what the sidebar indicator
//! counts. This example opens one or more such WebSockets to a running
//! danceinterpreter server and keeps them open, so the sidebar should light up
//! green and report the number of connected clients (with the loopback IP).
//!
//! Run the app first, enable the server in the sidebar, then:
//!
//!     cargo run --example dummy_traktor_client                  # 1 client -> 127.0.0.1:8080
//!     cargo run --example dummy_traktor_client -- 127.0.0.1:8080 3   # 3 clients
//!
//! Press Ctrl-C to disconnect them again.

use futures_util::StreamExt;
use tokio_tungstenite::connect_async;

#[tokio::main]
async fn main() {
    let mut args = std::env::args().skip(1);
    let addr = args.next().unwrap_or_else(|| "127.0.0.1:8080".to_owned());
    let count: usize = args.next().and_then(|s| s.parse().ok()).unwrap_or(1);

    let url = format!("ws://{addr}/cover");
    println!("connecting {count} dummy client(s) to {url} (Ctrl-C to stop)");

    let mut handles = Vec::with_capacity(count);
    for i in 0..count {
        let url = url.clone();
        handles.push(tokio::spawn(async move {
            match connect_async(&url).await {
                Ok((mut ws, _)) => {
                    println!("client {i}: connected");

                    // Hold the socket open. The server pushes cover-file paths
                    // over it; we just log them. The connection stays alive
                    // until the process is killed or the server shuts down.
                    while let Some(msg) = ws.next().await {
                        match msg {
                            Ok(msg) => {
                                if let Ok(text) = msg.to_text()
                                    && !text.is_empty()
                                {
                                    println!("client {i}: server requested cover for {text}");
                                }
                            }
                            Err(e) => {
                                println!("client {i}: connection error: {e}");
                                break;
                            }
                        }
                    }

                    println!("client {i}: disconnected");
                }
                Err(e) => {
                    println!(
                        "client {i}: failed to connect to {url}: {e} \
                         (is the app running with the server enabled?)"
                    );
                }
            }
        }));
    }

    for handle in handles {
        let _ = handle.await;
    }
}
