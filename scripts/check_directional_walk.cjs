const { walk } = require('../frontend/activity-presentation.js');

const origin = { x: 100, y: 100 };
const cases = [
  [{ x: 200, y: 200 }, 'down', false],
  [{ x: 0, y: 0 }, 'up', false],
  [{ x: 300, y: 100 }, 'diag_toward', true],
  [{ x: 100, y: 300 }, 'diag_toward', false],
  [{ x: 200, y: 0 }, 'side', true],
];

for (const [target, direction, flip] of cases) {
  const actual = walk(origin, target);
  if (actual.direction !== direction || actual.flip !== flip) {
    throw new Error(
      `${JSON.stringify(target)} expected ${direction}/${flip}, got ${JSON.stringify(actual)}`
    );
  }
}

console.log(`PASS ${cases.length} projected directional-walk cases`);
