// LintDrop - Main JavaScript
const tauri = window.__TAURI__;
const invoke = tauri?.core?.invoke ?? (async () => {
  throw new Error('请在 LintDrop 桌面应用中使用此功能');
});
const listen = tauri?.event?.listen ?? (() => Promise.resolve(() => {}));
const { computeLCS, MAX_RENDER_LINES } = window.LintDropDiff;

// State
let currentFiles = {};
let currentResults = [];
let isBatchMode = false;
const MAX_FILE_SIZE = 10 * 1024 * 1024;
const MAX_BATCH_SIZE = MAX_FILE_SIZE * 2;

// DOM Elements
const dropZone = document.getElementById('drop-zone');
const fileInput = document.getElementById('file-input');
const btnPaste = document.getElementById('btn-paste');
const pasteModal = document.getElementById('paste-modal');
const pasteInput = document.getElementById('paste-input');
const pasteFormat = document.getElementById('paste-format');
const btnPasteCancel = document.getElementById('btn-paste-cancel');
const btnPasteConfirm = document.getElementById('btn-paste-confirm');
const results = document.getElementById('results');
const singleView = document.getElementById('single-view');
const batchView = document.getElementById('batch-view');
const fileExt = document.getElementById('file-ext');
const fileName = document.getElementById('file-name');
const fileSize = document.getElementById('file-size');
const btnClose = document.getElementById('btn-close');
const statusCard = document.getElementById('status-card');
const statusIcon = document.getElementById('status-icon');
const statusText = document.getElementById('status-text');
const errorList = document.getElementById('error-list');
const actionBar = document.getElementById('action-bar');
const btnCopyErrors = document.getElementById('btn-copy-errors');
const btnCopyRaw = document.getElementById('btn-copy-raw');
const btnSchema = document.getElementById('btn-schema');
const btnDownload = document.getElementById('btn-download');
const diffPanel = document.getElementById('diff-panel');
const diffBefore = document.getElementById('diff-before');
const diffAfter = document.getElementById('diff-after');
const sourcePreview = document.getElementById('source-preview');
const sourceCode = document.getElementById('source-code');
const batchBody = document.getElementById('batch-body');
const batchDetail = document.getElementById('batch-detail');
const batchDetailName = document.getElementById('batch-detail-name');
const batchDetailErrors = document.getElementById('batch-detail-errors');
const btnBatchDetailClose = document.getElementById('btn-batch-detail-close');
const btnBatchCopy = document.getElementById('btn-batch-copy');
const btnBatchDownload = document.getElementById('btn-batch-download');
const btnBatchClear = document.getElementById('btn-batch-clear');
const statusBar = document.getElementById('status-bar');
const btnAbout = document.getElementById('btn-about');
const aboutModal = document.getElementById('about-modal');
const aboutOverlay = document.getElementById('about-overlay');
const btnAboutClose = document.getElementById('btn-about-close');

// ---- Tauri Native File Drop (v2 events) ----
listen('tauri://drag-drop', async (event) => {
  dropZone.classList.remove('dragover');

  const paths = event.payload?.paths || [];
  if (paths.length === 0) return;

  let fileContents;
  try {
    fileContents = await invoke('cmd_read_files', { paths });
  } catch (err) {
    updateStatus(`无法读取拖拽文件: ${err}`, 'error');
    return;
  }

  if (fileContents.length === 1) {
    const [name, content] = fileContents[0];
    await checkSingleFile(content, name);
  } else {
    await checkBatch(fileContents);
  }
});

listen('tauri://drag-enter', () => {
  dropZone.classList.add('dragover');
});

listen('tauri://drag-leave', () => {
  dropZone.classList.remove('dragover');
});

btnAbout.addEventListener('click', () => {
  aboutModal.classList.remove('hidden');
});

aboutOverlay.addEventListener('click', () => {
  aboutModal.classList.add('hidden');
});

btnAboutClose.addEventListener('click', () => {
  aboutModal.classList.add('hidden');
});

// ---- Click to browse (fallback) ----
dropZone.addEventListener('click', (e) => {
  if (e.target === btnPaste) return;
  fileInput.click();
});

