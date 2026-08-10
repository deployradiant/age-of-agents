'use strict';

const canvas = document.getElementById('world');
const ctx = canvas.getContext('2d', { alpha: false });
const woodValue = document.querySelector('#wood strong');
const foodValue = document.querySelector('#food strong');
const stoneValue = document.querySelector('#stone strong');
const connection = document.getElementById('connection');
const selection = document.getElementById('selection');
const buildingPopover = document.getElementById('building-popover');
const buildingPopoverTitle = document.getElementById('building-popover-title');
const buildingProducts = document.getElementById('building-products');
const buildingJob = document.getElementById('building-job');
const buildingJobLabel = document.getElementById('building-job-label');
const buildingProgress = document.getElementById('building-progress');
const buildingResearches = document.getElementById('building-researches');
const buildButton = document.getElementById('build');
const trainButton = document.getElementById('train-villager');
const cancelButton = document.getElementById('cancel');
const resetWorldButton = document.getElementById('reset-world');
const placementHint = document.getElementById('placement');
const toast = document.getElementById('toast');
const laptopModeQuery = window.matchMedia('(min-width: 900px) and (hover: hover) and (pointer: fine)');

const TILE = 80;
const MIN_ZOOM = 0.35;
const MAX_ZOOM = 2.5;
const DRAG_THRESHOLD = 8;
const sprites = {};
const hits = [];
const pointers = new Map();
let groundLayer = null;
let groundLayerKey = '';
let cssWidth = 1;
let cssHeight = 1;
let dpr = 1;
let selectedUnitId = null;
let selectedBuildingId = null;
let buildMode = false;
let cameraReady = false;
let receivedSnapshot = false;
let toastTimer = 0;
let gestureUsed = false;
let pinch = null;
let ws = null;
let reconnectTimer = 0;
let reconnectAttempt = 0;
let requestSequence = 0;
let lastSnapshotSequence = 0;
let minimumSnapshotSequence = 0;
let world = { width: 2400, height: 1600, cellSize: TILE, wood: 0, food: 0, stone: 0, terrain: [], units: [], resources: [], buildings: [] };
let tick = 0;
const camera = { x: 0, y: 0, zoom: 0.8 };
const pressedPanKeys = new Set();
let previousFrameTime = null;
let hoveredHit = null;
let mousePosition = null;

const assetPaths = {
  idle: '/assets/game/agent_idle.png',
  walk1: '/assets/game/agent_walk_01.png',
  walk2: '/assets/game/agent_walk_02.png',
  walkDiagToward1: '/assets/game/agent_walk_diag_toward_01.png',
  walkDiagToward2: '/assets/game/agent_walk_diag_toward_02.png',
  walkDown1: '/assets/game/agent_walk_down_01.png',
  walkDown2: '/assets/game/agent_walk_down_02.png',
  walkUp1: '/assets/game/agent_walk_up_01.png',
  walkUp2: '/assets/game/agent_walk_up_02.png',
  gather: '/assets/game/agent_gather.png',
  build: '/assets/game/agent_build.png',
  tree: '/assets/game/resource_tree.png',
  treeDepleted: '/assets/game/resource_tree_depleted.png',
  berries: '/assets/game/resource_berries.png',
  stone: '/assets/game/resource_stone.png',
  building: '/assets/game/building_town_center.png',
  meadow: '/assets/game/terrain_meadow.png',
  forest: '/assets/game/terrain_forest.png',
  prairie: '/assets/game/terrain_grassland.png',
  highland: '/assets/game/terrain_highland.png',
  wetland: '/assets/game/terrain_deep_forest.png',
  scrubland: '/assets/game/terrain_scrub.png',
  heath: '/assets/game/terrain_rock.png',
  clayland: '/assets/game/terrain_dirt.png'
};

const biomeTextures = {
  meadow: 'meadow', forest: 'forest', prairie: 'prairie', highland: 'highland',
  wetland: 'wetland', scrubland: 'scrubland', heath: 'heath', clayland: 'clayland'
};

function loadSprite(name, url) {
  const asset = { image: null };
  sprites[name] = asset;
  const image = new Image();
  image.onload = () => {
    asset.image = image;
    if (Object.prototype.hasOwnProperty.call(biomeTextures, name)) groundLayer = null;
  };
  image.onerror = () => { console.warn('Sprite failed to load', url); };
  image.src = url;
}

Object.entries(assetPaths).forEach(([name, url]) => loadSprite(name, url));

function rawProject(x, y) {
  return { x: (x - y) / 2, y: (x + y) / 4 };
}

function project(x, y) {
  const raw = rawProject(x, y);
  return {
    x: cssWidth / 2 + (raw.x - camera.x) * camera.zoom,
    y: cssHeight * 0.42 + (raw.y - camera.y) * camera.zoom
  };
}

function buildingPopoverAnchor(building, projectWorld, zoom) {
  const ground = projectWorld(Number(building.x) || 0, Number(building.y) || 0);
  return { x: ground.x, y: ground.y - 132 * Math.max(0, Number(zoom) || 0) };
}

