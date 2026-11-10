# Strobe

## 18+ — Adults Only

This game contains adult content. It is intended only for players 18 and older. *Strobe* is a sex-positive game about kinky women taking their own desire seriously: consent, comfort limits, and aftercare are built into its systems, not bolted on as warnings.

### Content notes

- Adult sexual content with ten adult heroines (ages 24–31)
- Kink and BDSM themes, specific to each heroine: bondage, discipline, lactation, pet play, chastity, enslavement fantasy, corruption, and more
- Optional connected-device play (off by default, hard output ceilings, stop means stop)

All characters depicted are adults.

## An adult session game where you learn what ten women secretly want — and become the one each of them finally chooses to show it to.

<p align="center"><a href="https://github.com/NTRanch/Strobe">GitHub</a> · <a href="https://ntranch.itch.io">itch.io studio page</a></p>

## Built with

![Rust](https://img.shields.io/badge/Rust-000000?logo=rust&logoColor=white) ![Bevy](https://img.shields.io/badge/Bevy-232326?logo=bevy&logoColor=white) ![Neovim](https://img.shields.io/badge/Neovim-nightly-57A143?logo=neovim&logoColor=white)

## What you do

Everything runs through one in-fiction phone with six apps. In **Session**, you play to each heroine's *frequency* — the learnable sweet spot where her kink sings — and two bars answer: **Response**, how deep she is right now, and **Corruption**, how far her desire has grown. The **Store** sells new verbs, never stat sticks; **Protocols** deliver her capstones as app upgrades, each with her reaction attached; the **Feed** shows her changing between sessions; and every session closes in **Aftercare** — a ritual, not a reward screen.

Each heroine's arc runs six stages, from *Unaware* to the flip where she stops hiding and starts choosing. Her desire levels along four Response tracks — Voice, Gaze, Touch, Rhythm — and progress only ever grows along the desire she already has. Rush her or play off her desire and things cool; learn her, and she opens like a lock turning. At **Craving** she starts initiating sessions herself. By the end, you're not pursuing her. You're keeping up.

## Features

- Ten heroines, each with her own kink, her own six-stage arc, and two portraits: the woman she performs, and the woman she becomes
- A hard comfort limit that is a stop, not a fail state — the session ends gracefully, no progress lost, straight to aftercare
- Optional device sync in two layers (a music layer that pulses with the track, and a judgement layer where your timing becomes touch), off by default and clamped to the lowest of your ceiling, her unlocked tier, and an absolute cap

## Media

Begin / End portraits from the cast book in this repo:

![Cordelia — Begin](../portraits/cordelia_begin.png)
![Cordelia — End](../portraits/cordelia_end.png)

## Status

**In active development.** The cast book is written, all twenty portraits are done, and the vertical slice — one heroine, end to end — is in production. The design's logic core was proven first (82 automated tests, green) and is being ported to Rust against those tests as its specification. No public build yet; the release route is itch.io first, and this GitHub page is the game's home in the meantime. Free on release.

## Credits

A Nice Thoughts Ranch game.

Made with Rust on the Bevy engine, with deterministic rig-based animation.
