# Flappy Jack integration contract

## Frontend

The desktop frontend uses Freya `=0.5.0-rc.13` with its engine feature. There is no Ratatui or Crossterm dependency, terminal lifecycle or embedded terminal. Keep runtime assets procedural and offline.

## Simulation

`src/game.rs` remains independent of Freya and rendering. Keep the seeded 100 × 24 world, fixed 1/120 second ticks, current gravity and flap impulse, bounded difficulty, conservative collision box, gate +1 and candy +3 scoring, and deterministic autopilot.

`src/app.rs` owns the fractional-tick accumulator and clamps elapsed stalls to 250 ms. The frontend supplies elapsed time and suspension status. Actions are `Activate`, `Retry`, `Pause` and `Quit`. Starting or retrying must not also flap. Preserve the title, playing, paused and game-over phases, retained death state, 0.35-second seed burst, and five-second demo title.

## Window and input

`src/window.rs` owns Freya launch, rendering ticker updates, keyboard press/release tracking and focus-loss suspension. The default window is 1100 × 700 with a 700 × 520 minimum. Pause active gameplay on focus loss, require explicit resume, and discard suspended time. Window close and Q/Ctrl-C exit through Freya.

Space starts, flaps or retries. R retries after death. P/Escape toggles pause. Ignore held-key repetition. Native action buttons support pointer, accessibility and Enter activation, while Space remains the global gameplay shortcut.

## Rendering

`src/scene.rs` draws the playfield through a Freya Skia render callback. Map horizontal world units to 10 logical pixels and vertical units to 20, then scale uniformly into the available area with letterboxing. Keep gate boundaries and the player core aligned with simulation hitboxes.

`src/pumpkin.rs` retains the real implicit 3D shell, carving, stem, surface normals and candle shading. Its output is RGBA, not terminal cells. Render at a bounded 256 × 256 resolution on a background thread, using bounded request/result queues driven by Freya's `RenderingTicker`, without a fixed FPS cap. Keep UI updates and image creation on Freya's thread.

`src/title.rs` composes the title and result screens. `src/ui.rs` composes the HUD, pause overlay and accessible action buttons. Keep the nearly black, purple, orange and warm candle palette, procedural moon, cemetery, ghosts, candy and death seeds.

## Persistence

Preserve `$XDG_DATA_HOME/flappy-jack/best-score`, falling back to `$HOME/.local/share/flappy-jack/best-score`. Storage errors remain nonfatal. Demo mode must never access personal records. Use explicitly injected isolated paths for any future UI tests.

## Migration validation

Only `cargo check --all-targets` is authorized for this migration. Do not run tests or claim runtime/visual verification. Existing simulation and storage tests remain, app and pumpkin tests are adapted to the new interfaces, and terminal rendering tests are removed with the old frontend.
