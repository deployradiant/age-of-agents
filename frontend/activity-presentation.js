(function exposeActivityPresentation(root, factory) {
  const api = factory();
  if (typeof module === 'object' && module.exports) module.exports = api;
  else root.ActivityPresentation = api;
})(typeof globalThis === 'object' ? globalThis : this, () => {
  'use strict';

  function walk(position, target) {
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
        : Math.abs(screenY) > Math.abs(screenX) * 0.2 ? 'up' : 'side';
    return { direction, flip: !vertical && screenX > 0 };
  }

  function nearestBuilding(position, buildings) {
    return buildings.reduce((nearest, building) => {
      const distance = Math.hypot(position.x - building.position.x, position.y - building.position.y);
      return !nearest || distance < nearest.distance ? { position: building.position, distance } : nearest;
    }, null)?.position || null;
  }

  function presentUnit(unit, world) {
    const action = unit.action?.type || 'idle';
    const position = unit.position || { x: 0, y: 0 };
    let state = 'idle';
    let target = null;
    let resource = null;
    let gatherPhase = null;
    if (action === 'move') {
      state = 'moving';
      target = unit.action;
    } else if (action === 'gather') {
      resource = world.resources.find(item => item.id === unit.action.resource_id) || null;
      gatherPhase = unit.action.phase || 'to_resource';
      state = gatherPhase === 'gathering' ? 'gathering' : 'moving';
      target = ['to_resource', 'gathering'].includes(gatherPhase)
        ? resource?.position || null
        : nearestBuilding(position, world.buildings);
    } else if (action === 'build') {
      const distance = Math.hypot(position.x - unit.action.x, position.y - unit.action.y);
      state = distance > 0.5 ? 'moving' : 'building';
      target = unit.action;
    }
    const presentation = walk(position, target);
    return {
      ...unit,
      x: Number(position.x) || 0,
      y: Number(position.y) || 0,
      name: unit.name || unit.id.replace(/^villager-/, 'Villager '),
      state,
      resourceId: resource?.id || unit.action?.resource_id || null,
      resourceKind: resource?.kind || null,
      gatherPhase,
      walkDirection: presentation.direction,
      walkFlip: presentation.flip
    };
  }

  function entities(world) {
    const gatherers = new Map();
    for (const unit of world.units) {
      if (unit.state !== 'gathering' || !unit.resourceId) continue;
      const group = gatherers.get(unit.resourceId) || [];
      group.push(unit);
      gatherers.set(unit.resourceId, group);
    }
    for (const group of gatherers.values()) group.sort((a, b) => a.id.localeCompare(b.id));

    const items = [
      ...world.resources
        .filter(resource => !gatherers.has(resource.id))
        .map(data => ({ type: 'resource', data })),
      ...world.buildings.map(data => ({ type: 'building', data })),
      ...world.units.map(data => {
        const group = gatherers.get(data.resourceId);
        if (!group) return { type: 'unit', data };
        const rank = group.findIndex(unit => unit.id === data.id);
        return {
          type: 'activity',
          data: {
            ...data,
            activityResource: world.resources.find(resource => resource.id === data.resourceId) || null,
            screenOffsetX: (rank - (group.length - 1) / 2) * 24
          }
        };
      })
    ];
    const priority = { resource: 0, building: 1, unit: 2, activity: 2 };
    return items.sort((left, right) => {
      const depth = item => (Number(item.data.x) || 0) + (Number(item.data.y) || 0);
      return depth(left) - depth(right) || priority[left.type] - priority[right.type];
    });
  }

  return { walk, presentUnit, entities };
});
