import * as THREE from 'three';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';
import { GLTFLoader } from 'three/examples/jsm/loaders/GLTFLoader.js';

type ObjectSpec = {
  type: string;
  name: string;
  label: string;
  position: [number, number, number];
};
type SceneSpec = {
  scene: { title: string; environment: string; frames: number };
  objects: ObjectSpec[];
};

const renderer = new THREE.WebGLRenderer({ antialias: true });
renderer.setSize(innerWidth, innerHeight);
renderer.outputColorSpace = THREE.SRGBColorSpace;
document.body.appendChild(renderer.domElement);

const scene = new THREE.Scene();
scene.background = new THREE.Color(0xeef3f6);

const camera = new THREE.PerspectiveCamera(55, innerWidth / innerHeight, 0.1, 200);
camera.position.set(6, -8, 5);

const controls = new OrbitControls(camera, renderer.domElement);
controls.target.set(0, 0, 0.6);
controls.update();

scene.add(new THREE.HemisphereLight(0xffffff, 0x888888, 2.2));
const sun = new THREE.DirectionalLight(0xffffff, 1.6);
sun.position.set(4, -3, 9);
scene.add(sun);

const callouts: THREE.Sprite[] = [];
function textSprite(text: string, x: number, y: number, z: number): THREE.Sprite {
  const canvas = document.createElement('canvas');
  canvas.width = 1024; canvas.height = 256;
  const ctx = canvas.getContext('2d')!;
  ctx.fillStyle = 'rgba(255,255,255,0.88)';
  ctx.fillRect(0, 0, 1024, 256);
  ctx.strokeStyle = '#35a7c8'; ctx.lineWidth = 8;
  ctx.strokeRect(4, 4, 1016, 248);
  ctx.font = '600 72px system-ui, Inter, sans-serif';
  ctx.fillStyle = '#1c5264';
  ctx.fillText(text, 40, 160);
  const tex = new THREE.CanvasTexture(canvas);
  tex.colorSpace = THREE.SRGBColorSpace;
  const sp = new THREE.Sprite(new THREE.SpriteMaterial({ map: tex, transparent: true }));
  sp.position.set(x, y, z);
  sp.scale.set(2.4, 0.6, 1);
  scene.add(sp);
  callouts.push(sp);
  return sp;
}

const hud = document.getElementById('hud')!;
const status = document.createElement('div');
status.style.marginTop = '6px';
status.style.fontSize = '12px';
status.style.color = '#456';
status.textContent = 'Loading scene.json…';
hud.appendChild(status);

(async () => {
  const specResp = await fetch('/scene.json');
  if (!specResp.ok) throw new Error('scene.json missing — run `cargo run -- gen-three` first');
  const spec: SceneSpec = await specResp.json();

  const title = hud.querySelector('b')!;
  title.textContent = spec.scene.title;

  const loader = new GLTFLoader();
  const glbResp = await fetch('/minimal_scene.glb', { method: 'HEAD' });
  if (!glbResp.ok) {
    status.textContent = 'minimal_scene.glb not found in public/. Run blender then re-run gen-three.';
    status.style.color = '#a33';
    animate();
    return;
  }

  status.textContent = 'Loading minimal_scene.glb…';
  const gltf = await loader.loadAsync('/minimal_scene.glb');
  scene.add(gltf.scene);

  // Place a billboard callout above each labeled object spec.
  for (const obj of spec.objects) {
    const [x, y, z] = obj.position;
    textSprite(obj.label, x, y, z + 1.6);
  }

  status.textContent = `Loaded ${spec.objects.length} labeled objects from ${spec.scene.title}.`;
  animate();
})().catch((e) => {
  status.textContent = `Error: ${e.message}`;
  status.style.color = '#a33';
  console.error(e);
});

function animate() {
  requestAnimationFrame(animate);
  for (const c of callouts) c.quaternion.copy(camera.quaternion);
  controls.update();
  renderer.render(scene, camera);
}

addEventListener('resize', () => {
  camera.aspect = innerWidth / innerHeight;
  camera.updateProjectionMatrix();
  renderer.setSize(innerWidth, innerHeight);
});
