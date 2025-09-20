# Mirror - Tauri + React + TypeScript Project

## Project Architecture

This is a **Tauri v2 desktop application** with a React frontend. The architecture follows a strict separation:

- **Frontend**: React + TypeScript in `src/` (runs in webview)
- **Backend**: Rust in `src-tauri/src/` (native desktop app)
- **Build System**: Vite for frontend, Cargo for Rust, orchestrated by Tauri CLI

## Development Workflow

### Key Commands

- `pnpm dev` - Start development with hot reload (frontend only)
- `pnpm tauri dev` - Start full Tauri app in development mode
- `pnpm build` - Build frontend assets
- `pnpm tauri build` - Build complete desktop application

### Package Manager

This project uses **pnpm** exclusively. The `pnpm-workspace.yaml` configures esbuild to be ignored as a built dependency.

## Tauri-Specific Patterns

### Frontend-Backend Communication

- Use `invoke("command_name", { args })` from `@tauri-apps/api/core` to call Rust functions
- Rust commands are defined with `#[tauri::command]` and registered in `lib.rs`
- Example: `greet` command in `lib.rs` called from `App.tsx`

### Configuration

- **tauri.conf.json**: Main Tauri configuration
  - `beforeDevCommand: "pnpm dev"` - Starts Vite dev server
  - `devUrl: "http://localhost:1420"` - Fixed dev port
  - `beforeBuildCommand: "pnpm build"` - Builds frontend assets
  - `frontendDist: "../dist"` - Where Vite outputs built files

### File Structure Conventions

- Never modify `src-tauri/target/` (Rust build artifacts)
- Frontend development happens entirely in `src/`
- Rust application logic goes in `src-tauri/src/lib.rs`
- `src-tauri/src/main.rs` is minimal entry point calling `lib.rs`

## Build System Integration

### Vite Configuration

- Port 1420 for dev server (matches Tauri config)
- HMR on port 1421
- Ignores watching `src-tauri/**` to prevent conflicts
- `clearScreen: false` to show Rust errors clearly

### Dependencies

- `@tauri-apps/api` for frontend-backend communication
- `@tauri-apps/plugin-opener` for system integration
- React 19 with TypeScript 5.8
- Rust dependencies managed in `Cargo.toml`

## Development Guidelines

1. **Adding Rust Commands**: Define in `lib.rs` with `#[tauri::command]`, register in `invoke_handler`
2. **Frontend Changes**: Standard React patterns, use `invoke()` for backend calls
3. **Building**: Always use `pnpm tauri build` for final builds, not separate commands
4. **Testing**: Use `pnpm tauri dev` to test full integration during development
