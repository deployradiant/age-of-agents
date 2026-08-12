#!/usr/bin/env node
'use strict';

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const root = path.join(__dirname, '..');
const app = fs.readFileSync(path.join(root, 'frontend/app.js'), 'utf8');
const grid = fs.readFileSync(path.join(root, 'frontend/grid-overlay.js'), 'utf8');
const html = fs.readFileSync(path.join(root, 'frontend/index.html'), 'utf8');

assert.match(html, /grid-overlay\.js[\s\S]*app\.js/, 'grid module loads before the renderer');
assert.match(app, /window\.GridOverlay\.draw\(ctx, world, project, camera\)/, 'renderer delegates to the dedicated grid module');
assert.match(grid, /world\.terrain\.filter\(cell => cell\.visibility !== 'unseen'\)/, 'grid does not reveal unseen terrain');
assert.match(grid, /ctx\.strokeStyle = '#050806cc'/, 'grid uses a texture-independent dark structural pass');
assert.match(grid, /ctx\.strokeStyle = '#d9bd6638'/, 'grid uses only a restrained warm ink accent');
assert.match(grid, /ctx\.strokeStyle = '#efd277b8'/, 'visibility frontier receives a stronger boundary');
assert.match(app, /drawGround\(\);\s*drawGridOverlay\(\);\s*drawConstructionSites\(\);/, 'grid is rendered over terrain but below gameplay entities');
assert.match(app, /ctx\.strokeStyle = '#070907e6'[\s\S]*ctx\.strokeStyle = '#ffe073'/, 'selection ring uses dark separation plus a bright semantic edge');

console.log('PASS projected grid legibility, fog privacy, render order, and selected-unit separation');
