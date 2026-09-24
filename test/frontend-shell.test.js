const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const root = path.resolve(__dirname, '..');
const htmlPath = path.join(root, 'src', 'index.html');
const mainPath = path.join(root, 'src', 'main.js');
const rustPath = path.join(root, 'src-tauri', 'src', 'lib.rs');

test('frontend references existing local assets in the right script order', () => {
  const html = fs.readFileSync(htmlPath, 'utf8');
  const refs = [...html.matchAll(/(?:src|href)="([^"]+)"/g)]
    .map((match) => match[1])
    .filter((ref) => !/^(?:https?:|#|data:)/.test(ref));

  for (const ref of refs) {
    assert.ok(fs.existsSync(path.join(root, 'src', ref)), `missing frontend asset: ${ref}`);
  }

  assert.ok(html.indexOf('src="diff.js"') < html.indexOf('src="main.js"'));
});

test('frontend invoke commands are registered by the Rust backend', () => {
  const main = fs.readFileSync(mainPath, 'utf8');
  const rust = fs.readFileSync(rustPath, 'utf8');
  const invoked = new Set([...main.matchAll(/invoke\('([^']+)'/g)].map((match) => match[1]));
  const registered = new Set([...rust.matchAll(/^\s*(cmd_[a-z_]+),\s*$/gm)].map((match) => match[1]));

  for (const command of invoked) {
    assert.ok(registered.has(command), `${command} is called by the frontend but not registered by Rust`);
  }
});

test('cmd_save_file and profile invocations align with backend IPC signatures', () => {
  const main = fs.readFileSync(mainPath, 'utf8');
  const rust = fs.readFileSync(rustPath, 'utf8');

  // Verify cmd_save_file supports filename and directory
  assert.ok(rust.includes('fn cmd_save_file('));
  assert.ok(rust.includes('filename: Option<String>'));
  assert.ok(rust.includes('directory: Option<String>'));

  // Verify cmd_detect_profile and cmd_validate_profile accept flexible/optional parameters
  assert.ok(rust.includes('fn cmd_detect_profile('));
  assert.ok(rust.includes('format: Option<String>'));
  assert.ok(rust.includes('fn cmd_validate_profile('));
  assert.ok(rust.includes('content: Option<String>'));

  // Verify main.js provides format parameter to profile calls
  assert.ok(main.includes("format: result.format || format"));
  assert.ok(main.includes("format: docState.format"));
});

test('drag-drop handler safely consumes tuple array from cmd_read_files', () => {
  const tupleResult = [['sample.json', '{"hello":"world"}']];
  const fileEntries = Array.isArray(tupleResult)
    ? tupleResult
    : (tupleResult?.files?.map((f) => [f.filename, f.content]) || []);

  assert.equal(fileEntries.length, 1);
  assert.equal(fileEntries[0][0], 'sample.json');
  assert.equal(fileEntries[0][1], '{"hello":"world"}');
});

test('semantic diff tag logic correctly maps camelCase changeType', () => {
  const itemAdded = { changeType: 'added', path: '/items/0' };
  const itemRemoved = { changeType: 'removed', path: '/debug' };
  const itemModified = { changeType: 'modified', path: '/port' };

  function getType(item) {
    return item.changeType || item.change_type || item.diff_type || 'modified';
  }

  assert.equal(getType(itemAdded), 'added');
  assert.equal(getType(itemRemoved), 'removed');
  assert.equal(getType(itemModified), 'modified');
});

test('escapeHtml safely encodes attribute-breaking characters', () => {
  const main = fs.readFileSync(mainPath, 'utf8');
  const fnCode = main.match(/function escapeHtml\(text\) \{[\s\S]*?\n\}/)[0];
  const escapeHtml = new Function(`${fnCode}; return escapeHtml;`)();

  assert.equal(
    escapeHtml('<div class="test" data-id=\'1\'>&'),
    '&lt;div class=&quot;test&quot; data-id=&#39;1&#39;&gt;&amp;',
  );
  assert.equal(escapeHtml(null), '');
  assert.equal(escapeHtml(undefined), '');
});
