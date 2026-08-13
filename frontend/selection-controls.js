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
    },

    gesture(pointerType, shiftKey, emptyGround, buildMode) {
      return {
        box: pointerType === 'mouse' && shiftKey && emptyGround && !buildMode,
        additive: shiftKey
      };
    },

    additiveTap(pointerType, shiftKey, heldMilliseconds) {
      return shiftKey || (pointerType === 'touch' && heldMilliseconds >= 450);
    },

    beginPointer(pointerType, shiftKey, emptyGround, buildMode, x, y) {
      const mode = this.gesture(pointerType, shiftKey, emptyGround, buildMode);
      return { x, y, startX: x, startY: y, lastX: x, lastY: y, dragging: false, mouseBox: mode.box, additive: mode.additive, pointerType, shiftKey, startedAt: Date.now() };
    },

    movePointer(pointer, x, y, threshold) {
      pointer.x = x;
      pointer.y = y;
      if (Math.hypot(x - pointer.startX, y - pointer.startY) > threshold) pointer.dragging = true;
      return pointer.mouseBox && pointer.dragging ? 'box' : pointer.dragging ? 'pan' : 'pending';
    },

    finishPointer(pointer, cancelled, threshold) {
      if (cancelled) return 'cancel';
      if (pointer.mouseBox && pointer.dragging) return 'box';
      return !pointer.dragging && Math.hypot(pointer.x - pointer.startX, pointer.y - pointer.startY) <= threshold ? 'tap' : 'pan';
    }
  };

  if (typeof module !== 'undefined' && module.exports) module.exports = api;
  root.SelectionControls = api;
}(typeof window === 'undefined' ? globalThis : window));
