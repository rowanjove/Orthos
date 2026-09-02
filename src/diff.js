// Pure diff helpers shared by the browser UI and Node regression tests.
(function initDiffHelpers(root) {
  const MAX_EXACT_LCS_CELLS = 1_000_000;
  const MAX_RENDER_LINES = 20_000;

  function computeLCS(a, b) {
    const m = a.length;
    const n = b.length;
    if (m === 0 || n === 0) return [];

    // The quadratic table is useful for small edits, but becomes a large
    // allocation quickly. Fall back before either dimension can exhaust the
    // renderer, including cases where both arrays are just below the old
    // per-dimension limit.
    if (m * n > MAX_EXACT_LCS_CELLS) {
      return computeLCSGreedy(a, b);
    }

    const dp = Array.from({ length: m + 1 }, () => new Array(n + 1).fill(0));
    for (let i = 1; i <= m; i++) {
      for (let j = 1; j <= n; j++) {
        if (a[i - 1] === b[j - 1]) {
          dp[i][j] = dp[i - 1][j - 1] + 1;
        } else {
          dp[i][j] = Math.max(dp[i - 1][j], dp[i][j - 1]);
        }
      }
    }

    const result = [];
    let i = m;
    let j = n;
    while (i > 0 && j > 0) {
      if (a[i - 1] === b[j - 1]) {
        result.unshift(a[i - 1]);
        i--;
        j--;
      } else if (dp[i - 1][j] > dp[i][j - 1]) {
        i--;
      } else {
        j--;
      }
    }
    return result;
  }

  function computeLCSGreedy(a, b) {
    // Map each line to sorted positions in b. A moving cursor plus binary
    // search keeps repeated-line inputs near O((m + n) log n), instead of the
    // previous O(m * n) scan through a `used` array.
    const positions = new Map();
    for (let i = 0; i < b.length; i++) {
      const line = b[i];
      const indices = positions.get(line);
      if (indices) {
        indices.push(i);
      } else {
        positions.set(line, [i]);
      }
    }

    const cursors = new Map();
    const result = [];
    let lastIndex = -1;

    for (const line of a) {
      const indices = positions.get(line);
      if (!indices) continue;

      let low = cursors.get(line) || 0;
      let high = indices.length;
      while (low < high) {
        const middle = low + Math.floor((high - low) / 2);
        if (indices[middle] <= lastIndex) {
          low = middle + 1;
        } else {
          high = middle;
        }
      }

      if (low < indices.length) {
        lastIndex = indices[low];
        cursors.set(line, low + 1);
        result.push(line);
      }
    }
    return result;
  }

  const api = {
    MAX_EXACT_LCS_CELLS,
    MAX_RENDER_LINES,
    computeLCS,
    computeLCSGreedy,
  };

  root.OrthosDiff = api;
  if (typeof module !== 'undefined' && module.exports) {
    module.exports = api;
  }
})(typeof window !== 'undefined' ? window : globalThis);
