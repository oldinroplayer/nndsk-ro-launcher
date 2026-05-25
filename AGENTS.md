# Repository Guidelines

## Project Structure & Module Organization

This is a Tauri v2 desktop launcher for Ragnarok Online on Linux. The React/TypeScript frontend lives in `src/`: `app/` wires the shell UI, `features/` contains user-facing domains such as launcher, servers, settings, logs, and autopot, and `shared/` holds reusable APIs, hooks, types, stores, and UI components. Frontend tests sit next to feature logic as `*.test.ts`.

Rust backend code lives in `src-tauri/src/`, grouped into `commands/`, `models/`, `tools/`, `utils/`, and `state/`. Shared Rust workspace crates are under `crates/ro-tools-core` and `crates/ro-tools-linux`. Bundled runtime resources are in `src-tauri/resources/`.

## Build, Test, and Development Commands

- `npm install`: install JavaScript and Tauri CLI dependencies.
- `npm run dev`: run the Vite frontend only.
- `npm run tauri:dev`: run the full desktop app with Linux WebView environment fixes.
- `npm run build`: type-check with `tsc` and build the frontend.
- `npm test`: run Vitest unit tests once.
- `cargo test --workspace`: run Rust tests for `src-tauri` and local crates.
- `cargo fmt --all`: format Rust workspace code.
- `npm run tauri:build`: create the production Tauri bundle.

## Coding Style & Naming Conventions

Use TypeScript, React function components, and named exports. Keep feature-specific state and logic inside the matching `src/features/<domain>/` directory, with reusable pieces in `src/shared/`. Component files use `PascalCase.tsx`; hooks use `useThing.ts`; stores use `<domain>.store.ts`; pure logic modules use `<domain>.logic.ts`.

Frontend code currently uses 2-space indentation, single quotes, and no semicolons. Rust code should follow `rustfmt`; use snake_case modules and functions, and keep Tauri command payloads in `models/`.

## Testing Guidelines

Use Vitest for frontend logic. Prefer focused tests for pure functions and state transitions, colocated beside the implementation as `name.logic.test.ts`. Run `npm test` before changing TypeScript behavior. For Rust behavior, add unit tests in the relevant crate/module and run `cargo test --workspace`.

## Commit & Pull Request Guidelines

Git history uses short, informal imperative summaries such as `refactor code/clean v3`. Keep commits concise and scoped to one change. Pull requests should describe the user-visible behavior, list test commands run, link related issues when available, and include screenshots or screen recordings for UI changes.

## Security & Configuration Tips

Do not commit user data from `~/.local/share/ro-launcher/`, local game client paths, WINE prefixes, or generated build output. Keep bundled dgVoodoo files limited to `src-tauri/resources/dgvoodoo/` and avoid adding server-specific executables to the repo.
