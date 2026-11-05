// Messages between the page and the vectorizer worker.
export type Detail = 0 | 1 | 2; // low, medium, high

export interface TraceRequest { type: 'trace'; id: number; file: Blob; detail: Detail }

export type WorkerMessage =
  | { type: 'stage'; id: number; stage: 'decode' | 'colors' | 'trace' }
  | { type: 'done'; id: number; svg: string; colors: number; paths: number; ms: number; width: number; height: number; downscaled: boolean }
  | { type: 'error'; id: number; message: string };

/** Working-size cap per detail level, in pixels (see replica/build-log.md for the timings). */
export const WORK_PIXELS: Record<Detail, number> = { 0: 1_000_000, 1: 1_600_000, 2: 2_600_000 };
/** Refuse decoded images over this size: the browser would run out of memory first. */
export const MAX_SOURCE_PIXELS = 50_000_000;
