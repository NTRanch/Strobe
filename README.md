# STROBE — Cast Book

Design document for the publishable Strobe title (Track B, Nice Thoughts Ranch).
Ten women, ten backstories, ten netorare arcs. Each arc is built on the same
load-bearing rules as the rest of the design:

- **The trait is the vector, never the casualty.** Every heroine has a hidden
  trait or desire she has been performing *against*. The arc does not break
  her; it stops her performance. What changes across the stages is how much
  of her life is organized around hiding.
- **Agency flips at Stage 3.** Stages 0–2 happen *to* her (exposure,
  temptation, secrecy). From Stage 3 on, every step is her choice, made with
  open eyes. The player witnesses a woman taking her own life over, not
  being taken.
- **No decay, no punishment.** Nobody is ruined. The endings are the
  happiest states these characters reach in the game — by her own measure,
  not the narrator's.
- **The before-bond is a mismatch, not a monster.** The partner she leaves
  (or renegotiates) is not a villain; he loved the performed version of her.
  The NTR sting lives in that contrast: he keeps the mask, someone else gets
  the woman.

## Stage model (W5)

| Stage | Name | Meaning |
|------:|------|---------|
| 0 | Unaware | The trait is buried; she would deny it if asked. |
| 1 | First Spark | The trait is touched by the one person/situation that fits it. |
| 2 | Secret Practice | She feeds it in secret and tells herself it means nothing. |
| 3 | Choosing | The flip: she acts on it deliberately, knowing the cost. |
| 4 | No More Hiding | The before-bond breaks or is renegotiated on the truth. |
| 5 | Self-Driven | The trait is integrated; she directs her own desire. Happiest state. |

## Contents

- `CHARACTERS.md` — the cast and their identity anchors (invariant across
  both portraits and all scenes): face, natural eye color, hair color,
  defining traits. The Strobe girls are **not** the Light Show companions:
  different names, different looks, different women — see CHARACTERS.md
  for the per-slot differentiation.
- `stories/` — one file per heroine: backstory, hidden trait, the
  before-bond, the vector, stage-by-stage arc beats, end state, and the
  portrait briefs her two portraits were drawn from.
- `portraits/` — `<name>_begin.png` (Stage 0 presentation) and
  `<name>_end.png` (Stage 5 presentation). Anchors hold; styling,
  expression, and bearing carry the transformation.

## Portrait status

- Vivienne, Kaisa — begin/end portrait pairs generated, in
  `portraits/` (identity held across each pair via snapshot resume).
- Valentina, Maren, Yuna, Katya, Manon, Zuzana, Cordelia, Sadie —
  briefs written in their story files; portraits pending generation
  (16 of 20 remain).

*Draft for the studio gate. Nothing here is final
until it passes.*