fileInput.addEventListener('change', async () => {
  const files = Array.from(fileInput.files);
  fileInput.value = '';
  if (files.length === 0) return;

  let totalSize = 0;
  for (const f of files) {
    if (f.size > MAX_FILE_SIZE) {
      updateStatus(`这个文件有点大，最多支持 10 MB：${f.name}`, 'error');
      return;
    }
    totalSize += f.size;
    if (totalSize > MAX_BATCH_SIZE) {
      updateStatus('这批文件有点大，合计最多支持 20 MB', 'error');
      return;
    }
  }

  const fileContents = [];
  for (const f of files) {
    try {
      const text = await f.text();
      fileContents.push([f.name, text]);
    } catch (err) {
      updateStatus(`读不了这个文件：${f.name}（${err.message}）`, 'error');
      return;
    }
  }

  if (fileContents.length === 1) {
    const [name, content] = fileContents[0];
    await checkSingleFile(content, name);
  } else {
    await checkBatch(fileContents);
  }
});

// ---- Paste Modal ----
btnPaste.addEventListener('click', (e) => {
  e.stopPropagation();
  pasteModal.classList.remove('hidden');
  pasteInput.focus();
});

btnPasteCancel.addEventListener('click', () => {
  pasteModal.classList.add('hidden');
  pasteInput.value = '';
});

document.querySelector('.modal-overlay').addEventListener('click', () => {
  pasteModal.classList.add('hidden');
  pasteInput.value = '';
});

btnPasteConfirm.addEventListener('click', async () => {
  const text = pasteInput.value;
  if (!text.trim()) return;

  const format = pasteFormat.value === 'auto' ? detectFormatFromContent(text) : pasteFormat.value;
  pasteModal.classList.add('hidden');
  pasteInput.value = '';

  await checkSingleFile(text, 'pasted.' + format, format);
});

// ---- Check Logic ----
async function checkSingleFile(content, filename, format) {
  try {
    // 如果提供了 format，直接传给后端；否则让后端推断
    const result = format
      ? await invoke('cmd_check_file', { content, filename: filename || ('file.' + format) })
      : await invoke('cmd_check_file', { content, filename });
    currentFiles = { [filename]: content };
    currentResults = [{ filename, result }];
    isBatchMode = false;
    renderSingleResult(filename, result);
  } catch (err) {
    updateStatus('检查没完成：' + err, 'error');
  }
}

async function checkBatch(files) {
  const normalizedFiles = makeUniqueFileEntries(files);
  try {
    const batchResults = await invoke('cmd_check_batch', { files: normalizedFiles });
    currentFiles = {};
    normalizedFiles.forEach(([name, content]) => { currentFiles[name] = content; });
    currentResults = batchResults;
    isBatchMode = true;
    renderBatchUI(batchResults);
  } catch (err) {
    updateStatus('批量检查没完成：' + err, 'error');
  }
}

function makeUniqueFileEntries(files) {
  const seen = new Map();
  return files.map(([name, content]) => {
    const count = seen.get(name) || 0;
    seen.set(name, count + 1);
    if (count === 0) return [name, content];
    return [appendFilenameSuffix(name, ` (${count + 1})`), content];
  });
}

function appendFilenameSuffix(name, suffix) {
  const dot = name.lastIndexOf('.');
  if (dot > 0) {
    return name.slice(0, dot) + suffix + name.slice(dot);
  }
  return name + suffix;
}

// ---- Render Single Result ----
function renderSingleResult(filename, result) {
  dropZone.classList.add('hidden');
  results.classList.remove('hidden');
  singleView.classList.remove('hidden');
  batchView.classList.add('hidden');

  fileExt.textContent = result.format.toUpperCase();
  fileName.textContent = filename;
  fileSize.textContent = formatSize(getByteLength(currentFiles[filename] || ''));

  if (result.valid) {
    statusCard.className = 'status-card status-ok';
    statusIcon.textContent = '\u2713';
    statusText.textContent = `检查通过 · ${result.format.toUpperCase()} · ${filename}`;
    errorList.classList.add('hidden');
    configureActionButtons(result);
    actionBar.classList.remove('hidden');
    diffPanel.classList.add('hidden');
    sourcePreview.classList.remove('hidden');
    sourceCode.textContent = currentFiles[filename] || '';
  } else {
    const fixable = Boolean(result.corrected);
    statusCard.className = fixable ? 'status-card status-fixable' : 'status-card status-error';
    statusIcon.textContent = fixable ? '!' : '\u2717';
    statusText.textContent = fixable
      ? `发现 ${result.errors.length} 个问题，已经准备好修正`
      : `发现 ${result.errors.length} 个问题，需要手动处理`;

    errorList.classList.remove('hidden');
    errorList.innerHTML = result.errors.map((err) => `
      <div class="error-item">
        <div class="error-location">${formatLocation(err)}</div>
        <div class="error-message">${escapeHtml(err.friendly)}</div>
        ${err.near ? `<div class="error-near">${escapeHtml(err.near)}</div>` : ''}
      </div>
    `).join('');

    configureActionButtons(result);
    actionBar.classList.remove('hidden');
    sourcePreview.classList.add('hidden');

    if (result.corrected) {
      diffPanel.classList.remove('hidden');
      renderDiff(currentFiles[filename], result.corrected);
    } else {
      diffPanel.classList.add('hidden');
    }
  }

  statusBar.textContent = `${result.format.toUpperCase()} · ${formatSize(getByteLength(currentFiles[filename] || ''))} · 检查完成`;
}

