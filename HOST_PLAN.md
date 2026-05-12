# LnMai Host Plan

## Goal

Run `lnmai /path/to/chart.zip` as a playable simulator with Lean as the
judgment core and Rust as the host runtime.

## Repo Layout

- Use `LnMai` as the root repo.
- Keep `lnmai-core` as a git submodule inside the Rust host tree.
- Keep the Lean core and Rust host side-by-side so the FFI boundary stays local.
- Treat the submodule as the authoritative Lean source checkout for host builds.

## Bootstrap Scope

- Accept one chart path from the command line.
- Open a game window immediately.
- Load chart assets from the zip path on the Rust side.
- Render a video/background layer and UI overlays.
- Map keyboard input to button and sensor events.
- Step Lean by timed input batches and apply emitted commands.

## Runtime Split

- Lean: chart state, note lifecycle, judgment, score, command emission.
- Rust: file IO, archive loading, windowing, rendering, audio, video, input.
- FFI: state in, batched input in, commands out.

## Host Modules

- `cli`: parse `lnmai <chart.zip>` and optional debug flags.
- `archive`: open chart zips and resolve files lazily.
- `clock`: provide monotonic music time and pause/seek hooks.
- `input`: translate keyboard/mouse/controller events into button/sensor batches.
- `video`: decode chart-linked video into texture frames.
- `audio`: play BGM plus judge SFX from Lean commands.
- `render`: draw background, video, notes, judges, and overlays.
- `bridge`: serialize state/inputs/commands across the Lean boundary.

## Frame Flow

1. Poll OS input.
2. Convert input to timestamped events.
3. Build `TimedInputBatch` for the current slice.
4. Call Lean step.
5. Consume `AudioCommand` and `RenderCommand`.
6. Present the next frame.

## Keyboard Mapping

- Buttons: 8 fixed keys for the ring/lane buttons.
- Sensors: 33 fixed keys for the touch areas.
- Keep bindings in a single table so they can be remapped later.

## Phased Delivery

1. Rust window + CLI + keyboard state.
2. Chart archive loader and metadata reader.
3. Lean bridge and step loop.
4. Audio playback and judge SFX.
5. Video background playback.
6. Rendering polish and result flow.

## Build Environment

- Use Nix as the base developer environment.
- Provide a shell with Lean toolchain support via `elan` and Rust toolchain
  support via `rustc`/`cargo`.
- Keep native build deps in the shell so `lake build` and `cargo check` work
  from the same entry point.

## Display Model

- Follow the reference ordering: background/video first, then note field,
  then judge effects and HUD.
- Keep time/progress as a dedicated overlay rather than blending it into note
  drawing.
- Treat judge text and combo feedback as transient effect layers.
- Keep video playback host-side, synchronized from the shared music clock.
