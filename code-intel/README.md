# Code intelligence (SCIP)

SCIP indexes of the Strobe program's Rust workspaces, kept here so
Diver's SCIP tooling has one stable place to read them from.

## Contents

- `lumen-index.scip` — SCIP index of the lumen workspace
  (qompassai/lumen), the rig factory that builds Strobe's heroines.

## Provenance

- Source workspace: qompassai/lumen @ 6b0ccb5
  (6b0ccb592bc5a941f460d54443bd8cef136183da), primo
  `~/.GH/Qompass/lumen`
- Generator: rust-analyzer 1 (9074e9b4c6 2026-09-06), `scip`
  subcommand
- Generated: 2026-10-26T02:50:18Z
- sha256: 2da728e07399402d5c5628f31d7e3d7e5ea8fcf4cb12e9f0f559fca06357d28b
- Size: 5,152,925 bytes

## Regeneration

    cd ~/.GH/Qompass/lumen
    rust-analyzer scip . --output index.scip
    cp index.scip ~/.GH/NTRanch/Strobe/code-intel/lumen-index.scip
    cp index.scip ~/.GH/NTRanch/Strobe-NSFW/code-intel/lumen-index.scip

Then update the provenance block above (source commit, timestamp,
sha256, size) and commit in both repos.

## Standing rule

Every Strobe-program Rust workspace — lumen today; the lumen rig
runtime and the Bevy core port in `game/` when they land — gets its
SCIP index regenerated after its gates pass and refreshed in this
directory. Indexes are generated artifacts: never hand-edit them;
regenerate instead. Consumer: Diver's SCIP tooling.