function configureActionButtons(result) {
  btnCopyErrors.classList.toggle('hidden', result.valid);
  btnSchema.classList.toggle('hidden', result.format !== 'json');
  btnDownload.classList.toggle('hidden', !result.valid && !result.corrected);
  btnDownload.textContent = result.valid ? '保存原文件' : '保存修正文件';
}

// ---- Render Batch Results ----
function renderBatchUI(batchResults) {
  dropZone.classList.add('hidden');
  results.classList.remove('hidden');
  batchView.classList.remove('hidden');
  singleView.classList.add('hidden');

  batchBody.innerHTML = batchResults.map((item, i) => {
    const ok = item.result.valid;
    const fixable = !ok && Boolean(item.result.corrected);
    const count = item.result.errors.length;
    const statusClass = ok ? 'batch-status-ok' : (fixable ? 'batch-status-fixable' : 'batch-status-err');
    const statusText = ok ? '\u2713 通过' : (fixable ? `! ${count} 个问题，已准备修正` : `\u2717 ${count} 个问题`);
    return `
      <tr>
        <td title="${escapeHtml(item.filename)}">${escapeHtml(item.filename)}</td>
        <td>${item.result.format.toUpperCase()}</td>
        <td class="${statusClass}">${statusText}</td>
        <td><button class="batch-expand" data-index="${i}">展开</button></td>
      </tr>
    `;
  }).join('');

  batchBody.querySelectorAll('.batch-expand').forEach(btn => {
    btn.addEventListener('click', () => {
      const idx = parseInt(btn.dataset.index);
      showBatchDetail(batchResults[idx]);
    });
  });

  const totalErrors = batchResults.reduce((sum, item) => sum + item.result.errors.length, 0);
  const totalOk = batchResults.filter(item => item.result.valid).length;
  const totalFixable = batchResults.filter(item => !item.result.valid && item.result.corrected).length;
  statusBar.textContent = `批量检查完成 · ${batchResults.length} 个文件 · ${totalOk} 个通过 · ${totalFixable} 个待修正 · ${totalErrors} 个问题`;
}

function showBatchDetail(item) {
  batchDetail.classList.remove('hidden');
  batchDetailName.textContent = item.filename;
  if (item.result.valid) {
    batchDetailErrors.innerHTML = '<div class="error-item"><div class="error-message" style="color:var(--green)">检查通过</div></div>';
  } else {
    batchDetailErrors.innerHTML = item.result.errors.map(err => `
      <div class="error-item">
        <div class="error-location">${formatLocation(err)}</div>
        <div class="error-message">${escapeHtml(err.friendly)}</div>
        ${err.near ? `<div class="error-near">${escapeHtml(err.near)}</div>` : ''}
      </div>
    `).join('');
  }
}

btnBatchDetailClose.addEventListener('click', () => {
  batchDetail.classList.add('hidden');
});

