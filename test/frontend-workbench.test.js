const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const root = path.resolve(__dirname, '..');
const htmlPath = path.join(root, 'src', 'index.html');
const mainPath = path.join(root, 'src', 'main.js');
const samplesPath = path.join(root, 'src', 'samples.js');

test('index.html contains workbench tabs, guide, samples, and editor mode panels', () => {
  const html = fs.readFileSync(htmlPath, 'utf8');

  // Verify workbench tabs
  assert.ok(html.includes('id="tab-code"'), 'missing tab-code');
  assert.ok(html.includes('id="tab-visual"'), 'missing tab-visual');
  assert.ok(html.includes('id="tab-form"'), 'missing tab-form');
  assert.ok(html.includes('id="tab-diff"'), 'missing tab-diff');

  // Verify multi-tab file bar and badges
  assert.ok(html.includes('id="file-tabs-bar"'), 'missing file-tabs-bar');
  assert.ok(html.includes('id="file-profile"'), 'missing file-profile');
  assert.ok(html.includes('id="file-large-warning"'), 'missing file-large-warning');

  // Verify onboarding guide & samples modal
  assert.ok(html.includes('id="btn-guide"'), 'missing btn-guide');
  assert.ok(html.includes('id="guide-modal"'), 'missing guide-modal');
  assert.ok(html.includes('id="samples-modal"'), 'missing samples-modal');
  assert.ok(html.includes('id="sample-chips-list"'), 'missing sample-chips-list');

  // Verify editor panels
  assert.ok(html.includes('id="code-panel"'), 'missing code-panel');
  assert.ok(html.includes('id="visual-panel"'), 'missing visual-panel');
  assert.ok(html.includes('id="form-panel"'), 'missing form-panel');
  assert.ok(html.includes('id="diff-panel"'), 'missing diff-panel');

  // Verify code editor components
  assert.ok(html.includes('id="code-gutter"'), 'missing code-gutter');
  assert.ok(html.includes('id="code-editor"'), 'missing code-editor');

  // Verify visual editor components
  assert.ok(html.includes('id="visual-frozen-banner"'), 'missing visual-frozen-banner');
  assert.ok(html.includes('id="visual-tree-container"'), 'missing visual-tree-container');
  assert.ok(html.includes('id="btn-visual-undo"'), 'missing btn-visual-undo');
  assert.ok(html.includes('id="btn-visual-redo"'), 'missing btn-visual-redo');

  // Verify schema form & semantic diff components
  assert.ok(html.includes('id="schema-form-container"'), 'missing schema-form-container');
  assert.ok(html.includes('id="btn-diff-mode-text"'), 'missing btn-diff-mode-text');
  assert.ok(html.includes('id="btn-diff-mode-semantic"'), 'missing btn-diff-mode-semantic');
  assert.ok(html.includes('id="diff-semantic-body"'), 'missing diff-semantic-body');
  assert.ok(html.includes('id="btn-quick-fix"'), 'missing btn-quick-fix');
});

test('main.js handles format registry, profile system, auto-repair, and semantic diff commands', () => {
  const main = fs.readFileSync(mainPath, 'utf8');

  // Verify all essential Tauri commands invoked
  assert.ok(main.includes("cmd_list_formats"), 'missing cmd_list_formats in main.js');
  assert.ok(main.includes("cmd_detect_format"), 'missing cmd_detect_format in main.js');
  assert.ok(main.includes("cmd_parse_document"), 'missing cmd_parse_document in main.js');
  assert.ok(main.includes("cmd_apply_patch"), 'missing cmd_apply_patch in main.js');
  assert.ok(main.includes("cmd_format_document"), 'missing cmd_format_document in main.js');
  assert.ok(main.includes("cmd_validate_document"), 'missing cmd_validate_document in main.js');
  assert.ok(main.includes("cmd_simple_fix"), 'missing cmd_simple_fix in main.js');
  assert.ok(main.includes("cmd_list_profiles"), 'missing cmd_list_profiles in main.js');
  assert.ok(main.includes("cmd_detect_profile"), 'missing cmd_detect_profile in main.js');
  assert.ok(main.includes("cmd_validate_profile"), 'missing cmd_validate_profile in main.js');
  assert.ok(main.includes("cmd_compute_semantic_diff"), 'missing cmd_compute_semantic_diff in main.js');

  // Verify secret masking logic
  assert.ok(main.includes("isSensitiveKey"), 'missing isSensitiveKey in main.js');
  assert.ok(main.includes("btn-secret-toggle"), 'missing btn-secret-toggle in main.js');

  // Verify large file threshold fallback
  assert.ok(main.includes("LARGE_FILE_THRESHOLD"), 'missing LARGE_FILE_THRESHOLD in main.js');

  // Verify onboarding guide logic
  assert.ok(main.includes("initGuideTour"), 'missing initGuideTour in main.js');
  assert.ok(main.includes("renderSamplesUI"), 'missing renderSamplesUI in main.js');
});

test('samples.js contains comprehensive format templates', () => {
  const samplesContent = fs.readFileSync(samplesPath, 'utf8');
  const sandbox = {};
  const fn = new Function('window', samplesContent);
  fn(sandbox);

  const samples = sandbox.OrthosSamples;
  assert.ok(Array.isArray(samples) && samples.length >= 15, 'samples list must contain at least 15 formats');

  const requiredFormats = ['json', 'yaml', 'toml', 'xml', 'csv', 'ini', 'env', 'jsonc', 'json5', 'jsonl', 'tsv', 'properties', 'editorconfig', 'gitconfig', 'hcl'];
  for (const fmt of requiredFormats) {
    assert.ok(samples.some((s) => s.format === fmt), `missing sample for format: ${fmt}`);
  }

  // Verify key profile samples exist
  assert.ok(samples.some((s) => s.id === 'package.json'), 'missing package.json sample');
  assert.ok(samples.some((s) => s.id === 'docker-compose'), 'missing docker-compose sample');
});
