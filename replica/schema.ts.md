# IndexedDB schema (v1)

```ts
// db name: 'vectorizer', version: 1
export type ImageType = 'photo' | 'artwork_blended' | 'artwork_sharp' | 'scan_color' | 'scan_bw';
export type Detail = 'low' | 'medium' | 'high';

export interface TraceSettings {
  imageType: ImageType;
  detail: Detail;
  colorMode: 'unlimited' | 'custom';
  palette?: string[];            // '#rrggbb', length 2..12 when colorMode === 'custom'
}

export interface ImageRecord {            // store 'images', keyPath 'id'
  id: string;                             // crypto.randomUUID()
  name: string;
  mime: 'image/jpeg' | 'image/png' | 'image/bmp' | 'image/gif';
  width: number; height: number; bytes: number;
  createdAt: string;                      // ISO 8601, UTC
  original: Blob;
  thumb: Blob;                            // 256px PNG
}
// indexes: 'createdAt'

export interface VectorizationRecord {    // store 'vectorizations', keyPath 'id'
  id: string;
  imageId: string;                        // -> ImageRecord.id; deleting an image deletes these in the same transaction
  settings: TraceSettings;
  auto: boolean;
  status: 'done' | 'failed';
  error?: string;
  svg: string;                            // empty when failed
  shapeCount: number;
  ms: number;
  createdAt: string;
}
// indexes: 'imageId', 'imageId_createdAt' on ['imageId','createdAt']

export interface Prefs {                  // store 'prefs', key 'singleton'
  lastSettings?: TraceSettings;
  maxMegapixels: number;                  // default 4
}
```

Validation (in code, since IndexedDB has no constraints): palette length 2..12 when custom;
every vectorization's imageId must exist; mime in the list above.
