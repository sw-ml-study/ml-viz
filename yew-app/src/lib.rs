use ml_viz_scene::{Anchor, Callout, ObjectSpec, Scene, resolve_anchor};
use wasm_bindgen::prelude::*;
use yew::prelude::*;

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
    let objects: Vec<ObjectSpec> = scene
        .as_ref()
        .map(|s| s.objects.clone())
        .unwrap_or_default();
    let callouts: Vec<Callout> = scene
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
                        let [x, y, z] = o.position();
                        let name = o.name().to_string();
                        let onclick = Callback::from(move |_| focus_object(&name, x, y, z));
                        let kind = o.kind().to_string();
                        let label = o.label().to_string();
                        html! {
                            <li>
                                <button {onclick}>
                                    <span class="kind">{ kind }</span>
                                    <span class="label">{ label }</span>
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
        .map(|o| serde_json::json!({ "name": o.name(), "position": o.position() }))
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
