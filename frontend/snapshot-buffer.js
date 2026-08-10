'use strict';

(function exposeSnapshotBuffer(root, factory) {
  const api = factory();
  if (typeof module === 'object' && module.exports) module.exports = api;
  else root.SnapshotBuffer = api;
}(typeof globalThis === 'object' ? globalThis : this, () => {
  function movementKey(unit) {
    const action = unit.action || {};
    if (unit.state !== 'moving') return '';
    if (action.type === 'move' || action.type === 'build') {
      return `${action.type}:${Number(action.x)}:${Number(action.y)}`;
    }
    if (action.type === 'gather') return `gather:${action.resource_id || ''}`;
    return '';
  }

  function createSnapshotBuffer({ maxInterpolationDistance = 48 } = {}) {
    let previous = null;
    let current = null;

    return {
      clear() {
        previous = null;
        current = null;
      },

      push(sequence, state, receivedAt = performance.now()) {
        const number = Number(sequence);
        if (!Number.isFinite(number) || number <= 0 || (current && number <= current.sequence)) return false;
        previous = current;
        current = { sequence: number, state, receivedAt: Number(receivedAt) };
        return true;
      },

      authoritative() {
        return current?.state || null;
      },

      sequences() {
        return [previous?.sequence, current?.sequence].filter(Number.isFinite);
      },

      presentation(now = performance.now()) {
        if (!current || !previous) return current?.state || null;
        const duration = Math.max(1, current.receivedAt - previous.receivedAt);
        const alpha = Math.max(0, Math.min(1, (Number(now) - current.receivedAt) / duration));
        if (alpha >= 1) return current.state;

        const olderUnits = new Map(previous.state.units.map(unit => [unit.id, unit]));
        const units = current.state.units.map(unit => {
          const older = olderUnits.get(unit.id);
          const key = movementKey(unit);
          if (!older || !key || key !== movementKey(older)) return unit;
          const dx = Number(unit.x) - Number(older.x);
          const dy = Number(unit.y) - Number(older.y);
          if (!Number.isFinite(dx) || !Number.isFinite(dy) || Math.hypot(dx, dy) > maxInterpolationDistance) return unit;
          return { ...unit, x: Number(older.x) + dx * alpha, y: Number(older.y) + dy * alpha };
        });
        return { ...current.state, units };
      }
    };
  }

  return { createSnapshotBuffer };
}));
