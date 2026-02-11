# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**betc** is a Tauri 2 desktop application with a Svelte 5 frontend. The frontend lives in `src/` (TypeScript + Svelte) and the backend in `src-tauri/` (Rust).

## Commands

```bash
pnpm install              # Install dependencies
pnpm dev                  # Vite dev server on port 5173
pnpm tauri dev            # Run full Tauri desktop app in dev mode
pnpm tauri build          # Build native desktop app bundle
pnpm build                # Build frontend only (tsc + vite build)
pnpm check                # Type-check Svelte + TypeScript
```

No test runner or linter is configured yet.

## Tool Versions (mise.toml)

Node 25.6.1, pnpm 10.29.2, Rust 1.93.0

## Architecture

- **Frontend:** Svelte 5 (using `$state` runes), Vite 7, Tailwind CSS v4
- **Desktop:** Tauri 2 with `tauri-plugin-log`
- **UI Components:** shadcn/svelte — add new components via the shadcn CLI. Components live in `src/lib/components/ui/`. Uses `tailwind-variants` for variant styling and `cn()` from `$lib/utils` for class merging.
- **Path alias:** `$lib` → `src/lib` (configured in both tsconfig and vite)
- **Styling:** Tailwind v4 with OKLCH design tokens defined as CSS variables in `src/app.css`. Dark mode via `@custom-variant`. Zinc base color.
- **Icons:** `@lucide/svelte`

## Conventions

- Conventional commits (`chore:`, `feat:`, `fix:`, `build:`)
- Barrel exports via `index.ts` for component directories
- All dependencies are devDependencies (Vite handles bundling)