function screenToRaw(x, y) {
  return {
    x: camera.x + (x - cssWidth / 2) / camera.zoom,
    y: camera.y + (y - cssHeight * 0.42) / camera.zoom
  };
}

function screenToWorld(x, y) {
  const raw = screenToRaw(x, y);
  return { x: raw.y * 2 + raw.x, y: raw.y * 2 - raw.x };
}

function centerCamera() {
  const focus = world.units.length
    ? world.units.reduce((sum, unit) => ({ x: sum.x + unit.x / world.units.length, y: sum.y + unit.y / world.units.length }), { x: 0, y: 0 })
    : { x: world.width / 2, y: world.height / 2 };
  const center = rawProject(focus.x, focus.y);
  camera.x = center.x;
  camera.y = center.y;
  camera.zoom = Math.max(MIN_ZOOM, Math.min(1, cssWidth < 700 ? 0.55 : 0.8));
  cameraReady = true;
}

function clampCamera() {
  const corners = [rawProject(0, 0), rawProject(world.width, 0), rawProject(0, world.height), rawProject(world.width, world.height)];
  const marginX = cssWidth * 0.35 / camera.zoom;
  const marginY = cssHeight * 0.35 / camera.zoom;
  camera.x = Math.max(Math.min(...corners.map(point => point.x)) - marginX, Math.min(Math.max(...corners.map(point => point.x)) + marginX, camera.x));
  camera.y = Math.max(Math.min(...corners.map(point => point.y)) - marginY, Math.min(Math.max(...corners.map(point => point.y)) + marginY, camera.y));
}

function panCameraWithKeyboard(elapsedSeconds) {
  if (!laptopModeQuery.matches || pressedPanKeys.size === 0) return;
  let x = 0;
  let y = 0;
  if (pressedPanKeys.has('KeyA') || pressedPanKeys.has('ArrowLeft')) x -= 1;
  if (pressedPanKeys.has('KeyD') || pressedPanKeys.has('ArrowRight')) x += 1;
  if (pressedPanKeys.has('KeyW') || pressedPanKeys.has('ArrowUp')) y -= 1;
  if (pressedPanKeys.has('KeyS') || pressedPanKeys.has('ArrowDown')) y += 1;
  if (x === 0 && y === 0) return;
  const length = Math.hypot(x, y);
  const distance = 440 * Math.max(0, elapsedSeconds) / camera.zoom;
  camera.x += x / length * distance;
  camera.y += y / length * distance;
  clampCamera();
}

function resize() {
  const rect = canvas.getBoundingClientRect();
  cssWidth = Math.max(1, rect.width);
  cssHeight = Math.max(1, rect.height);
  dpr = Math.min(window.devicePixelRatio || 1, 2.5);
  const width = Math.round(cssWidth * dpr);
  const height = Math.round(cssHeight * dpr);
  if (canvas.width !== width || canvas.height !== height) {
    canvas.width = width;
    canvas.height = height;
  }
  if (!cameraReady) centerCamera();
  else clampCamera();
  positionBuildingPopover();
}

function diamondPath(x, y, size) {
  const a = project(x, y);
  const b = project(x + size, y);
  const c = project(x + size, y + size);
  const d = project(x, y + size);
  ctx.beginPath();
  ctx.moveTo(a.x, a.y);
  ctx.lineTo(b.x, b.y);
  ctx.lineTo(c.x, c.y);
  ctx.lineTo(d.x, d.y);
  ctx.closePath();
}

function buildGroundLayer(columns, rows) {
  const textures = Object.values(biomeTextures).map(name => sprites[name]);
  if (textures.some(asset => !asset?.image) || world.terrain.length !== columns * rows) return null;
  const key = world.terrain.map(cell => `${cell.biome}:${cell.visibility}`).join('|');
  if (groundLayer && groundLayerKey === key) return groundLayer;
  const sourceSize = 96;
  const layer = document.createElement('canvas');
  layer.width = columns * sourceSize;
  layer.height = rows * sourceSize;
  const layerContext = layer.getContext('2d', { alpha: false });
  for (const cell of world.terrain) {
    const left = cell.column * sourceSize;
    const top = cell.row * sourceSize;
    if (cell.visibility === 'unseen' || !cell.biome) {
      layerContext.fillStyle = '#020504';
      layerContext.fillRect(left, top, sourceSize, sourceSize);
      continue;
    }
    layerContext.drawImage(sprites[biomeTextures[cell.biome]].image, left, top, sourceSize, sourceSize);
    if (cell.visibility === 'explored') {
      layerContext.fillStyle = '#08100bc4';
      layerContext.fillRect(left, top, sourceSize, sourceSize);
    }
  }
  groundLayer = layer;
  groundLayerKey = key;
  return layer;
}

