//! Small cross-platform HTTP delivery adapter for the bundled browser runtime.
//!
//! Runtime filenames and application paths are allowlisted, keeping `--web`
//! from becoming an ambient file server while serving the same ordinary-memory
//! runtime used by static Web delivery.

use std::{
    collections::HashMap,
    env,
    fmt::Write as _,
    fs,
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    path::{Path, PathBuf},
    time::Duration,
};

use crate::{
    ApplicationAssets, DEFAULT_PLUGIN_FILE, PluginSource, read_application_assets,
    read_application_file,
};

pub const WEB_RUNTIME_ENV: &str = "AEDICULE_WEB_RUNTIME";
const MAX_REQUEST_BYTES: usize = 8 * 1024;
const RUNTIME_FILES: &[&str] = &[
    "index.html",
    "bootstrap.js",
    "audio.mjs",
    "motion-input.mjs",
    "touch-input.mjs",
    "local-application.mjs",
    "startup-lock.mjs",
    "service-worker-retirement.mjs",
    "launcher-i18n.mjs",
    "manifest.webmanifest",
    "icon.png",
    "aedicule_web.js",
];
const CONTENT_ADDRESSED_MODULE_SHAPES: &[(&str, &str)] = &[
	("aedicule_web.", ".js"),
	("audio.", ".mjs"),
	("bootstrap.", ".js"),
	("launcher-i18n.", ".mjs"),
	("local-application.", ".mjs"),
	("motion-input.", ".mjs"),
	("service-worker-retirement.", ".mjs"),
	("startup-lock.", ".mjs"),
	("touch-input.", ".mjs"),
];
const IMMUTABLE_CACHE_CONTROL: &str = "public, max-age=31536000, immutable";
const MUTABLE_CACHE_CONTROL: &str = "no-store";

pub struct WebServer {
    listener: TcpListener,
    files: HashMap<String, Vec<u8>>,
    code: Vec<u8>,
    asset_catalog: Vec<u8>,
    assets: ApplicationAssets,
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
            files.insert(name.to_owned(), bytes);
        }
		for path in find_content_addressed_runtime_modules(runtime)? {
			let name = path
				.file_name()
				.and_then(|name| name.to_str())
				.expect("validated runtime module filename is UTF-8")
				.to_owned();
			let bytes = fs::read(&path)
				.map_err(|error| format!("read web runtime {}: {error}", path.display()))?;
			if bytes.is_empty() {
				return Err(format!("web runtime asset is empty: {}", path.display()));
			}
			files.insert(name, bytes);
		}
        let runtime_wasm = find_immutable_runtime_wasm(runtime)?;
        let runtime_wasm_name = runtime_wasm
            .file_name()
            .and_then(|name| name.to_str())
            .expect("validated runtime Wasm filename is UTF-8")
            .to_owned();
        let runtime_wasm_bytes = fs::read(&runtime_wasm)
            .map_err(|error| format!("read web runtime {}: {error}", runtime_wasm.display()))?;
        if runtime_wasm_bytes.is_empty() {
            return Err(format!(
                "web runtime asset is empty: {}",
                runtime_wasm.display()
            ));
        }
        files.insert(runtime_wasm_name, runtime_wasm_bytes);
        let code = read_application_file(source, DEFAULT_PLUGIN_FILE)?;
        let assets = read_application_assets(source)?;
        let asset_catalog = json_asset_catalog(assets.keys().map(String::as_str));
        let listener = TcpListener::bind((bind, port))
            .map_err(|error| format!("bind web server to {bind}:{port}: {error}"))?;
        Ok(Self {
            listener,
            files,
            code,
            asset_catalog,
            assets,
        })
    }

    pub fn url(&self) -> Result<String, String> {
        let address = self
            .listener
            .local_addr()
            .map_err(|error| format!("read web listener address: {error}"))?;
        Ok(format_http_url(address))
    }

    /// Serves the small fixed runtime request set while treating browser-side
    /// cancellation as local to one connection rather than server-fatal.
    pub fn serve(self) -> Result<(), String> {
        for connection in self.listener.incoming() {
            let mut connection =
                connection.map_err(|error| format!("accept web request: {error}"))?;
            connection
                .set_read_timeout(Some(Duration::from_secs(5)))
                .map_err(|error| format!("configure web request timeout: {error}"))?;
            if let Err(error) = self.serve_connection(&mut connection) {
                if is_recoverable_connection_error(error.kind()) {
                    continue;
                }
                return Err(format!("serve web request: {error}"));
            }
        }
        Ok(())
    }

    fn serve_connection(&self, connection: &mut TcpStream) -> std::io::Result<()> {
        let mut request = [0_u8; MAX_REQUEST_BYTES];
        let length = connection.read(&mut request)?;
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
                MUTABLE_CACHE_CONTROL,
            );
        }
        let path = target.split('?').next().unwrap_or_default();
        let (content_type, body, cache_control) = match path {
            "/" | "/index.html" => (
                mime_type("index.html"),
                self.files["index.html"].as_slice(),
                MUTABLE_CACHE_CONTROL,
            ),
            "/code.wat" => (
                mime_type(DEFAULT_PLUGIN_FILE),
                self.code.as_slice(),
                MUTABLE_CACHE_CONTROL,
            ),
            "/application-assets.json" => (
                "application/json; charset=utf-8",
                self.asset_catalog.as_slice(),
                MUTABLE_CACHE_CONTROL,
            ),
            path if path.starts_with('/') => {
                let name = percent_decode_path(&path[1..]);
                let body = name.as_deref().and_then(|name| {
                    self.files
                        .get(name)
                        .map(Vec::as_slice)
                        .or_else(|| self.assets.get(name).map(Vec::as_slice))
                });
                match body {
                    Some(body) => (
                        mime_type(name.as_deref().unwrap()),
                        body,
                        cache_control(name.as_deref().unwrap()),
                    ),
                    None => {
                        return write_response(
                            connection,
                            "404 Not Found",
                            "text/plain; charset=utf-8",
                            b"not found\n",
                            method == "HEAD",
                            MUTABLE_CACHE_CONTROL,
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
                    MUTABLE_CACHE_CONTROL,
                );
            }
        };
        write_response(
            connection,
            "200 OK",
            content_type,
            body,
            method == "HEAD",
            cache_control,
        )
    }
}

