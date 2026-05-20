use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use yew::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct SceneMeta {
    title: String,
    environment: String,
    frames: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct ObjectSpec {
    #[serde(rename = "type")]
    kind: String,
    name: String,
    label: String,
    position: [f32; 3],
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
enum Anchor {
    Object { object: String },
    World { world: [f32; 3] },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum Callout {
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
    fn anchor(&self) -> &Anchor {
        match self {
            Callout::Tooltip { anchor, .. }
            | Callout::Link { anchor, .. }
            | Callout::SvgBillboard { anchor, .. }
            | Callout::HtmlPanel { anchor, .. } => anchor,
        }
    }
    fn offset(&self) -> [f32; 3] {
        match self {
            Callout::Tooltip { offset, .. }
            | Callout::Link { offset, .. }
            | Callout::SvgBillboard { offset, .. }
            | Callout::HtmlPanel { offset, .. } => *offset,
        }
    }
    fn kind_str(&self) -> &'static str {
        match self {
            Callout::Tooltip { .. } => "tooltip",
            Callout::Link { .. } => "link",
            Callout::SvgBillboard { .. } => "svg_billboard",
            Callout::HtmlPanel { .. } => "html_panel",
        }
    }
}

fn resolve_anchor(anchor: &Anchor, objects: &[ObjectSpec]) -> Option<[f32; 3]> {
    match anchor {
        Anchor::Object { object } => objects
            .iter()
            .find(|o| &o.name == object)
            .map(|o| o.position),
        Anchor::World { world } => Some(*world),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Scene {
    scene: SceneMeta,
    objects: Vec<ObjectSpec>,
    #[serde(default)]
    callouts: Vec<Callout>,
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = window, js_name = initThree)]
    fn init_three(canvas_id: &str, glb_url: &str);
    #[wasm_bindgen(js_namespace = window, js_name = focusObject)]
    fn focus_object(name: &str, x: f32, y: f32, z: f32);
    #[wasm_bindgen(js_namespace = window, js_name = initCallouts)]
    fn init_callouts(json: &str);
}

#[function_component(App)]
fn app() -> Html {
    let scene = use_state(|| None::<Scene>);
    let error = use_state(|| None::<String>);

    {
        let scene = scene.clone();
        let error = error.clone();
        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                match fetch_scene().await {
                    Ok(s) => {
                        scene.set(Some(s));
                    }
                    Err(e) => error.set(Some(e)),
                }
            });
            || ()
        });
    }

    // Once the scene is in state, fire init_three + init_callouts on the next
    // render so the canvas and DOM callouts exist in the document.
    {
        let scene_dep = scene.clone();
        use_effect_with((*scene_dep).clone(), move |s| {
            if let Some(scene) = s {
                init_three("viz", "/minimal_scene.glb");
                let payload = build_callout_payload(scene);
                init_callouts(&payload);
            }
            || ()
        });
    }

    if let Some(msg) = (*error).clone() {
        return html! { <div class="error">{ format!("scene.json error: {msg}") }</div> };
    }

    let title = scene
        .as_ref()
        .map(|s| s.scene.title.clone())
        .unwrap_or_else(|| "Loading scene…".into());
    let objects = scene
        .as_ref()
        .map(|s| s.objects.clone())
        .unwrap_or_default();
    let callouts = scene
        .as_ref()
        .map(|s| s.callouts.clone())
        .unwrap_or_default();

    html! {
        <>
            <canvas id="viz"></canvas>
            <div id="callouts-layer">
                { for callouts.iter().enumerate().map(|(i, c)| render_callout(i, c)) }
            </div>
            <div id="tooltip" class="callout-tooltip hidden">
                <h3 class="t-title"></h3>
                <p class="t-text"></p>
            </div>
            <aside id="hud">
                <h1>{ title }</h1>
                <p class="hint">{ "Left-drag to orbit · right-drag or arrow keys to pan · scroll to zoom · hover an object for its tooltip · click below to focus" }</p>
                <ul class="objects">
                {
                    for objects.into_iter().map(|o| {
                        let [x, y, z] = o.position;
                        let name = o.name.clone();
                        let onclick = Callback::from(move |_| focus_object(&name, x, y, z));
                        html! {
                            <li>
                                <button {onclick}>
                                    <span class="kind">{ o.kind.clone() }</span>
                                    <span class="label">{ o.label.clone() }</span>
                                </button>
                            </li>
                        }
                    })
                }
                </ul>
                <footer>{ "Rust → wasm-bindgen → Yew · three.js via JS glue" }</footer>
            </aside>
        </>
    }
}

fn render_callout(i: usize, c: &Callout) -> Html {
    let id = format!("callout-{i}");
    match c {
        // Tooltips are dynamic: JS manages the single #tooltip element on hover.
        Callout::Tooltip { .. } => html! {},
        Callout::Link { text, url, .. } => html! {
            <a id={id} class="callout callout-link" href={url.clone()} target="_blank" rel="noopener">
                { text.clone() } <span class="arrow">{ "↗" }</span>
            </a>
        },
        Callout::SvgBillboard { svg, width, .. } => html! {
            <img id={id} class="callout callout-billboard"
                 src={svg.clone()}
                 style={format!("width:{width}px")}
                 alt="callout" />
        },
        Callout::HtmlPanel { title, html, .. } => {
            let body = Html::from_html_unchecked(AttrValue::from(html.clone()));
            yew::html! {
                <div id={id} class="callout callout-panel">
                    { for title.as_ref().map(|t| yew::html! { <h3>{ t.clone() }</h3> }) }
                    <div class="content">{ body }</div>
                </div>
            }
        }
    }
}

fn build_callout_payload(scene: &Scene) -> String {
    let entries: Vec<_> = scene
        .callouts
        .iter()
        .enumerate()
        .filter_map(|(i, c)| {
            let pos = resolve_anchor(c.anchor(), &scene.objects)?;
            let [ox, oy, oz] = c.offset();
            let world = [pos[0] + ox, pos[1] + oy, pos[2] + oz];
            let anchor_object = match c.anchor() {
                Anchor::Object { object } => Some(object.clone()),
                Anchor::World { .. } => None,
            };
            let mut entry = serde_json::json!({
                "id": format!("callout-{i}"),
                "kind": c.kind_str(),
                "position": world,
                "anchor_object": anchor_object,
            });
            if let Callout::Tooltip { title, text, .. } = c {
                entry["title"] = serde_json::Value::String(title.clone().unwrap_or_default());
                entry["text"] = serde_json::Value::String(text.clone());
            }
            Some(entry)
        })
        .collect();
    let scene_objects: Vec<_> = scene
        .objects
        .iter()
        .map(|o| serde_json::json!({ "name": o.name, "position": o.position }))
        .collect();
    serde_json::to_string(&serde_json::json!({
        "callouts": entries,
        "objects": scene_objects,
    }))
    .unwrap_or_else(|_| "{}".into())
}

async fn fetch_scene() -> Result<Scene, String> {
    let resp = gloo_net::http::Request::get("/scene.json")
        .send()
        .await
        .map_err(|e| e.to_string())?;
    resp.json::<Scene>().await.map_err(|e| e.to_string())
}

#[wasm_bindgen(start)]
pub fn start() {
    yew::Renderer::<App>::new().render();
}