// ---- Diff Rendering (LCS-based) ----
function renderDiff(before, after) {
  const beforeLines = before.split('\n');
  const afterLines = after.split('\n');

  if (beforeLines.length + afterLines.length > MAX_RENDER_LINES) {
    diffBefore.textContent = `文件较大，已跳过逐行对比（${beforeLines.length.toLocaleString()} 行）`;
    diffAfter.textContent = `文件较大，已跳过逐行对比（${afterLines.length.toLocaleString()} 行）`;
    return;
  }

  // LCS 计算最长公共子序列
  const lcs = computeLCS(beforeLines, afterLines);

  let beforeHtml = '';
  let afterHtml = '';
  let bi = 0, ai = 0, li = 0;

  while (bi < beforeLines.length || ai < afterLines.length) {
    if (li < lcs.length && bi < beforeLines.length && ai < afterLines.length
        && beforeLines[bi] === lcs[li] && afterLines[ai] === lcs[li]) {
      // 两行都匹配 LCS
      beforeHtml += escapeHtml(beforeLines[bi]) + '\n';
      afterHtml += escapeHtml(afterLines[ai]) + '\n';
      bi++; ai++; li++;
    } else if (li < lcs.length && bi < beforeLines.length && beforeLines[bi] !== lcs[li]) {
      // before 中有删除的行
      beforeHtml += `<span class="diff-line-del">${escapeHtml(beforeLines[bi])}</span>\n`;
      bi++;
    } else if (li < lcs.length && ai < afterLines.length && afterLines[ai] !== lcs[li]) {
      // after 中有新增的行
      afterHtml += `<span class="diff-line-add">${escapeHtml(afterLines[ai])}</span>\n`;
      ai++;
    } else if (li >= lcs.length && bi < beforeLines.length) {
      // LCS 已用完，before 剩余的都是删除
      beforeHtml += `<span class="diff-line-del">${escapeHtml(beforeLines[bi])}</span>\n`;
      bi++;
    } else if (li >= lcs.length && ai < afterLines.length) {
      // LCS 已用完，after 剩余的都是新增
      afterHtml += `<span class="diff-line-add">${escapeHtml(afterLines[ai])}</span>\n`;
      ai++;
    } else {
      break;
    }
  }

  diffBefore.innerHTML = beforeHtml;
  diffAfter.innerHTML = afterHtml;
}

// ---- Actions ----
btnClose.addEventListener('click', resetAll);

btnCopyErrors.addEventListener('click', async () => {
  const result = currentResults[0]?.result;
  if (!result) return;
  const text = result.errors.map(err =>
    `${formatLocation(err)}: ${err.friendly}`
  ).join('\n');
  try {
    await navigator.clipboard.writeText(text);
    flashStatus('问题已复制');
  } catch {
    flashStatus('复制没成功');
  }
});

btnCopyRaw.addEventListener('click', async () => {
  const filename = Object.keys(currentFiles)[0];
  if (filename) {
    try {
      await navigator.clipboard.writeText(currentFiles[filename]);
      flashStatus('已复制原文');
    } catch {
      flashStatus('复制没成功');
    }
  }
});

btnDownload.addEventListener('click', () => {
  const result = currentResults[0]?.result;
  const filename = Object.keys(currentFiles)[0];
  if (!result || !filename) return;

  const content = result.corrected || currentFiles[filename];
  const fixedName = result.valid ? filename : 'fixed_' + filename;
  showDownloadModal(fixedName, content);
});

btnSchema.addEventListener('click', () => {
  schemaInput.value = '';
  schemaResult.classList.add('hidden');
  schemaResult.innerHTML = '';
  schemaModal.classList.remove('hidden');
  schemaInput.focus();
});

btnBatchCopy.addEventListener('click', async () => {
  const text = currentResults
    .filter(item => !item.result.valid)
    .map(item => {
      const errs = item.result.errors.map(e => `  ${formatLocation(e)}: ${e.friendly}`).join('\n');
      return `${item.filename}:\n${errs}`;
    }).join('\n\n');
  try {
    await navigator.clipboard.writeText(text);
    flashStatus('全部问题已复制');
  } catch {
    flashStatus('复制失败');
  }
});

btnBatchDownload.addEventListener('click', async () => {
  const items = currentResults.filter(item => item.result.corrected);
  if (items.length === 0) {
    flashStatus('没有可以保存的修正文件');
    return;
  }
  let successCount = 0;
  for (const item of items) {
    const fixedName = 'fixed_' + item.filename;
    const ok = await saveFileDirect(fixedName, item.result.corrected);
    if (ok) successCount++;
  }
  if (successCount === items.length) {
    flashStatus(`已保存 ${successCount} 个修正文件`);
  } else {
    flashStatus(`已保存 ${successCount}/${items.length} 个，部分文件没保存成功`);
  }
});

btnBatchClear.addEventListener('click', resetAll);

