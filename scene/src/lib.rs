//! Shared scene-spec types for ml-viz.
//!
//! Both the CLI (`ml_viz` binary) and the Yew/WASM viewer (`ml_viz_yew`
//! cdylib) deserialize the same `scene.json` / `scenes/*.yaml`, so the
//! `Scene` schema lives here as the single source of truth.

use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Scene {
    pub scene: SceneMeta,
    pub objects: Vec<ObjectSpec>,
    #[serde(default)]
    pub callouts: Vec<Callout>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneMeta {
    pub title: String,
    pub environment: String,
    pub frames: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ObjectSpec {
    #[serde(rename = "layer_stack")]
    LayerStack {
        name: String,
        label: String,
        position: [f32; 3],
        layers: u32,
        #[serde(default)]
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

impl ObjectSpec {
    pub fn name(&self) -> &str {
        match self {
            ObjectSpec::LayerStack { name, .. }
            | ObjectSpec::NetworkGraph { name, .. }
            | ObjectSpec::MlCube { name, .. }
            | ObjectSpec::RagPipeline { name, .. } => name,
        }
    }

    pub fn label(&self) -> &str {
        match self {
            ObjectSpec::LayerStack { label, .. }
            | ObjectSpec::NetworkGraph { label, .. }
            | ObjectSpec::MlCube { label, .. }
            | ObjectSpec::RagPipeline { label, .. } => label,
        }
    }

    pub fn position(&self) -> [f32; 3] {
        match self {
            ObjectSpec::LayerStack { position, .. }
            | ObjectSpec::NetworkGraph { position, .. }
            | ObjectSpec::MlCube { position, .. }
            | ObjectSpec::RagPipeline { position, .. } => *position,
        }
    }

    pub fn kind(&self) -> &'static str {
        match self {
            ObjectSpec::LayerStack { .. } => "layer_stack",
            ObjectSpec::NetworkGraph { .. } => "network_graph",
            ObjectSpec::MlCube { .. } => "ml_cube",
            ObjectSpec::RagPipeline { .. } => "rag_pipeline",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Anchor {
    Object { object: String },
    World { world: [f32; 3] },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Callout {
    Tooltip {
        anchor: Anchor,
        #[serde(default)]
        offset: [f32; 3],
        #[serde(default)]
        title: Option<String>,
        text: String,
    },
    Link {
        anchor: Anchor,
        #[serde(default)]
        offset: [f32; 3],
        text: String,
        url: String,
    },
    SvgBillboard {
        anchor: Anchor,
        #[serde(default)]
        offset: [f32; 3],
        svg: String,
        #[serde(default = "default_billboard_width")]
        width: u32,
    },
    HtmlPanel {
        anchor: Anchor,
        #[serde(default)]
        offset: [f32; 3],
        #[serde(default)]
        title: Option<String>,
        html: String,
    },
}

fn default_billboard_width() -> u32 {
    260
}

impl Callout {
    pub fn anchor(&self) -> &Anchor {
        match self {
            Callout::Tooltip { anchor, .. }
            | Callout::Link { anchor, .. }
            | Callout::SvgBillboard { anchor, .. }
            | Callout::HtmlPanel { anchor, .. } => anchor,
        }
    }

    pub fn offset(&self) -> [f32; 3] {
        match self {
            Callout::Tooltip { offset, .. }
            | Callout::Link { offset, .. }
            | Callout::SvgBillboard { offset, .. }
            | Callout::HtmlPanel { offset, .. } => *offset,
        }
    }

    pub fn kind_str(&self) -> &'static str {
        match self {
            Callout::Tooltip { .. } => "tooltip",
            Callout::Link { .. } => "link",
            Callout::SvgBillboard { .. } => "svg_billboard",
            Callout::HtmlPanel { .. } => "html_panel",
        }
    }
}

pub fn resolve_anchor(anchor: &Anchor, objects: &[ObjectSpec]) -> Option<[f32; 3]> {
    match anchor {
        Anchor::Object { object } => objects
            .iter()
            .find(|o| o.name() == object)
            .map(|o| o.position()),
        Anchor::World { world } => Some(*world),
    }
}

impl Scene {
    pub fn validate(&self) -> Result<()> {
        ensure!(!self.objects.is_empty(), "scene has no objects");
        for obj in &self.objects {
            match obj {
                ObjectSpec::LayerStack { layers, label, .. } => {
                    ensure!(*layers > 0, "layer stack has zero layers");
                    ensure!(!label.trim().is_empty(), "empty label");
                }
                ObjectSpec::NetworkGraph { nodes, label, .. } => {
                    ensure!(*nodes > 1, "network graph needs at least two nodes");
                    ensure!(!label.trim().is_empty(), "empty label");
                }
                ObjectSpec::MlCube { label, .. } | ObjectSpec::RagPipeline { label, .. } => {
                    ensure!(!label.trim().is_empty(), "empty label");
                }
            }
        }
        Ok(())
    }
}
