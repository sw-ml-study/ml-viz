//! `cargo run --example ml-test-scene`
//!
//! Builds the Yew/WASM frontend with wasm-pack, then serves the static HTML,
//! CSS, JS glue, wasm bundle, scene spec, and the Blender-exported .glb on
//! 0.0.0.0:9521 so the page is reachable from other devices on the LAN.

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};
use tiny_http::{Header, Method, Response, Server, StatusCode};

const BIND: &str = "0.0.0.0:9521";

fn main() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let yew_app = manifest_dir.join("yew-app");
    let static_dir = yew_app.join("static");
    let pkg_dir = yew_app.join("pkg");
    let generated = manifest_dir.join("generated");
    let scenes = manifest_dir.join("scenes");

    if !yew_app.is_dir() {
        eprintln!("error: yew-app/ not found at {}", yew_app.display());
        std::process::exit(1);
    }

    build_wasm(&yew_app);

    let scene_json = load_scene_json(&scenes.join("minimal.yaml"))
        .unwrap_or_else(|e| {
            eprintln!("warn: could not load scenes/minimal.yaml — {e}");
            eprintln!("      run `cargo run -- init-scene scenes/minimal.yaml` first");
            "{}".into()
        });

    let glb_path = generated.join("minimal_scene.glb");
    if !glb_path.is_file() {
        eprintln!("warn: {} not found — run", glb_path.display());
        eprintln!("      cargo run -- gen-blender scenes/minimal.yaml --out generated/minimal_scene.py");
        eprintln!("      blender -b --python generated/minimal_scene.py");
        eprintln!("      to generate it; the page will fall back to a placeholder cube.");
    }

    let server = Server::http(BIND).expect("bind 0.0.0.0:9521");
    println!();
    println!("blender_ml_viz Yew/WASM demo");
    println!("  http://localhost:9521    (this machine)");
    println!("  http://{}                (LAN)", lan_url(BIND));
    println!();
    println!("Press Ctrl-C to stop.");

    for req in server.incoming_requests() {
        if !matches!(req.method(), Method::Get | Method::Head) {
            let _ = req.respond(Response::empty(StatusCode(405)));
            continue;
        }
        let raw = req.url().to_string();
        let path = raw.split('?').next().unwrap_or("/");
        let logical = if path == "/" { "/index.html" } else { path };
        log_req(req.method(), logical);

        if logical == "/scene.json" {
            let _ = req.respond(
                Response::from_string(scene_json.clone())
                    .with_header(header("Content-Type", "application/json"))
                    .with_header(header("Cache-Control", "no-store")),
            );
            continue;
        }

        match resolve(logical, &static_dir, &pkg_dir, &generated, &manifest_dir) {
            Some(file) => match fs::read(&file) {
                Ok(bytes) => {
                    let mime = mime_for(&file);
                    let _ = req.respond(
                        Response::from_data(bytes)
                            .with_header(header("Content-Type", mime))
                            .with_header(header("Cache-Control", "no-store")),
                    );
                }
                Err(e) => {
                    eprintln!("  500 {}: {e}", file.display());
                    let _ = req.respond(
                        Response::from_string(format!("read error: {e}"))
                            .with_status_code(500),
                    );
                }
            },
            None => {
                let _ = req.respond(
                    Response::from_string(format!("not found: {logical}"))
                        .with_status_code(404),
                );
            }
        }
    }
}

fn build_wasm(yew_app: &Path) {
    println!("→ wasm-pack build (yew-app)");
    let status = Command::new("wasm-pack")
        .args([
            "build",
            "--target",
            "web",
            "--out-dir",
            "pkg",
            "--out-name",
            "viz",
            "--dev",
        ])
        .current_dir(yew_app)
        .status();
    match status {
        Ok(s) if s.success() => {}
        Ok(s) => {
            eprintln!("wasm-pack exited with status {s}");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("failed to invoke wasm-pack: {e}");
            eprintln!("install with: cargo install wasm-pack");
            std::process::exit(1);
        }
    }
}

fn load_scene_json(yaml_path: &Path) -> Result<String, String> {
    let text = fs::read_to_string(yaml_path).map_err(|e| e.to_string())?;
    let value: serde_yaml::Value = serde_yaml::from_str(&text).map_err(|e| e.to_string())?;
    serde_json::to_string_pretty(&value).map_err(|e| e.to_string())
}

fn resolve(
    logical: &str,
    static_dir: &Path,
    pkg_dir: &Path,
    generated: &Path,
    manifest_dir: &Path,
) -> Option<PathBuf> {
    let trimmed = logical.trim_start_matches('/');
    if trimmed.contains("..") {
        return None;
    }
    if let Some(rest) = trimmed.strip_prefix("pkg/") {
        return file_in(pkg_dir, rest);
    }
    if let Some(rest) = trimmed.strip_prefix("assets/") {
        return file_in(&manifest_dir.join("assets"), rest);
    }
    if let Some(p) = file_in(static_dir, trimmed) {
        return Some(p);
    }
    if let Some(p) = file_in(generated, trimmed) {
        return Some(p);
    }
    None
}

fn file_in(base: &Path, rest: &str) -> Option<PathBuf> {
    let candidate = base.join(rest);
    if candidate.is_file() {
        Some(candidate)
    } else {
        None
    }
}

fn mime_for(p: &Path) -> &'static str {
    match p.extension().and_then(|s| s.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") | Some("mjs") => "application/javascript; charset=utf-8",
        Some("wasm") => "application/wasm",
        Some("json") => "application/json",
        Some("glb") => "model/gltf-binary",
        Some("gltf") => "model/gltf+json",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("ico") => "image/x-icon",
        _ => "application/octet-stream",
    }
}

fn header(name: &str, value: &str) -> Header {
    Header::from_bytes(name.as_bytes(), value.as_bytes()).expect("static header")
}

fn log_req(method: &Method, path: &str) {
    println!("  {method:>4} {path}");
}

fn lan_url(bind: &str) -> String {
    // Best-effort LAN hint without bringing in extra deps: ask the OS for a
    // routable address by connecting a UDP socket to a public IP.
    use std::net::UdpSocket;
    let port = bind.rsplit(':').next().unwrap_or("9521");
    let ip = UdpSocket::bind("0.0.0.0:0")
        .and_then(|s| {
            s.connect("1.1.1.1:80")?;
            s.local_addr()
        })
        .map(|a| a.ip().to_string())
        .unwrap_or_else(|_| "0.0.0.0".into());
    format!("{ip}:{port}")
}
