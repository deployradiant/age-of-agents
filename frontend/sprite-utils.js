// ── Sprite / Texture utilities for Age of Agents ────────────────────────
// Handles white-background removal via canvas chroma key,
// loads texture atlases, and manages sprite animations.

/**
 * Load an image and convert it to a DataTexture with transparency.
 * Pure white pixels (>240 R, G, B) become transparent.
 */
function loadSpriteTexture(url, threshold = 240) {
  return new Promise((resolve) => {
    const img = new Image();
    img.crossOrigin = "anonymous";
    img.onload = () => {
      const canvas = document.createElement("canvas");
      canvas.width = img.width;
      canvas.height = img.height;
      const ctx = canvas.getContext("2d");
      ctx.drawImage(img, 0, 0);
      
      const imageData = ctx.getImageData(0, 0, canvas.width, canvas.height);
      const data = imageData.data;
      
      for (let i = 0; i < data.length; i += 4) {
        const r = data[i], g = data[i + 1], b = data[i + 2];
        // Chroma key: white pixels → transparent
        if (r >= threshold && g >= threshold && b >= threshold) {
          data[i + 3] = 0; // alpha = 0
        }
      }
      ctx.putImageData(imageData, 0, 0);
      
      const texture = new THREE.CanvasTexture(canvas);
      texture.needsUpdate = true;
      resolve(texture);
    };
    img.onerror = () => {
      // Fallback: just load it anyway
      const tex = new THREE.TextureLoader().load(url);
      resolve(tex);
    };
    img.src = url;
  });
}

/**
 * Create a sprite (billboard) from a texture — always faces camera.
 */
function createBillboardSprite(texture, scale = 1) {
  const material = new THREE.SpriteMaterial({
    map: texture,
    transparent: true,
    depthWrite: false,
    sizeAttenuation: true,
  });
  const sprite = new THREE.Sprite(material);
  sprite.scale.set(scale, scale, 1);
  return sprite;
}

/**
 * Create a ground-plane tile from a texture.
 */
function createTile(texture, width, height) {
  const material = new THREE.MeshBasicMaterial({
    map: texture,
    transparent: true,
    side: THREE.DoubleSide,
  });
  const geo = new THREE.PlaneGeometry(width, height);
  const mesh = new THREE.Mesh(geo, material);
  mesh.rotation.x = -Math.PI / 2;
  return mesh;
}

/**
 * Create an upright billboard (for buildings — always faces camera).
 */
function createBillboard(texture, width, height) {
  const material = new THREE.MeshBasicMaterial({
    map: texture,
    transparent: true,
    side: THREE.DoubleSide,
  });
  const geo = new THREE.PlaneGeometry(width, height);
  const mesh = new THREE.Mesh(geo, material);
  return mesh;
}

/**
 * Simple sprite animator: cycles through textures.
 */
class SpriteAnimator {
  constructor(frames, fps = 4) {
    this.frames = frames;     // Array of textures
    this.fps = fps;
    this.currentFrame = 0;
    this.lastTime = performance.now();
  }
  
  update(time) {
    if (this.frames.length <= 1) return this.frames[0];
    const elapsed = time - this.lastTime;
    if (elapsed >= 1000 / this.fps) {
      this.currentFrame = (this.currentFrame + 1) % this.frames.length;
      this.lastTime = time;
    }
    return this.frames[this.currentFrame];
  }
}