// ---- Helpers ----
function resetAll() {
  currentFiles = {};
  currentResults = [];
  isBatchMode = false;
  dropZone.classList.remove('hidden');
  results.classList.add('hidden');
  singleView.classList.add('hidden');
  batchView.classList.add('hidden');
  batchDetail.classList.add('hidden');
  statusBar.textContent = '准备好了';
}

function updateStatus(msg, type) {
  results.classList.remove('hidden');
  singleView.classList.remove('hidden');
  batchView.classList.add('hidden');
  dropZone.classList.add('hidden');
  statusCard.className = `status-card status-${type}`;
  statusIcon.textContent = type === 'error' ? '\u2717' : '\u2713';
  statusText.textContent = msg;
  errorList.classList.add('hidden');
  actionBar.classList.add('hidden');
  diffPanel.classList.add('hidden');
  sourcePreview.classList.add('hidden');
}

function flashStatus(msg) {
  const original = statusBar.textContent;
  statusBar.textContent = msg;
  statusBar.style.color = 'var(--green)';
  setTimeout(() => {
    statusBar.textContent = original;
    statusBar.style.color = '';
  }, 2000);
}

function getByteLength(str) {
  return new TextEncoder().encode(str).length;
}

function formatSize(bytes) {
  if (bytes < 1024) return bytes + ' B';
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB';
  return (bytes / (1024 * 1024)).toFixed(1) + ' MB';
}

function formatLocation(err) {
  const parts = [];
  if (err.line) parts.push(`第 ${err.line} 行`);
  if (err.col) parts.push(`第 ${err.col} 列`);
  return parts.length > 0 ? parts.join('\uff0c') : '未知位置';
}

function escapeHtml(text) {
  const div = document.createElement('div');
  div.textContent = text;
  return div.innerHTML;
}

function detectFormatFromContent(text) {
  const trimmed = text.trim();
  if (trimmed.startsWith('{')) return 'json';
  if (trimmed.startsWith('[')) return detectBracketedFormat(trimmed);
  if (trimmed.startsWith('<')) return 'xml';
  if (trimmed.startsWith('---') || trimmed.includes('\n---\n')) return 'yaml';
  // TOML: 有 section [header] 且有 key = value
  if (trimmed.split('\n').some(l => {
    const t = l.trim();
    return t.startsWith('[') && t.includes(']');
  }) && trimmed.includes('=') && !trimmed.includes('{')) {
    return 'toml';
  }
  if (trimmed.includes(': ') || trimmed.split('\n').some(l => l.trim().startsWith('- '))) return 'yaml';
  if (trimmed.includes(',') && trimmed.split('\n').length > 1) return 'csv';
  if (trimmed.includes('=') && !trimmed.includes('{')) return 'env';
  return 'txt';
}

function detectBracketedFormat(trimmed) {
  try {
    JSON.parse(trimmed);
    return 'json';
  } catch {
    // Sectioned config files such as TOML/INI also begin with '['.
  }

  const lines = trimmed.split('\n');
  const hasSection = lines.some((line) => {
    const t = line.trim();
    return t.startsWith('[') && t.includes(']') && !t.startsWith('[{');
  });
  if (!hasSection) return 'json';

  if (looksLikeIni(lines)) return 'ini';
  if (lines.some((line) => line.includes('='))) return 'toml';
  return 'ini';
}

function looksLikeIni(lines) {
  return lines.some((line) => {
    const trimmed = line.trim();
    if (!trimmed || trimmed.startsWith('#')) return false;
    if (trimmed.startsWith(';')) return true;
    const eq = trimmed.indexOf('=');
    if (eq === -1) return false;

    const value = trimmed.slice(eq + 1).trim();
    return value === '' || isIniBareValue(value);
  });
}

function isIniBareValue(value) {
  const lower = value.toLowerCase();
  if (['true', 'false', 'nan', 'inf', '+inf', '-inf'].includes(lower)) return false;
  const first = value[0];
  if (!first) return true;
  if (['"', "'", '[', '{', '+', '-'].includes(first)) return false;
  if (first >= '0' && first <= '9') return false;
  return /[A-Za-z\u0080-\uFFFF]/.test(value);
}

