# Strobe — Tracker

**Nice Thoughts Ranch**

This is the detailed Strobe tracker. A summary row for Strobe also
lives on the Qompass progress board; this file is the workstream-level
record. Bars move only on gated or verified evidence, never on claims:
a percentage changes when a stage's gate has actually passed, and
every change is explained in the dated log below.

| Workstream | Progress | State | Next step | Updated |
|---|---|---|---|---|
| Strobe (overall) | `███████░░░░░░░░░░░░░` 35% | Design decisions complete; generation test under way | Write the Track B brief; generation test phase 2 through the media pipeline | 2026-10-20 |
| Design | `████████████░░░░░░░░` 60% | Decisions D1–D6 all ruled (below) | Write the Track B brief (evidence base closed 2026-10-20) | 2026-10-20 |
| Cast book | `██████████░░░░░░░░░░` 50% | Text complete for all 10 heroines; portraits 4 of 20 done | Remaining 16 portraits | 2026-10-20 |
| Game scaffold | `███░░░░░░░░░░░░░░░░░` 15% | Unity scaffold and test-animation content built, uncommitted | Commit decision; URP migration pass | 2026-10-20 |
| Lovense integration | `████████░░░░░░░░░░░░` 40% | Bridge and real local transport gated (tests and stub smoke passed) | Live-toy rung, when un-tabled by the owner | 2026-10-20 |
| Art generation pipeline | `████░░░░░░░░░░░░░░░░` 20% | Image model verified on the build machine; generation test phase 1 gated | Phase 2 regenerations under the studio profile | 2026-10-20 |
| Publication | `█░░░░░░░░░░░░░░░░░░░` 5% | Route ruled (Steam first); nothing filed | Store fee is the owner's call | 2026-10-20 |

## Design decisions (all ruled)

- D1 — Differentiated selves: each cast member keeps a distinct
  personality track rather than converging on one template.
- D2 — Unity 6000 + URP as the engine and render pipeline.
- D3 — Systems W1–W6 carry over from the design study unchanged.
- D4 — Release route: Steam first, then EroLabs, then Nutaku.
- D5 — Full adult content, disclosed; the cast is depicted 18+.
- D6 — The W4 sync model, two layers. Music layer: output follows the
  music's rhythm, beat pulses derived from the chart, strength
  following track intensity, running whether or not notes are hit.
  Judgement layer: a matched note fires output scaled by accuracy as
  a continuous function of timing offset (dead-center full, window
  edge faint). A miss fires one short, dull, low-strength thud.
  Design evidence base: complete.

## Log

### 2026-10-20

- Tracker seeded at the percentages above.
- Design closed: decisions D1–D6 all ruled, including the D6
  two-layer sync model. The design evidence base closed the same day,
  which makes the Track B brief writable.
- Cast book text complete for all 10 heroines (W5 stages 0–5,
  dialogue voice = player); portraits stand at 4 of 20.
- Lovense integration gated: the bridge safety state machine, the
  real local transport, and the probe CLI are built and green (82/82
  tests; stub smoke passed, including the intensity clamp proof).
  The live-toy rung is tabled by the owner.
- Generation test phase 1 gated: 23 scenes cataloged from a reference
  title, age gate closed with zero exclusions, and a 3-scene test
  set briefed. Phase 1.5 (composite references and tag-form prompts)
  is running; phase 2 runs through the media pipeline's stage 1
  under the studio profile.
- Game scaffold stands built but uncommitted; the commit decision
  and the URP migration pass are next.