function drawGround() {
  const columns = Math.ceil(world.width / world.cellSize);
  const rows = Math.ceil(world.height / world.cellSize);
  const layer = buildGroundLayer(columns, rows);
  if (layer) {
    // Compose square tiles edge-to-edge first, then project the complete layer
    // once. This keeps the isometric diamonds interlocked without per-tile
    // antialiasing seams or overlap-based edge bleed.
    const a = project(0, 0);
    const b = project(columns * TILE, 0);
    const d = project(0, rows * TILE);
    ctx.save();
    ctx.transform(
      (b.x - a.x) / layer.width,
      (b.y - a.y) / layer.width,
      (d.x - a.x) / layer.height,
      (d.y - a.y) / layer.height,
      a.x,
      a.y
    );
    ctx.drawImage(layer, 0, 0);
    ctx.restore();
    return;
  }
  for (let row = 0; row < rows; row += 1) {
    for (let column = 0; column < columns; column += 1) {
      const x = column * world.cellSize;
      const y = row * world.cellSize;
      diamondPath(x, y, world.cellSize);
      ctx.fillStyle = '#020504';
      ctx.fill();
    }
  }
}

function drawConstructionSites() {
  for (const unit of world.units) {
    if (unit.action?.type !== 'build') continue;
    const point = project(unit.action.x, unit.action.y);
    const progress = Math.max(0, Math.min(1, Number(unit.action.work_seconds) / 4));
    ctx.save();
    ctx.translate(point.x, point.y);
    ctx.globalAlpha = 0.35 + progress * 0.45;
    ctx.fillStyle = '#c8a35d';
    ctx.beginPath();
    ctx.moveTo(0, -28 * camera.zoom);
    ctx.lineTo(52 * camera.zoom, 0);
    ctx.lineTo(0, 28 * camera.zoom);
    ctx.lineTo(-52 * camera.zoom, 0);
    ctx.closePath();
    ctx.fill();
    ctx.globalAlpha = 1;
    ctx.strokeStyle = '#f0cc75';
    ctx.lineWidth = 2;
    ctx.stroke();
    ctx.fillStyle = '#171b12';
    ctx.fillRect(-42 * camera.zoom, 34 * camera.zoom, 84 * camera.zoom, 5 * camera.zoom);
    ctx.fillStyle = '#e6bd58';
    ctx.fillRect(-42 * camera.zoom, 34 * camera.zoom, 84 * camera.zoom * progress, 5 * camera.zoom);
    ctx.restore();
  }
}

function walkPresentation(position, target) {
  if (!target) return { direction: 'side', flip: false };
  const dx = Number(target.x) - Number(position.x);
  const dy = Number(target.y) - Number(position.y);
  const screenX = dx - dy;
  const screenY = (dx + dy) / 2;
  const vertical = Math.abs(screenX) <= Math.abs(screenY) * 0.55;
  const direction = vertical
    ? (screenY >= 0 ? 'down' : 'up')
    : screenY > 0
      ? 'diag_toward'
      : Math.abs(screenY) > Math.abs(screenX) * 0.2
        ? 'up'
        : 'side';
  return { direction, flip: !vertical && screenX > 0 };
}

function spriteForUnit(unit, now) {
  if (unit.state === 'gathering') return sprites.gather;
  if (unit.state === 'building') {
    const frames = [sprites.build, sprites.idle];
    return frames[Math.floor(now / 260) % frames.length];
  }
  if (unit.state === 'moving') {
    const frames = unit.walkDirection === 'down'
      ? [sprites.walkDown1, sprites.walkDown2]
      : unit.walkDirection === 'up'
        ? [sprites.walkUp1, sprites.walkUp2]
        : unit.walkDirection === 'diag_toward'
          ? [sprites.walkDiagToward1, sprites.walkDiagToward2]
          : [sprites.walk1, sprites.walk2];
    return frames[Math.floor(now / 190) % frames.length];
  }
  return sprites.idle;
}

function fallbackEntity(entity, type, point, scale) {
  ctx.save();
  ctx.translate(point.x, point.y);
  if (type === 'unit') {
    ctx.fillStyle = '#e6c273';
    ctx.beginPath(); ctx.arc(0, -22 * scale, 12 * scale, 0, Math.PI * 2); ctx.fill();
    ctx.fillStyle = '#31586b'; ctx.fillRect(-10 * scale, -14 * scale, 20 * scale, 28 * scale);
  } else if (type === 'resource') {
    if (entity.kind === 'food') {
      ctx.fillStyle = '#6c8240'; ctx.beginPath(); ctx.arc(0, -18 * scale, 22 * scale, 0, Math.PI * 2); ctx.fill();
      ctx.fillStyle = '#a94f58'; ctx.beginPath(); ctx.arc(-6 * scale, -20 * scale, 4 * scale, 0, Math.PI * 2); ctx.fill();
    } else if (entity.kind === 'stone') {
      ctx.fillStyle = '#69706a'; ctx.beginPath(); ctx.moveTo(-24 * scale, 0); ctx.lineTo(-12 * scale, -38 * scale); ctx.lineTo(10 * scale, -46 * scale); ctx.lineTo(26 * scale, 0); ctx.fill();
    } else {
      ctx.fillStyle = '#6a4328'; ctx.fillRect(-5 * scale, -30 * scale, 10 * scale, 32 * scale);
      ctx.fillStyle = '#275a30'; ctx.beginPath(); ctx.arc(0, -42 * scale, 24 * scale, 0, Math.PI * 2); ctx.fill();
    }
  } else {
    ctx.fillStyle = '#765639'; ctx.fillRect(-32 * scale, -38 * scale, 64 * scale, 38 * scale);
    ctx.fillStyle = '#a56742'; ctx.beginPath(); ctx.moveTo(-38 * scale, -38 * scale); ctx.lineTo(0, -68 * scale); ctx.lineTo(38 * scale, -38 * scale); ctx.fill();
  }
  ctx.restore();
}

