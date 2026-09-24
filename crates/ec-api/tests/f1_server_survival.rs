#![forbid(unsafe_code)]

//! F1: an analysis-worker crash must not terminate the real HTTP server.

use std::fs::{self, File};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};
use tempfile::TempDir;

const KEY: &str = "f1-test-key-not-a-secret";

struct Server(Child);

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn request(port: u16, method: &str, path: &str, body: &str) -> Result<(u16, String), String> {
    let address = SocketAddr::from(([127, 0, 0, 1], port));
    let mut stream =
        TcpStream::connect_timeout(&address, Duration::from_secs(2)).map_err(|e| e.to_string())?;
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .map_err(|e| e.to_string())?;
    stream
        .set_write_timeout(Some(Duration::from_secs(5)))
        .map_err(|e| e.to_string())?;

    let headers = format!(
        "{method} {path} HTTP/1.1\r\n\
         Host: 127.0.0.1\r\n\
         Connection: close\r\n\
         Content-Type: application/json\r\n\
         x-api-key: {KEY}\r\n\
         Content-Length: {}\r\n\r\n",
        body.len()
    );
    stream
        .write_all(headers.as_bytes())
        .and_then(|_| stream.write_all(body.as_bytes()))
        .map_err(|e| e.to_string())?;

    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .map_err(|e| e.to_string())?;
    let status = response
        .lines()
        .next()
        .ok_or("empty HTTP response")?
        .split_whitespace()
        .nth(1)
        .ok_or("missing HTTP status")?
        .parse::<u16>()
        .map_err(|e| e.to_string())?;

    Ok((status, response))
}

#[test]
fn crashing_analysis_does_not_kill_server() {
    let dir = TempDir::new().expect("temporary server directory");
    let reserved = TcpListener::bind(("127.0.0.1", 0)).expect("allocate local port");
    let port = reserved.local_addr().expect("local address").port();
    drop(reserved);

    let stderr_path = dir.path().join("server.stderr");
    let stderr = File::create(&stderr_path).expect("create server log");
    let child = Command::new(env!("CARGO_BIN_EXE_ec-server"))
        .env("EC_BIND_ADDR", format!("127.0.0.1:{port}"))
        .env("EC_API_KEY", KEY)
        .env("EC_DB", dir.path().join("gate.sqlite"))
        .current_dir(dir.path())
        .stdout(Stdio::null())
        .stderr(Stdio::from(stderr))
        .spawn()
        .expect("start real ec-server");
    let mut server = Server(child);

    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if let Some(status) = server.0.try_wait().expect("inspect server") {
            panic!("server exited before readiness: {status}");
        }
        if matches!(request(port, "GET", "/api/v1/health", ""), Ok((200, _))) {
            break;
        }
        assert!(Instant::now() < deadline, "server did not become ready");
        thread::sleep(Duration::from_millis(40));
    }

    let normal = serde_json::json!({"code": "fn f() {}"}).to_string();
    let (normal_status, normal_response) =
        request(port, "POST", "/api/v1/analyze", &normal).expect("normal analysis must respond");
    assert_eq!(normal_status, 200, "{normal_response}");
    assert!(
        normal_response.contains("\"parse_successful\":true"),
        "normal analysis did not run: {normal_response}"
    );

    // Small enough to pass the 2 MiB body limit; the same family of
    // deeply nested generic types previously terminated CLI and server.
    let code = format!(
        "fn f() {{ let x: {}i32{} = todo!(); }}",
        "Vec<".repeat(600),
        ">".repeat(600)
    );
    let payload = serde_json::json!({"code": code}).to_string();
    let attack_response = request(port, "POST", "/api/v1/analyze", &payload);

    // Probe the SAME child server after the hostile request.
    let health_after = request(port, "GET", "/api/v1/health", "");
    let server_exit = server.0.try_wait().expect("inspect server after request");
    let server_log = fs::read_to_string(&stderr_path).unwrap_or_default();
    assert!(
        matches!(health_after.as_ref(), Ok((status, _)) if *status == 200) && server_exit.is_none(),
        "server died or stopped responding: attack={attack_response:?}, \
         health={health_after:?}, exit={server_exit:?}, stderr={server_log}"
    );

    // A child crash is an infrastructure failure, not successful analysis.
    let (status, response) =
        attack_response.expect("server must answer even when its worker crashes");
    assert_eq!(
        status, 502,
        "worker crash must produce HTTP 502: {response}"
    );
    assert!(response.contains("\"error\""), "missing error: {response}");
}
