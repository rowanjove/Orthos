// Orthos - Main JavaScript (v2.0)
const tauri = window.__TAURI__;
const invoke = tauri?.core?.invoke ?? (async (cmd, args) => {
  throw new Error(`请在 Orthos 桌面应用中使用此功能: ${cmd}`);
});
const listen = tauri?.event?.listen ?? (() => Promise.resolve(() => {}));
const { computeLCS, computeLCSMatchSets, MAX_RENDER_LINES } = window.OrthosDiff;

// Global Registry & State
let formatRegistry = [];
let formatMap = new Map();
let currentFiles = {};
let currentResults = [];
let isBatchMode = false;

const MAX_FILE_SIZE = 10 * 1024 * 1024;
const MAX_BATCH_SIZE = MAX_FILE_SIZE * 2;
const LARGE_FILE_THRESHOLD = 2 * 1024 * 1024; // 2MB

// Multi-Tab Documents Workspace State
let tabDocuments = [];
let activeTabIndex = 0;

// Active Document State Reference
let activeMode = 'code'; // 'code' | 'visual' | 'form' | 'diff'
let docState = {
  filename: '',
  format: 'json',
  content: '',
  savedContent: '',
  dirty: false,
  tree: null,
  lastValidTree: null,
  valid: true,
  errors: [],
  corrected: null,
  profile: null,
  profileDiagnostics: [],
  schema: null,
  undoStack: [],
  redoStack: [],
};

let validationTimer = null;
let diffMode = 'text'; // 'text' | 'semantic'

// Guide Tour State
let currentGuideStep = 0;
const GUIDE_STEPS = [
  {
    title: '1. 智能格式嗅探与语法校验',
    desc: 'Orthos 本地离线运行，支持 JSON、YAML、TOML、XML、CSV、INI、ENV、JSONC、JSON5、TSV 等 15+ 种配置格式。',
    features: [
      '拖拽或粘贴任意配置，毫秒级启发式格式嗅探',
      '实时语法校验，行号槽精确标注错误位置与附近代码',
      '支持错误行一键跳转定位与 Ctrl+S 安全保存'
    ]
  },
  {
    title: '2. 双向同步多维可视化工作台',
    desc: '摆脱单一纯文本编辑，自动根据配置类型匹配最适宜的可视化交互视图。',
    features: [
      'Tree 树形编辑器（JSON / YAML / TOML）、Grid 表格（CSV / TSV）、KV 键值对（INI / ENV）与 DOM 视图（XML）',
      '代码编辑与可视化操作毫秒级双向无损同步',
      '非法语法安全冻结机制：输入错误时锁定结构树并保留快照，修复后自动恢复'
    ]
  },
  {
    title: '3. 业务 Profile 规范与 Schema 表单',
    desc: '超越单一语法检查，深入业务工程规范与数据约束。',
    features: [
      '内置 package.json、docker-compose、tsconfig、GitHub Actions 等规范语义诊断',
      '依赖冲突、大写包名、端口冲突即时预警与修复建议',
      '支持一键加载 JSON Schema 动态生成交互式表单控件'
    ]
  },
  {
    title: '4. 语义差异对比与一键自修复',
    desc: '安全可控的配置维护体验，避免意外修改与凭据泄漏。',
    features: [
      '结构化差异（Semantic Diff）：直观查看字段级新增、删除与数值变更卡片',
      '敏感字段自动遮罩：识别 password、token、api_key 等并提供明文切换',
      '一键自修复：秒级修正单双引号、尾逗号、缩进与重复依赖等典型异常'
    ]
  }
];

// DOM Elements
const dropZone = document.getElementById('drop-zone');
const formatBadges = document.getElementById('format-badges');
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

// Header & Guide
const btnGuide = document.getElementById('btn-guide');
const guideModal = document.getElementById('guide-modal');
const guideOverlay = document.getElementById('guide-overlay');
const guideTitle = document.getElementById('guide-title');
const guideStepBadge = document.getElementById('guide-step-badge');
const guideBody = document.getElementById('guide-body');
const btnGuidePrev = document.getElementById('btn-guide-prev');
const btnGuideNext = document.getElementById('btn-guide-next');
const btnGuideSample = document.getElementById('btn-guide-sample');

// Samples UI
const btnQuickSample = document.getElementById('btn-quick-sample');
const sampleChipsList = document.getElementById('sample-chips-list');
const sampleSelectDropdown = document.getElementById('sample-select-dropdown');
const btnOpenSamples = document.getElementById('btn-open-samples');
const samplesModal = document.getElementById('samples-modal');
const samplesOverlay = document.getElementById('samples-overlay');
const btnSamplesClose = document.getElementById('btn-samples-close');
const samplesGrid = document.getElementById('samples-grid');

// Multi-Tab Elements
const fileTabsBar = document.getElementById('file-tabs-bar');
const fileTabsList = document.getElementById('file-tabs-list');
const btnTabNew = document.getElementById('btn-tab-new');

// File Info Bar Elements
const fileExt = document.getElementById('file-ext');
const fileName = document.getElementById('file-name');
const fileSize = document.getElementById('file-size');
const fileDirty = document.getElementById('file-dirty');
const fileProfile = document.getElementById('file-profile');
const fileLargeWarning = document.getElementById('file-large-warning');
const btnClose = document.getElementById('btn-close');

// Workbench Tabs
const tabCode = document.getElementById('tab-code');
const tabVisual = document.getElementById('tab-visual');
const tabForm = document.getElementById('tab-form');
const tabDiff = document.getElementById('tab-diff');

// Editor Panels
const codePanel = document.getElementById('code-panel');
const visualPanel = document.getElementById('visual-panel');
const formPanel = document.getElementById('form-panel');
const diffPanel = document.getElementById('diff-panel');
const sourcePreview = document.getElementById('source-preview');

// Status & Error
const statusCard = document.getElementById('status-card');
const statusIcon = document.getElementById('status-icon');
const statusText = document.getElementById('status-text');
const errorList = document.getElementById('error-list');

// Action Bar
const actionBar = document.getElementById('action-bar');
const btnFormatDoc = document.getElementById('btn-format-doc');
const btnQuickFix = document.getElementById('btn-quick-fix');
const btnCopyErrors = document.getElementById('btn-copy-errors');
const btnCopyRaw = document.getElementById('btn-copy-raw');
const btnSchema = document.getElementById('btn-schema');
const btnDownload = document.getElementById('btn-download');

// Code Editor Elements
const codeGutter = document.getElementById('code-gutter');
const codeEditor = document.getElementById('code-editor');
const codeCursorPos = document.getElementById('code-cursor-pos');

// Visual Editor Elements
const visualFrozenBanner = document.getElementById('visual-frozen-banner');
const btnGotoFixSyntax = document.getElementById('btn-goto-fix-syntax');
const visualTreeContainer = document.getElementById('visual-tree-container');
const visualFormatBadge = document.getElementById('visual-format-badge');
const visualNodeCount = document.getElementById('visual-node-count');
const btnVisualUndo = document.getElementById('btn-visual-undo');
const btnVisualRedo = document.getElementById('btn-visual-redo');
const btnVisualExpandAll = document.getElementById('btn-visual-expand-all');
const btnVisualCollapseAll = document.getElementById('btn-visual-collapse-all');
const btnVisualAddRoot = document.getElementById('btn-visual-add-root');

// Schema Form Elements
const schemaFormContainer = document.getElementById('schema-form-container');
const btnFormClearSchema = document.getElementById('btn-form-clear-schema');

// Diff Elements
const btnDiffModeText = document.getElementById('btn-diff-mode-text');
const btnDiffModeSemantic = document.getElementById('btn-diff-mode-semantic');
const diffTextBody = document.getElementById('diff-text-body');
const diffSemanticBody = document.getElementById('diff-semantic-body');
const diffSemanticSummary = document.getElementById('diff-semantic-summary');
const diffSemanticList = document.getElementById('diff-semantic-list');
const diffBefore = document.getElementById('diff-before');
const diffAfter = document.getElementById('diff-after');

// Batch Elements
const batchBody = document.getElementById('batch-body');
const batchDetail = document.getElementById('batch-detail');
const batchDetailName = document.getElementById('batch-detail-name');
const batchDetailErrors = document.getElementById('batch-detail-errors');
const btnBatchDetailClose = document.getElementById('btn-batch-detail-close');
const btnBatchCopy = document.getElementById('btn-batch-copy');
const btnBatchDownload = document.getElementById('btn-batch-download');
const btnBatchClear = document.getElementById('btn-batch-clear');

// Status & About
const statusBar = document.getElementById('status-bar');
const btnAbout = document.getElementById('btn-about');
const aboutModal = document.getElementById('about-modal');
const aboutOverlay = document.getElementById('about-overlay');
const btnAboutClose = document.getElementById('btn-about-close');

// Download Modal
const downloadModal = document.getElementById('download-modal');
const downloadOverlay = document.getElementById('download-overlay');
const downloadFilename = document.getElementById('download-filename');
const downloadPath = document.getElementById('download-path');
const btnBrowse = document.getElementById('btn-browse');
const btnDownloadCancel = document.getElementById('btn-download-cancel');
const btnDownloadConfirm = document.getElementById('btn-download-confirm');

// Schema Modal
const schemaModal = document.getElementById('schema-modal');
const schemaOverlay = document.getElementById('schema-overlay');
const schemaInput = document.getElementById('schema-input');
const schemaResult = document.getElementById('schema-result');
const btnSchemaCancel = document.getElementById('btn-schema-cancel');
const btnSchemaConfirm = document.getElementById('btn-schema-confirm');

// ---- Initialization ----
async function initApp() {
  try {
    const list = await invoke('cmd_list_formats');
    if (Array.isArray(list) && list.length > 0) {
      formatRegistry = list;
      formatMap.clear();
      list.forEach((f) => formatMap.set(f.id, f));
      renderFormatBadges(list);
      renderFormatSelectOptions(list);
    }
    try {
      await invoke('cmd_list_profiles');
    } catch {
      // ignore
    }
  } catch (err) {
    console.warn('获取格式注册表失败，使用默认配置:', err);
  }

  renderSamplesUI();
  initGuideTour();
}

