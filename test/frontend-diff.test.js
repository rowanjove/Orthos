const test = require('node:test');
const assert = require('node:assert/strict');
const {
  MAX_EXACT_LCS_CELLS,
  MAX_RENDER_LINES,
  computeLCS,
  computeLCSGreedy,
  computeLCSMatchSets,
} = require('../src/diff.js');

test('exact LCS keeps the expected common sequence', () => {
  assert.deepEqual(
    computeLCS(['a', 'b', 'c'], ['a', 'x', 'c']),
    ['a', 'c'],
  );
});

test('large repeated-line inputs use the bounded greedy path', () => {
  const lineCount = 4_000;
  const before = Array(lineCount).fill('same');
  const after = Array(lineCount).fill('same');
  after[after.length - 1] = 'changed';

  const started = Date.now();
  const result = computeLCS(before, after);
  const elapsed = Date.now() - started;

  assert.equal(result.length, lineCount - 1);
  assert.ok(elapsed < 1_000, `greedy diff took ${elapsed}ms`);
  assert.ok(lineCount * lineCount > MAX_EXACT_LCS_CELLS);
});

test('greedy matching preserves order with duplicate lines', () => {
  assert.deepEqual(
    computeLCSGreedy(['a', 'b', 'a', 'c'], ['b', 'a', 'a', 'c']),
    ['a', 'a', 'c'],
  );
});

test('computeLCSMatchSets correctly identifies matched index sets', () => {
  const before = ['first', 'deleted', 'common', 'last'];
  const after = ['first', 'inserted', 'common', 'last'];
  const { beforeMatches, afterMatches } = computeLCSMatchSets(before, after);

  assert.ok(beforeMatches instanceof Set);
  assert.ok(afterMatches instanceof Set);
  assert.equal(beforeMatches.has(0), true); // 'first'
  assert.equal(beforeMatches.has(1), false); // 'deleted'
  assert.equal(beforeMatches.has(2), true); // 'common'
  assert.equal(beforeMatches.has(3), true); // 'last'

  assert.equal(afterMatches.has(0), true); // 'first'
  assert.equal(afterMatches.has(1), false); // 'inserted'
  assert.equal(afterMatches.has(2), true); // 'common'
  assert.equal(afterMatches.has(3), true); // 'last'
});

test('render guard is large enough for normal files but finite', () => {
  assert.ok(MAX_RENDER_LINES >= 10_000);
  assert.ok(MAX_RENDER_LINES < 100_000);
});
