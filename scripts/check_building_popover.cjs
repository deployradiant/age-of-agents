const assert = require('assert');
const fs = require('fs');
const path = require('path');

const appPath = path.join(__dirname, '..', 'frontend', 'app.js');
const source = fs.readFileSync(appPath, 'utf8');

function extractFunction(name) {
  const start = source.indexOf(`function ${name}(`);
  if (start < 0) throw new Error(`${name} function not found`);
  const body = source.indexOf('{', start);
  let depth = 0;
  for (let index = body; index < source.length; index += 1) {
    if (source[index] === '{') depth += 1;
    if (source[index] === '}') depth -= 1;
    if (depth === 0) return source.slice(start, index + 1);
  }
  throw new Error(`${name} function is incomplete`);
}

const definitions = [
  'rawProject',
  'project',
  'buildingPopoverAnchor',
  'resolveSelectedBuilding',
  'capabilityId',
  'buildingCapabilityView'
].map(extractFunction).join('\n');

function appFunctions(width, height, camera) {
  return Function('cssWidth', 'cssHeight', 'camera', `${definitions}; return {
    project, buildingPopoverAnchor, resolveSelectedBuilding, buildingCapabilityView
  };`)(width, height, camera);
}

const camera = { x: 50, y: 70, zoom: 1.25 };
const api = appFunctions(1000, 800, camera);
const building = { id: 'base-1', x: 120, y: 80 };
assert.deepStrictEqual(api.buildingPopoverAnchor(building, api.project, camera.zoom), { x: 462.5, y: 146 });
camera.x = 70;
camera.y = 60;
assert.deepStrictEqual(api.buildingPopoverAnchor(building, api.project, camera.zoom), { x: 437.5, y: 158.5 });
const resized = appFunctions(720, 500, { x: 70, y: 60, zoom: 2 });
assert.deepStrictEqual(resized.buildingPopoverAnchor(building, resized.project, 2), { x: 260, y: -74 });

const buildings = [building];
assert.strictEqual(api.resolveSelectedBuilding(buildings, 'base-1', () => 'visible'), building);
assert.strictEqual(api.resolveSelectedBuilding(buildings, null, () => 'visible'), null);
assert.strictEqual(api.resolveSelectedBuilding([], 'base-1', () => 'visible'), null);
assert.strictEqual(api.resolveSelectedBuilding(buildings, 'base-1', () => 'explored'), null);

const legacy = api.buildingCapabilityView({ produces: ['villager'], production: { product: 'villager', elapsed_seconds: 3 } });
assert.deepStrictEqual(legacy.products, ['villager']);
assert.strictEqual(legacy.job.progress, 0.5);
const unified = api.buildingCapabilityView({
  produces: ['villager'],
  researches: ['wheelbarrow'],
  researched_technologies: ['loom'],
  job: { type: 'research', target: 'wheelbarrow', elapsed_seconds: 3, required_seconds: 12 }
});
assert.deepStrictEqual(unified.researches, ['wheelbarrow']);
assert.deepStrictEqual(unified.researched, ['loom']);
assert.deepStrictEqual(unified.job, { type: 'research', subject: 'wheelbarrow', progress: 0.25 });

for (const name of ['resize', 'render', 'updateBuildingPopover']) {
  assert.match(extractFunction(name), /positionBuildingPopover\(\)/, `${name} must refresh the anchor`);
}
assert.match(extractFunction('positionBuildingPopover'), /buildingPopoverAnchor\(building, project, camera\.zoom\)/);

console.log('PASS building popover canonical anchor, lifecycle, and capability compatibility checks');