function visibilityAtWorldPosition(x, y) {
  const column = Math.floor(x / world.cellSize);
  const row = Math.floor(y / world.cellSize);
  return world.terrain.find(cell => cell.column === column && cell.row === row)?.visibility || 'unseen';
}

function drawEntity(item, now) {
  const point = project(Number(item.data.x) || 0, Number(item.data.y) || 0);
  const resourceScales = item.data.kind === 'wood' ? [105, 112] : item.data.kind === 'food' ? [92, 78] : [100, 92];
  const scales = item.type === 'building' ? [150, 145] : item.type === 'resource' ? resourceScales : [88, 94];
  const width = scales[0] * camera.zoom;
  const height = scales[1] * camera.zoom;
  if (point.x < -width || point.x > cssWidth + width || point.y < -height || point.y > cssHeight + height) return;

  const selected = (item.type === 'unit' && item.data.id === selectedUnitId)
    || (item.type === 'building' && item.data.id === selectedBuildingId);
  if (selected) {
    ctx.beginPath();
    const radius = item.type === 'building' ? 40 : 24;
    ctx.ellipse(point.x, point.y, radius * camera.zoom, radius * 0.46 * camera.zoom, 0, 0, Math.PI * 2);
    ctx.fillStyle = '#f6c74a44'; ctx.fill();
    ctx.strokeStyle = '#ffdb67'; ctx.lineWidth = 2; ctx.stroke();
  }

  if (hoveredHit?.type === item.type && hoveredHit.data.id === item.data.id) {
    ctx.beginPath();
    ctx.ellipse(point.x, point.y, 29 * camera.zoom, 14 * camera.zoom, 0, 0, Math.PI * 2);
    ctx.fillStyle = item.type === 'resource' ? '#8ed06a33' : '#ffe08a22';
    ctx.fill();
    ctx.strokeStyle = item.type === 'resource' ? '#9bdc76' : '#ffe08a';
    ctx.lineWidth = 2;
    ctx.stroke();
  }

  const depletedResource = item.type === 'resource' && Number(item.data.amount) <= 0;
  let asset;
  if (item.type === 'unit') asset = spriteForUnit(item.data, now);
  else if (item.type === 'building') asset = sprites.building;
  else if (item.data.kind === 'wood') asset = depletedResource && sprites.treeDepleted?.image ? sprites.treeDepleted : sprites.tree;
  else if (item.data.kind === 'food') asset = sprites.berries;
  else asset = sprites.stone;
  const remembered = item.type !== 'unit'
    && visibilityAtWorldPosition(Number(item.data.x) || 0, Number(item.data.y) || 0) === 'explored';
  ctx.save();
  if (remembered) ctx.globalAlpha = 0.58;
  if (depletedResource && item.data.kind !== 'wood') {
    ctx.globalAlpha *= 0.48;
    ctx.filter = 'grayscale(1) brightness(.65)';
  }
  if (asset && asset.image) {
    const ratio = asset.image.width / asset.image.height;
    const drawWidth = width * Math.min(1.35, Math.max(0.72, ratio));
    if (item.type === 'unit' && item.data.walkFlip) {
      ctx.save();
      ctx.translate(point.x, 0);
      ctx.scale(-1, 1);
      ctx.drawImage(asset.image, -drawWidth / 2, point.y - height * 0.88, drawWidth, height);
      ctx.restore();
    } else {
      ctx.drawImage(asset.image, point.x - drawWidth / 2, point.y - height * 0.88, drawWidth, height);
    }
  } else {
    fallbackEntity(item.data, item.type, point, camera.zoom);
  }
  ctx.restore();

  if (item.type === 'building' && item.data.production) {
    const progress = Math.max(0, Math.min(1, Number(item.data.production.elapsed_seconds) / 6));
    const barWidth = 78 * camera.zoom;
    ctx.fillStyle = '#111b';
    ctx.fillRect(point.x - barWidth / 2, point.y + 15 * camera.zoom, barWidth, 6 * camera.zoom);
    ctx.fillStyle = '#e8bd51';
    ctx.fillRect(point.x - barWidth / 2, point.y + 15 * camera.zoom, barWidth * progress, 6 * camera.zoom);
  }

  if (depletedResource) return;
  hits.push({
    type: item.type,
    data: item.data,
    left: point.x - width * 0.38,
    top: point.y - height * 0.82,
    right: point.x + width * 0.38,
    bottom: point.y + height * 0.08
  });
}

