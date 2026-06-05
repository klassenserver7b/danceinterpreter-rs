//! Dummy Traktor client for manually testing ui elements like the connection indicator
//!
//! Run the app first, enable the server in the sidebar, then:
//!     cargo run --example dummy_traktor_client                  # 127.0.0.1:8080

use tokio_tungstenite::tungstenite::connect;

fn main() {
    let mut args = std::env::args().skip(1);
    let addr = args.next().unwrap_or_else(|| "127.0.0.1:8080".to_owned());

    let url = format!("ws://{addr}/cover");
    println!("connecting dummy client to {url} (Ctrl-C to stop)");

    match connect(&url) {
        Ok((mut ws, _)) => {
            println!("client connected");

            loop {
                let msg = ws.read();
                match msg {
                    Ok(msg) => {
                        if let Ok(text) = msg.to_text()
                            && !text.is_empty()
                        {
                            println!("client: server requested cover for {text}");
                        } else {
                            println!("client: received non-text message: {msg}");
                        }
                    }
                    Err(e) => {
                        println!("client: connection error: {e}");
                        break;
                    }
                }
            }

            println!("client: disconnected");
        }
        Err(e) => {
            println!(
                "client: failed to connect to {url}: {e} \
                         (is the app running with the server enabled?)"
            );
        }
    }
}
