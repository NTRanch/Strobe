# Current state — 2026-10-25

- Track B Unity project lives in `game/` (Unity 6000.6.6f1). Core drop 1 +
  Surface Beat landed locally as commit 33eb135 on main; GitHub push is
  pending the Owner's GPG unlock of the NTRanch credential. Core gates:
  dotnet test 62/62 (48 drop-1 + 14 Surface Beat). LangVersion pinned 9.0
  (Unity compiler ceiling). Unity asmdef batchmode compile check still owed.
- No levels exist yet: game/ Art/Audio/Data/Prefabs/Scenes/Shaders hold only
  .gitkeep files. First build target = one vertical-slice level (one heroine:
  hub → session → Surface Beat → aftercare), gated on the fathom
  animation-RE motion-record schema freeze.
- Prose: STYLE.md §9 applied 2026-10-25 (revision-pass menu, continuity tail,
  seam, prompt-card assembly, interactive lane); PROMPT-CARDS.md completion
  pass in progress (Kaisa first, then Manon, then fleet order); revision pass
  targets the three §8.7-WEAK vignettes (Vivienne, Katya, Sadie) first.