function render(now) {
  requestAnimationFrame(render);
  if (previousFrameTime !== null) panCameraWithKeyboard((now - previousFrameTime) / 1000);
  previousFrameTime = now;
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  ctx.fillStyle = '#17231b';
  ctx.fillRect(0, 0, cssWidth, cssHeight);
  drawGround();
  drawConstructionSites();
  hits.length = 0;
  const entities = [
    ...world.resources.map(data => ({ type: 'resource', data })),
    ...world.buildings.map(data => ({ type: 'building', data })),
    ...world.units.map(data => ({ type: 'unit', data }))
  ];
  entities.sort((a, b) => {
    const depth = item => (Number(item.data.x) || 0) + (Number(item.data.y) || 0);
    return depth(a) - depth(b);
  });
  entities.forEach(item => drawEntity(item, now));
  positionBuildingPopover();
  if (mousePosition) updateCursor(mousePosition.x, mousePosition.y);
}

function showToast(message) {
  window.clearTimeout(toastTimer);
  toast.textContent = message;
  toast.classList.add('visible');
  toastTimer = window.setTimeout(() => toast.classList.remove('visible'), 2200);
}

function selectedUnit() {
  return world.units.find(unit => unit.id === selectedUnitId) || null;
}

function resolveSelectedBuilding(buildings, selectedId, visibilityAt) {
  const building = buildings.find(item => item.id === selectedId) || null;
  if (!building || visibilityAt(Number(building.x) || 0, Number(building.y) || 0) !== 'visible') return null;
  return building;
}

function selectedBuilding() {
  return resolveSelectedBuilding(world.buildings, selectedBuildingId, visibilityAtWorldPosition);
}

function capabilityId(value) {
  if (typeof value === 'string') return value;
  return value?.id || value?.kind || value?.product || value?.technology || '';
}

function buildingCapabilityView(building) {
  const list = value => Array.isArray(value) ? value.map(capabilityId).filter(Boolean) : [];
  const products = list(building?.produces);
  const researches = list(building?.researches);
  const researched = list(building?.researched_technologies);
  const rawJob = building?.job || building?.active_job || building?.production || null;
  if (!rawJob) return { products, researches, researched, job: null };
  const product = capabilityId(rawJob.product);
  const technology = capabilityId(rawJob.technology);
  const type = rawJob.type || rawJob.kind || (technology ? 'research' : 'produce');
  const subject = product || technology || capabilityId(rawJob.item ?? rawJob.target) || 'task';
  const duration = Math.max(0, Number(rawJob.required_seconds ?? rawJob.duration_seconds ?? rawJob.total_seconds) || (subject === 'villager' ? 6 : 0));
  const elapsed = Math.max(0, Number(rawJob.elapsed_seconds ?? rawJob.elapsed ?? rawJob.progress_seconds) || 0);
  const explicit = rawJob.progress == null ? NaN : Number(rawJob.progress);
  const fraction = Number.isFinite(explicit) ? (explicit > 1 ? explicit / 100 : explicit) : (duration > 0 ? elapsed / duration : 0);
  return { products, researches, researched, job: { type, subject, progress: Math.max(0, Math.min(1, fraction)) } };
}

function capabilityLabel(value) {
  return String(value || 'task').replaceAll('_', ' ').replace(/\b\w/g, letter => letter.toUpperCase());
}

function positionBuildingPopover() {
  const building = selectedBuilding();
  if (!building) {
    buildingPopover.hidden = true;
    return;
  }
  if (buildingPopover.hidden) return;
  const anchor = buildingPopoverAnchor(building, project, camera.zoom);
  buildingPopover.style.left = `${anchor.x}px`;
  buildingPopover.style.top = `${anchor.y}px`;
}

function updateBuildingPopover(building) {
  buildingPopover.hidden = !building;
  if (!building) return;
  const view = buildingCapabilityView(building);
  const job = view.job;
  buildingPopoverTitle.textContent = building.kind === 'town_center' ? 'Town Center' : capabilityLabel(building.kind);
  buildingProducts.textContent = `Produces: ${view.products.map(capabilityLabel).join(', ') || 'Nothing'}`;
  buildingJob.hidden = !job;
  if (job) {
    const verb = String(job.type).includes('research') ? 'Researching' : 'Producing';
    const percent = Math.floor(job.progress * 100);
    buildingJobLabel.textContent = `${verb} ${capabilityLabel(job.subject)} · ${percent}%`;
    buildingProgress.value = job.progress;
    buildingProgress.setAttribute('aria-label', `${verb} ${capabilityLabel(job.subject)} progress`);
  }
  const researchText = [];
  if (view.researches.length) researchText.push(`Researches: ${view.researches.map(capabilityLabel).join(', ')}`);
  if (view.researched.length) researchText.push(`Researched: ${view.researched.map(capabilityLabel).join(', ')}`);
  buildingResearches.hidden = researchText.length === 0;
  buildingResearches.textContent = researchText.join(' · ');
  trainButton.hidden = !view.products.includes('villager');
  trainButton.disabled = Boolean(job) || world.food < 50;
  trainButton.title = job ? 'Town center is already busy' : world.food < 50 ? 'Train villager (50 food required)' : 'Train villager (50 food)';
  positionBuildingPopover();
}

