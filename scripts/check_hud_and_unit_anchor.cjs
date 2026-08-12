#!/usr/bin/env node
'use strict';

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const root = path.join(__dirname, '..');
const html = fs.readFileSync(path.join(root, 'frontend/index.html'), 'utf8');
const app = fs.readFileSync(path.join(root, 'frontend/app.js'), 'utf8');

assert.match(html, /id="tick">Tick <strong>0<\/strong>/, 'top bar exposes the authoritative tick');
assert.match(app, /const tickValue = document\.querySelector\('#tick strong'\)/, 'app binds the top tick value');
assert.match(app, /tickValue\.textContent = String\(tick\)/, 'HUD refresh publishes the latest authoritative tick');
assert.match(app, /asset\.alphaBottom = \(y \+ 1\) \/ sample\.height/, 'sprite alpha bounds define the frame-specific feet anchor');
assert.match(app, /const top = villager \? point\.y - height \* asset\.alphaBottom : point\.y - height \* 0\.88/, 'villagers use frame alpha while non-villager art preserves its existing anchor');

console.log('PASS top-level authoritative tick and frame-aware villager ground anchoring');
