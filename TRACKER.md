# Strobe — Tracker

**Nice Thoughts Ranch**

This is the detailed Strobe tracker. A summary row for Strobe also
lives on the Qompass progress board; this file is the workstream-level
record. Bars move only on gated or verified evidence, never on claims:
a percentage changes when a stage's gate has actually passed, and
every change is explained in the dated log below.

**Program priority (Owner directive, 2026-10-25): all resources to
the animation problem.** The geometry-first route (rigged models,
animation as curve data, audio first, deterministic renders) is the
critical path; other workstreams advance only as they feed it.

| Workstream | Progress | State | Next step | Updated |
|---|---|---|---|---|
| Strobe (overall) | `████████░░░░░░░░░░░░` 40% | Engine ruled (Bevy); animation pivot under way | Vivienne proof unit, identity gate | 2026-10-25 |
| Design | `██████████████░░░░░░` 70% | D1–D7 ruled; D2 superseded (Bevy, below) | Slice SPEC §3/§8 rewrite as Bevy systems | 2026-10-25 |
| Cast book | `██████████████████░░` 90% | Text 10/10; portraits 20 BEGIN + 10 END v3 keepers landed in `portraits/` | Expression sets for the slice | 2026-10-25 |
| Game core | `█████████░░░░░░░░░░░` 45% | C# Core 82/82 in the primo trackb tree (drop 1 + Surface Beat + 5A + weighting hook); `game/` here is the pre-5A Unity drop, reference only | Rust port checked against the 82 tests, inside the slice | 2026-10-25 |
| Vertical slice (Vivienne) | `███░░░░░░░░░░░░░░░░░` 15% | SPEC written (968 lines, 10-step spine, gates) | Pax review with the Bevy rewrite; build after proof G1 | 2026-10-25 |
| Animation | `██░░░░░░░░░░░░░░░░░░` 10% | Fathom motion-record/3 schema complete; rig runtime commissioned (lumen); 3D proof dispatched to identity gate | G1 eye gate: GLB + turntable vs portrait anchors | 2026-10-25 |
| Lovense integration | `██████████░░░░░░░░░░` 50% | MCP S1 shipped (safety contract: master OFF default, ceiling clamp, synchronous stop, no resume, 5 s watchdog) | Bridge in slice stage 7 vs mock; live rung S4 on Owner go + presence | 2026-10-25 |
| Art pipeline | `███████████░░░░░░░░░` 55% | Catalog v7: 601 rows, 74 keepers; eye gates: 117-image pass + BDSM batches (58 keeps); Pony lanes adopted (scenes; portraits/eyes dual-lane); Wan tranche-1: 0/24 keeps, retries queued and deprioritized under the pivot | Catalog v8 validation, then 6d fold-in | 2026-10-25 |
| Audio | `█░░░░░░░░░░░░░░░░░░░` 5% | AUDIO-FACTORY standard written; no assets landed | Aria pilot (Vivienne) inside the proof unit | 2026-10-25 |
| Publication | `██░░░░░░░░░░░░░░░░░░` 10% | Route ruled (D4); PUBLICATION.md compiled | Store fee is the owner's call | 2026-10-25 |

## Design decisions (all ruled)

- D1 — Differentiated selves: each cast member keeps a distinct
  personality track rather than converging on one template.
- D2 — Engine. Originally ruled 2026-10-19: Unity 6000.
  **Superseded 2026-10-25 (Owner): the engine is Bevy / Rust.** The
  C# Core survives as the reference implementation; the Rust port is
  verified against its 82 passing tests. Flip condition recorded
  with the ruling: if Live2D-grade continuous heroine motion ever
  becomes a hard requirement, the ruling reopens.
- D3 — Systems W1–W6 carry over from the design study unchanged.
- D4 — Release route: Steam first, then EroLabs, then Nutaku.
- D5 — Full adult content, disclosed; the cast is depicted 18+.
- D6 — The W4 sync model, two layers. Music layer: output follows the
  music's rhythm, beat pulses derived from the chart, strength
  following track intensity, running whether or not notes are hit.
  Judgement layer: a matched note fires output scaled by accuracy as
  a continuous function of timing offset (dead-center full, window
  edge faint). A miss fires one short, dull, low-strength thud.
- D7 — Phone hub (six apps hard cap: Status, Session, Store,
  Protocols, Feed, Aftercare); Session = Response × Corruption with
  kink-aligned progression only; campaign clock of per-heroine
  seasons; expression deltas on every surface from W5.

## Rulings log (program level)

- 2026-10-25 — Engine: Bevy / Rust (D2 superseded, above).
- 2026-10-25 — Animation priority: all resources to the animation
  problem; geometry-first route (this tracker, Animation row).
- 2026-10-25 — Rig runtime commissioned under lumen (IDEA-0002):
  from-scratch Rust rig-animation runtime, own format first, moc3
  only via a gated deterministic importer; one lane-neutral,
  content-free crate family for SFW and NSFW consumers alike.
- 2026-10-25 — Pratique (IDEA-0091) ntranch pack in ENFORCE mode
  (ratified): asset/upload gate checks run as enforcement, not audit.
- 2026-10-25 — Cast law re-asserted: Strobe heroines are different
  women from the Light Show cast; never key Strobe content to Light
  Show names. Manon's vector is bimbofication (gradual, cumulative
  intellect removal by her own escalating request) — the feeding /
  dancer material is superseded wherever it survives.

## Log

### 2026-10-25

- Portraits completed and landed here: 20 BEGIN keepers (eye-gate
  PASS 2026-10-23) + 10 END v3 keepers (Owner verdict KEEPER
  2026-10-23, sha256-verified against the verdict file at landing).
  The four stale files previously in `portraits/` were replaced by
  the current masters.
- Core reached 82/82 (5A Aftercare-in-hub + per-track weighting hook
  landed in the primo trackb tree; Suite was 62/62). The `game/`
  copy in this repo remains the earlier Unity drop and is now
  reference material under the Bevy ruling.
- Surface Beat implemented and gated in Core (trance → act →
  surface → fluster → re-trance; comfort limit is never the trigger).
- Fathom animation-RE closed: motion-record/3 schema with face
  tracks, scenario-asserted phases, and progression arcs; cases A–G
  measured from the corpus.
- Vivienne vertical-slice SPEC written; the §3/§8 Bevy rewrite and
  Pax's full review are next, after the animation proof's G1.
- Animation proof unit dispatched: Vivienne portrait anchors →
  lumen factory → rigged GLB → identity gate (G1). Wan I2V is
  demoted to short flourishes after tranche-1 went 0/24 at the eye
  gate; the smoothness target is the 3D rigged class (persistent
  geometry, authored curves, rendered frames, produced audio).
- `.ai/memory` completed to the estate form (handoff + inbox +
  archive added; current state and decisions refreshed).
- Code intelligence: SCIP index of the lumen workspace (the rig
  factory) generated with rust-analyzer and landed in `code-intel/`;
  every Strobe-program Rust workspace refreshes its index there
  after its gates pass.

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
