import * as THREE from 'three';
import { OrbitControls } from 'three/addons/controls/OrbitControls.js';
import { GLTFLoader } from 'three/addons/loaders/GLTFLoader.js';

let camera = null;
let controls = null;
let gltfRoot = null;
let canvas = null;

// Callouts state: each entry has { id, kind, world: Vector3, anchor_object?, title?, text? }
const callouts = [];
const sceneObjects = []; // [{ name, position: [bx, by, bz] }] in Blender Z-up

// Blender's glTF exporter rotates Z-up -> Y-up. Scene-spec positions in
// scene.json are still in Blender's Z-up convention, so we re-map them
// (Bx, By, Bz) -> (Bx, Bz, -By) before handing values to three.js.
function fromBlender(bx, by, bz) {
  return new THREE.Vector3(bx, bz, -by);
}

async function waitForCanvas(id, attempts = 80) {
  for (let i = 0; i < attempts; i++) {
    const c = document.getElementById(id);
    if (c) return c;
    await new Promise(r => setTimeout(r, 16));
  }
  throw new Error(`canvas #${id} did not appear`);
}

window.initThree = async function (canvasId, glbUrl) {
  canvas = await waitForCanvas(canvasId);
  const renderer = new THREE.WebGLRenderer({ canvas, antialias: true });
  renderer.setSize(innerWidth, innerHeight, false);
  renderer.outputColorSpace = THREE.SRGBColorSpace;

  const scene = new THREE.Scene();
  scene.background = new THREE.Color(0xeef3f6);

  camera = new THREE.PerspectiveCamera(55, innerWidth / innerHeight, 0.1, 200);
  camera.position.set(8, 6, 8);
  camera.up.set(0, 1, 0);

  controls = new OrbitControls(camera, canvas);
  controls.target.set(0, 0.6, 0);
  controls.enableDamping = true;
  controls.dampingFactor = 0.08;
  controls.enablePan = true;
  controls.screenSpacePanning = true;
  controls.keyPanSpeed = 16;
  controls.listenToKeyEvents(window);

  scene.add(new THREE.HemisphereLight(0xffffff, 0x889099, 2.2));
  const sun = new THREE.DirectionalLight(0xffffff, 1.6);
  sun.position.set(4, -3, 9);
  scene.add(sun);

  try {
    const gltf = await new GLTFLoader().loadAsync(glbUrl);
    gltfRoot = gltf.scene;
    scene.add(gltfRoot);
  } catch (e) {
    console.warn('GLB load failed:', e);
    const ph = new THREE.Mesh(
      new THREE.BoxGeometry(1.5, 1.5, 1.5),
      new THREE.MeshStandardMaterial({ color: 0xcc6666 })
    );
    ph.position.set(0, 0, 0.75);
    scene.add(ph);
  }

  setupRaycaster();

  function frame() {
    requestAnimationFrame(frame);
    controls.update();
    updateCalloutPositions();
    renderer.render(scene, camera);
  }
  frame();

  addEventListener('resize', () => {
    camera.aspect = innerWidth / innerHeight;
    camera.updateProjectionMatrix();
    renderer.setSize(innerWidth, innerHeight, false);
  });
};

window.focusObject = function (_name, bx, by, bz) {
  if (!camera || !controls) return;
  const target = fromBlender(bx, by, bz);
  controls.target.copy(target);
  camera.position.set(target.x + 4, target.y + 3, target.z + 6);
  controls.update();
};

window.initCallouts = function (jsonStr) {
  const payload = JSON.parse(jsonStr);
  callouts.length = 0;
  sceneObjects.length = 0;
  for (const c of payload.callouts || []) {
    const [bx, by, bz] = c.position;
    callouts.push({
      ...c,
      world: fromBlender(bx, by, bz),
    });
  }
  for (const o of payload.objects || []) {
    sceneObjects.push(o);
  }
};

function updateCalloutPositions() {
  if (!camera) return;
  const tmp = new THREE.Vector3();
  for (const c of callouts) {
    if (c.kind === 'tooltip') continue; // not a DOM element
    const el = document.getElementById(c.id);
    if (!el) continue;
    tmp.copy(c.world).project(camera);
    const onScreen = tmp.z > -1 && tmp.z < 1;
    if (!onScreen) {
      el.style.opacity = '0';
      el.style.pointerEvents = 'none';
      continue;
    }
    const sx = (tmp.x * 0.5 + 0.5) * innerWidth;
    const sy = (-tmp.y * 0.5 + 0.5) * innerHeight;
    el.style.transform = `translate(${sx}px, ${sy}px) translate(-50%, -50%)`;
    el.style.opacity = '1';
    el.style.pointerEvents = 'auto';
  }
}

// --- Raycaster tooltip -------------------------------------------------------

const raycaster = new THREE.Raycaster();
const ndc = new THREE.Vector2();
let currentHover = null;

function setupRaycaster() {
  if (!canvas) return;
  canvas.addEventListener('mousemove', onCanvasMove);
  canvas.addEventListener('mouseleave', () => {
    currentHover = null;
    hideTooltip();
  });
}

function onCanvasMove(e) {
  if (!camera || !gltfRoot) return;
  ndc.x = (e.clientX / innerWidth) * 2 - 1;
  ndc.y = -(e.clientY / innerHeight) * 2 + 1;
  raycaster.setFromCamera(ndc, camera);
  const hits = raycaster.intersectObject(gltfRoot, true);
  const sceneName = hits.length ? findSceneObjectName(hits[0].object.name) : null;
  if (sceneName !== currentHover) {
    currentHover = sceneName;
    if (currentHover) showTooltip(currentHover, e.clientX, e.clientY);
    else hideTooltip();
  } else if (currentHover) {
    positionTooltip(e.clientX, e.clientY);
  }
}

function findSceneObjectName(hitName) {
  // The .glb names primitives like 'encoder_stack_layer_0'. Find the longest
  // scene-object name that is a prefix of the hit name.
  let best = null;
  for (const o of sceneObjects) {
    if (hitName === o.name || hitName.startsWith(o.name + '_')) {
      if (!best || o.name.length > best.length) best = o.name;
    }
  }
  return best;
}

function showTooltip(objectName, mx, my) {
  const data = callouts.find(c => c.kind === 'tooltip' && c.anchor_object === objectName);
  const tip = document.getElementById('tooltip');
  if (!tip) return;
  if (!data) {
    tip.classList.add('hidden');
    return;
  }
  tip.querySelector('.t-title').textContent = data.title || objectName;
  tip.querySelector('.t-text').textContent = data.text || '';
  tip.classList.remove('hidden');
  positionTooltip(mx, my);
}

function hideTooltip() {
  const tip = document.getElementById('tooltip');
  if (tip) tip.classList.add('hidden');
}

function positionTooltip(mx, my) {
  const tip = document.getElementById('tooltip');
  if (!tip || tip.classList.contains('hidden')) return;
  tip.style.left = (mx + 14) + 'px';
  tip.style.top = (my + 14) + 'px';
}