// ---- Download Modal ----
const downloadModal = document.getElementById('download-modal');
const downloadOverlay = document.getElementById('download-overlay');
const downloadFilename = document.getElementById('download-filename');
const downloadPath = document.getElementById('download-path');
const btnBrowse = document.getElementById('btn-browse');
const btnDownloadCancel = document.getElementById('btn-download-cancel');
const btnDownloadConfirm = document.getElementById('btn-download-confirm');
const schemaModal = document.getElementById('schema-modal');
const schemaOverlay = document.getElementById('schema-overlay');
const schemaInput = document.getElementById('schema-input');
const schemaResult = document.getElementById('schema-result');
const btnSchemaCancel = document.getElementById('btn-schema-cancel');
const btnSchemaConfirm = document.getElementById('btn-schema-confirm');

let pendingDownload = { filename: '', content: '' };

function showDownloadModal(filename, content) {
  pendingDownload = { filename, content };
  downloadFilename.value = filename;
  downloadPath.value = '';
  downloadModal.classList.remove('hidden');
}

downloadOverlay.addEventListener('click', () => {
  downloadModal.classList.add('hidden');
});

btnDownloadCancel.addEventListener('click', () => {
  downloadModal.classList.add('hidden');
});

schemaOverlay.addEventListener('click', () => {
  schemaModal.classList.add('hidden');
});

btnSchemaCancel.addEventListener('click', () => {
  schemaModal.classList.add('hidden');
});

btnSchemaConfirm.addEventListener('click', async () => {
  const filename = Object.keys(currentFiles)[0];
  const content = filename ? currentFiles[filename] : '';
  const schema = schemaInput.value.trim();

  if (!schema) return;

  try {
    const result = await invoke('cmd_check_schema', { content, schema });
    schemaResult.classList.remove('hidden');
    if (result.valid) {
      schemaResult.className = 'schema-result schema-ok';
      schemaResult.textContent = 'Schema 检查通过';
    } else {
      schemaResult.className = 'schema-result schema-error';
      schemaResult.innerHTML = result.errors.map((err) => `
        <div class="error-item">
          <div class="error-location">${formatLocation(err)}</div>
          <div class="error-message">${escapeHtml(err.friendly)}</div>
          ${err.near ? `<div class="error-near">${escapeHtml(err.near)}</div>` : ''}
        </div>
      `).join('');
    }
  } catch (err) {
    schemaResult.className = 'schema-result schema-error';
    schemaResult.classList.remove('hidden');
    schemaResult.textContent = 'Schema 检查没完成：' + err;
  }
});

btnBrowse.addEventListener('click', async () => {
  const askForPath = () => {
    const p = prompt('请输入保存路径（文件夹）:', downloadPath.value || '');
    if (p !== null) downloadPath.value = p;
  };

  try {
    if (tauri?.dialog?.open) {
      const dir = await tauri.dialog.open({
        directory: true,
        title: '选择保存位置'
      });
      if (dir) downloadPath.value = dir;
    } else {
      askForPath();
    }
  } catch {
    askForPath();
  }
});

btnDownloadConfirm.addEventListener('click', async () => {
  const filename = downloadFilename.value.trim() || pendingDownload.filename;
  const dir = downloadPath.value.trim();
  const content = pendingDownload.content;

  let fullPath;
  if (dir) {
    const sep = dir.includes('\\') ? '\\' : '/';
    fullPath = dir.replace(/[\/\\]$/, '') + sep + filename;
  } else {
    // 使用后端获取桌面路径，而非硬编码
    try {
      const desktop = await invoke('cmd_get_desktop');
      const sep = desktop.includes('\\') ? '\\' : '/';
      fullPath = desktop + sep + filename;
    } catch {
      // 回退到 home/Desktop
      const home = await invoke('cmd_get_home');
      const sep = home.includes('\\') ? '\\' : '/';
      fullPath = home + sep + 'Desktop' + sep + filename;
    }
  }

  try {
    await invoke('cmd_save_file', { path: fullPath, content });
    flashStatus('文件已保存: ' + fullPath);
    downloadModal.classList.add('hidden');
  } catch (err) {
    flashStatus('保存失败: ' + err);
  }
});

async function saveFileDirect(filename, content) {
  try {
    const desktop = await invoke('cmd_get_desktop');
    const sep = desktop.includes('\\') ? '\\' : '/';
    const fullPath = desktop + sep + filename;
    await invoke('cmd_save_file', { path: fullPath, content });
    return true;
  } catch (err) {
    flashStatus('保存失败: ' + filename + ' - ' + err);
    return false;
  }
}
