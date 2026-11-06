# Current state — 2026-11-06

- ENGINE: Bevy / Rust (Owner ruling 2026-10-25). The Rust game is
  COMPLETE and gated: workspace `strobe-core` + `strobe-bridge` +
  `strobe-game` (Bevy 0.20 subcrates), 124 passed / 0 failed, clippy
  and fmt clean, golden determinism SHA-256
  `631d1066ef0221871999deb9bf9f2eb86a0e8ba03f940b7a719a411d4fcb79ca`
  pinned in tests and reproduced from fresh state twice
  (re-gated 2026-11-05 on the final tree).
- DEMO: the windowed `strobe-play` demo covers the full first
  chapter under the D-181 mapping (guided consent + practice →
  first real session → first banked clear → Stage 1 "Curious" +
  Feed unlock → end card; free play continues). Packages:
  `strobe-play-demo-linux-x86_64.zip` sha256 `48c96be8…`,
  `strobe-play-demo-windows-x86_64.zip` sha256 `2f7f644c…`,
  byte-identical to the gated build. The demo ships SILENT (no
  audio assets landed yet).
- Vivienne keeper pair RULED (2026-10-28): Begin = C1 cel,
  End = E2 magenta gown (E1 black lace alternate). The demo's C1
  portrait is the keeper Begin. Her expression set + 12-frame
  animated idle are unblocked and queued as follow-on work.
- Player-facing copy: the agent-drafted demo copy (welcome,
  consent, narration, end card) was ACCEPTED for the demo
  (2026-11-06). No character dialogue was invented; her lines
  remain owed keys (extraction from PROMPT-CARDS.md).
- Publication: the itch demo upload was AUTHORIZED (2026-11-06).
  Both zips are staged on primo, hash-verified; the upload runs
  as soon as the Owner completes `butler login` (session-only
  credential posture), then Pax pushes, verifies, and removes
  the session credential.
- C# Core: reference implementation only. The primo trackb
  checkout was RETIRED (2026-11-06, Owner ruling): its full git
  history (all local branches) is preserved as a verified git
  bundle plus a byte-verified dirty-state snapshot in the
  finish-up record. The weighting hook's behaviour is fully
  implemented in the Rust port (`strobe-core` session XP award,
  same formula and rounding; 124 tests pass).
- D-654 rigged 3D fleet (10 girls, 54 bones / 36 morph targets):
  the Owner REJECTED the fleet's look (2026-11-06, on Vivienne's
  renders) — adoption is gated on a real face + hair pass. The
  rigs stand as proven infrastructure; no fleet girl is
  adoption-grade on screen yet.
- D-802 kink stylesheet (D-826 re-base at IPA 0.15): ADOPTED as
  the stylesheet of record (2026-11-06, all ten cells). One
  caption nit carried: Cordelia's caption reads "Corruption"
  where the fold names her kink overstimulation.
- Asset-factory verdict packs (Strobe, 150): object — RULED as
  a set (2026-11-06): #18 kept, the other 17 rejected as props,
  refire with isolation-hardened prompts queued. Clothing —
  RULED: all 17 rejected as clothing assets, kept as outfit
  design references, refire queued. Character stills — WALKED:
  9 of 63 rejected, the rest stay candidates; the cast record
  and three ref-clips rule by file in the follow-on batch.
  Audio (45) and effect (2) packs are deferred to their own
  ear/file sittings. Cordelia is absent from every pack
  (D-799 freeze honoured).
- Prose: demo v8 canonical; fleet v2 verdicts still owed on
  eight stories (Vivienne, Maren, Kaisa, Cordelia, Zuzana,
  Valentina, Yuna, Manon-final) — deferred by the Owner to
  dedicated reading sittings (2026-11-06).
- Audio: no landed assets yet; the factory audio pack awaits
  the Owner's ear pass (files only).
