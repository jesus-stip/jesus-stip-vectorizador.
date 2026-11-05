import './styles/tokens.css';
import './styles/app.css';
import type { Detail, TraceRequest, WorkerMessage } from './protocol';

const MAX_BYTES = 25 * 1024 * 1024;
const ACCEPTED = ['image/png', 'image/jpeg', 'image/bmp', 'image/gif'];

const $ = <T extends HTMLElement>(id: string) => document.getElementById(id) as T;
const s01 = $('s01'), s02 = $('s02'), s03 = $('s03');
const drop = $<HTMLLabelElement>('drop');
const fileInput = $<HTMLInputElement>('file');
const s01Error = $('s01-error');
const thumb = $<HTMLImageElement>('s02-thumb');
const stage = $('s02-stage');
const origImg = $<HTMLImageElement>('s03-orig');
const vecImg = $<HTMLImageElement>('s03-vec');

const STAGES: Record<'decode' | 'colors' | 'trace', string> = {
  decode: 'Leyendo la imagen…',
  colors: 'Buscando los colores principales…',
  trace: 'Trazando las formas… (las imágenes grandes tardan unos segundos)',
};

let worker = spawn();
let jobId = 0;
let urls: string[] = [];
let current: { name: string; svg: string } | null = null;

/** If the worker sends nothing at all for this long, it never started. */
const START_TIMEOUT_MS = 15_000;
let watchdog = 0;

function spawn(): Worker | null {
  try {
    const w = new Worker(new URL('./worker.ts', import.meta.url), { type: 'module' });
    w.onmessage = (ev: MessageEvent<WorkerMessage>) => onWorker(ev.data);
    w.onerror = (e) => fail(`El motor no pudo arrancar${e.message ? ` (${e.message})` : ''}. Prueba con Chrome, Edge o Firefox actualizados.`);
    return w;
  } catch (e) {
    showError(`Este navegador no puede ejecutar el motor (${e instanceof Error ? e.message : e}). Usa Chrome, Edge o Firefox actualizados.`);
    return null;
  }
}

function restartWorker() { clearTimeout(watchdog); worker?.terminate(); worker = spawn(); }

function show(section: HTMLElement) {
  for (const s of [s01, s02, s03]) s.hidden = s !== section;
}

function revokeAll() { urls.forEach(URL.revokeObjectURL); urls = []; }
function objectUrl(b: Blob) { const u = URL.createObjectURL(b); urls.push(u); return u; }

function showError(message: string) {
  show(s01);
  s01Error.textContent = message;
  s01Error.hidden = false;
  fileInput.value = '';
}

function fail(message: string) {
  restartWorker();
  showError(message);
}

function start(file: File) {
  s01Error.hidden = true;
  if (!ACCEPTED.includes(file.type)) return fail('Ese formato no se puede usar. Prueba con PNG, JPG, BMP o GIF.');
  if (file.size > MAX_BYTES) return fail('El archivo pesa más de 25 MB.');
  revokeAll();
  current = { name: file.name.replace(/\.[^.]+$/, '') || 'imagen', svg: '' };
  const src = objectUrl(file);
  thumb.src = src;
  origImg.src = src;
  stage.textContent = STAGES.decode;
  show(s02);
  $<HTMLButtonElement>('s02-cancel').focus();
  if (!worker) return fail('Este navegador no puede ejecutar el motor. Usa Chrome, Edge o Firefox actualizados.');
  const req: TraceRequest = { type: 'trace', id: ++jobId, file, detail: 1 as Detail };
  worker.postMessage(req);
  clearTimeout(watchdog);
  watchdog = window.setTimeout(() => fail('El motor no respondió. Puede que el servidor no esté entregando los archivos de assets/ (revisa que se hayan subido completos).'), START_TIMEOUT_MS);
}

function onWorker(m: WorkerMessage) {
  if (m.id !== jobId) return; // a cancelled job finishing late
  clearTimeout(watchdog);
  if (m.type === 'stage') { stage.textContent = STAGES[m.stage]; return; }
  if (m.type === 'error') return fail(m.message || 'No se pudo vectorizar esta imagen.');
  current!.svg = m.svg;
  vecImg.src = objectUrl(new Blob([m.svg], { type: 'image/svg+xml' }));
  $('s03-title').textContent = current!.name;
  const n = new Intl.NumberFormat('es');
  $('s03-stats').textContent =
    `${m.width} × ${m.height} px · ${m.colors} colores · ${n.format(m.paths)} formas · ${(m.ms / 1000).toFixed(1)} s` +
    (m.downscaled ? ' · se trabajó a tamaño reducido' : '');
  selectTab('vec');
  show(s03);
  $<HTMLButtonElement>('s05-svg').focus();
}

function download(blob: Blob, name: string) {
  const a = document.createElement('a');
  a.href = URL.createObjectURL(blob);
  a.download = name;
  document.body.append(a);
  a.click();
  a.remove();
  setTimeout(() => URL.revokeObjectURL(a.href), 10_000);
}

function selectTab(which: 'vec' | 'orig') {
  for (const k of ['vec', 'orig'] as const) {
    const on = k === which;
    $(`tab-${k}`).setAttribute('aria-selected', String(on));
    const pane = $(`pane-${k}`);
    if (on) pane.removeAttribute('hidden-on-mobile'); else pane.setAttribute('hidden-on-mobile', '');
  }
}

// S01: pick or drop
fileInput.addEventListener('change', () => { const f = fileInput.files?.[0]; if (f) start(f); });
for (const t of ['dragenter', 'dragover'] as const)
  drop.addEventListener(t, (e) => { e.preventDefault(); drop.classList.add('drop--over'); });
for (const t of ['dragleave', 'drop'] as const)
  drop.addEventListener(t, () => drop.classList.remove('drop--over'));
drop.addEventListener('drop', (e) => { e.preventDefault(); const f = e.dataTransfer?.files[0]; if (f) start(f); });
// dropping outside the box should not navigate away from the app
window.addEventListener('dragover', (e) => e.preventDefault());
window.addEventListener('drop', (e) => e.preventDefault());
// paste an image from the clipboard
window.addEventListener('paste', (e) => {
  const f = [...(e.clipboardData?.files ?? [])].find((x) => ACCEPTED.includes(x.type));
  if (f && !s01.hidden) start(f);
});

// S02: cancel
$('s02-cancel').addEventListener('click', () => { jobId++; restartWorker(); show(s01); fileInput.value = ''; });

// S03 / S05
$('s05-svg').addEventListener('click', () => {
  if (current?.svg) download(new Blob([current.svg], { type: 'image/svg+xml' }), `${current.name}.svg`);
});
$('s03-new').addEventListener('click', () => { show(s01); fileInput.value = ''; fileInput.focus(); });
$('tab-vec').addEventListener('click', () => selectTab('vec'));
$('tab-orig').addEventListener('click', () => selectTab('orig'));

(window as unknown as { __vzReady: boolean }).__vzReady = true;