/// Locates the one hash-named Wasm payload emitted by the web build, rejecting
/// ambiguous runtime directories before their assets can become HTTP routes.
fn find_immutable_runtime_wasm(runtime: &Path) -> Result<PathBuf, String> {
    let mut candidates = fs::read_dir(runtime)
        .map_err(|error| format!("read web runtime {}: {error}", runtime.display()))?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(is_immutable_runtime_asset)
        })
        .collect::<Vec<_>>();
    candidates.sort();
    match candidates.as_slice() {
        [path] => Ok(path.clone()),
        [] => Err("web runtime has no content-addressed Aedicule Wasm".to_owned()),
        _ => Err("web runtime has multiple content-addressed Aedicule Wasm files".to_owned()),
    }
}

/// Finds only build-produced, hash-named modules with known runtime roles, so
/// `--web` can serve a versioned import graph without exposing other files.
fn find_content_addressed_runtime_modules(runtime: &Path) -> Result<Vec<PathBuf>, String> {
	let mut modules = fs::read_dir(runtime)
		.map_err(|error| format!("read web runtime {}: {error}", runtime.display()))?
		.filter_map(|entry| entry.ok())
		.map(|entry| entry.path())
		.filter(|path| {
			path.is_file()
				&& path
					.file_name()
					.and_then(|name| name.to_str())
					.is_some_and(is_content_addressed_runtime_module)
		})
		.collect::<Vec<_>>();
	modules.sort();
	Ok(modules)
}