function updateHud() {
  woodValue.textContent = String(Math.floor(world.wood ?? 0));
  foodValue.textContent = String(Math.floor(world.food ?? 0));
  stoneValue.textContent = String(Math.floor(world.stone ?? 0));
  const unit = selectedUnit();
  const building = selectedBuilding();
  if (!unit) selectedUnitId = null;
  if (!building) selectedBuildingId = null;
  buildButton.disabled = !unit;
  updateBuildingPopover(building);

  if (unit) {
    selection.className = '';
    selection.replaceChildren();
    const name = document.createElement('strong');
    name.textContent = unit.name || `Villager ${unit.id}`;
    const status = document.createElement('span');
    status.textContent = `${unit.state || 'idle'} · tick ${tick}`;
    selection.append(name, status);
    return;
  }

  if (building) {
    if (buildMode) cancelBuild();
    selection.className = '';
    selection.replaceChildren();
    const name = document.createElement('strong');
    name.textContent = building.kind === 'town_center' ? 'Town Center' : building.id;
    const status = document.createElement('span');
    status.textContent = 'Capabilities shown above building';
    selection.append(name, status);
    return;
  }

  selection.className = 'empty';
  selection.textContent = 'Tap a villager or town center';
  if (buildMode) cancelBuild();
}

function setBuildMode(enabled) {
  buildMode = enabled && Boolean(selectedUnit());
  buildButton.classList.toggle('active', buildMode);
  cancelButton.classList.toggle('visible', buildMode);
  placementHint.classList.toggle('visible', buildMode);
  updateCursor();
}

function cancelBuild() {
  setBuildMode(false);
}

function sendCommand(command) {
  if (!ws || ws.readyState !== WebSocket.OPEN) {
    showToast('Not connected');
    return false;
  }
  const requestId = `${Date.now().toString(36)}-${(++requestSequence).toString(36)}`;
  ws.send(JSON.stringify({ type: 'command', request_id: requestId, command }));
  return true;
}

function hitAt(x, y) {
  for (let i = hits.length - 1; i >= 0; i -= 1) {
    const hit = hits[i];
    if (x >= hit.left && x <= hit.right && y >= hit.top && y <= hit.bottom) return hit;
  }
  return null;
}

function updateCursor(x, y) {
  if (Number.isFinite(x) && Number.isFinite(y)) {
    mousePosition = { x, y };
    hoveredHit = hitAt(x, y);
  }
  if (buildMode) canvas.style.cursor = 'crosshair';
  else if (laptopModeQuery.matches && (['unit', 'resource', 'building'].includes(hoveredHit?.type) || selectedUnit())) canvas.style.cursor = 'pointer';
  else canvas.style.cursor = laptopModeQuery.matches ? 'grab' : '';
}

function handleTap(x, y) {
  const hit = hitAt(x, y);
  if (buildMode) {
    const ground = screenToWorld(x, y);
    if (ground.x < 0 || ground.y < 0 || ground.x > world.width || ground.y > world.height) {
      showToast('Choose ground inside the map');
      return;
    }
    const unit = selectedUnit();
    if (unit && sendCommand({ type: 'build', unit_id: unit.id, x: ground.x, y: ground.y })) cancelBuild();
    return;
  }
  if (!hit) {
    if (selectedBuildingId) {
      selectedBuildingId = null;
      updateHud();
      return;
    }
    const unit = selectedUnit();
    if (!unit) return;
    const ground = screenToWorld(x, y);
    if (ground.x < 0 || ground.y < 0 || ground.x > world.width || ground.y > world.height) return;
    sendCommand({ type: 'move', unit_id: unit.id, x: ground.x, y: ground.y });
    return;
  }
  if (hit.type === 'unit') {
    selectedUnitId = hit.data.id;
    selectedBuildingId = null;
    updateHud();
  } else if (hit.type === 'resource') {
    if (Number(hit.data.amount) <= 0) return;
    const unit = selectedUnit();
    if (unit) sendCommand({ type: 'gather', unit_id: unit.id, resource_id: hit.data.id });
    else showToast('Select an agent first');
  } else if (hit.type === 'building') {
    selectedBuildingId = hit.data.id;
    selectedUnitId = null;
    cancelBuild();
    updateHud();
  }
}

function pointerDistance(a, b) {
  return Math.hypot(a.x - b.x, a.y - b.y);
}

function pointerMidpoint(a, b) {
  return { x: (a.x + b.x) / 2, y: (a.y + b.y) / 2 };
}

function beginPinch() {
  const pair = [...pointers.values()].slice(0, 2);
  const middle = pointerMidpoint(pair[0], pair[1]);
  pinch = {
    distance: Math.max(1, pointerDistance(pair[0], pair[1])),
    zoom: camera.zoom,
    anchor: screenToRaw(middle.x, middle.y)
  };
  gestureUsed = true;
}

function onPointerDown(event) {
  if (event.pointerType === 'mouse' && event.button !== 0) return;
  event.preventDefault();
  canvas.setPointerCapture(event.pointerId);
  pointers.set(event.pointerId, { x: event.clientX, y: event.clientY, startX: event.clientX, startY: event.clientY, lastX: event.clientX, lastY: event.clientY, dragging: false });
  if (pointers.size === 1) gestureUsed = false;
  if (pointers.size === 2) beginPinch();
}

