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
| Strobe (overall) | `████████████░░░░░░░░` 60% | Rust game complete and gated; demo packaged; Owner verdict walk under way | itch upload (authorized, staged); fleet face pass | 2026-11-06 |
| Design | `██████████████░░░░░░` 70% | D1–D7 ruled; D2 superseded (Bevy, below) | Slice SPEC §3/§8 rewrite as Bevy systems | 2026-10-25 |
| Cast book | `██████████████████░░` 90% | Text 10/10; portraits 20 BEGIN + 10 END v3 keepers landed in `portraits/`; Vivienne keeper pair ruled (C1 Begin + E2 End) | Expression sets (Vivienne's unblocked) | 2026-11-06 |
| Game core | `██████████████████░░` 90% | Rust workspace complete: 124 passed / 0 failed, clippy + fmt clean, golden determinism hash reproduced; C# Core is reference only (primo checkout retired 2026-11-06, history preserved as a verified bundle) | Full-game content beyond the demo chapter; audio landing | 2026-11-06 |
| Vertical slice (Vivienne) | `████████████████████` 100% | `strobe-play` demo covers the full first chapter under the D-181 mapping; packages byte-identical to the gated build | Ship via the authorized itch upload | 2026-11-06 |
| Animation | `█████░░░░░░░░░░░░░░░` 25% | D-654 rigged fleet built (10 girls, 54 bones / 36 morph targets, viewer gates passed); Owner rejected the fleet's look 2026-11-06 | Real face + hair pass before any adoption | 2026-11-06 |
| Lovense integration | `██████████░░░░░░░░░░` 50% | MCP S1 shipped (safety contract: master OFF default, ceiling clamp, synchronous stop, no resume, 5 s watchdog) | Bridge in slice stage 7 vs mock; live rung S4 on Owner go + presence | 2026-10-25 |
| Art pipeline | `█████████████░░░░░░░` 65% | D-802 kink stylesheet adopted as record (D-826 re-base); factory packs walked: object ruled (1 kept / 17 rejected, refire queued), clothing ruled (design refs, refire queued), character stills walked (9 of 63 rejected) | Isolation-hardened refires; audio + effect pack sittings | 2026-11-06 |
| Audio | `█░░░░░░░░░░░░░░░░░░░` 5% | AUDIO-FACTORY standard written; factory audio pack (45) awaits the Owner's ear pass; no assets landed | Ear pass, then Vivienne pilot | 2026-11-06 |
| Publication | `██████░░░░░░░░░░░░░░` 30% | itch demo upload authorized 2026-11-06; zips staged on primo, hash-verified; Steam route ruled (D4) for later | Owner `butler login`, then push + verify | 2026-11-06 |

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

- 2026-11-06 — Owner verdict walk on the finish-up queue: Vivienne
  keeper pair confirmed (C1 + E2); D-654 fleet look rejected (face +
  hair pass gates adoption); D-802 stylesheet adopted as record;
  object and clothing factory packs ruled as sets with refires
  queued; character stills walked (9 of 63 rejected); draft demo
  copy accepted; itch upload authorized; primo C# checkout retired
  with its history preserved as a verified git bundle.
- 2026-10-28 — Vivienne keeper pair ruled: Begin = C1 cel,
  End = E2 magenta gown (E1 black lace alternate).
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

### 2026-11-06

- Finish-up re-gate (2026-11-05 night, verified from run files):
  fmt clean, clippy clean (incl. the render feature), 124 passed /
  0 failed, release builds rc 0, golden determinism hash
  `631d1066…` reproduced from fresh state twice; demo zips verified
  byte-identical to the gated build. The unused `bevy_time`
  workspace dependency was removed and the tree re-gated green.
- The shipped Rust source tree, which had no version control, was
  given a local-only git repo (no remote, nothing pushed); its
  long-term home in this repo is a separate Owner ruling.
- Owner verdict walk (see Rulings log): keeper pair confirmed,
  fleet look rejected pending a face + hair pass, stylesheet
  adopted, three factory packs ruled or walked, copy accepted,
  itch upload authorized and staged, C# checkout retired with a
  verified bundle preserving its full history.
- Prose v2 verdicts deferred by the Owner to dedicated reading
  sittings; the audio and effect factory packs deferred to their
  own ear/file sittings.

### 2026-10-25

- Portraits completed and landed here: 20 BEGIN keepers (eye-gate
  PASS 2026-10-23) + 10 END v3 keepers (Owner verdict KEEPER
  2026-10-23, sha256-verified against the verdict file at landing).
  The four stale files previously in `portraits/` were replaced by
  the current masters.
- Core reached 82/82 (5A Aftercare-in-hub + per-track weighting hook
  landed in the primo trackb tree; suite was 62/62). The `game/`
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
