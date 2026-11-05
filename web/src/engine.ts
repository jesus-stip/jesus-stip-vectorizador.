// Thin wrapper over the C ABI exported by engine/ (Rust + VTracer). Runs inside the worker.
import wasmUrl from './engine/vectorizer.wasm?url';

interface Exports {
  memory: WebAssembly.Memory;
  alloc(len: number): number;
  dealloc(ptr: number, len: number): void;
  result_ptr(): number;
  result_len(): number;
  suggest_colors(ptr: number, w: number, h: number): number;
  trace(ptr: number, w: number, h: number, colors: number, detail: number, maxPixels: number): number;
}

let ready: Promise<Exports> | null = null;

async function instantiate(): Promise<Exports> {
  const res = await fetch(wasmUrl);
  if (!res.ok) throw new Error(`no se pudo cargar el motor (${res.status})`);
  try {
    // needs "application/wasm"; the .htaccess sets it, but fall back if the host gets it wrong
    const { instance } = await WebAssembly.instantiateStreaming(res.clone(), {});
    return instance.exports as unknown as Exports;
  } catch {
    const { instance } = await WebAssembly.instantiate(await res.arrayBuffer(), {});
    return instance.exports as unknown as Exports;
  }
}

export function engine(): Promise<Exports> {
  ready ??= instantiate();
  return ready;
}

/** Copies RGBA into WASM memory, runs `fn`, always frees the copy. */
export async function withPixels<T>(rgba: Uint8ClampedArray, fn: (e: Exports, ptr: number) => T): Promise<T> {
  const e = await engine();
  const ptr = e.alloc(rgba.length);
  try {
    new Uint8Array(e.memory.buffer, ptr, rgba.length).set(rgba);
    return fn(e, ptr);
  } finally {
    e.dealloc(ptr, rgba.length);
  }
}

export function readResult(e: Exports): string {
  return new TextDecoder().decode(new Uint8Array(e.memory.buffer, e.result_ptr(), e.result_len()));
}
