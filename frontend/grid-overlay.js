'use strict';

(function attachGridOverlay(global) {
  function draw(ctx, world, project, camera) {
    const visibleCells = world.terrain.filter(cell => cell.visibility !== 'unseen');
    if (!visibleCells.length) return;
    const visibility = new Map(world.terrain.map(cell => [`${cell.column}:${cell.row}`, cell.visibility]));

    const traceCells = () => {
      ctx.beginPath();
      for (const cell of visibleCells) {
        const x = cell.column * world.cellSize;
        const y = cell.row * world.cellSize;
        const a = project(x, y);
        const b = project(x + world.cellSize, y);
        const c = project(x + world.cellSize, y + world.cellSize);
        const d = project(x, y + world.cellSize);
        ctx.moveTo(a.x, a.y);
        ctx.lineTo(b.x, b.y);
        ctx.lineTo(c.x, c.y);
        ctx.lineTo(d.x, d.y);
        ctx.closePath();
      }
    };

    ctx.save();
    ctx.lineJoin = 'round';
    traceCells();
    ctx.strokeStyle = '#050806cc';
    ctx.lineWidth = Math.max(1.35, 2.15 * camera.zoom);
    ctx.stroke();
    traceCells();
    ctx.strokeStyle = '#d9bd6638';
    ctx.lineWidth = Math.max(0.45, 0.62 * camera.zoom);
    ctx.stroke();

    ctx.beginPath();
    for (const cell of visibleCells) {
      const x = cell.column * world.cellSize;
      const y = cell.row * world.cellSize;
      const a = project(x, y);
      const b = project(x + world.cellSize, y);
      const c = project(x + world.cellSize, y + world.cellSize);
      const d = project(x, y + world.cellSize);
      const edges = [
        [[0, -1], a, b],
        [[1, 0], b, c],
        [[0, 1], c, d],
        [[-1, 0], d, a]
      ];
      for (const [[dc, dr], start, end] of edges) {
        if (visibility.get(`${cell.column + dc}:${cell.row + dr}`) !== 'unseen') continue;
        ctx.moveTo(start.x, start.y);
        ctx.lineTo(end.x, end.y);
      }
    }
    ctx.strokeStyle = '#efd277b8';
    ctx.lineWidth = Math.max(1.25, 1.8 * camera.zoom);
    ctx.stroke();
    ctx.restore();
  }

  global.GridOverlay = { draw };
}(window));
