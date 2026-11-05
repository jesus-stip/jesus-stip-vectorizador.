/// <reference lib="webworker" />
import { engine, readResult, withPixels } from './engine';
import { MAX_SOURCE_PIXELS, WORK_PIXELS, type TraceRequest, type WorkerMessage } from './protocol';

const post = (m: WorkerMessage) => (self as unknown as DedicatedWorkerGlobalScope).postMessage(m);

engine().catch(() => { /* reported on the first request */ });

self.onmessage = async (ev: MessageEvent<TraceRequest>) => {
  const { id, file, detail } = ev.data;
  try {
    post({ type: 'stage', id, stage: 'decode' });
    const bmp = await createImageBitmap(file); // first frame for GIFs
    const { width, height } = bmp;
    if (width * height > MAX_SOURCE_PIXELS) throw new Error('La imagen es demasiado grande (más de 50 megapíxeles).');
    const canvas = new OffscreenCanvas(width, height);
    const ctx = canvas.getContext('2d', { willReadFrequently: true });
    if (!ctx) throw new Error('Este navegador no puede leer la imagen.');
    ctx.drawImage(bmp, 0, 0);
    bmp.close();
    const { data } = ctx.getImageData(0, 0, width, height);

    post({ type: 'stage', id, stage: 'colors' });
    const colors = await withPixels(data, (e, ptr) => e.suggest_colors(ptr, width, height));

    post({ type: 'stage', id, stage: 'trace' });
    const cap = WORK_PIXELS[detail];
    const t0 = performance.now();
    const svg = await withPixels(data, (e, ptr) => {
      const rc = e.trace(ptr, width, height, colors, detail, cap);
      const text = readResult(e);
      if (rc !== 0) throw new Error(text);
      return text;
    });
    post({
      type: 'done', id, svg, colors, width, height,
      ms: Math.round(performance.now() - t0),
      paths: (svg.match(/<path /g) ?? []).length,
      downscaled: width * height > cap,
    });
  } catch (err) {
    const msg = err instanceof Error ? err.message : String(err);
    post({ type: 'error', id, message: msg.startsWith('no se pudo cargar el motor') || err instanceof WebAssembly.CompileError
      ? `No se pudo cargar el motor de vectorización: ${msg}` : msg });
  }
};
