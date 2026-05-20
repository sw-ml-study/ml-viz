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

#[cfg(test)]
mod tests {
    use super::*;

    const MINIMAL_YAML: &str = include_str!("../../scenes/minimal.yaml");
    const INIT_TEMPLATE_YAML: &str = include_str!("../../templates/minimal_scene.yaml");

    fn meta() -> SceneMeta {
        SceneMeta {
            title: "t".into(),
            environment: "e".into(),
            frames: 1,
        }
    }

    fn ml_cube(name: &str, label: &str) -> ObjectSpec {
        ObjectSpec::MlCube {
            name: name.into(),
            label: label.into(),
            position: [0.0, 0.0, 0.0],
        }
    }

    fn scene(objects: Vec<ObjectSpec>) -> Scene {
        Scene {
            scene: meta(),
            objects,
            callouts: vec![],
        }
    }

    #[test]
    fn validate_accepts_minimal_scene() {
        let s: Scene = serde_yaml::from_str(MINIMAL_YAML).expect("parse minimal.yaml");
        s.validate().expect("minimal scene is valid");
    }

    #[test]
    fn validate_rejects_empty_objects() {
        let err = scene(vec![]).validate().unwrap_err().to_string();
        assert!(err.contains("no objects"), "got {err}");
    }

    #[test]
    fn validate_rejects_zero_layers() {
        let s = scene(vec![ObjectSpec::LayerStack {
            name: "enc".into(),
            label: "Encoder".into(),
            position: [0.0, 0.0, 0.0],
            layers: 0,
            svg: None,
        }]);
        let err = s.validate().unwrap_err().to_string();
        assert!(err.contains("zero layers"), "got {err}");
    }

    #[test]
    fn validate_rejects_single_node_graph() {
        for n in [0u32, 1] {
            let s = scene(vec![ObjectSpec::NetworkGraph {
                name: "g".into(),
                label: "Graph".into(),
                position: [0.0, 0.0, 0.0],
                nodes: n,
            }]);
            let err = s.validate().unwrap_err().to_string();
            assert!(err.contains("at least two nodes"), "n={n}: {err}");
        }
    }

    #[test]
    fn validate_rejects_empty_label() {
        let err = scene(vec![ml_cube("c", "")])
            .validate()
            .unwrap_err()
            .to_string();
        assert!(err.contains("empty label"), "got {err}");
    }

    #[test]
    fn validate_rejects_whitespace_label() {
        let err = scene(vec![ml_cube("c", "   \t")])
            .validate()
            .unwrap_err()
            .to_string();
        assert!(err.contains("empty label"), "got {err}");
    }

    #[test]
    fn resolve_anchor_object_match() {
        let objects = vec![ml_cube("decoder", "Decoder")];
        let got = resolve_anchor(
            &Anchor::Object {
                object: "decoder".into(),
            },
            &objects,
        );
        assert_eq!(got, Some([0.0, 0.0, 0.0]));
    }

    #[test]
    fn resolve_anchor_object_no_match() {
        let objects = vec![ml_cube("decoder", "Decoder")];
        let got = resolve_anchor(
            &Anchor::Object {
                object: "missing".into(),
            },
            &objects,
        );
        assert_eq!(got, None);
    }

    #[test]
    fn resolve_anchor_world() {
        let got = resolve_anchor(
            &Anchor::World {
                world: [1.0, 2.0, 3.0],
            },
            &[],
        );
        assert_eq!(got, Some([1.0, 2.0, 3.0]));
    }

    #[test]
    fn init_template_parses_and_validates() {
        let s: Scene =
            serde_yaml::from_str(INIT_TEMPLATE_YAML).expect("parse templates/minimal_scene.yaml");
        s.validate()
            .expect("init template must satisfy the live schema");
    }

    #[test]
    fn minimal_yaml_serde_roundtrip() {
        let first: Scene = serde_yaml::from_str(MINIMAL_YAML).expect("parse 1");
        let yaml = serde_yaml::to_string(&first).expect("serialize");
        let second: Scene = serde_yaml::from_str(&yaml).expect("parse 2");
        assert_eq!(first, second);
    }
}