function onPointerMove(event) {
  const pointer = pointers.get(event.pointerId);
  if (!pointer) {
    if (event.pointerType === 'mouse') updateCursor(event.clientX, event.clientY);
    return;
  }
  event.preventDefault();
  pointer.x = event.clientX;
  pointer.y = event.clientY;
  if (pointers.size >= 2) {
    if (!pinch) beginPinch();
    const pair = [...pointers.values()].slice(0, 2);
    const middle = pointerMidpoint(pair[0], pair[1]);
    const nextZoom = Math.max(MIN_ZOOM, Math.min(MAX_ZOOM, pinch.zoom * pointerDistance(pair[0], pair[1]) / pinch.distance));
    camera.zoom = nextZoom;
    camera.x = pinch.anchor.x - (middle.x - cssWidth / 2) / nextZoom;
    camera.y = pinch.anchor.y - (middle.y - cssHeight * 0.42) / nextZoom;
    clampCamera();
  } else {
    const moved = Math.hypot(pointer.x - pointer.startX, pointer.y - pointer.startY);
    if (moved > DRAG_THRESHOLD) pointer.dragging = true;
    if (pointer.dragging) {
      camera.x -= (pointer.x - pointer.lastX) / camera.zoom;
      camera.y -= (pointer.y - pointer.lastY) / camera.zoom;
      clampCamera();
      gestureUsed = true;
    }
  }
  pointer.lastX = pointer.x;
  pointer.lastY = pointer.y;
}

function onPointerLeave(event) {
  if (event.pointerType !== 'mouse') return;
  mousePosition = null;
  hoveredHit = null;
  updateCursor();
}

function finishPointer(event, cancelled) {
  const pointer = pointers.get(event.pointerId);
  if (!pointer) return;
  const wasSingle = pointers.size === 1;
  pointers.delete(event.pointerId);
  if (!cancelled && wasSingle && !pointer.dragging && !gestureUsed && Math.hypot(pointer.x - pointer.startX, pointer.y - pointer.startY) <= DRAG_THRESHOLD) {
    handleTap(pointer.x, pointer.y);
  }
  if (pointers.size < 2) {
    pinch = null;
    const remaining = pointers.values().next().value;
    if (remaining) remaining.dragging = true;
  }
}

function zoomAt(x, y, factor) {
  const anchor = screenToRaw(x, y);
  const next = Math.max(MIN_ZOOM, Math.min(MAX_ZOOM, camera.zoom * factor));
  camera.zoom = next;
  camera.x = anchor.x - (x - cssWidth / 2) / next;
  camera.y = anchor.y - (y - cssHeight * 0.42) / next;
  clampCamera();
}

function shortcutTargetIsInteractive(target) {
  return target instanceof Element && Boolean(target.closest('input, textarea, select, button, summary, a[href], [contenteditable], [role="button"], [role="textbox"]'));
}

function clearPanInput() {
  pressedPanKeys.clear();
}

function onKeyDown(event) {
  if (!laptopModeQuery.matches || shortcutTargetIsInteractive(event.target) || event.ctrlKey || event.metaKey || event.altKey) {
    clearPanInput();
    return;
  }
  if (['KeyW', 'KeyA', 'KeyS', 'KeyD', 'ArrowUp', 'ArrowLeft', 'ArrowDown', 'ArrowRight'].includes(event.code)) {
    pressedPanKeys.add(event.code);
    event.preventDefault();
  } else if (event.code === 'KeyB' && !event.repeat && selectedUnit()) {
    setBuildMode(true);
    event.preventDefault();
  } else if (event.key === 'Escape' && buildMode) {
    cancelBuild();
    event.preventDefault();
  }
}

function onKeyUp(event) {
  pressedPanKeys.delete(event.code);
}

function updateLaptopMode() {
  clearPanInput();
  placementHint.textContent = laptopModeQuery.matches ? 'Click ground to build · Escape to cancel' : 'Tap ground to build · Escape to cancel';
  updateCursor();
}

