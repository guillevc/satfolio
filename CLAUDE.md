# CLAUDE.md

## Project

satfolio — Tauri 2 desktop app. Frontend: `src/` (Svelte 5 + TypeScript). Backend: `src-tauri/` (Rust).

## Stack

- Svelte 5 with `$state` runes, Vite 7, Tailwind CSS v4, TypeScript
- Tauri 2 with `tauri-plugin-log`
- shadcn/svelte for UI (components in `src/lib/components/ui/`, add via `pnpm dlx shadcn-svelte@latest add <name>`)
- `tailwind-variants` for variant styling, `cn()` from `$lib/utils` for class merging
- `@lucide/svelte` for icons
- Path alias: `$lib` → `src/lib`
- Tailwind v4 with OKLCH design tokens in `src/app.css`, dark mode via `@custom-variant`, zinc base

## Commands

Use `just`

```
Available recipes:
    default

    [build]
    build          # Build native desktop bundle
    build-ui       # Build frontend only (tsc + vite)

    [ci]
    check          # Full check: typecheck + lint + format [alias: c]
    smoke-app      # Build Tauri app without bundling (smoke test)

    [dev]
    dev            # Run full Tauri desktop app [alias: d]
    dev-ui         # Run Vite dev server only (:5173) [alias: du]

    [format]
    fmt            # Format all [alias: f]
    fmt-check      # Check formatting (no write)

    [gen]
    gen-icons      # Generate app icons from src-tauri/app-icon.svg
    gen-types-core # Generate TS types from Rust models

    [lint]
    lint           # Lint all [alias: l]
    lint-fix       # Lint & fix all

    [setup]
    install        # Install all dependencies [alias: i]

    [test]
    test           # Run all tests [alias: t]
    test-core      # Run app-core tests only

    [typecheck]
    typecheck      # Typecheck all (cargo + svelte)
```

## Conventions

- Conventional commits: `feat:`, `fix:`, `chore:`, `build:`
- Barrel exports via `index.ts` for component directories
- All dependencies are devDependencies (Vite bundles everything)

## Tool Versions

Node 25.6.1, pnpm 10.29.2, Rust 1.93.0

---

## Documentation Lookup

### shadcn-svelte

Fetch docs directly via WebFetch — no index fetch needed.

- **Topic docs:** `https://shadcn-svelte.com/docs/{topic}.md`
  Key topics: `cli`, `components-json`, `theming`, `dark-mode/svelte`, `installation/vite`, `migration/tailwind-v4`
- **Component docs:** `https://shadcn-svelte.com/docs/components/{name}.md`
  All components: accordion, alert, alert-dialog, aspect-ratio, avatar, badge, breadcrumb, button, button-group, calendar, card, carousel, chart, checkbox, collapsible, combobox, command, context-menu, data-table, date-picker, dialog, drawer, dropdown-menu, empty, field, formsnap, hover-card, input, input-group, input-otp, item, kbd, label, menubar, native-select, navigation-menu, pagination, popover, progress, radio-group, range-calendar, resizable, scroll-area, select, separator, sheet, sidebar, skeleton, slider, sonner, spinner, switch, table, tabs, textarea, toggle, toggle-group, tooltip, typography

### Tauri 2

Fetch docs directly via WebFetch — no index fetch needed.

**URL pattern:** `https://v2.tauri.app/{section}/{topic}`

Key paths:

- Concepts: `concept/architecture`, `concept/process-model`, `concept/inter-process-communication`
- Development: `develop/calling-rust`, `develop/calling-frontend`, `develop/configuration-files`, `develop/state-management`, `develop/resources`, `develop/sidecar`, `develop/plugins`, `develop/icons`
- Security: `security/capabilities`, `security/permissions`, `security/scope`, `security/csp`
- Distribution: `distribute/macos`, `distribute/windows`, `distribute/linux`, `distribute/sign/macos`
- Tutorials: `learn/system-tray`, `learn/splashscreen`, `learn/window-customization`, `learn/window-menu`

### Svelte 5 — MCP Server

This project uses **Svelte 5 runes, NOT SvelteKit**. Use the Svelte MCP server:

1. Call **list-sections** first to discover available docs, then **get-documentation** for relevant sections.
2. **MUST** run **svelte-autofixer** on all Svelte code before presenting to user. Repeat until no issues remain.
3. Only use **playground-link** when user explicitly asks — never for code written to project files.

#### MCP Tools

##### 1. list-sections
Use this FIRST to discover all available documentation sections. Returns a structured list with titles, use_cases, and paths. When asked about Svelte topics, ALWAYS call this at the start to find relevant sections.

##### 2. get-documentation
Retrieves full documentation content for specific sections. Accepts single or multiple sections. After calling list-sections, analyze the returned sections (especially use_cases) and fetch ALL relevant sections for the user's task.

##### 3. svelte-autofixer
Analyzes Svelte code and returns issues and suggestions. **MUST** use this whenever writing Svelte code before presenting to user. Keep calling until no issues or suggestions remain.

##### 4. playground-link
Generates a Svelte Playground link with the provided code. Only use when user explicitly asks — **never** for code written to project files.

Commonly needed section paths (pass to get-documentation):
`svelte/$state`, `svelte/$derived`, `svelte/$effect`, `svelte/$props`, `svelte/$bindable`, `svelte/$inspect`, `svelte/what-are-runes`, `svelte/svelte-files`, `svelte/svelte-js-files`, `svelte/basic-markup`, `svelte/if`, `svelte/each`, `svelte/await`, `svelte/snippet`, `svelte/@render`, `svelte/@html`, `svelte/@attach`, `svelte/bind`, `svelte/use`, `svelte/transition`, `svelte/scoped-styles`, `svelte/context`, `svelte/lifecycle-hooks`, `svelte/stores`, `svelte/typescript`, `svelte/svelte-window`, `svelte/svelte-boundary`, `svelte/svelte-reactivity`, `svelte/svelte-motion`, `svelte/testing`

### Other Libraries — Context7 MCP

For any dependency not covered above (e.g. tailwind-variants, bits-ui, serde, clsx): call **resolve-library-id** with the library name, then **query-docs** with the returned ID and a specific query.