fn is_lowercase_sha256(value: &str) -> bool {
	value.len() == 64
		&& value
			.bytes()
			.all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn is_content_addressed_runtime_module(name: &str) -> bool {
	CONTENT_ADDRESSED_MODULE_SHAPES.iter().any(|(prefix, suffix)| {
		name.strip_prefix(prefix)
			.and_then(|rest| rest.strip_suffix(suffix))
			.is_some_and(is_lowercase_sha256)
	})
}

/// Recognizes only the lowercase SHA-256 URL form produced by the Nix build,
/// making a one-year immutable cache lifetime safe by construction.
fn is_immutable_runtime_asset(name: &str) -> bool {
    const PREFIX: &str = "aedicule_web_bg.";
    const SUFFIX: &str = ".wasm";
    let Some(hash) = name
        .strip_prefix(PREFIX)
        .and_then(|rest| rest.strip_suffix(SUFFIX))
    else {
        return false;
    };
	is_lowercase_sha256(hash)
}

fn cache_control(name: &str) -> &'static str {
    if is_immutable_runtime_asset(name) {
        IMMUTABLE_CACHE_CONTROL
    } else {
        MUTABLE_CACHE_CONTROL
    }
}

/// Browser preloaders routinely cancel speculative requests; those client-local
/// disconnects must not terminate the application server's accept loop.
fn is_recoverable_connection_error(kind: std::io::ErrorKind) -> bool {
    matches!(
        kind,
        std::io::ErrorKind::BrokenPipe
            | std::io::ErrorKind::ConnectionReset
            | std::io::ErrorKind::ConnectionAborted
            | std::io::ErrorKind::UnexpectedEof
            | std::io::ErrorKind::TimedOut
            | std::io::ErrorKind::WouldBlock
    )
}

/// Emits a sorted JSON path list without adding a serializer to the tiny
/// server adapter; package validation already guarantees well-formed UTF-8.
fn json_asset_catalog<'a>(names: impl IntoIterator<Item = &'a str>) -> Vec<u8> {
    let mut output = String::from("[");
    for (index, name) in names.into_iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push('"');
        for character in name.chars() {
            match character {
                '"' => output.push_str("\\\""),
                '\\' => output.push_str("\\\\"),
                '\u{08}' => output.push_str("\\b"),
                '\u{0c}' => output.push_str("\\f"),
                '\n' => output.push_str("\\n"),
                '\r' => output.push_str("\\r"),
                '\t' => output.push_str("\\t"),
                character if character <= '\u{1f}' => {
                    write!(&mut output, "\\u{:04x}", u32::from(character))
                        .expect("writing JSON to a String cannot fail");
                }
                character => output.push(character),
            }
        }
        output.push('"');
    }
    output.push_str("]\n");
    output.into_bytes()
}

/// Decodes one URL path into its validated virtual-package name; only an
/// exact key in the preloaded asset map can ever become a response.
fn percent_decode_path(path: &str) -> Option<String> {
    let bytes = path.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            let high = *bytes.get(index + 1)?;
            let low = *bytes.get(index + 2)?;
            decoded.push(hex_digit(high)? << 4 | hex_digit(low)?);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(decoded).ok()
}

fn hex_digit(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
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
        Some("js" | "mjs") => "text/javascript; charset=utf-8",
        Some("json" | "webmanifest") => "application/manifest+json; charset=utf-8",
        Some("png") => "image/png",
        Some("flac") => "audio/flac",
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
    cache_control: &str,
) -> std::io::Result<()> {
    write!(
        connection,
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nCache-Control: {cache_control}\r\nConnection: close\r\n\r\n",
        body.len(),
    )?;
    if !head_only {
        connection.write_all(body)?;
    }
    connection.flush()
}

#[cfg(test)]
mod tests {
    use std::io::ErrorKind;

    use super::{
		cache_control, is_content_addressed_runtime_module, is_immutable_runtime_asset,
		is_recoverable_connection_error, json_asset_catalog, mime_type, percent_decode_path,
    };

	#[test]
	fn content_addressed_runtime_module_names_are_classified_over_a_set() {
		let hash = "0123456789abcdef".repeat(4);
		let cases = [
			(format!("bootstrap.{hash}.js"), true),
			(format!("touch-input.{hash}.mjs"), true),
			(format!("motion-input.{hash}.mjs"), true),
			(format!("unknown.{hash}.mjs"), false),
			(format!("bootstrap.{}.js", hash.to_uppercase()), false),
			(format!("bootstrap.{hash}0.js"), false),
			("bootstrap.js".to_owned(), false),
		];
		assert_eq!(
			cases
				.iter()
				.map(|(name, _)| is_content_addressed_runtime_module(name))
				.collect::<Vec<_>>(),
			cases.iter().map(|(_, expected)| *expected).collect::<Vec<_>>()
		);
	}

