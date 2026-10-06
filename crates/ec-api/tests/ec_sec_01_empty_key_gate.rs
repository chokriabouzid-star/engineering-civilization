#![forbid(unsafe_code)]

//! EC-SEC-01: an empty `EC_API_KEY` must never become a valid credential.
//!
//! Defence 1 (real process): `ec-server` refuses to start when `EC_API_KEY`
//! is present but empty, and says why on stderr.
//! Defence 2 (middleware): an explicitly empty `x-api-key` header never
//! authenticates, even when the configured key is empty.
//! A missing header and an empty header are distinct cases and both are covered.

use axum::http::{HeaderName, HeaderValue};
use axum_test::TestServer;
use ec_api::auth::API_KEY_HEADER;
use ec_api::build_router;
use ec_api::state::AppState;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant};
use tempfile::TempDir;

/// Protected route already used by week50 Gate 0 (200 on an empty audit log).
const PROTECTED: &str = "/api/v1/governance/audit";

struct Server(Child);

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

enum Outcome {
    Exited(ExitStatus),
    Ready,
}

/// Raw HTTP GET against the real process. `key: None` sends no header;
/// `key: Some("")` sends an explicit empty `x-api-key` header.
fn request(port: u16, path: &str, key: Option<&str>) -> Result<u16, String> {
    let address = SocketAddr::from(([127, 0, 0, 1], port));
    let mut stream =
        TcpStream::connect_timeout(&address, Duration::from_secs(2)).map_err(|e| e.to_string())?;
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .map_err(|e| e.to_string())?;
    stream
        .set_write_timeout(Some(Duration::from_secs(5)))
        .map_err(|e| e.to_string())?;

    let key_line = match key {
        Some(k) => format!("x-api-key: {k}\r\n"),
        None => String::new(),
    };
    let head =
        format!("GET {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n{key_line}\r\n");
    stream
        .write_all(head.as_bytes())
        .map_err(|e| e.to_string())?;

    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .map_err(|e| e.to_string())?;
    response
        .lines()
        .next()
        .ok_or("empty HTTP response")?
        .split_whitespace()
        .nth(1)
        .ok_or("missing HTTP status")?
        .parse::<u16>()
        .map_err(|e| e.to_string())
}

fn show(result: &Result<u16, String>) -> String {
    match result {
        Ok(status) => status.to_string(),
        Err(e) => format!("ERR({e})"),
    }
}

#[test]
fn real_server_refuses_to_start_with_empty_api_key() {
    let dir = TempDir::new().expect("temporary server directory");
    let reserved = TcpListener::bind(("127.0.0.1", 0)).expect("allocate local port");
    let port = reserved.local_addr().expect("local address").port();
    drop(reserved);

    let stderr_path = dir.path().join("server.stderr");
    let stderr = File::create(&stderr_path).expect("create server log");
    let child = Command::new(env!("CARGO_BIN_EXE_ec-server"))
        .env("EC_BIND_ADDR", format!("127.0.0.1:{port}"))
        .env("EC_API_KEY", "")
        .env("EC_DB", dir.path().join("gate.sqlite"))
        .current_dir(dir.path())
        .stdout(Stdio::null())
        .stderr(Stdio::from(stderr))
        .spawn()
        .expect("start real ec-server");
    let mut server = Server(child);

    let deadline = Instant::now() + Duration::from_secs(10);
    let outcome = loop {
        if let Some(status) = server.0.try_wait().expect("inspect server") {
            break Outcome::Exited(status);
        }
        if matches!(request(port, "/api/v1/health", None), Ok(200)) {
            break Outcome::Ready;
        }
        assert!(
            Instant::now() < deadline,
            "server neither exited nor became ready within 10s"
        );
        thread::sleep(Duration::from_millis(40));
    };

    match outcome {
        Outcome::Exited(status) => {
            let log = fs::read_to_string(&stderr_path).unwrap_or_default();
            assert!(
                !status.success(),
                "server exited successfully with an empty EC_API_KEY: status={status}, stderr={log}"
            );
            assert!(
                log.contains("EC_API_KEY"),
                "server exited for a reason unrelated to the key: status={status}, stderr={log}"
            );
        }
        Outcome::Ready => {
            let empty = request(port, PROTECTED, Some(""));
            let absent = request(port, PROTECTED, None);
            panic!(
                "EC-SEC-01 server accepted an empty EC_API_KEY and stayed up; \
                 RED_EVIDENCE empty_header={} no_header={}",
                show(&empty),
                show(&absent)
            );
        }
    }
}

fn server_with_configured_key(configured: &'static str) -> TestServer {
    let state = AppState::in_memory(configured).expect("in-memory state");
    TestServer::new(build_router(state)).expect("test server")
}

#[tokio::test]
async fn middleware_rejects_empty_header_when_configured_key_is_empty() {
    let mut server = server_with_configured_key("");
    let absent = server.get(PROTECTED).await.status_code();
    assert_eq!(
        absent, 401,
        "precondition: protected route must reject a missing header"
    );
    server.add_header(
        HeaderName::from_static(API_KEY_HEADER),
        HeaderValue::from_static(""),
    );
    let empty = server.get(PROTECTED).await.status_code();
    assert_eq!(
        empty, 401,
        "EC-SEC-01 middleware accepted an empty x-api-key against an empty configured key: status={empty}"
    );
}

#[tokio::test]
async fn middleware_rejects_empty_header_when_configured_key_is_set() {
    let mut server = server_with_configured_key("ec-sec-01-configured-key");
    server.add_header(
        HeaderName::from_static(API_KEY_HEADER),
        HeaderValue::from_static(""),
    );
    let empty = server.get(PROTECTED).await.status_code();
    assert_eq!(
        empty, 401,
        "control: an empty x-api-key must not match a configured key: status={empty}"
    );
}
