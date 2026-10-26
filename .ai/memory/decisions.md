# Decisions (Owner rulings)

- CAST LAW (ruled 2026-10-19; re-asserted 2026-10-25): the Strobe heroines
  are different women from the Light Show cast — different names and looks,
  same archetype slots. Never key Strobe content to Light Show names.
  Authority: CHARACTERS.md. Anchors are invariant within Strobe;
  transformation is styling/arrangement/makeup/expression, never hair color.
- NAME RULE (2026-10-20): never use the Owner's personal name in any NTRanch
  repo or property — studio attribution only.
- PLAY ANALYTICS (2026-10-25): one shared system across light-show, ontrack,
  and strobe (IDEA-0090) — one schema/codebase, qompass/ntranch lane profiles,
  disjoint data roots, no identity in schema. Schema + analysis skill now;
  collector/server only on named triggers. Design:
  ~/workspace/play-analytics-20261025/DESIGN.md.
- MCP GAPS (2026-10-25): storefront/community ops = a skill, not a server;
  periscope is the standard verification path for this build; phaedrus
  research output routes into lupin; broadside never extends to NTRanch.

- PRATIQUE (2026-10-25): the NTRanch enforcement-gate MCP is named pratique (Owner ruling; IDEA-0091). Brief at ~/workspace/pratique-20261025/BRIEF.md (sandbox); build on Owner go.

- ENGINE (2026-10-25): Bevy / Rust. DECISIONS D2 (Unity 6000) is
  superseded; the C# Core is the reference implementation and the
  Rust port is verified against its 82 tests. Flip condition: if
  Live2D-grade continuous heroine motion becomes a hard requirement,
  the ruling reopens.
- ANIMATION PRIORITY (2026-10-25): all resources to the animation
  problem. Geometry-first route: rigged models, animation as curve
  data, audio first, deterministic renders. Wan-class I2V is demoted
  to short flourishes (tranche-1 eye gate: 0/24 keeps).
- RIG RUNTIME (2026-10-25): a from-scratch Rust rig-animation runtime
  is commissioned under lumen (IDEA-0002) — own format first, moc3
  only via a gated deterministic importer, one lane-neutral
  content-free crate family for SFW and NSFW use. Brief:
  ~/workspace/lumen-rig-runtime-20261025/BRIEF.md.
- PRATIQUE MODE (2026-10-25): the ntranch pack runs in ENFORCE mode
  (ratified after the mode flip), pack v1.2.0.
- MANON VECTOR (re-asserted 2026-10-25): bimbofication — gradual,
  cumulative intellect removal at her own escalating request, per
  the 2026-10-22 re-canon. The feeding / dancer material is
  superseded wherever it survives.
