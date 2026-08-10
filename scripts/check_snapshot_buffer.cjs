'use strict';

const assert = require('node:assert/strict');
const { createSnapshotBuffer } = require('../frontend/snapshot-buffer.js');

function unit(id, x, action = { type: 'move', x: 100, y: 0 }, state = 'moving') {
  return { id, x, y: 0, action, state };
}

function state(units) {
  return { units, resources: [], buildings: [], terrain: [] };
}

const buffer = createSnapshotBuffer({ maxInterpolationDistance: 40 });
assert.equal(buffer.push(1, state([unit('a', 0)]), 100), true);
assert.equal(buffer.push(3, state([unit('a', 20)]), 300), true);
assert.equal(buffer.push(2, state([unit('a', 10)]), 200), false, 'older snapshots must not overwrite newer state');
assert.equal(buffer.authoritative().units[0].x, 20, 'commands read the newest authoritative position');
assert.deepEqual(buffer.sequences(), [1, 3], 'only the newest two accepted snapshots are retained');
assert.equal(buffer.presentation(400).units[0].x, 10, 'rendering interpolates halfway across the retained snapshots');
assert.equal(buffer.presentation(500).units[0].x, 20, 'presentation converges on authority without extrapolation');

buffer.push(4, state([unit('a', 25)]), 500);
buffer.push(5, state([unit('a', 30)]), 600);
assert.deepEqual(buffer.sequences(), [4, 5], 'bursts coalesce to two snapshots');
assert.equal(buffer.presentation(650).units[0].x, 27.5);

const stopped = createSnapshotBuffer();
stopped.push(1, state([unit('a', 0)]), 0);
stopped.push(2, state([unit('a', 10, { type: 'idle' }, 'idle')]), 100);
assert.equal(stopped.presentation(150).units[0].x, 10, 'action changes snap to authoritative position');

const teleported = createSnapshotBuffer({ maxInterpolationDistance: 40 });
teleported.push(1, state([unit('a', 0)]), 0);
teleported.push(2, state([unit('a', 500)]), 100);
assert.equal(teleported.presentation(150).units[0].x, 500, 'teleports are never animated through');

const visibility = createSnapshotBuffer();
visibility.push(1, state([unit('hidden-next', 0)]), 0);
visibility.push(2, state([]), 100);
assert.deepEqual(visibility.presentation(150).units, [], 'entities absent from current fog-filtered authority stay hidden');
visibility.push(3, state([unit('newly-visible', 12)]), 200);
assert.equal(visibility.presentation(250).units[0].x, 12, 'newly visible entities are not fabricated from an old position');

console.log('PASS snapshot ordering, coalescing, interpolation, teleport, and visibility checks');