function renderFormatBadges(formats) {
  if (!formatBadges) return;
  formatBadges.innerHTML = formats
    .map((f) => `<span class="badge" title="${f.extensions.join(', ')}">${escapeHtml(f.name)}</span>`)
    .join('');
}

function renderFormatSelectOptions(formats) {
  if (!pasteFormat) return;
  pasteFormat.innerHTML = '<option value="auto">自动检测</option>' +
    formats.map((f) => `<option value="${f.id}">${escapeHtml(f.name)}</option>`).join('');
}

// ---- Built-in Samples UI ----
function renderSamplesUI() {
  const samples = window.OrthosSamples || [];
  if (samples.length === 0) return;

  // 1. Quick Chips in DropZone (Popular Formats)
  if (sampleChipsList) {
    const popularSamples = ['package.json', 'docker-compose', 'json', 'yaml', 'toml', 'csv', 'env', 'xml'];
    sampleChipsList.innerHTML = popularSamples.map((sid) => {
      const s = samples.find((item) => item.id === sid);
      if (!s) return '';
      return `<button type="button" class="sample-chip" data-sample-id="${s.id}">${escapeHtml(s.name)}</button>`;
    }).join('');

    sampleChipsList.querySelectorAll('.sample-chip').forEach((btn) => {
      btn.addEventListener('click', (e) => {
        e.stopPropagation();
        const sid = btn.getAttribute('data-sample-id');
        loadSampleById(sid);
      });
    });
  }

  // 2. Dropdown in DropZone (All 15+ Formats)
  if (sampleSelectDropdown) {
    sampleSelectDropdown.innerHTML = '<option value="">更多格式示例 (15+ 种)...</option>' +
      samples.map((s) => `<option value="${s.id}">${escapeHtml(s.name)} (${escapeHtml(s.format.toUpperCase())})</option>`).join('');

    sampleSelectDropdown.addEventListener('change', (e) => {
      e.stopPropagation();
      const sid = sampleSelectDropdown.value;
      if (sid) {
        loadSampleById(sid);
        sampleSelectDropdown.value = '';
      }
    });
  }

  // 3. Samples Modal Grid
  if (samplesGrid) {
    samplesGrid.innerHTML = samples.map((s) => `
      <div class="sample-grid-card" data-sample-id="${s.id}">
        <div class="sample-card-head">
          <span class="sample-card-title">${escapeHtml(s.name)}</span>
          <span class="badge">${escapeHtml(s.badge || s.format.toUpperCase())}</span>
        </div>
        <span class="sample-card-fn">${escapeHtml(s.filename)}</span>
      </div>
    `).join('');

    samplesGrid.querySelectorAll('.sample-grid-card').forEach((card) => {
      card.addEventListener('click', () => {
        const sid = card.getAttribute('data-sample-id');
        loadSampleById(sid);
        samplesModal.classList.add('hidden');
      });
    });
  }
}

function loadSampleById(sampleId) {
  const samples = window.OrthosSamples || [];
  const s = samples.find((item) => item.id === sampleId);
  if (!s) return;
  checkSingleFile(s.content, s.filename, s.format, s.schema);
  flashStatus(`已载入示例：${s.name}`);
}

btnQuickSample?.addEventListener('click', (e) => {
  e.stopPropagation();
  loadSampleById('package.json');
});

btnOpenSamples?.addEventListener('click', () => {
  samplesModal?.classList.remove('hidden');
});

btnSamplesClose?.addEventListener('click', () => {
  samplesModal?.classList.add('hidden');
});

samplesOverlay?.addEventListener('click', () => {
  samplesModal?.classList.add('hidden');
});

// ---- Onboarding Guide Tour ----
function initGuideTour() {
  btnGuide?.addEventListener('click', () => {
    openGuideTour(0);
  });

  guideOverlay?.addEventListener('click', () => {
    guideModal.classList.add('hidden');
  });

  btnGuidePrev?.addEventListener('click', () => {
    if (currentGuideStep > 0) {
      renderGuideStep(currentGuideStep - 1);
    }
  });

  btnGuideNext?.addEventListener('click', () => {
    if (currentGuideStep < GUIDE_STEPS.length - 1) {
      renderGuideStep(currentGuideStep + 1);
    } else {
      guideModal.classList.add('hidden');
      try {
        localStorage.setItem('orthos_guide_seen', 'true');
      } catch {
        // ignore
      }
    }
  });

  btnGuideSample?.addEventListener('click', () => {
    guideModal.classList.add('hidden');
    loadSampleById('package.json');
  });

  // Automatically show on first load
  try {
    const seen = localStorage.getItem('orthos_guide_seen');
    if (!seen) {
      openGuideTour(0);
    }
  } catch {
    // ignore
  }
}

function openGuideTour(stepIdx) {
  if (!guideModal) return;
  guideModal.classList.remove('hidden');
  renderGuideStep(stepIdx || 0);
}

function renderGuideStep(stepIdx) {
  currentGuideStep = stepIdx;
  const step = GUIDE_STEPS[stepIdx];
  if (!step) return;

  guideTitle.textContent = step.title;
  guideStepBadge.textContent = `${stepIdx + 1} / ${GUIDE_STEPS.length}`;

  guideBody.innerHTML = `
    <div class="guide-step-card">
      <p class="guide-step-desc">${escapeHtml(step.desc)}</p>
      <div class="guide-feature-list">
        ${step.features.map((f) => `<div class="guide-feature-item"><span>${escapeHtml(f)}</span></div>`).join('')}
      </div>
    </div>
  `;

  btnGuidePrev.disabled = (stepIdx === 0);
  btnGuideNext.textContent = (stepIdx === GUIDE_STEPS.length - 1) ? '完成指引' : '下一步';
}

// ---- File Drag & Drop ----
dropZone.addEventListener('dragover', (e) => {
  e.preventDefault();
  dropZone.classList.add('dragover');
});

dropZone.addEventListener('dragleave', (e) => {
  if (!dropZone.contains(e.relatedTarget)) {
    dropZone.classList.remove('dragover');
  }
});

