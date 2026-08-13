'use strict';

(function selectionControls(root) {
  const api = {
    normalizedRectangle(start, end) {
      return {
        left: Math.min(start.x, end.x),
        top: Math.min(start.y, end.y),
        right: Math.max(start.x, end.x),
        bottom: Math.max(start.y, end.y)
      };
    },

    enclosedUnitIds(units, rectangle, project) {
      return units
        .filter(unit => {
          const feet = project(Number(unit.x) || 0, Number(unit.y) || 0);
          return feet.x >= rectangle.left && feet.x <= rectangle.right
            && feet.y >= rectangle.top && feet.y <= rectangle.bottom;
        })
        .map(unit => unit.id);
    },

    reconcile(selectedIds, units) {
      const visibleIds = new Set(units.map(unit => unit.id));
      return new Set([...selectedIds].filter(id => visibleIds.has(id)));
    },

    update(selectedIds, unitId, additive) {
      const next = additive ? new Set(selectedIds) : new Set();
      if (additive && next.has(unitId)) next.delete(unitId);
      else next.add(unitId);
      return next;
    }
  };

  if (typeof module !== 'undefined' && module.exports) module.exports = api;
  root.SelectionControls = api;
}(typeof window === 'undefined' ? globalThis : window));