function connect() {
  window.clearTimeout(reconnectTimer);
  if (ws && (ws.readyState === WebSocket.OPEN || ws.readyState === WebSocket.CONNECTING)) return;
  const protocol = location.protocol === 'https:' ? 'wss:' : 'ws:';
  const socket = new WebSocket(`${protocol}//${location.host}/ws`);
  ws = socket;
  connection.className = '';
  connection.textContent = 'Connecting';
  socket.onopen = () => {
    if (ws !== socket) return;
    reconnectAttempt = 0;
    lastSnapshotSequence = 0;
    minimumSnapshotSequence = 0;
    connection.className = 'online';
    connection.textContent = 'Connected';
  };
  socket.onmessage = event => {
    try {
      const message = JSON.parse(event.data);
      if (message.type === 'snapshot' && message.world) {
        const sequence = Number(message.sequence) || 0;
        if (sequence < minimumSnapshotSequence || sequence <= lastSnapshotSequence) return;
        lastSnapshotSequence = sequence;
        const next = message.world;
        const withPosition = item => ({
          ...item,
          x: Number(item.position?.x) || 0,
          y: Number(item.position?.y) || 0
        });
        const unitForDisplay = unit => {
          const action = unit.action?.type || 'idle';
          let state = 'idle';
          let target = null;
          if (action === 'move') {
            state = 'moving';
            target = unit.action;
          } else if (action === 'gather') {
            const resource = next.resources?.find(item => item.id === unit.action.resource_id);
            const distance = resource ? Math.hypot(unit.position.x - resource.position.x, unit.position.y - resource.position.y) : Infinity;
            state = distance > 0.5 ? 'moving' : 'gathering';
            target = resource?.position || null;
          } else if (action === 'build') {
            const distance = Math.hypot(unit.position.x - unit.action.x, unit.position.y - unit.action.y);
            state = distance > 0.5 ? 'moving' : 'building';
            target = unit.action;
          }
          const walk = walkPresentation(unit.position, target);
          return {
            ...withPosition(unit),
            name: unit.name || unit.id.replace(/^villager-/, 'Villager '),
            state,
            walkDirection: walk.direction,
            walkFlip: walk.flip
          };
        };
        world = {
          width: Number(next.width) || 2400,
          height: Number(next.height) || 1600,
          cellSize: Number(next.cell_size) || TILE,
          wood: Number(next.stockpile?.wood) || 0,
          food: Number(next.stockpile?.food) || 0,
          stone: Number(next.stockpile?.stone) || 0,
          terrain: Array.isArray(next.terrain) ? next.terrain : [],
          units: Array.isArray(next.units) ? next.units.map(unitForDisplay) : [],
          resources: Array.isArray(next.resources) ? next.resources.map(withPosition) : [],
          buildings: Array.isArray(next.buildings) ? next.buildings.map(withPosition) : []
        };
        tick = Number(next.tick) || 0;
        if (!receivedSnapshot) {
          centerCamera();
          receivedSnapshot = true;
        }
        updateHud();
      } else if (message.type === 'command_result') {
        const failed = message.ok === false || message.success === false || message.error;
        if (!failed) minimumSnapshotSequence = Math.max(minimumSnapshotSequence, Number(message.applied_sequence) || 0);
        showToast(failed ? (message.error || message.message || 'Command failed') : (message.message || 'Command accepted'));
      }
    } catch (error) {
      console.warn('Ignored invalid server message', error);
    }
  };
  socket.onerror = () => socket.close();
  socket.onclose = () => {
    if (ws !== socket) return;
    ws = null;
    connection.className = '';
    connection.textContent = 'Reconnecting';
    const delay = Math.min(15000, 700 * (2 ** reconnectAttempt)) * (0.8 + Math.random() * 0.4);
    reconnectAttempt = Math.min(reconnectAttempt + 1, 6);
    reconnectTimer = window.setTimeout(connect, delay);
  };
}

async function resetWorld() {
  if (!window.confirm('Reset the entire world? All progress will be lost.')) return;
  resetWorldButton.disabled = true;
  try {
    const response = await fetch('/reset', { method: 'POST' });
    if (!response.ok) throw new Error(await response.text() || `Reset failed (${response.status})`);
    selectedUnitId = null;
    selectedBuildingId = null;
    cancelBuild();
    updateHud();
    showToast('World reset');
  } catch (error) {
    showToast(error instanceof Error ? error.message : 'World reset failed');
  } finally {
    resetWorldButton.disabled = false;
  }
}

buildButton.addEventListener('click', () => setBuildMode(!buildMode));
trainButton.addEventListener('click', () => {
  const building = selectedBuilding();
  if (building) sendCommand({ type: 'produce', building_id: building.id, product: 'villager' });
});
cancelButton.addEventListener('click', cancelBuild);
resetWorldButton.addEventListener('click', resetWorld);
canvas.addEventListener('pointerdown', onPointerDown);
canvas.addEventListener('pointermove', onPointerMove);
canvas.addEventListener('pointerleave', onPointerLeave);
canvas.addEventListener('pointerup', event => finishPointer(event, false));
canvas.addEventListener('pointercancel', event => finishPointer(event, true));
canvas.addEventListener('contextmenu', event => event.preventDefault());
canvas.addEventListener('wheel', event => {
  event.preventDefault();
  zoomAt(event.clientX, event.clientY, Math.exp(-event.deltaY * 0.0012));
}, { passive: false });
window.addEventListener('keydown', onKeyDown);
window.addEventListener('keyup', onKeyUp);
window.addEventListener('blur', clearPanInput);
document.addEventListener('focusin', clearPanInput);
laptopModeQuery.addEventListener('change', updateLaptopMode);
window.addEventListener('online', connect);
document.addEventListener('visibilitychange', () => {
  clearPanInput();
  if (!document.hidden && (!ws || ws.readyState > WebSocket.OPEN)) connect();
});
new ResizeObserver(resize).observe(canvas);
resize();
updateLaptopMode();
updateHud();
connect();
requestAnimationFrame(render);
