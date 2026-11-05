# Build log

| ID | date | status | missing | harder than expected |
| --- | --- | --- | --- | --- |
| engine | 2026-11-05 | done | quality targets below | auto color count: bin histograms could not tell busy art from flat art; cluster-and-merge (k-means 32, drop < 0.8%, merge < 34 RGB) does: Peanuts 24-25, Snoopy 14-18 |
| S01 upload | 2026-11-05 | done | example images | nothing |
| S02 processing | 2026-11-05 | done | real % progress (the engine is one blocking call; stages only) | `.working { display:flex }` beat the `hidden` attribute and S02 stayed under the result: fixed with a global `[hidden]` rule, now covered by the e2e test |
| S03 result | 2026-11-05 | partial | zoom and pan, flip view, settings summary, re-vectorize (S04) | nothing |
| S05 download | 2026-11-05 | partial | PDF and EPS (only SVG in the slice) | nothing |
| S04 settings | | not started | | |
| S07 history | | not started | | |

## Engine timings (WASM in Chromium, this container)

Working-size cap by detail level: low 1.0 MP, medium 1.6 MP (default), high 2.6 MP.
Medium: Snoopy 736x883 in 2.4-2.9 s, Peanuts 900x900 in 3.0 s. Native at 2.6 MP was 6 s, at 1.6 MP 2.8 s.
The spike's fixed 2x upscale is now "up to 2x, never over the cap".

## Known quality gaps (for /replica-diff and the next engine pass)

- Grey bands next to black outlines: k-means gives anti-aliasing its own cluster. A 3x3 majority filter
  was tried and did not help (bands are wider than 2 px); reverted. Next: snap pixels that lie between two
  clusters (on the line between their centers) to the nearer end, or merge clusters by edge adjacency.
- Small lettering ("GOOD DOG GOOD FRIEND") comes out heavier and blobbier than the original.
- The spike's median-cut at 2x with 16 colors looked cleaner on outlines than k-means at 1.4x. Worth an A/B.
