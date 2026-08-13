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

function pointerSequence(pointerType, shiftKey, points, cancelled = false) {
  const pointer = controls.beginPointer(pointerType, shiftKey, true, false, points[0].x, points[0].y);
  const modes = points.slice(1).map(point => controls.movePointer(pointer, point.x, point.y, 8));
  return { pointer, modes, result: controls.finishPointer(pointer, cancelled, 8) };
}
assert.deepEqual(pointerSequence('mouse', false, [{ x: 10, y: 10 }, { x: 30, y: 30 }]).modes, ['pan']);
assert.equal(pointerSequence('mouse', true, [{ x: 10, y: 10 }, { x: 30, y: 30 }]).result, 'box');
assert.equal(pointerSequence('touch', false, [{ x: 10, y: 10 }]).result, 'tap');
assert.equal(pointerSequence('touch', false, [{ x: 10, y: 10 }, { x: 30, y: 30 }]).result, 'pan');
assert.equal(pointerSequence('mouse', true, [{ x: 10, y: 10 }, { x: 30, y: 30 }], true).result, 'cancel');
assert.equal(controls.beginPointer('touch', false, true, false, 0, 0).additive, false);
assert.equal(controls.beginPointer('mouse', false, true, false, 0, 0).additive, false);
assert.equal(controls.additiveTap('touch', false, 449), false);
assert.equal(controls.additiveTap('touch', false, 450), true);
assert.equal(controls.additiveTap('mouse', true, 0), true);

const root = path.join(__dirname, '..');
const app = fs.readFileSync(path.join(root, 'frontend/app.js'), 'utf8');
const html = fs.readFileSync(path.join(root, 'frontend/index.html'), 'utf8');
assert.match(html, /selection-controls\.js/, 'selection helpers load before app');
assert.match(app, /SelectionControls\.beginPointer/, 'pointer sequences use the tested gesture state machine');
assert.match(app, /SelectionControls\.finishPointer/, 'pointer completion uses the tested gesture state machine');
assert.match(app, /selectionRectangle = null;[\s\S]*result === 'tap'/, 'pointer completion and cancellation clear box preview before tap handling');
assert.match(app, /type: 'group_move', unit_ids: \[\.\.\.selectedUnitIds\]/, 'selected collection issues one group command');
assert.match(app, /SelectionControls\.reconcile/, 'snapshots remove stale selected IDs');
assert.match(app, /villagers selected/, 'HUD reports selected count');
console.log('PASS mouse pan, Shift-box, touch tap/pan/additive, cancellation, stale cleanup, HUD, and group command contracts');
