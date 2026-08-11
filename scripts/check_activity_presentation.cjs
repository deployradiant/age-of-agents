#!/usr/bin/env node
'use strict';

const assert = require('node:assert/strict');
const { presentUnit, entities } = require('../frontend/activity-presentation.js');

const resource = { id: 'tree-1', kind: 'wood', position: { x: 400, y: 400 }, x: 400, y: 400, amount: 25 };
const base = { id: 'base-1', kind: 'town_center', position: { x: 100, y: 100 }, x: 100, y: 100 };
const raw = (id, phase) => ({ id, position: { x: 400, y: 400 }, action: { type: 'gather', resource_id: 'tree-1', phase } });
const world = { resources: [resource], buildings: [base] };

const gathering = presentUnit(raw('villager-1', 'gathering'), world);
assert.equal(gathering.state, 'gathering');
assert.equal(gathering.resourceId, 'tree-1');
const returning = presentUnit(raw('villager-2', 'returning'), world);
assert.equal(returning.state, 'moving', 'server phase, not resource distance, controls presentation');

const items = entities({ resources: [resource], buildings: [base], units: [gathering, { ...gathering, id: 'villager-2' }] });
assert.equal(items.filter(item => item.type === 'resource').length, 0, 'combined activities replace the standalone node');
const activities = items.filter(item => item.type === 'activity');
assert.equal(activities.length, 2, 'one activity is retained per associated villager');
assert.deepEqual(activities.map(item => item.data.screenOffsetX), [-12, 12]);
assert.ok(activities.every(item => item.data.activityResource.id === 'tree-1'));

const idle = { id: 'villager-3', x: 400, y: 400, state: 'idle' };
const layered = entities({ resources: [resource], buildings: [], units: [idle] });
assert.equal(layered.at(-1).type, 'unit', 'villagers win equal-depth ties over resources');
console.log('PASS phase-driven gathering, combined activities, 1:n offsets, and villager layering');
