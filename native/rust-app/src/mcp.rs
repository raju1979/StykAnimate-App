use crate::{api, model::Result};
use serde_json::{json, Value};
use std::{
    io::{self, BufRead, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, SyncSender},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};
pub struct Request {
    pub method: String,
    pub args: Value,
    pub deadline: Instant,
    pub response: SyncSender<Result<Value>>,
}
pub struct Bridge {
    pub receiver: Receiver<Request>,
    stop: Arc<AtomicBool>,
}
impl Drop for Bridge {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}
fn line(reader: &mut impl BufRead) -> Result<String> {
    let mut output = String::new();
    reader
        .take(65537)
        .read_line(&mut output)
        .map_err(|e| e.to_string())?;
    if output.len() > 65536 {
        return Err("Request is too large".into());
    }
    Ok(output)
}
fn config() -> Result<(u16, String)> {
    let port = std::env::var("STYK_MCP_PORT")
        .unwrap_or_else(|_| "47833".into())
        .parse::<u16>()
        .map_err(|e| e.to_string())?;
    let token = std::env::var("STYK_MCP_TOKEN").unwrap_or_default();
    if port == 0 || token.len() < 32 || token.len() > 128 || !token.is_ascii() {
        return Err("Set STYK_MCP_TOKEN to a private 32–128 character token and use a nonzero STYK_MCP_PORT".into());
    }
    Ok((port, token))
}
pub fn start(context: eframe::egui::Context) -> Result<Bridge> {
    let (port, token) = config()?;
    let listener = TcpListener::bind(("127.0.0.1", port)).map_err(|e| e.to_string())?;
    listener.set_nonblocking(true).map_err(|e| e.to_string())?;
    let (tx, receiver) = mpsc::sync_channel::<Request>(8);
    let stop = Arc::new(AtomicBool::new(false));
    let done = stop.clone();
    thread::spawn(move || {
        while !done.load(Ordering::Relaxed) {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
                    let _ = stream.set_write_timeout(Some(Duration::from_secs(2)));
                    let result = (|| {
                        let input: Value =
                            serde_json::from_str(&line(&mut BufReader::new(&stream))?)
                                .map_err(|e| e.to_string())?;
                        let supplied = input["token"].as_str().unwrap_or("").as_bytes();
                        if supplied.len() != token.len()
                            || supplied
                                .iter()
                                .zip(token.as_bytes())
                                .fold(0u8, |sum, (a, b)| sum | (a ^ b))
                                != 0
                        {
                            return Err("Unauthorized".into());
                        }
                        let method = input["method"].as_str().ok_or("Missing method")?.to_owned();
                        let args = input.get("args").cloned().unwrap_or(json!({}));
                        let (response, rx) = mpsc::sync_channel(1);
                        tx.try_send(Request {
                            method,
                            args,
                            deadline: Instant::now() + Duration::from_secs(3),
                            response,
                        })
                        .map_err(|_| "Editor command queue is busy")?;
                        context.request_repaint();
                        rx.recv_timeout(Duration::from_secs(3)).map_err(|_| {
                            "Editor timed out; inspect state before retrying an edit"
                        })?
                    })();
                    let output = match result {
                        Ok(value) => json!({"result":value}),
                        Err(error) => json!({"error":error}),
                    };
                    let _ = writeln!(stream, "{output}");
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(10))
                }
                Err(_) => break,
            }
        }
    });
    Ok(Bridge { receiver, stop })
}
fn remote(method: &str, args: Value) -> Result<Value> {
    let (port, token) = config()?;
    let mut stream = TcpStream::connect_timeout(
        &format!("127.0.0.1:{port}")
            .parse()
            .map_err(|e: std::net::AddrParseError| e.to_string())?,
        Duration::from_secs(2),
    )
    .map_err(|e| e.to_string())?;
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .map_err(|e| e.to_string())?;
    writeln!(
        stream,
        "{}",
        json!({"token":token,"method":method,"args":args})
    )
    .map_err(|e| e.to_string())?;
    // Responses can contain a PNG, and have a separate 32 MiB output bound.
    let mut output = String::new();
    BufReader::new(stream)
        .take(32 * 1024 * 1024)
        .read_line(&mut output)
        .map_err(|e| e.to_string())?;
    let response: Value = serde_json::from_str(&output).map_err(|e| e.to_string())?;
    if let Some(error) = response["error"].as_str() {
        Err(error.into())
    } else {
        Ok(response["result"].clone())
    }
}
pub fn stdio() -> Result<()> {
    config()?;
    let stdin = io::stdin();
    let mut reader = stdin.lock();
    let stdout = io::stdout();
    let mut writer = stdout.lock();
    loop {
        let input = line(&mut reader)?;
        if input.is_empty() {
            break;
        }
        let request: Value = serde_json::from_str(&input).map_err(|e| e.to_string())?;
        let Some(id) = request.get("id") else {
            continue;
        };
        let method = request["method"].as_str().unwrap_or("");
        let params = &request["params"];
        let response = match method {
            "initialize" => {
                let requested = params["protocolVersion"].as_str().unwrap_or("2024-11-05");
                let version = if ["2024-11-05", "2025-03-26", "2025-06-18", "2025-11-25"]
                    .contains(&requested)
                {
                    requested
                } else {
                    "2024-11-05"
                };
                json!({"result":{"protocolVersion":version,"serverInfo":{"name":"stykanimate-rust-preview","version":env!("CARGO_PKG_VERSION")},"capabilities":{"tools":{}}}})
            }
            "ping" => json!({"result":{}}),
            "tools/list" => json!({"result":{"tools":api::tools()}}),
            "tools/call" => {
                let name = params["name"].as_str().unwrap_or("");
                if !api::tools().iter().any(|tool| tool["name"] == name) {
                    json!({"error":{"code":-32602,"message":"Unknown tool"}})
                } else {
                    match remote(name, params.get("arguments").cloned().unwrap_or(json!({}))) {
                        Ok(value) if name == "get_frame_preview" => {
                            json!({"result":{"content":[{"type":"image","mimeType":"image/png","data":value["png"]}]}})
                        }
                        Ok(value) => {
                            json!({"result":{"content":[{"type":"text","text":value.to_string()}]}})
                        }
                        Err(error) => {
                            json!({"result":{"isError":true,"content":[{"type":"text","text":error}]}})
                        }
                    }
                }
            }
            _ => json!({"error":{"code":-32601,"message":"Method not found"}}),
        };
        let mut response = response;
        response["jsonrpc"] = json!("2.0");
        response["id"] = id.clone();
        writeln!(writer, "{response}").map_err(|e| e.to_string())?;
        writer.flush().map_err(|e| e.to_string())?;
    }
    Ok(())
}
