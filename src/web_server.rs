//! Small cross-platform HTTP delivery adapter for the bundled browser runtime.
//!
//! Runtime filenames and application paths are allowlisted, keeping `--web`
//! from becoming an ambient file server while preserving GPUI Web's required
//! cross-origin-isolation response headers.

use std::{
    collections::HashMap,
    env, fs,
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    path::{Path, PathBuf},
    time::Duration,
};

use crate::{DEFAULT_PLUGIN_FILE, PluginSource, read_application_file};

pub const WEB_RUNTIME_ENV: &str = "AEDICULE_WEB_RUNTIME";
const MAX_REQUEST_BYTES: usize = 8 * 1024;
const RUNTIME_FILES: &[&str] = &[
    "index.html",
    "bootstrap.js",
    "coi-serviceworker.js",
    "manifest.webmanifest",
    "icon.png",
    "aedicule_web.js",
    "aedicule_web_bg.wasm",
];

pub struct WebServer {
    listener: TcpListener,
    files: HashMap<&'static str, Vec<u8>>,
    code: Vec<u8>,
}

impl WebServer {
    /// Binds only after the runtime bundle and application entrypoint have
    /// passed validation, so a printed URL always denotes a servable app.
    pub fn bind(
        source: &PluginSource,
        runtime: &Path,
        bind: &str,
        port: u16,
    ) -> Result<Self, String> {
        let mut files = HashMap::new();
        for &name in RUNTIME_FILES {
            let path = runtime.join(name);
            let bytes = fs::read(&path)
                .map_err(|error| format!("read web runtime {}: {error}", path.display()))?;
            if bytes.is_empty() {
                return Err(format!("web runtime asset is empty: {}", path.display()));
            }
            files.insert(name, bytes);
        }
        let code = read_application_file(source, DEFAULT_PLUGIN_FILE)?;
        let listener = TcpListener::bind((bind, port))
            .map_err(|error| format!("bind web server to {bind}:{port}: {error}"))?;
        Ok(Self {
            listener,
            files,
            code,
        })
    }

    pub fn url(&self) -> Result<String, String> {
        let address = self
            .listener
            .local_addr()
            .map_err(|error| format!("read web listener address: {error}"))?;
        Ok(format_http_url(address))
    }

    /// Serves requests serially: browser startup fetches a tiny fixed set, and
    /// avoiding a worker pool keeps shutdown/resource ownership predictable.
    pub fn serve(self) -> Result<(), String> {
        for connection in self.listener.incoming() {
            let mut connection =
                connection.map_err(|error| format!("accept web request: {error}"))?;
            connection
                .set_read_timeout(Some(Duration::from_secs(5)))
                .map_err(|error| format!("configure web request timeout: {error}"))?;
            self.serve_connection(&mut connection)?;
        }
        Ok(())
    }

    fn serve_connection(&self, connection: &mut TcpStream) -> Result<(), String> {
        let mut request = [0_u8; MAX_REQUEST_BYTES];
        let length = connection
            .read(&mut request)
            .map_err(|error| format!("read web request: {error}"))?;
        let first_line = request[..length]
            .split(|byte| *byte == b'\n')
            .next()
            .and_then(|line| std::str::from_utf8(line).ok())
            .map(str::trim_end)
            .unwrap_or_default();
        let mut parts = first_line.split_whitespace();
        let method = parts.next().unwrap_or_default();
        let target = parts.next().unwrap_or_default();
        if !matches!(method, "GET" | "HEAD") {
            return write_response(
                connection,
                "405 Method Not Allowed",
                "text/plain; charset=utf-8",
                b"method not allowed\n",
                method == "HEAD",
            );
        }
        let path = target.split('?').next().unwrap_or_default();
        let (content_type, body) = match path {
            "/" | "/index.html" => (mime_type("index.html"), self.files["index.html"].as_slice()),
            "/code.wat" => (mime_type(DEFAULT_PLUGIN_FILE), self.code.as_slice()),
            path if path.starts_with('/') => {
                let name = &path[1..];
                match self.files.get(name) {
                    Some(body) => (mime_type(name), body.as_slice()),
                    None => {
                        return write_response(
                            connection,
                            "404 Not Found",
                            "text/plain; charset=utf-8",
                            b"not found\n",
                            method == "HEAD",
                        );
                    }
                }
            }
            _ => {
                return write_response(
                    connection,
                    "400 Bad Request",
                    "text/plain; charset=utf-8",
                    b"bad request\n",
                    method == "HEAD",
                );
            }
        };
        write_response(connection, "200 OK", content_type, body, method == "HEAD")
    }
}

/// Finds the immutable browser runtime in development or in each native
/// delivery layout; an explicit environment path wins for testability.
pub fn discover_web_runtime() -> Result<PathBuf, String> {
    if let Some(path) = env::var_os(WEB_RUNTIME_ENV).map(PathBuf::from) {
        return validate_runtime_directory(path);
    }
    let executable = env::current_exe()
        .map_err(|error| format!("locate current executable for web runtime: {error}"))?;
    let executable_directory = executable
        .parent()
        .ok_or_else(|| "current executable has no parent directory".to_owned())?;
    let candidates = [
        executable_directory.join("Web"),
        executable_directory.join("web-runtime"),
        executable_directory.join("../Resources/Web"),
        executable_directory.join("../share/aedicule/web"),
        executable_directory.join("../Resources/web-runtime"),
    ];
    for candidate in candidates {
        if candidate.join("index.html").is_file() {
            return validate_runtime_directory(candidate);
        }
    }
    Err(format!(
        "web runtime not found; set {WEB_RUNTIME_ENV} or install the bundled Web runtime"
    ))
}

fn validate_runtime_directory(path: PathBuf) -> Result<PathBuf, String> {
    if path.is_dir() {
        Ok(path)
    } else {
        Err(format!(
            "web runtime is not a directory: {}",
            path.display()
        ))
    }
}

fn format_http_url(address: SocketAddr) -> String {
    match address {
        SocketAddr::V4(address) => format!("http://{}:{}/", address.ip(), address.port()),
        SocketAddr::V6(address) => format!("http://[{}]:{}/", address.ip(), address.port()),
    }
}

fn mime_type(name: &str) -> &'static str {
    match Path::new(name)
        .extension()
        .and_then(|extension| extension.to_str())
    {
        Some("html") => "text/html; charset=utf-8",
        Some("js") => "text/javascript; charset=utf-8",
        Some("json" | "webmanifest") => "application/manifest+json; charset=utf-8",
        Some("png") => "image/png",
        Some("wasm") => "application/wasm",
        Some("wat") => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}

fn write_response(
    connection: &mut TcpStream,
    status: &str,
    content_type: &str,
    body: &[u8],
    head_only: bool,
) -> Result<(), String> {
    write!(
        connection,
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nCross-Origin-Opener-Policy: same-origin\r\nCross-Origin-Embedder-Policy: require-corp\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
        body.len()
    )
    .map_err(|error| format!("write web response headers: {error}"))?;
    if !head_only {
        connection
            .write_all(body)
            .map_err(|error| format!("write web response body: {error}"))?;
    }
    connection
        .flush()
        .map_err(|error| format!("flush web response: {error}"))
}
