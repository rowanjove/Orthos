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

