# Flappy Jack

A midnight flight through a haunted cemetery, built with **Freya 0.5.0-rc.13**. The opening jack-o'-lantern is a live, rotating **3D hollow, ribbed surface**, with carved openings and a flickering candle inside. No prerecorded animation or runtime assets.

```sh
cargo run --release
# Five seconds of title, followed by a deterministic autopilot flight.
cargo run --release -- --demo
```

A native desktop window replaces the original Ratatui interface. The window starts at 1100 × 700 and can be resized down to 700 × 520. Build requirements are those of [Freya](https://freyaui.dev/), including a current Rust toolchain and the platform libraries required by Skia and winit. Building fetches dependencies, playing needs no network connection.

## Controls

| Input | Action |
| --- | --- |
| Space | Awaken / flap / retry |
| Click or tap the playfield | Flap |
| R | Retry after dying |
| P or Escape | Pause / resume |
| Q or Ctrl-C | Quit |

Start, Resume and Retry also have clickable buttons with keyboard focus and Enter activation. Space is reserved for the game's global start/flap/retry action. Held gameplay keys do not repeat. Starting and retrying do not also flap.

Each gate gives **+1**, candy gives **+3**. Candy does not increase difficulty. Gate speed and gap size gradually become harder, then stop increasing. The pumpkin's collision box is smaller than its decorative leaves and wings.

Losing window focus pauses an active flight. Returning does not resume it automatically, use Resume or P/Escape. Manual pause survives resizing. The world scales uniformly and is letterboxed, preserving the original game's collision geometry and aspect correction.

Best scores are saved atomically to `$XDG_DATA_HOME/flappy-jack/best-score`, or `$HOME/.local/share/flappy-jack/best-score`. Missing, malformed or unwritable storage never prevents play. Concurrent instances are not locked. Demo mode never reads or writes your personal record and uses the same physics as normal play.

## Architecture

- `src/game.rs`: unchanged seeded 100 × 24 world, 120 Hz physics, collision, scoring, difficulty and autopilot.
- `src/app.rs`: backend-independent actions, phases, fixed-step accumulator and persistence integration. Storage can be injected for isolated checks.
- `src/window.rs`: Freya launch, frame notifications, keyboard press/release tracking, focus handling and a bounded background pumpkin renderer.
- `src/pumpkin.rs`: implicit carved shell, inner wall, stem, ray intersection, normals and candle shading, producing RGBA pixels.
- `src/scene.rs`: Skia drawing through Freya render callbacks, including the cemetery, gates, expressive player, candy and seed burst.
- `src/title.rs`: native title/game-over composition and the live 3D pumpkin image.
- `src/ui.rs`: HUD, accessible action buttons and pause overlay.
- `src/storage.rs`: validated best-score loading and atomic replacement.

The title pumpkin renders at 256 × 256 on a worker thread, with requests driven by Freya's `RenderingTicker` instead of a fixed FPS cap. Requests are accepted only when the worker is ready, avoiding a backlog of stale animation frames. The game code is optimized even in development builds to keep CPU ray tracing practical. Freya scales the latest image with linear filtering. Gameplay drawing follows Freya's rendering cadence, independently of the 120 Hz simulation. Animation time freezes while paused or unfocused.

## Verification

```sh
cargo check --all-targets
```

The Freya migration was compile-checked only. Tests and interactive runtime verification were not run.
