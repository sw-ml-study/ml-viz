use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};
use tera::{Context as TeraContext, Tera};

#[derive(Parser)]
#[command(name = "ml-viz")]
#[command(about = "Generate Blender batch scripts and three.js/Yew examples from ML scene specs")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    InitScene {
        #[arg(default_value = "scenes/minimal.yaml")]
        out: PathBuf,
    },
    GenBlender {
        scene: PathBuf,
        #[arg(short, long, default_value = "generated/minimal_scene.py")]
        out: PathBuf,
    },
    GenThree {
        scene: PathBuf,
        #[arg(short, long, default_value = "web-example")]
        out_dir: PathBuf,
    },
    GenSvg {
        #[arg(short, long, default_value = "Encoder Stack")]
        title: String,
        #[arg(short, long, default_value = "assets/svg/callout_encoder.svg")]
        out: PathBuf,
    },
    Validate {
        scene: PathBuf,
    },
}

#[derive(Debug, Serialize, Deserialize)]
struct Scene {
    scene: SceneMeta,
    objects: Vec<ObjectSpec>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SceneMeta {
    title: String,
    environment: String,
    frames: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum ObjectSpec {
    #[serde(rename = "layer_stack")]
    LayerStack {
        name: String,
        label: String,
        position: [f32; 3],
        layers: u32,
        svg: Option<String>,
    },
    #[serde(rename = "network_graph")]
    NetworkGraph {
        name: String,
        label: String,
        position: [f32; 3],
        nodes: u32,
    },
    #[serde(rename = "ml_cube")]
    MlCube {
        name: String,
        label: String,
        position: [f32; 3],
    },
    #[serde(rename = "rag_pipeline")]
    RagPipeline {
        name: String,
        label: String,
        position: [f32; 3],
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::InitScene { out } => init_scene(&out),
        Commands::GenBlender { scene, out } => gen_blender(&scene, &out),
        Commands::GenThree { scene, out_dir } => gen_three(&scene, &out_dir),
        Commands::GenSvg { title, out } => gen_svg(&title, &out),
        Commands::Validate { scene } => validate(&scene),
    }
}

fn init_scene(out: &Path) -> Result<()> {
    let yaml = include_str!("../templates/minimal_scene.yaml");
    write_file(out, yaml)?;
    Ok(())
}

fn gen_svg(title: &str, out: &Path) -> Result<()> {
    let body = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="900" height="420" viewBox="0 0 900 420">
  <defs>
    <filter id="shadow"><feDropShadow dx="0" dy="8" stdDeviation="8" flood-opacity="0.25"/></filter>
  </defs>
  <rect x="24" y="24" width="852" height="372" rx="32" fill="#f9fbff" stroke="#35a7c8" stroke-width="6" filter="url(#shadow)"/>
  <text x="70" y="95" font-family="Inter, Arial" font-size="48" font-weight="700" fill="#1c5264">{}</text>
  <text x="70" y="155" font-family="Inter, Arial" font-size="30" fill="#445">Reusable labeled SVG callout</text>
  <g transform="translate(70,205)">
    <rect x="0" y="0" width="150" height="44" rx="12" fill="#dff7ff" stroke="#35a7c8"/>
    <rect x="190" y="0" width="150" height="44" rx="12" fill="#eaf2ff" stroke="#709ee8"/>
    <rect x="380" y="0" width="150" height="44" rx="12" fill="#fff4da" stroke="#d8a431"/>
    <path d="M150 22 L190 22 M340 22 L380 22" stroke="#555" stroke-width="4" marker-end="url(#arrow)"/>
    <text x="35" y="30" font-family="Inter, Arial" font-size="22">tokens</text>
    <text x="220" y="30" font-family="Inter, Arial" font-size="22">layers</text>
    <text x="418" y="30" font-family="Inter, Arial" font-size="22">logits</text>
  </g>
  <defs><marker id="arrow" markerWidth="10" markerHeight="10" refX="9" refY="3" orient="auto" markerUnits="strokeWidth"><path d="M0,0 L0,6 L9,3 z" fill="#555"/></marker></defs>
</svg>"##,
        xml_escape(title)
    );
    write_file(out, &body)
}

fn validate(scene_path: &Path) -> Result<()> {
    let scene = read_scene(scene_path)?;
    anyhow::ensure!(!scene.objects.is_empty(), "scene has no objects");
    for obj in &scene.objects {
        match obj {
            ObjectSpec::LayerStack { layers, label, .. } => {
                anyhow::ensure!(*layers > 0, "layer stack has zero layers");
                anyhow::ensure!(!label.trim().is_empty(), "empty label");
            }
            ObjectSpec::NetworkGraph { nodes, label, .. } => {
                anyhow::ensure!(*nodes > 1, "network graph needs at least two nodes");
                anyhow::ensure!(!label.trim().is_empty(), "empty label");
            }
            ObjectSpec::MlCube { label, .. } | ObjectSpec::RagPipeline { label, .. } => {
                anyhow::ensure!(!label.trim().is_empty(), "empty label")
            }
        }
    }
    println!("OK: {} objects", scene.objects.len());
    Ok(())
}

fn gen_blender(scene_path: &Path, out: &Path) -> Result<()> {
    let scene = read_scene(scene_path)?;
    let mut tera = Tera::default();
    tera.add_raw_template(
        "blender.py",
        include_str!("../templates/blender/scene.py.tera"),
    )?;
    let mut ctx = TeraContext::new();
    ctx.insert("scene", &scene);
    let rendered = tera.render("blender.py", &ctx)?;
    write_file(out, &rendered)
}

fn gen_three(scene_path: &Path, out_dir: &Path) -> Result<()> {
    let scene = read_scene(scene_path)?;
    fs::create_dir_all(out_dir.join("src"))?;
    fs::create_dir_all(out_dir.join("public/assets/svg"))?;
    let scene_json = serde_json::to_string_pretty(&scene)?;
    write_file(&out_dir.join("public/scene.json"), &scene_json)?;
    write_file(
        &out_dir.join("package.json"),
        include_str!("../templates/threejs/package.json"),
    )?;
    write_file(
        &out_dir.join("index.html"),
        include_str!("../templates/threejs/index.html"),
    )?;
    write_file(
        &out_dir.join("src/main.ts"),
        include_str!("../templates/threejs/main.ts"),
    )?;
    copy_glb_if_present(
        Path::new("generated/minimal_scene.glb"),
        &out_dir.join("public/minimal_scene.glb"),
    );
    Ok(())
}

fn copy_glb_if_present(src: &Path, dst: &Path) {
    if src.exists() {
        if let Some(parent) = dst.parent() {
            let _ = fs::create_dir_all(parent);
        }
        match fs::copy(src, dst) {
            Ok(_) => println!("copied {} -> {}", src.display(), dst.display()),
            Err(e) => eprintln!("warn: failed to copy {}: {}", src.display(), e),
        }
    } else {
        eprintln!(
            "note: {} not found yet — run `blender -b --python generated/minimal_scene.py` then re-run this command",
            src.display()
        );
    }
}

fn read_scene(path: &Path) -> Result<Scene> {
    let s = fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    serde_yaml::from_str(&s).with_context(|| format!("parsing {}", path.display()))
}

fn write_file(path: &Path, body: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, body).with_context(|| format!("writing {}", path.display()))
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