    #[test]
    fn asset_catalog_escapes_all_json_sensitive_path_characters() {
        assert_eq!(
			json_asset_catalog([
				"assets/audio/a.flac",
				"assets/quote\"and\nline.flac",
				"assets/snowman-☃.flac",
			]),
			b"[\"assets/audio/a.flac\",\"assets/quote\\\"and\\nline.flac\",\"assets/snowman-\xe2\x98\x83.flac\"]\n"
		);
    }

    #[test]
    fn percent_decoding_classifies_paths_as_a_set() {
        let cases = [
            ("assets/audio/a.flac", Some("assets/audio/a.flac")),
            ("assets/space%20name.flac", Some("assets/space name.flac")),
            (
                "assets/snowman-%E2%98%83.flac",
                Some("assets/snowman-☃.flac"),
            ),
            (
                "assets/encoded%2fslash.flac",
                Some("assets/encoded/slash.flac"),
            ),
            ("assets/bad%", None),
            ("assets/bad%gg", None),
            ("assets/bad%ff", None),
        ];
        assert_eq!(
            cases
                .iter()
                .map(|(input, _)| percent_decode_path(input))
                .collect::<Vec<_>>(),
            cases
                .iter()
                .map(|(_, expected)| expected.map(str::to_owned))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn browser_disconnect_errors_are_classified_over_a_set() {
        let cases = [
            (ErrorKind::BrokenPipe, true),
            (ErrorKind::ConnectionReset, true),
            (ErrorKind::ConnectionAborted, true),
            (ErrorKind::UnexpectedEof, true),
            (ErrorKind::TimedOut, true),
            (ErrorKind::WouldBlock, true),
            (ErrorKind::InvalidData, false),
            (ErrorKind::Other, false),
        ];
        assert_eq!(
            cases
                .iter()
                .map(|(kind, _)| is_recoverable_connection_error(*kind))
                .collect::<Vec<_>>(),
            cases
                .iter()
                .map(|(_, expected)| *expected)
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn browser_runtime_mime_types_are_classified_over_a_set() {
        for (name, expected) in [
            ("index.html", "text/html; charset=utf-8"),
            ("bootstrap.js", "text/javascript; charset=utf-8"),
            ("audio.mjs", "text/javascript; charset=utf-8"),
            ("startup-lock.mjs", "text/javascript; charset=utf-8"),
            ("launcher-i18n.mjs", "text/javascript; charset=utf-8"),
            ("icon.png", "image/png"),
            ("sample.flac", "audio/flac"),
            ("aedicule_web_bg.wasm", "application/wasm"),
            ("code.wat", "text/plain; charset=utf-8"),
        ] {
            assert_eq!(mime_type(name), expected, "{name}");
        }
    }

    #[test]
    fn immutable_runtime_cache_policy_is_classified_over_a_set() {
        let hash = "0123456789abcdef".repeat(4);
        let cases = [
            (
                format!("aedicule_web_bg.{hash}.wasm"),
                true,
                "public, max-age=31536000, immutable",
            ),
			(
				format!("bootstrap.{hash}.js"),
				false,
				"no-store",
			),
            (
                format!("aedicule_web_bg.{}.wasm", hash.to_uppercase()),
                false,
                "no-store",
            ),
            (format!("aedicule_web_bg.{}0.wasm", hash), false, "no-store"),
            ("aedicule_web_bg.wasm".to_owned(), false, "no-store"),
            ("bootstrap.js".to_owned(), false, "no-store"),
            ("code.wat".to_owned(), false, "no-store"),
        ];

        assert_eq!(
            cases
                .iter()
                .map(|(name, _, _)| is_immutable_runtime_asset(name))
                .collect::<Vec<_>>(),
            cases
                .iter()
                .map(|(_, expected, _)| *expected)
                .collect::<Vec<_>>()
        );
        assert_eq!(
            cases
                .iter()
                .map(|(name, _, _)| cache_control(name))
                .collect::<Vec<_>>(),
            cases
                .iter()
                .map(|(_, _, expected)| *expected)
                .collect::<Vec<_>>()
        );
    }
}