dropZone.addEventListener('drop', async (e) => {
  e.preventDefault();
  dropZone.classList.remove('dragover');

  const dtFiles = e.dataTransfer.files;
  if (!dtFiles || dtFiles.length === 0) return;

  const files = Array.from(dtFiles);
  let totalSize = 0;
  for (const f of files) {
    if (f.size > MAX_FILE_SIZE) {
      updateStatus(`单文件超出大小限制 (上限 10 MB)：${f.name}`, 'error');
      return;
    }
    totalSize += f.size;
    if (totalSize > MAX_BATCH_SIZE) {
      updateStatus('批量文件总容量超出限制 (上限 20 MB)', 'error');
      return;
    }
  }

  const fileContents = [];
  for (const f of files) {
    try {
      const text = await f.text();
      fileContents.push([f.name, text]);
    } catch (err) {
      updateStatus(`文件读取失败：${f.name}（${err.message}）`, 'error');
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

// Tauri drag-drop handler
try {
  listen('tauri://drag-drop', async (event) => {
    const paths = event.payload?.paths;
    if (!paths || paths.length === 0) return;

    try {
      const readResult = await invoke('cmd_read_files', { paths });
      const fileEntries = Array.isArray(readResult)
        ? readResult
        : (readResult?.files?.map((f) => [f.filename, f.content]) || []);

      if (fileEntries.length === 1) {
        const [filename, content] = fileEntries[0];
        await checkSingleFile(content, filename);
      } else if (fileEntries.length > 1) {
        await checkBatch(fileEntries);
      }
    } catch (err) {
      updateStatus('文件读取失败: ' + err, 'error');
    }
  });
} catch {
  // Not in Tauri environment
}

// ---- Click to browse ----
dropZone.addEventListener('click', (e) => {
  if (e.target.closest('button') || e.target.closest('select') || e.target.closest('.sample-chip')) return;
  fileInput.click();
});

fileInput.addEventListener('change', async () => {
  const files = Array.from(fileInput.files);
  fileInput.value = '';
  if (files.length === 0) return;

  let totalSize = 0;
  for (const f of files) {
    if (f.size > MAX_FILE_SIZE) {
      updateStatus(`单文件超出大小限制 (上限 10 MB)：${f.name}`, 'error');
      return;
    }
    totalSize += f.size;
    if (totalSize > MAX_BATCH_SIZE) {
      updateStatus('批量文件总容量超出限制 (上限 20 MB)', 'error');
      return;
    }
  }

  const fileContents = [];
  for (const f of files) {
    try {
      const text = await f.text();
      fileContents.push([f.name, text]);
    } catch (err) {
      updateStatus(`文件读取失败：${f.name}（${err.message}）`, 'error');
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

pasteModal.querySelector('.modal-overlay').addEventListener('click', () => {
  pasteModal.classList.add('hidden');
  pasteInput.value = '';
});

btnPasteConfirm.addEventListener('click', async () => {
  const text = pasteInput.value;
  if (!text.trim()) return;

  let format = pasteFormat.value;
  if (format === 'auto') {
    try {
      format = await invoke('cmd_detect_format', { content: text, filename: null }) || 'json';
    } catch {
      format = 'json';
    }
  }

  pasteModal.classList.add('hidden');
  pasteInput.value = '';
  await checkSingleFile(text, 'pasted.' + format, format);
});

// ---- Check Single File & Initialize Workbench Document ----
async function checkSingleFile(content, filename, formatHint, schemaHint) {
  try {
    let format = formatHint;
    if (!format) {
      format = await invoke('cmd_detect_format', { content, filename }) || 'json';
    }

    const result = await invoke('cmd_check_file', {
      content,
      filename: filename || ('file.' + format),
    });

    let tree = null;
    if (result.valid) {
      try {
        tree = await invoke('cmd_parse_document', { content, format: result.format || format });
      } catch (e) {
        console.warn('解析结构树失败:', e);
      }
    }

    // Detect Profile & Profile Diagnostics
    let profile = null;
    let profileDiagnostics = [];
    if (result.valid && tree) {
      try {
        profile = await invoke('cmd_detect_profile', {
          filename: filename || '',
          content,
          format: result.format || format,
          doc: tree,
        });
        if (profile) {
          profileDiagnostics = await invoke('cmd_validate_profile', {
            profileId: profile,
            content,
            format: result.format || format,
            doc: tree,
          }) || [];
        }
      } catch (err) {
        console.warn('Profile 检测或校验失败:', err);
      }
    }

    const newDoc = {
      id: 'doc_' + Date.now() + '_' + Math.random().toString(36).substring(2, 6),
      filename: filename || ('file.' + format),
      format: result.format || format,
      content,
      savedContent: content,
      dirty: false,
      tree,
      lastValidTree: tree,
      valid: result.valid,
      errors: result.errors || [],
      corrected: result.corrected || null,
      profile,
      profileDiagnostics,
      schema: schemaHint || null,
      undoStack: [],
      redoStack: [],
      activeMode: 'code',
    };

    saveCurrentTabState();

    // Check if same filename is already open
    const existingIndex = tabDocuments.findIndex((d) => d.filename === newDoc.filename);
    if (existingIndex >= 0) {
      tabDocuments[existingIndex] = newDoc;
      activeTabIndex = existingIndex;
    } else {
      tabDocuments.push(newDoc);
      activeTabIndex = tabDocuments.length - 1;
    }

    currentFiles = { [newDoc.filename]: content };
    currentResults = [{ filename: newDoc.filename, result }];
    isBatchMode = false;

    loadActiveTabState();
    renderTabsBar();
    renderWorkbenchUI();
  } catch (err) {
    updateStatus('校验失败：' + err, 'error');
  }
}

// ---- Multi-Tab Workspace Functions ----
function saveCurrentTabState() {
  if (tabDocuments.length === 0 || activeTabIndex < 0 || activeTabIndex >= tabDocuments.length) return;
  tabDocuments[activeTabIndex] = {
    ...tabDocuments[activeTabIndex],
    content: docState.content,
    savedContent: docState.savedContent,
    dirty: docState.dirty,
    valid: docState.valid,
    errors: docState.errors,
    corrected: docState.corrected,
    tree: docState.tree,
    lastValidTree: docState.lastValidTree,
    profile: docState.profile,
    profileDiagnostics: docState.profileDiagnostics,
    schema: docState.schema,
    undoStack: [...visualUndoStack],
    redoStack: [...visualRedoStack],
    activeMode,
  };
}

function loadActiveTabState() {
  if (tabDocuments.length === 0) return;
  if (activeTabIndex < 0 || activeTabIndex >= tabDocuments.length) {
    activeTabIndex = 0;
  }
  const cur = tabDocuments[activeTabIndex];
  docState = { ...cur };
  visualUndoStack = [...(cur.undoStack || [])];
  visualRedoStack = [...(cur.redoStack || [])];
  activeMode = cur.activeMode || 'code';
}

function renderTabsBar() {
  if (!fileTabsBar || !fileTabsList) return;
  if (tabDocuments.length === 0) {
    fileTabsBar.classList.add('hidden');
    return;
  }

  fileTabsBar.classList.remove('hidden');
  fileTabsList.innerHTML = tabDocuments.map((doc, idx) => `
    <div class="file-tab${idx === activeTabIndex ? ' active' : ''}" data-index="${idx}">
      <span class="file-tab-name">${escapeHtml(doc.filename)}</span>
      ${doc.dirty ? '<span class="file-tab-dirty">●</span>' : ''}
      <span class="file-tab-close" data-close-index="${idx}" title="关闭标签">&times;</span>
    </div>
  `).join('');

  fileTabsList.querySelectorAll('.file-tab').forEach((el) => {
    el.addEventListener('click', (e) => {
      const closeTarget = e.target.closest('.file-tab-close');
      if (closeTarget) {
        e.stopPropagation();
        const closeIdx = parseInt(closeTarget.getAttribute('data-close-index'), 10);
        closeTab(closeIdx);
        return;
      }
      const idx = parseInt(el.getAttribute('data-index'), 10);
      if (idx !== activeTabIndex) {
        saveCurrentTabState();
        activeTabIndex = idx;
        loadActiveTabState();
        renderTabsBar();
        renderWorkbenchUI();
      }
    });
  });
}

function closeTab(index) {
  if (index < 0 || index >= tabDocuments.length) return;
  if (index === activeTabIndex) {
    tabDocuments.splice(index, 1);
    if (tabDocuments.length === 0) {
      activeTabIndex = 0;
      singleView.classList.add('hidden');
      results.classList.add('hidden');
      dropZone.classList.remove('hidden');
      fileTabsBar.classList.add('hidden');
      updateStatus('就绪');
      return;
    }
    if (activeTabIndex >= tabDocuments.length) {
      activeTabIndex = tabDocuments.length - 1;
    }
  } else {
    saveCurrentTabState();
    tabDocuments.splice(index, 1);
    if (index < activeTabIndex) {
      activeTabIndex--;
    }
  }
  loadActiveTabState();
  renderTabsBar();
  renderWorkbenchUI();
}

btnTabNew?.addEventListener('click', () => {
  const untitledCount = tabDocuments.filter((d) => d.filename.startsWith('untitled')).length + 1;
  const filename = `untitled${untitledCount}.json`;
  checkSingleFile('{\n  \n}', filename, 'json');
});

// ---- Render Single Result / Workbench UI ----
function renderWorkbenchUI() {
  dropZone.classList.add('hidden');
  results.classList.remove('hidden');
  singleView.classList.remove('hidden');
  batchView.classList.add('hidden');

  fileExt.textContent = docState.format.toUpperCase();
  fileName.textContent = docState.filename;
  const byteLen = getByteLength(docState.content);
  fileSize.textContent = formatSize(byteLen);
  updateDirtyIndicator();

  // Profile Badge
  if (docState.profile) {
    fileProfile.textContent = `Profile: ${docState.profile}`;
    fileProfile.classList.remove('hidden');
  } else {
    fileProfile.classList.add('hidden');
  }

  // Large File Protection (v2.0)
  const isLargeFile = byteLen > LARGE_FILE_THRESHOLD;
  if (isLargeFile) {
    fileLargeWarning.classList.remove('hidden');
    tabVisual.disabled = true;
    tabVisual.title = '文件超过 2MB，已自动降级为高性能代码编辑模式以保护界面流畅度';
    tabForm.disabled = true;
    if (activeMode === 'visual' || activeMode === 'form') {
      activeMode = 'code';
    }
  } else {
    fileLargeWarning.classList.add('hidden');
    tabVisual.disabled = false;
    tabVisual.title = '结构化可视化编辑';
    tabForm.disabled = false;
  }

  // Schema Form tab visibility
  if (docState.schema) {
    tabForm.classList.remove('hidden');
  } else {
    tabForm.classList.add('hidden');
    if (activeMode === 'form') {
      activeMode = 'code';
    }
  }

  // Status card & action bar
  updateStatusCard();
  renderErrorList(docState.errors);

  // Setup Code Editor
  codeEditor.value = docState.content;
  updateCodeGutter();
  updateCursorPos();

  // Switch to current active mode
  switchMode(activeMode);
}

function updateDirtyIndicator() {
  if (docState.dirty) {
    fileDirty.classList.remove('hidden');
  } else {
    fileDirty.classList.add('hidden');
  }
}

function updateStatusCard() {
  const profileIssueCount = docState.profileDiagnostics ? docState.profileDiagnostics.length : 0;
  const canSelfFix = !docState.valid || profileIssueCount > 0;

  if (docState.valid) {
    if (profileIssueCount > 0) {
      statusCard.className = 'status-card status-fixable';
      statusIcon.textContent = 'ℹ';
      statusText.textContent = `语法校验通过 · 发现 ${profileIssueCount} 处工程 Profile 优化建议 · ${docState.format.toUpperCase()}`;
      btnQuickFix.classList.remove('hidden');
      btnQuickFix.textContent = '优化建议';
    } else {
      statusCard.className = 'status-card status-ok';
      statusIcon.textContent = '\u2713';
      statusText.textContent = `校验通过 · ${docState.format.toUpperCase()} · ${docState.filename}`;
      btnQuickFix.classList.add('hidden');
    }
  } else {
    const fixable = Boolean(docState.corrected);
    statusCard.className = fixable ? 'status-card status-fixable' : 'status-card status-error';
    statusIcon.textContent = fixable ? '!' : '\u2717';
    statusText.textContent = fixable
      ? `检测到 ${docState.errors.length} 处语法异常（支持自动修复）`
      : `检测到 ${docState.errors.length} 处语法异常`;

    btnQuickFix.classList.remove('hidden');
    btnQuickFix.textContent = '自动修复';
  }
}

function renderErrorList(errors) {
  const hasSyntaxErrors = errors && errors.length > 0;
  const hasProfileDiags = docState.profileDiagnostics && docState.profileDiagnostics.length > 0;

  if (!hasSyntaxErrors && !hasProfileDiags) {
    errorList.classList.add('hidden');
    errorList.innerHTML = '';
    return;
  }
  errorList.classList.remove('hidden');

  let html = '';
  // 1. Syntax / Format Errors
  if (hasSyntaxErrors) {
    html += errors.map((err, idx) => `
      <div class="error-item" data-index="${idx}" data-line="${err.line || ''}">
        <div class="error-location">${formatLocation(err)}${err.line ? ' <button class="btn btn-outline btn-xs btn-jump-line" data-line="' + err.line + '">定位代码</button>' : ''}</div>
        <div class="error-message">${escapeHtml(err.friendly)}</div>
        ${err.near ? `<div class="error-near">${escapeHtml(err.near)}</div>` : ''}
      </div>
    `).join('');
  }

  // 2. Profile Semantic Diagnostics (v1.8)
  if (hasProfileDiags) {
    html += docState.profileDiagnostics.map((diag, idx) => `
      <div class="error-item profile-diag" style="border-left: 3px solid var(--accent); background: rgba(211, 154, 98, 0.08);">
        <div class="error-location">
          <span class="badge" style="background: rgba(211, 154, 98, 0.2); color: var(--accent); border-color: rgba(211, 154, 98, 0.4);">
            Profile 建议 · ${escapeHtml(docState.profile || '规范')}
          </span>
          <span style="margin-left: 8px; font-family: monospace;">${escapeHtml(diag.path || '/')}</span>
        </div>
        <div class="error-message" style="margin-top: 4px; font-weight: 500;">${escapeHtml(diag.message)}</div>
        ${diag.suggestion ? `<div class="error-near" style="color: var(--green); margin-top: 4px;">建议处理: ${escapeHtml(diag.suggestion)}</div>` : ''}
      </div>
    `).join('');
  }

  errorList.innerHTML = html;

  errorList.querySelectorAll('.btn-jump-line').forEach((btn) => {
    btn.addEventListener('click', (e) => {
      e.stopPropagation();
      const line = parseInt(btn.getAttribute('data-line'), 10);
      if (line) {
        jumpToLine(line);
      }
    });
  });
}

function jumpToLine(lineNumber) {
  switchMode('code');
  const lines = codeEditor.value.split('\n');
  let charPos = 0;
  for (let i = 0; i < lineNumber - 1 && i < lines.length; i++) {
    charPos += lines[i].length + 1;
  }
  const lineLength = lines[lineNumber - 1] ? lines[lineNumber - 1].length : 0;
  codeEditor.focus();
  codeEditor.setSelectionRange(charPos, charPos + lineLength);

  const lineHeight = 19.5;
  codeEditor.scrollTop = Math.max(0, (lineNumber - 5) * lineHeight);
}

// ---- Mode Switcher ----
tabCode.addEventListener('click', () => switchMode('code'));
tabVisual.addEventListener('click', () => switchMode('visual'));
tabForm.addEventListener('click', () => switchMode('form'));
tabDiff.addEventListener('click', () => switchMode('diff'));
btnGotoFixSyntax.addEventListener('click', () => switchMode('code'));

function switchMode(mode) {
  activeMode = mode;
  docState.activeMode = mode;

  tabCode.classList.toggle('active', mode === 'code');
  tabVisual.classList.toggle('active', mode === 'visual');
  tabForm.classList.toggle('active', mode === 'form');
  tabDiff.classList.toggle('active', mode === 'diff');

  codePanel.classList.toggle('hidden', mode !== 'code');
  visualPanel.classList.toggle('hidden', mode !== 'visual');
  formPanel.classList.toggle('hidden', mode !== 'form');
  diffPanel.classList.toggle('hidden', mode !== 'diff');
  sourcePreview.classList.add('hidden');

  if (mode === 'code') {
    codeEditor.value = docState.content;
    updateCodeGutter();
    codeEditor.focus();
  } else if (mode === 'visual') {
    renderVisualView();
  } else if (mode === 'form') {
    renderFormView();
  } else if (mode === 'diff') {
    renderDiffView();
  }
}

// ---- Code Editor Logic ----
codeEditor.addEventListener('input', () => {
  docState.content = codeEditor.value;
  docState.dirty = (docState.content !== docState.savedContent);
  updateDirtyIndicator();
  renderTabsBar();
  fileSize.textContent = formatSize(getByteLength(docState.content));
  updateCodeGutter();

  clearTimeout(validationTimer);
  validationTimer = setTimeout(runDebouncedValidation, 300);
});

codeEditor.addEventListener('scroll', () => {
  codeGutter.scrollTop = codeEditor.scrollTop;
});

codeEditor.addEventListener('keyup', updateCursorPos);
codeEditor.addEventListener('click', updateCursorPos);

codeEditor.addEventListener('keydown', (e) => {
  if (e.key === 'Tab') {
    e.preventDefault();
    const start = codeEditor.selectionStart;
    const end = codeEditor.selectionEnd;
    codeEditor.value = codeEditor.value.substring(0, start) + '  ' + codeEditor.value.substring(end);
    codeEditor.selectionStart = codeEditor.selectionEnd = start + 2;
    codeEditor.dispatchEvent(new Event('input'));
  } else if ((e.ctrlKey || e.metaKey) && e.key === 's') {
    e.preventDefault();
    btnDownload.click();
  }
});

function updateCursorPos() {
  const pos = codeEditor.selectionStart;
  const lines = codeEditor.value.substring(0, pos).split('\n');
  const line = lines.length;
  const col = lines[lines.length - 1].length + 1;
  codeCursorPos.textContent = `第 ${line} 行, 第 ${col} 列`;
}

function updateCodeGutter() {
  const lineCount = codeEditor.value.split('\n').length;
  const errorLines = new Set(docState.errors.map((e) => e.line).filter(Boolean));
  let html = '';
  for (let i = 1; i <= lineCount; i++) {
    const isError = errorLines.has(i);
    html += `<div class="gutter-line${isError ? ' error-line' : ''}">${i}</div>`;
  }
  codeGutter.innerHTML = html;
}

async function runDebouncedValidation() {
  try {
    const checkResult = await invoke('cmd_validate_document', {
      content: docState.content,
      format: docState.format,
    });

    docState.valid = checkResult.valid;
    docState.errors = checkResult.errors || [];
    docState.corrected = checkResult.corrected || null;

    if (checkResult.valid) {
      try {
        const tree = await invoke('cmd_parse_document', {
          content: docState.content,
          format: docState.format,
        });
        docState.tree = tree;
        docState.lastValidTree = tree;

        // Detect & Validate Profile
        try {
          const profileId = await invoke('cmd_detect_profile', {
            filename: docState.filename,
            content: docState.content,
            format: docState.format,
            doc: tree,
          });
          docState.profile = profileId || null;
          if (profileId) {
            docState.profileDiagnostics = await invoke('cmd_validate_profile', {
              profileId,
              content: docState.content,
              format: docState.format,
              doc: tree,
            }) || [];
          } else {
            docState.profileDiagnostics = [];
          }
        } catch {
          docState.profile = null;
          docState.profileDiagnostics = [];
        }
      } catch (err) {
        console.warn('解析失败:', err);
      }
    } else {
      docState.profileDiagnostics = [];
    }

    if (docState.profile) {
      fileProfile.textContent = `Profile: ${docState.profile}`;
      fileProfile.classList.remove('hidden');
    } else {
      fileProfile.classList.add('hidden');
    }

    updateStatusCard();
    renderErrorList(docState.errors);
    updateCodeGutter();

    if (activeMode === 'visual') {
      renderVisualView();
    } else if (activeMode === 'form') {
      renderFormView();
    }
  } catch (err) {
    console.error('实时校验出错:', err);
  }
}

// Action Bar: Format Document
btnFormatDoc.addEventListener('click', async () => {
  try {
    const formatted = await invoke('cmd_format_document', {
      content: docState.content,
      format: docState.format,
    });
    pushVisualUndo();
    docState.content = formatted;
    docState.dirty = (docState.content !== docState.savedContent);
    codeEditor.value = formatted;
    updateDirtyIndicator();
    renderTabsBar();
    updateCodeGutter();
    await runDebouncedValidation();
    flashStatus('已完成一键排版');
  } catch (err) {
    flashStatus('排版未完成：' + err);
  }
});

// Action Bar: Quick Fix / Auto-Repair (自修复功能)
btnQuickFix.addEventListener('click', async () => {
  try {
    let repaired = false;
    let fixed = null;

    // 1. Syntax-level auto-repair via backend
    try {
      fixed = await invoke('cmd_simple_fix', {
        content: docState.content,
        format: docState.format,
      });
      if (fixed && fixed !== docState.content) {
        repaired = true;
      }
    } catch (err) {
      console.warn('后端语法自修复未命中:', err);
    }

    // 2. Profile-level semantic repair (e.g. duplicate dependencies in package.json)
    if (!repaired && docState.profile === 'package.json' && docState.tree) {
      try {
        const pkgObj = nodeToJsonValue(docState.tree);
        if (pkgObj && pkgObj.dependencies && pkgObj.devDependencies) {
          let hasDup = false;
          for (const dep of Object.keys(pkgObj.dependencies)) {
            if (pkgObj.devDependencies[dep]) {
              delete pkgObj.devDependencies[dep];
              hasDup = true;
            }
          }
          if (hasDup) {
            fixed = JSON.stringify(pkgObj, null, 2);
            repaired = true;
          }
        }
      } catch (err) {
        console.warn('Profile 自修复未命中:', err);
      }
    }

    if (repaired && fixed) {
      pushVisualUndo();
      docState.content = fixed;
      docState.dirty = (docState.content !== docState.savedContent);
      codeEditor.value = fixed;
      updateDirtyIndicator();
      renderTabsBar();
      updateCodeGutter();
      await runDebouncedValidation();
      flashStatus('已自动修复并校验');
    } else {
      flashStatus('未发现可自动修复的典型异常，请手动调整');
    }
  } catch (err) {
    flashStatus('自动修复失败: ' + err);
  }
});

// Action Bar: Copy Errors
btnCopyErrors.addEventListener('click', () => {
  const allIssues = [];
  if (docState.errors.length > 0) {
    allIssues.push(...docState.errors.map((e) => `${formatLocation(e)}: ${e.friendly}`));
  }
  if (docState.profileDiagnostics && docState.profileDiagnostics.length > 0) {
    allIssues.push(...docState.profileDiagnostics.map((d) => `[Profile: ${d.path || '/'}] ${d.message} (${d.suggestion || ''})`));
  }

  if (allIssues.length === 0) {
    flashStatus('当前无异常需要复制');
    return;
  }
  navigator.clipboard.writeText(allIssues.join('\n')).then(() => flashStatus('已复制所有问题至剪贴板'));
});

// Action Bar: Copy Raw
btnCopyRaw.addEventListener('click', () => {
  navigator.clipboard.writeText(docState.content).then(() => flashStatus('已复制配置文本至剪贴板'));
});

// ---- Secret Masking Utility (v2.0) ----
function isSensitiveKey(key) {
  if (!key) return false;
  return /password|secret|token|api_key|access_key|private_key|credential|auth_token|passwd/i.test(key);
}

// ---- Visual Editor Rendering ----
function renderVisualView() {
  visualFormatBadge.textContent = docState.format.toUpperCase();

  if (!docState.valid) {
    visualFrozenBanner.classList.remove('hidden');
    if (docState.lastValidTree) {
      renderVisualContent(docState.lastValidTree, true);
    } else {
      visualTreeContainer.innerHTML = '<div style="padding:20px;text-align:center;color:var(--text-dim)">暂无可用结构树，请先修正语法错误。</div>';
    }
    return;
  }

  visualFrozenBanner.classList.add('hidden');
  const treeToRender = docState.tree || docState.lastValidTree;
  if (!treeToRender) {
    visualTreeContainer.innerHTML = '<div style="padding:20px;text-align:center;color:var(--text-dim)">正在解析结构树...</div>';
    return;
  }

  renderVisualContent(treeToRender, false);
}

function renderVisualContent(rootNode, isFrozen) {
  const cap = formatMap.get(docState.format)?.capabilities;
  let nodeCount = 0;
  countNodes(rootNode, () => nodeCount++);
  visualNodeCount.textContent = `${nodeCount} 个节点`;

  if (cap?.supportsGridEditor) {
    renderGridEditor(rootNode, isFrozen);
  } else if (cap?.supportsDomEditor) {
    renderDomEditor(rootNode, isFrozen);
  } else if (cap?.supportsKvEditor && !cap?.supportsTreeEditor) {
    renderKvEditor(rootNode, isFrozen);
  } else {
    renderTreeEditor(rootNode, isFrozen);
  }
}

function countNodes(node, cb) {
  cb();
  if (node.children) {
    node.children.forEach((c) => countNodes(c, cb));
  }
}

// 1. Tree Editor
function renderTreeEditor(rootNode, isFrozen) {
  visualTreeContainer.innerHTML = '';
  const treeRootEl = document.createElement('div');
  treeRootEl.className = 'tree-root';
  treeRootEl.appendChild(createTreeNodeElement(rootNode, isFrozen));
  visualTreeContainer.appendChild(treeRootEl);
}

function createTreeNodeElement(node, isFrozen) {
  const el = document.createElement('div');
  el.className = 'tree-node';
  el.dataset.path = node.path;

  const row = document.createElement('div');
  row.className = 'tree-row';

  const isCompound = node.kind === 'object' || node.kind === 'array' || node.kind === 'section';
  const hasChildren = Boolean(node.children && node.children.length > 0);

  // Expander
  const expander = document.createElement('span');
  expander.className = 'tree-expander';
  expander.textContent = isCompound ? '▼' : '•';
  if (isCompound) {
    expander.addEventListener('click', (e) => {
      e.stopPropagation();
      const childrenEl = el.querySelector(':scope > .tree-children');
      if (childrenEl) {
        const isHidden = childrenEl.style.display === 'none';
        childrenEl.style.display = isHidden ? '' : 'none';
        expander.textContent = isHidden ? '▼' : '▶';
      }
    });
  }
  row.appendChild(expander);

  // Key
  if (node.key != null) {
    const keyInput = document.createElement('input');
    keyInput.type = 'text';
    keyInput.className = 'tree-key-input';
    keyInput.value = node.key;
    keyInput.disabled = isFrozen;
    keyInput.addEventListener('change', () => {
      applyVisualPatch({
        type: 'setKey',
        path: node.path,
        newKey: keyInput.value.trim(),
      });
    });
    row.appendChild(keyInput);

    const colon = document.createElement('span');
    colon.className = 'tree-colon';
    colon.textContent = ':';
    row.appendChild(colon);
  }

  // Type Selector
  const typeSelect = document.createElement('select');
  typeSelect.className = 'tree-type-select';
  typeSelect.disabled = isFrozen;
  ['string', 'number', 'boolean', 'null', 'object', 'array'].forEach((t) => {
    const opt = document.createElement('option');
    opt.value = t;
    opt.textContent = t;
    if (t === node.kind) opt.selected = true;
    typeSelect.appendChild(opt);
  });
  typeSelect.addEventListener('change', () => {
    applyVisualPatch({
      type: 'setType',
      path: node.path,
      targetType: typeSelect.value,
    });
  });
  row.appendChild(typeSelect);

  // Primitive Value
  if (!isCompound) {
    if (node.kind === 'boolean') {
      const boolInput = document.createElement('input');
      boolInput.type = 'checkbox';
      boolInput.checked = Boolean(node.value);
      boolInput.disabled = isFrozen;
      boolInput.addEventListener('change', () => {
        applyVisualPatch({
          type: 'setValue',
          path: node.path,
          value: boolInput.checked,
        });
      });
      row.appendChild(boolInput);
    } else if (node.kind !== 'null') {
      const isSecret = isSensitiveKey(node.key);
      const valInput = document.createElement('input');
      valInput.type = isSecret ? 'password' : (node.kind === 'number' ? 'number' : 'text');
      valInput.className = 'tree-value-input';
      valInput.value = node.value != null ? (typeof node.value === 'object' ? JSON.stringify(node.value) : node.value) : '';
      valInput.disabled = isFrozen;
      valInput.addEventListener('change', () => {
        let v = valInput.value;
        if (node.kind === 'number') {
          v = Number(v) || 0;
        }
        applyVisualPatch({
          type: 'setValue',
          path: node.path,
          value: v,
        });
      });
      row.appendChild(valInput);

      if (isSecret) {
        const toggleBtn = document.createElement('button');
        toggleBtn.type = 'button';
        toggleBtn.className = 'btn-secret-toggle';
        toggleBtn.textContent = '👁️';
        toggleBtn.title = '切换明文显示';
        toggleBtn.addEventListener('click', (e) => {
          e.stopPropagation();
          valInput.type = valInput.type === 'password' ? 'text' : 'password';
        });
        row.appendChild(toggleBtn);
      }
    }
  }

  // Node Actions (Add / Remove)
  if (!isFrozen) {
    const actions = document.createElement('div');
    actions.className = 'tree-actions';

    if (isCompound) {
      const btnAdd = document.createElement('button');
      btnAdd.className = 'tree-btn-add';
      btnAdd.textContent = '+';
      btnAdd.title = '添加子项';
      btnAdd.addEventListener('click', (e) => {
        e.stopPropagation();
        applyVisualPatch({
          type: 'addNode',
          parentPath: node.path,
          key: node.kind === 'object' ? 'new_prop' : null,
          value: '',
          kind: 'string',
        });
      });
      actions.appendChild(btnAdd);
    }

    if (node.path !== '/') {
      const btnDel = document.createElement('button');
      btnDel.className = 'tree-btn-delete';
      btnDel.textContent = '×';
      btnDel.title = '删除此项';
      btnDel.addEventListener('click', (e) => {
        e.stopPropagation();
        applyVisualPatch({
          type: 'removeNode',
          path: node.path,
        });
      });
      actions.appendChild(btnDel);
    }

    row.appendChild(actions);
  }

  el.appendChild(row);

  // Children
  if (isCompound && hasChildren) {
    const childrenContainer = document.createElement('div');
    childrenContainer.className = 'tree-children';
    node.children.forEach((child) => {
      childrenContainer.appendChild(createTreeNodeElement(child, isFrozen));
    });
    el.appendChild(childrenContainer);
  }

  return el;
}

// 2. Grid Editor (CSV, TSV)
function renderGridEditor(rootNode, isFrozen) {
  visualTreeContainer.innerHTML = '';
  const rows = rootNode.children || [];
  if (rows.length === 0) {
    visualTreeContainer.innerHTML = '<div style="padding:20px;text-align:center;color:var(--text-dim)">表格无数据。</div>';
    return;
  }

  const tableWrapper = document.createElement('div');
  tableWrapper.className = 'grid-table-container';

  const table = document.createElement('table');
  table.className = 'grid-table';

  // Thead
  const thead = document.createElement('thead');
  const headerTr = document.createElement('tr');
  const rowIdxTh = document.createElement('th');
  rowIdxTh.textContent = '#';
  rowIdxTh.style.width = '40px';
  headerTr.appendChild(rowIdxTh);

  const firstRow = rows[0]?.children || [];
  firstRow.forEach((col, cIdx) => {
    const th = document.createElement('th');
    th.textContent = col.value != null ? String(col.value) : `列 ${cIdx + 1}`;
    headerTr.appendChild(th);
  });
  thead.appendChild(headerTr);
  table.appendChild(thead);

  // Tbody
  const tbody = document.createElement('tbody');
  rows.slice(1).forEach((rNode, rIdx) => {
    const tr = document.createElement('tr');
    const idxTd = document.createElement('td');
    idxTd.textContent = String(rIdx + 1);
    idxTd.style.textAlign = 'center';
    idxTd.style.color = 'var(--text-dim)';
    tr.appendChild(idxTd);

    (rNode.children || []).forEach((cNode) => {
      const td = document.createElement('td');
      const input = document.createElement('input');
      input.type = 'text';
      input.className = 'grid-cell-input';
      input.value = cNode.value != null ? String(cNode.value) : '';
      input.disabled = isFrozen;
      input.addEventListener('change', () => {
        applyVisualPatch({
          type: 'setValue',
          path: cNode.path,
          value: input.value,
        });
      });
      td.appendChild(input);
      tr.appendChild(td);
    });
    tbody.appendChild(tr);
  });

  table.appendChild(tbody);
  tableWrapper.appendChild(table);
  visualTreeContainer.appendChild(tableWrapper);
}

// 3. Key-Value Editor (INI, ENV, Properties, etc.)
function renderKvEditor(rootNode, isFrozen) {
  visualTreeContainer.innerHTML = '';
  const container = document.createElement('div');
  container.className = 'kv-container';

  const sections = rootNode.kind === 'section' ? [rootNode] : (rootNode.children || []);

  sections.forEach((sec) => {
    const card = document.createElement('div');
    card.className = 'kv-section-card';

    const header = document.createElement('div');
    header.className = 'kv-section-header';
    header.textContent = sec.key ? `[${sec.key}]` : '全局默认设置';
    card.appendChild(header);

    const body = document.createElement('div');
    body.className = 'kv-table';

    (sec.children || []).forEach((child) => {
      const row = document.createElement('div');
      row.className = 'kv-row';

      const keyInput = document.createElement('input');
      keyInput.type = 'text';
      keyInput.className = 'tree-key-input kv-key';
      keyInput.value = child.key || '';
      keyInput.disabled = isFrozen;
      keyInput.addEventListener('change', () => {
        applyVisualPatch({
          type: 'setKey',
          path: child.path,
          newKey: keyInput.value.trim(),
        });
      });
      row.appendChild(keyInput);

      const colon = document.createElement('span');
      colon.textContent = '=';
      row.appendChild(colon);

      const isSecret = isSensitiveKey(child.key);
      const valInput = document.createElement('input');
      valInput.type = isSecret ? 'password' : 'text';
      valInput.className = 'tree-value-input kv-val';
      valInput.value = child.value != null ? String(child.value) : '';
      valInput.disabled = isFrozen;
      valInput.addEventListener('change', () => {
        applyVisualPatch({
          type: 'setValue',
          path: child.path,
          value: valInput.value,
        });
      });
      row.appendChild(valInput);

      if (isSecret) {
        const toggleBtn = document.createElement('button');
        toggleBtn.type = 'button';
        toggleBtn.className = 'btn-secret-toggle';
        toggleBtn.textContent = '👁️';
        toggleBtn.title = '切换明文显示';
        toggleBtn.addEventListener('click', (e) => {
          e.stopPropagation();
          valInput.type = valInput.type === 'password' ? 'text' : 'password';
        });
        row.appendChild(toggleBtn);
      }

      body.appendChild(row);
    });

    card.appendChild(body);
    container.appendChild(card);
  });

  visualTreeContainer.appendChild(container);
}

// 4. XML DOM Editor
function renderDomEditor(rootNode, isFrozen) {
  visualTreeContainer.innerHTML = '';
  const layout = document.createElement('div');
  layout.className = 'xml-dom-layout';

  const treePane = document.createElement('div');
  treePane.className = 'xml-tree-pane';

  const inspectorPane = document.createElement('div');
  inspectorPane.className = 'xml-inspector-pane';
  inspectorPane.innerHTML = '<div style="color:var(--text-dim);font-size:12px;">选中左侧节点查看并编辑属性与文本。</div>';

  function renderXmlElement(node) {
    const nodeEl = document.createElement('div');
    nodeEl.className = 'tree-node';

    const row = document.createElement('div');
    row.className = 'tree-row';
    row.style.cursor = 'pointer';

    const tagSpan = document.createElement('span');
    tagSpan.style.color = 'var(--accent)';
    tagSpan.style.fontWeight = 'bold';
    tagSpan.textContent = `<${node.key || 'element'}>`;
    row.appendChild(tagSpan);

    row.addEventListener('click', (e) => {
      e.stopPropagation();
      renderXmlInspector(node, inspectorPane, isFrozen);
    });

    nodeEl.appendChild(row);

    if (node.children && node.children.length > 0) {
      const childWrap = document.createElement('div');
      childWrap.className = 'tree-children';
      node.children.forEach((c) => childWrap.appendChild(renderXmlElement(c)));
      nodeEl.appendChild(childWrap);
    }
    return nodeEl;
  }

  treePane.appendChild(renderXmlElement(rootNode));
  layout.appendChild(treePane);
  layout.appendChild(inspectorPane);
  visualTreeContainer.appendChild(layout);
}

function renderXmlInspector(node, container, isFrozen) {
  const isSecret = isSensitiveKey(node.key);
  container.innerHTML = `
    <h4 style="font-size:13px;color:var(--accent);">元素: &lt;${escapeHtml(node.key)}&gt;</h4>
    <div style="font-size:11px;color:var(--text-dim);">路径: ${escapeHtml(node.path)}</div>
    <div style="margin-top:8px;">
      <label style="font-size:11px;color:var(--text-dim);">文本内容:</label>
      <div style="display:flex;gap:4px;align-items:flex-start;margin-top:4px;">
        <textarea id="xml-node-text" class="code-editor" style="height:60px;background:var(--bg);border:1px solid var(--border);border-radius:4px;padding:6px;flex:1;">${escapeHtml(node.value || '')}</textarea>
        ${isSecret ? '<button type="button" id="btn-xml-secret-toggle" class="btn-secret-toggle">👁️</button>' : ''}
      </div>
    </div>
  `;
  const textInput = container.querySelector('#xml-node-text');
  textInput.disabled = isFrozen;
  textInput.addEventListener('change', () => {
    applyVisualPatch({
      type: 'setValue',
      path: node.path,
      value: textInput.value,
    });
  });
}

// ---- Apply Visual Patch ----
async function applyVisualPatch(patch) {
  pushVisualUndo();
  try {
    const result = await invoke('cmd_apply_patch', {
      content: docState.content,
      format: docState.format,
      patch,
    });

    docState.content = result.content;
    docState.tree = result.node;
    docState.lastValidTree = result.node;
    docState.valid = result.valid;
    docState.errors = result.errors || [];
    docState.dirty = (docState.content !== docState.savedContent);

    codeEditor.value = result.content;
    updateDirtyIndicator();
    renderTabsBar();
    updateStatusCard();
    renderErrorList(docState.errors);
    updateCodeGutter();

    renderVisualView();
  } catch (err) {
    flashStatus('操作未完成: ' + err);
  }
}

// ---- Visual Undo / Redo ----
function pushVisualUndo() {
  visualUndoStack.push({
    content: docState.content,
    tree: JSON.parse(JSON.stringify(docState.tree || {})),
  });
  if (visualUndoStack.length > 50) visualUndoStack.shift();
  visualRedoStack = [];
  updateUndoRedoButtons();
}

btnVisualUndo.addEventListener('click', async () => {
  if (visualUndoStack.length === 0) return;
  const prev = visualUndoStack.pop();
  visualRedoStack.push({
    content: docState.content,
    tree: JSON.parse(JSON.stringify(docState.tree || {})),
  });
  updateUndoRedoButtons();

  docState.content = prev.content;
  docState.tree = prev.tree;
  docState.lastValidTree = prev.tree;
  docState.dirty = (docState.content !== docState.savedContent);
  codeEditor.value = prev.content;
  updateDirtyIndicator();
  renderTabsBar();
  updateCodeGutter();
  await runDebouncedValidation();
  renderVisualView();
});

btnVisualRedo.addEventListener('click', async () => {
  if (visualRedoStack.length === 0) return;
  const next = visualRedoStack.pop();
  visualUndoStack.push({
    content: docState.content,
    tree: JSON.parse(JSON.stringify(docState.tree || {})),
  });
  updateUndoRedoButtons();

  docState.content = next.content;
  docState.tree = next.tree;
  docState.lastValidTree = next.tree;
  docState.dirty = (docState.content !== docState.savedContent);
  codeEditor.value = next.content;
  updateDirtyIndicator();
  renderTabsBar();
  updateCodeGutter();
  await runDebouncedValidation();
  renderVisualView();
});

function updateUndoRedoButtons() {
  btnVisualUndo.disabled = visualUndoStack.length === 0;
  btnVisualRedo.disabled = visualRedoStack.length === 0;
}

btnVisualExpandAll.addEventListener('click', () => {
  visualTreeContainer.querySelectorAll('.tree-children').forEach((c) => { c.style.display = ''; });
  visualTreeContainer.querySelectorAll('.tree-expander').forEach((e) => {
    if (e.textContent !== '•') e.textContent = '▼';
  });
});

btnVisualCollapseAll.addEventListener('click', () => {
  visualTreeContainer.querySelectorAll('.tree-children').forEach((c) => { c.style.display = 'none'; });
  visualTreeContainer.querySelectorAll('.tree-expander').forEach((e) => {
    if (e.textContent !== '•') e.textContent = '▶';
  });
});

btnVisualAddRoot.addEventListener('click', () => {
  if (!docState.tree) return;
  applyVisualPatch({
    type: 'addNode',
    parentPath: '/',
    key: docState.tree.kind === 'object' ? 'new_field' : null,
    value: '',
    kind: 'string',
  });
});

// ---- Schema Form Dynamic Editor (v1.9) ----
btnFormClearSchema?.addEventListener('click', () => {
  docState.schema = null;
  tabForm.classList.add('hidden');
  switchMode('code');
  flashStatus('已退出 Schema 表单模式');
});

function nodeToJsonValue(node) {
  if (!node) return null;
  if (node.kind === 'object' || node.kind === 'section') {
    const obj = {};
    if (node.children) {
      for (const child of node.children) {
        if (child.key != null) {
          obj[child.key] = nodeToJsonValue(child);
        }
      }
    }
    return obj;
  }
  if (node.kind === 'array') {
    const arr = [];
    if (node.children) {
      for (const child of node.children) {
        arr.push(nodeToJsonValue(child));
      }
    }
    return arr;
  }
  return node.value;
}

function renderFormView() {
  if (!schemaFormContainer) return;
  schemaFormContainer.innerHTML = '';

  if (!docState.schema) {
    schemaFormContainer.innerHTML = '<div style="padding:20px;text-align:center;color:var(--text-dim);">未加载 JSON Schema 规范。可在操作栏点击“Schema 校验”提供定义。</div>';
    return;
  }

  let currentData = {};
  try {
    if (docState.tree) {
      currentData = nodeToJsonValue(docState.tree) || {};
    } else {
      currentData = JSON.parse(docState.content || '{}');
    }
  } catch {
    currentData = {};
  }

  const formRootCard = document.createElement('div');
  formRootCard.className = 'form-card';

  const title = document.createElement('div');
  title.className = 'form-card-title';
  title.textContent = docState.schema.title || `${docState.filename} 结构表单`;
  formRootCard.appendChild(title);

  if (docState.schema.description) {
    const desc = document.createElement('div');
    desc.className = 'form-desc';
    desc.textContent = docState.schema.description;
    formRootCard.appendChild(desc);
  }

  const props = docState.schema.properties || {};
  const requiredFields = new Set(docState.schema.required || []);

  Object.entries(props).forEach(([propKey, propDef]) => {
    const group = document.createElement('div');
    group.className = 'form-group';

    const label = document.createElement('label');
    label.className = 'form-label';
    label.innerHTML = `${escapeHtml(propDef.title || propKey)} ${requiredFields.has(propKey) ? '<span style="color:var(--red);">*</span>' : ''} <span style="font-size:10px;color:var(--text-dim);font-family:monospace;">(${escapeHtml(propKey)})</span>`;
    group.appendChild(label);

    if (propDef.description) {
      const pDesc = document.createElement('span');
      pDesc.className = 'form-desc';
      pDesc.textContent = propDef.description;
      group.appendChild(pDesc);
    }

    const curVal = currentData[propKey];

    // 1. Enum
    if (Array.isArray(propDef.enum)) {
      const select = document.createElement('select');
      select.className = 'form-select';
      propDef.enum.forEach((optVal) => {
        const opt = document.createElement('option');
        opt.value = optVal;
        opt.textContent = String(optVal);
        if (curVal === optVal) opt.selected = true;
        select.appendChild(opt);
      });
      select.addEventListener('change', () => {
        currentData[propKey] = select.value;
        onFormDataUpdated(currentData);
      });
      group.appendChild(select);
    }
    // 2. Boolean
    else if (propDef.type === 'boolean') {
      const row = document.createElement('div');
      row.className = 'form-checkbox-row';
      const chk = document.createElement('input');
      chk.type = 'checkbox';
      chk.checked = Boolean(curVal);
      chk.addEventListener('change', () => {
        currentData[propKey] = chk.checked;
        onFormDataUpdated(currentData);
      });
      row.appendChild(chk);
      const span = document.createElement('span');
      span.textContent = '启用 / 开启';
      span.style.fontSize = '12px';
      row.appendChild(span);
      group.appendChild(row);
    }
    // 3. Integer / Number
    else if (propDef.type === 'integer' || propDef.type === 'number') {
      const numInput = document.createElement('input');
      numInput.type = 'number';
      numInput.className = 'form-input';
      numInput.value = curVal != null ? curVal : '';
      numInput.addEventListener('change', () => {
        currentData[propKey] = Number(numInput.value) || 0;
        onFormDataUpdated(currentData);
      });
      group.appendChild(numInput);
    }
    // 4. String or other
    else {
      const isSecret = isSensitiveKey(propKey);
      const strInput = document.createElement('input');
      strInput.type = isSecret ? 'password' : 'text';
      strInput.className = 'form-input';
      strInput.value = curVal != null ? String(curVal) : '';
      strInput.addEventListener('change', () => {
        currentData[propKey] = strInput.value;
        onFormDataUpdated(currentData);
      });

      if (isSecret) {
        const wrap = document.createElement('div');
        wrap.style.display = 'flex';
        wrap.style.gap = '6px';
        wrap.appendChild(strInput);
        const toggleBtn = document.createElement('button');
        toggleBtn.type = 'button';
        toggleBtn.className = 'btn-secret-toggle';
        toggleBtn.textContent = '👁️';
        toggleBtn.title = '切换明文显示';
        toggleBtn.addEventListener('click', () => {
          strInput.type = strInput.type === 'password' ? 'text' : 'password';
        });
        wrap.appendChild(toggleBtn);
        group.appendChild(wrap);
      } else {
        group.appendChild(strInput);
      }
    }

    formRootCard.appendChild(group);
  });

  schemaFormContainer.appendChild(formRootCard);
}

function onFormDataUpdated(dataObj) {
  pushVisualUndo();
  try {
    let serialized = '';
    if (docState.format === 'json' || docState.format === 'jsonc' || docState.format === 'json5') {
      serialized = JSON.stringify(dataObj, null, 2);
    } else {
      serialized = JSON.stringify(dataObj, null, 2);
    }
    docState.content = serialized;
    docState.dirty = (docState.content !== docState.savedContent);
    codeEditor.value = serialized;
    updateDirtyIndicator();
    renderTabsBar();
    updateCodeGutter();
    runDebouncedValidation();
  } catch (err) {
    console.warn('表单数据同步失败:', err);
  }
}

// ---- Diff View (Text Diff + Semantic Diff) ----
btnDiffModeText?.addEventListener('click', () => {
  diffMode = 'text';
  btnDiffModeText.classList.add('active');
  btnDiffModeSemantic.classList.remove('active');
  diffTextBody.classList.remove('hidden');
  diffSemanticBody.classList.add('hidden');
  renderDiffView();
});

btnDiffModeSemantic?.addEventListener('click', () => {
  diffMode = 'semantic';
  btnDiffModeSemantic.classList.add('active');
  btnDiffModeText.classList.remove('active');
  diffTextBody.classList.add('hidden');
  diffSemanticBody.classList.remove('hidden');
  renderDiffView();
});

async function renderDiffView() {
  const original = docState.savedContent || '';
  const current = docState.content || '';

  if (diffMode === 'text') {
    diffTextBody.classList.remove('hidden');
    diffSemanticBody.classList.add('hidden');

    const beforeLines = original.split('\n');
    const afterLines = current.split('\n');

    if (beforeLines.length > MAX_RENDER_LINES || afterLines.length > MAX_RENDER_LINES) {
      diffBefore.textContent = original;
      diffAfter.textContent = current;
      return;
    }

    const { beforeMatches, afterMatches } = computeLCSMatchSets(beforeLines, afterLines);

    diffBefore.innerHTML = beforeLines.map((line, i) => {
      const isCommon = beforeMatches.has(i);
      const cls = isCommon ? 'diff-line diff-unchanged' : 'diff-line diff-removed';
      return `<div class="${cls}"><span class="diff-ln">${i + 1}</span><span class="diff-text">${escapeHtml(line)}</span></div>`;
    }).join('');

    diffAfter.innerHTML = afterLines.map((line, i) => {
      const isCommon = afterMatches.has(i);
      const cls = isCommon ? 'diff-line diff-unchanged' : 'diff-line diff-added';
      return `<div class="${cls}"><span class="diff-ln">${i + 1}</span><span class="diff-text">${escapeHtml(line)}</span></div>`;
    }).join('');
  } else {
    // Semantic Diff (v2.0)
    diffTextBody.classList.add('hidden');
    diffSemanticBody.classList.remove('hidden');

    try {
      const diffResult = await invoke('cmd_compute_semantic_diff', {
        oldContent: original,
        newContent: current,
        format: docState.format,
      });

      diffSemanticSummary.innerHTML = `
        <span>结构化对比:</span>
        <span class="semantic-tag mod">${diffResult.modified_count} 项修改</span>
        <span class="semantic-tag add">${diffResult.added_count} 项新增</span>
        <span class="semantic-tag del">${diffResult.removed_count} 项删除</span>
        <span style="color:var(--text-dim);font-size:11px;">(共 ${diffResult.items.length} 处变动)</span>
      `;

      if (diffResult.items.length === 0) {
        diffSemanticList.innerHTML = '<div style="padding:20px;text-align:center;color:var(--text-dim)">未检测到结构化差异（与已保存版本一致）。</div>';
      } else {
        diffSemanticList.innerHTML = diffResult.items.map((item) => {
          const changeType = item.changeType || item.change_type || item.diff_type || 'modified';
          let cardType = 'mod';
          let tagText = '修改';
          if (changeType === 'added') {
            cardType = 'add';
            tagText = '新增';
          } else if (changeType === 'removed') {
            cardType = 'del';
            tagText = '删除';
          }

          let valuesHtml = '';
          if (cardType === 'mod') {
            valuesHtml = `
              <span class="diff-val-old">${escapeHtml(item.old_value != null ? item.old_value : 'null')}</span>
              <span class="diff-val-arrow">→</span>
              <span class="diff-val-new">${escapeHtml(item.new_value != null ? item.new_value : 'null')}</span>
            `;
          } else if (cardType === 'add') {
            valuesHtml = `<span class="diff-val-new">+ ${escapeHtml(item.new_value != null ? item.new_value : 'null')}</span>`;
          } else {
            valuesHtml = `<span class="diff-val-old">- ${escapeHtml(item.old_value != null ? item.old_value : 'null')}</span>`;
          }

          return `
            <div class="semantic-diff-card ${cardType}">
              <div class="semantic-card-header">
                <span class="semantic-path">${escapeHtml(item.path)}</span>
                <span class="semantic-tag ${cardType}">${tagText}</span>
              </div>
              <div class="semantic-card-values">${valuesHtml}</div>
            </div>
          `;
        }).join('');
      }
    } catch (err) {
      diffSemanticSummary.textContent = '计算语义差异失败：' + err;
      diffSemanticList.innerHTML = '';
    }
  }
}

// ---- Batch View Logic ----
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
    updateStatus('批量校验失败：' + err, 'error');
  }
}

function renderBatchUI(items) {
  dropZone.classList.add('hidden');
  results.classList.remove('hidden');
  singleView.classList.add('hidden');
  batchView.classList.remove('hidden');

  batchBody.innerHTML = items.map((item, idx) => {
    const { filename, result } = item;
    const ok = result.valid;
    const fixable = !ok && Boolean(result.corrected);
    const statusBadge = ok
      ? '<span class="status-badge badge-ok">通过</span>'
      : fixable
        ? `<span class="status-badge badge-fixable">${result.errors.length} 处异常·可修复</span>`
        : `<span class="status-badge badge-error">${result.errors.length} 处异常</span>`;

    return `
      <tr data-index="${idx}">
        <td><strong>${escapeHtml(filename)}</strong></td>
        <td><span class="file-ext">${escapeHtml(result.format.toUpperCase())}</span></td>
        <td>${statusBadge}</td>
        <td>
          <button class="btn btn-outline btn-xs btn-open-workbench" data-index="${idx}">打开工作台</button>
          ${!ok ? `<button class="btn btn-outline btn-xs btn-batch-view-errors" data-index="${idx}">详情</button>` : ''}
          ${fixable ? `<button class="btn btn-primary btn-xs btn-batch-save-one" data-index="${idx}">保存修正</button>` : ''}
        </td>
      </tr>
    `;
  }).join('');

  batchBody.querySelectorAll('.btn-open-workbench').forEach((btn) => {
    btn.addEventListener('click', (e) => {
      e.stopPropagation();
      const idx = parseInt(btn.getAttribute('data-index'), 10);
      const item = items[idx];
      if (item) {
        checkSingleFile(currentFiles[item.filename], item.filename, item.result.format);
      }
    });
  });

  batchBody.querySelectorAll('.btn-batch-view-errors').forEach((btn) => {
    btn.addEventListener('click', (e) => {
      e.stopPropagation();
      const idx = parseInt(btn.getAttribute('data-index'), 10);
      const item = items[idx];
      showBatchDetail(item.filename, item.result.errors);
    });
  });

  batchBody.querySelectorAll('.btn-batch-save-one').forEach((btn) => {
    btn.addEventListener('click', (e) => {
      e.stopPropagation();
      const idx = parseInt(btn.getAttribute('data-index'), 10);
      const item = items[idx];
      openDownloadModal(suggestFixedFilename(item.filename), item.result.corrected);
    });
  });
}

function showBatchDetail(filename, errors) {
  batchDetailName.textContent = filename + ' 的异常详情';
  batchDetailErrors.innerHTML = errors.map((err) => `
    <div class="error-item">
      <div class="error-location">${formatLocation(err)}</div>
      <div class="error-message">${escapeHtml(err.friendly)}</div>
      ${err.near ? `<div class="error-near">${escapeHtml(err.near)}</div>` : ''}
    </div>
  `).join('');
  batchDetail.classList.remove('hidden');
}

btnBatchDetailClose.addEventListener('click', () => {
  batchDetail.classList.add('hidden');
});

btnBatchCopy.addEventListener('click', () => {
  const lines = [];
  currentResults.forEach((item) => {
    if (!item.result.valid) {
      lines.push(`--- ${item.filename} ---`);
      item.result.errors.forEach((e) => {
        lines.push(`  ${formatLocation(e)}: ${e.friendly}`);
      });
    }
  });
  if (lines.length === 0) {
    flashStatus('所有文件校验均通过');
    return;
  }
  navigator.clipboard.writeText(lines.join('\n')).then(() => flashStatus('已复制所有文件异常信息'));
});

btnBatchDownload.addEventListener('click', async () => {
  const fixableItems = currentResults.filter((item) => !item.result.valid && item.result.corrected);
  if (fixableItems.length === 0) {
    flashStatus('暂无可自动保存的修正文件');
    return;
  }
  let savedCount = 0;
  for (const item of fixableItems) {
    try {
      await invoke('cmd_save_file', {
        filename: suggestFixedFilename(item.filename),
        content: item.result.corrected,
        directory: null,
      });
      savedCount++;
    } catch (err) {
      console.error(err);
    }
  }
  flashStatus(`已批量保存 ${savedCount} 个修正文件至桌面`);
});

btnBatchClear.addEventListener('click', () => {
  currentFiles = {};
  currentResults = [];
  batchView.classList.add('hidden');
  results.classList.add('hidden');
  dropZone.classList.remove('hidden');
  updateStatus('就绪');
});

// Close Button (Single view)
btnClose.addEventListener('click', () => {
  if (tabDocuments.length > 0 && activeTabIndex >= 0) {
    closeTab(activeTabIndex);
  } else {
    singleView.classList.add('hidden');
    results.classList.add('hidden');
    dropZone.classList.remove('hidden');
    fileTabsBar.classList.add('hidden');
    updateStatus('就绪');
  }
});

function suggestFixedFilename(original) {
  const dot = original.lastIndexOf('.');
  if (dot === -1) return original + '.fixed';
  return original.substring(0, dot) + '.fixed' + original.substring(dot);
}

function makeUniqueFileEntries(files) {
  const seen = new Map();
  return files.map(([name, content]) => {
    const count = seen.get(name) || 0;
    seen.set(name, count + 1);
    if (count === 0) {
      return [name, content];
    }
    const dot = name.lastIndexOf('.');
    const uniqueName = dot === -1
      ? `${name} (${count})`
      : `${name.substring(0, dot)} (${count})${name.substring(dot)}`;
    return [uniqueName, content];
  });
}

// ---- About Modal ----
btnAbout.addEventListener('click', () => {
  aboutModal.classList.remove('hidden');
});

btnAboutClose.addEventListener('click', () => {
  aboutModal.classList.add('hidden');
});

aboutOverlay.addEventListener('click', () => {
  aboutModal.classList.add('hidden');
});

// ---- Download / Save Modal ----
btnDownload.addEventListener('click', () => {
  const targetContent = docState.content;
  openDownloadModal(docState.filename, targetContent);
});

function openDownloadModal(suggestedName, content) {
  downloadFilename.value = suggestedName;
  downloadModal.dataset.content = content;
  downloadModal.classList.remove('hidden');
  downloadFilename.focus();
}

btnDownloadCancel.addEventListener('click', () => {
  downloadModal.classList.add('hidden');
  delete downloadModal.dataset.content;
});

downloadOverlay.addEventListener('click', () => {
  downloadModal.classList.add('hidden');
  delete downloadModal.dataset.content;
});

btnBrowse.addEventListener('click', async () => {
  try {
    const desktop = await invoke('cmd_get_desktop');
    downloadPath.value = desktop;
  } catch (err) {
    console.warn(err);
  }
});

btnDownloadConfirm.addEventListener('click', async () => {
  const filename = downloadFilename.value.trim();
  const dir = downloadPath.value.trim() || null;
  const content = downloadModal.dataset.content;

  if (!filename) {
    flashStatus('请输入保存的文件名');
    return;
  }

  try {
    const savedPath = await invoke('cmd_save_file', {
      filename,
      content,
      directory: dir,
    });
    downloadModal.classList.add('hidden');
    delete downloadModal.dataset.content;

    docState.savedContent = docState.content;
    docState.dirty = false;
    updateDirtyIndicator();
    renderTabsBar();

    flashStatus(`文件已保存至：${savedPath}`);
  } catch (err) {
    flashStatus('保存失败：' + err);
  }
});

// ---- Schema Modal ----
btnSchema.addEventListener('click', () => {
  schemaModal.classList.remove('hidden');
  schemaResult.classList.add('hidden');
  schemaInput.value = '';
  schemaInput.focus();
});

btnSchemaCancel.addEventListener('click', () => {
  schemaModal.classList.add('hidden');
  schemaInput.value = '';
});

schemaOverlay.addEventListener('click', () => {
  schemaModal.classList.add('hidden');
  schemaInput.value = '';
});

btnSchemaConfirm.addEventListener('click', async () => {
  const schemaStr = schemaInput.value.trim();
  if (!schemaStr) {
    schemaResult.className = 'schema-result schema-error';
    schemaResult.classList.remove('hidden');
    schemaResult.textContent = '请输入 JSON Schema 规范内容';
    return;
  }

  let schemaObj = null;
  try {
    schemaObj = JSON.parse(schemaStr);
  } catch (parseErr) {
    schemaResult.className = 'schema-result schema-error';
    schemaResult.classList.remove('hidden');
    schemaResult.textContent = 'Schema 内容非有效 JSON：' + parseErr.message;
    return;
  }

  try {
    const result = await invoke('cmd_check_schema', {
      content: docState.content,
      schema: schemaStr,
    });

    schemaResult.classList.remove('hidden');
    if (result.valid) {
      schemaResult.className = 'schema-result schema-ok';
      schemaResult.textContent = 'Schema 校验通过！结构符合规范，已激活动态表单模式。';
      docState.schema = schemaObj;
      tabForm.classList.remove('hidden');
      setTimeout(() => {
        schemaModal.classList.add('hidden');
        switchMode('form');
      }, 1000);
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
    schemaResult.textContent = 'Schema 校验未完成：' + err;
  }
});

// ---- Helper Functions ----
function updateStatus(msg, type) {
  statusBar.textContent = msg;
  statusBar.style.color = type === 'error' ? 'var(--red)' : '';
}

function flashStatus(msg) {
  const original = statusBar.textContent;
  statusBar.textContent = msg;
  statusBar.style.color = 'var(--green)';
  setTimeout(() => {
    statusBar.textContent = original;
    statusBar.style.color = '';
  }, 2200);
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
  return parts.length > 0 ? parts.join('，') : '未知位置';
}

function escapeHtml(text) {
  if (text == null) return '';
  return String(text)
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;');
}

// Start app
initApp();
