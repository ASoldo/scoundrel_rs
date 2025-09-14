# Scoundrel (Terminal)

A single‑player roguelike card game played with a standard deck — in your terminal.
This is a ratatui (Rust) TUI implementation of Scoundrel (v1.0, Aug 15th 2011) by
Zach Gage and Kurt Bieg, adapted for keyboard play and a clean, focused terminal UI.

## Features

- Ratatui TUI with crisp card rendering and subtle background texture
- Full Scoundrel rules (v1.0) with rooms of 4 cards and avoid mechanic
- Binding weapons with ≤ last monster rule, per‑turn potion limit
- Color‑coded HP (green/yellow/orange/red) with always‑red projected damage
- Equipped card on the left; slain mini‑cards row to the right (left→right)
- Room cells show quick‑pick numbers (1–4) in corners and a caption (Weapon/Potion/Enemy)
- Unified Help popup (press `?`) on every screen
- Leaderboard with podium (gold/silver/bronze) and centered list below
- Name entry per run; results stored locally in JSON
- Mouse wheel + keys scroll the Game Over history

## Install

Prereqs: Rust (stable). Then:

```
cargo build --release
```

Run:

```
cargo run --release
```

## Controls

Global
- `?` — Help popup
- `q` or `Esc` — Quit

Menu
- `Up/Down` + `Enter` — Navigate/confirm
- Bottom border shows a dim “? - help” hint

Name Entry
- Type to enter a name (max ~20 chars), `Enter` to confirm, `Backspace` to edit
- `?` — Help

In‑Game
- `Left/Right` — Move selection among 4 room cards
- `Enter` / `Space` — Take selected (default: use weapon if allowed, else barehand)
- `1` `2` `3` `4` — Quick‑pick the corresponding room cell
- `w` — Force weapon; `b` — Force barehand
- `v` — Avoid room (not twice in a row)
- `?` — Help

Game Over
- `Up/Down`, `PageUp/PageDown`, `Home/End`, or mouse wheel — Scroll history
- `n` — New run; `l` — Leaderboard; `m` — Menu

## Rules (Scoundrel v1.0)

- Deck setup
  - Remove jokers, red faces (J/Q/K of hearts/diamonds), and red aces (A♥ A♦)
  - Monsters: all clubs/spades (2–10, J=11, Q=12, K=13, A=14)
  - Weapons: diamonds 2–10 (binding — equipping discards previous weapon+stack)
  - Potions: hearts 2–10 (max 1 potion used per turn)
- Rooms & turns
  - Always reveal 4 cards (a room). You must take 3, one carries over to the next room
  - You may avoid a room (put all 4 to the bottom), but not two rooms in a row
  - A turn ends after taking 3 cards or when only 1 card remains
- Combat
  - Barehand: take full monster damage
  - With weapon: monster hits for (monster − weapon) if positive
  - After using a weapon, it can only be used on monsters ≤ the last monster’s value it fought
- Potions
  - Only one potion can heal per turn (second and further potions that turn are discarded)
  - HP cannot exceed 20
- End & scoring
  - Lose when HP ≤ 0: score = current HP − sum(remaining monsters)
  - Win when deck is cleared: score = HP; if HP==20 and last card was a potion, score += that potion

## UI Notes

- Cards
  - Big cards: rounded white borders; rank top‑left and bottom‑right; suit centered and bold
  - Mini cards (slain): small rounded boxes; value top‑left; suit bottom‑right; rendered left→right
- Status box
  - Shows HP (color‑coded), projected damage (always red), Weapon (with ≤ last cap), Deck, Room
- Room view
  - 4 cells in one row; each shows its quick‑pick number in corners and a caption beneath
  - Selected cell: yellow border; selected empty: red border; otherwise dim gray borders
- Help
  - Popup uses a dim gray text color for body content and is available on all screens

## Data & Files

- Leaderboard file: `scoundrel_scores.json` in the working directory
  - Appends runs, sorts descending by score; shows top 10 in UI

## Compatibility

- Uses Unicode suit glyphs (♥ ♦ ♣ ♠) and box drawing characters
- For best results, use a terminal with Unicode and ANSI color support

## Acknowledgments

- Game design: Scoundrel — © 2011 Zach Gage and Kurt Bieg
- TUI: [ratatui](https://github.com/ratatui-org/ratatui), backend: [crossterm](https://github.com/crossterm-rs/crossterm)

## License

This project implements gameplay inspired by Scoundrel (2011). See original rules/credits above.
