#!/usr/bin/env node
'use strict';

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const controls = require('../frontend/selection-controls.js');

const rectangle = controls.normalizedRectangle({ x: 30, y: 40 }, { x: 10, y: 20 });
assert.deepEqual(rectangle, { left: 10, top: 20, right: 30, bottom: 40 });
const units = [{ id: 'a', x: 15, y: 25 }, { id: 'b', x: 31, y: 25 }];
assert.deepEqual(controls.enclosedUnitIds(units, rectangle, (x, y) => ({ x, y })), ['a']);
assert.deepEqual([...controls.reconcile(new Set(['a', 'gone']), units)], ['a']);
assert.deepEqual([...controls.update(new Set(['a']), 'b', true)], ['a', 'b']);
assert.deepEqual([...controls.update(new Set(['a', 'b']), 'a', true)], ['b']);
assert.deepEqual([...controls.update(new Set(['a']), 'b', false)], ['b']);

const root = path.join(__dirname, '..');
const app = fs.readFileSync(path.join(root, 'frontend/app.js'), 'utf8');
const html = fs.readFileSync(path.join(root, 'frontend/index.html'), 'utf8');
assert.match(html, /selection-controls\.js/, 'selection helpers load before app');
assert.match(app, /pointerType === 'mouse'[\s\S]*const mouseBox/, 'only mouse starts box selection');
assert.match(app, /pointerType === 'touch'/, 'touch taps use explicit additive selection');
assert.match(app, /type: 'group_move', unit_ids: \[\.\.\.selectedUnitIds\]/, 'selected collection issues one group command');
assert.match(app, /SelectionControls\.reconcile/, 'snapshots remove stale selected IDs');
assert.match(app, /villagers selected/, 'HUD reports selected count');
console.log('PASS selection rectangle, additive touch selection, stale cleanup, HUD, and group command contracts');
