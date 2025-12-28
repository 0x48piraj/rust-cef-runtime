# "Pure-GPU" HTML Renderer, minus the bullsh*t

A Rust-native Chromium runtime providing a high-performance foundation for GPU-accelerated desktop applications.

`rust-cef-runtime` is a **low-level, Rust-native Chromium runtime built on the Chromium Embedded Framework (CEF)** for developers who need **control, performance, and consistency** beyond what system WebViews provide.

It exposes Chromium directly: its rendering pipeline, lifecycle, and process model while remaining intentionally minimal, explicit, and unopinionated, **without Electron and without relying on system WebViews**.

## Motivation

This project started as a "_GPU-accelerated FPS toy demo built with Tauri for the boys_" that performed extremely well on **Windows (WebView2)** out-of-the-box but encountered hard limitations on **Linux**:

* VSync-locked rendering (~60 FPS)
* Inconsistent GPU behavior through and through
* Extremely limited control over the rendering pipeline

Those constraints are inherent to **system WebViews**, not Tauri itself.

To achieve consistent, high-performance rendering, we pivoted to **CEF**, unlocking:

* Native Chromium rendering everywhere
* Explicit lifecycle and process control
* Reliable GPU acceleration on Linux (and macOS)
* High-frequency rendering where the platform allows

The result is a **clean, reusable Rust + CEF runtime** you can build performant desktop apps on

## Why `rust-cef-runtime` (and not Tauri/Electron)

Using Chromium directly solves the rendering problem, but existing options have trade-offs:

* **Electron** bundles Node.js, adds runtime overhead, and constrains architecture
* **Custom Chromium builds** are complex, fragile, and expensive to maintain

* Tauri uses:

  * **WebView2 on Windows**: 🔥 fast, uncapped, GPU-accelerated
  * **WebKitGTK / WKWebView elsewhere**: ⚠️ vsync-locked, inconsistent GPU support

* For performance-heavy apps such as:

  * Real-time animations
  * Visualizations
  * WebGL
  * WASM
  * Games
  * High-refresh dashboards

    * Linux/macOS were capped ~60 FPS
    * GPU behavior varied wildly

**CEF + Rust** provides a middle ground:

* Native Chromium GPU pipeline
* Explicit application and window lifecycle
* No embedded Node.js runtime
* No opinionated framework
* Total control over process boundaries and IPC

## How `rust-cef-runtime` compares with the giants

| Capability                   | **rust-cef-runtime**                             | **Tauri (WebView2 / WKWebView)** | **Electron**     |
| ---------------------------- | ------------------------------------------------ | -------------------------------- | ---------------- |
| Rendering engine             | Chromium                                         | OS WebView                       | Chromium         |
| GPU pipeline                 | Chromium                                         | OS-managed                       | Chromium         |
| VSync control                | **Uncapped on Windows, Linux**                   | OS-locked                        | OS-locked        |
| High-FPS rendering           | **Yes**                                          | Limited                          | Limited          |
| Cross-platform consistency   | **Yes**                                          | No                               | Yes              |
| Engine-level control         | **Complete**                                     | No                               | Partial          |
| IPC model                    | **Native (CEF / Rust)**                          | JS <-> Rust                      | JS <-> Node      |
| Binary size                  | Compact                                          | **Small**                        | Large            |
| Runtime dependency           | **None**                                         | Tauri runtime                    | Electron runtime |
| Sandbox control              | **Explicit**                                     | OS-defined                       | Limited          |
| Linux GPU reliability        | **Excellent**                                    | VSync-locked (`WebViewGTK`)      | Good             |
| macOS GPU control            | **Untested**                                     | OS-restricted                    | Good             |
| Windows GPU stack            | **Excellent**                                    | **Best-in-class**                | Great            |
| Open source                  | Yes                                              | Yes                              | Yes              |
| Opinionated framework        | No                                               | Yes                              | Yes              |


## What this project optimizes for

> `rust-cef-runtime` is not a replacement for Tauri or Electron.

It exists for cases where **engine-level control and rendering behavior matter more than convenience**.

### This runtime is well-suited for:

* High-frequency rendering (render loops, visualization, tooling, engines)
* Developers who want **Chromium without Electron**
* WebGL, Canvas, WASM-heavy workloads
* Identical rendering semantics across platforms
* Rust-first architectures without embedded JS runtimes
* Anyone hitting performance or GPU limitations with OS WebViews
* Anyone who wants **complete control** over rendering & lifecycle
* A base to build **custom shells, engines, or non-standard apps**

## When you should *not* use this project

* If you want the smallest possible binary: **probably use Tauri**
* If your app is standard CRUD UI: use **Tauri or Electron**
* If you want Node.js APIs: **use Electron**
* If you want maximum native OS integration **with minimal effort**: **use Tauri**

## Architecture overview

```
Rust (CEF)
 ├─ App lifecycle (cef::App)
 ├─ BrowserProcessHandler
 ├─ Native window + browser_view
 ├─ JS <-> Rust IPC (cefQuery)
 └─ Asset loading (file:// or dev server)

HTML / CSS / JS
 ├─ Any framework (Vanilla / React / Vue / Svelte)
 ├─ requestAnimationFrame
 ├─ WebGL / Canvas / WASM
 └─ Calls into Rust via IPC
```

You explicitly control **everything**:

* Window creation
* Browser lifecycle
* Rendering backend
* IPC boundaries

There is no hidden runtime behavior.

## Setup

### 1. Install CEF binaries (once)

### Linux or macOS:

```bash
cargo run -p export-cef-dir -- --force $HOME/.local/share/cef
```

### Windows (using PowerShell):


```pwsh
cargo run -p export-cef-dir -- --force $env:USERPROFILE/.local/share/cef
```

### 2. Environment variables

### Linux

```sh
export CEF_PATH="$HOME/.local/share/cef"
export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:$CEF_PATH"
```

**Sandbox fix:**

```bash
sudo chown root:root ~/.local/share/cef/chrome-sandbox
sudo chmod 4755 ~/.local/share/cef/chrome-sandbox
```

(CEF will refuse to start without this.)


#### NixOS

Enter the dev-shell

```bash
nix develop
```

Run this (ONCE) to create the shared linked CEF directory:

```bash
setup-cef
```


### macOS (experimental)

```sh
export CEF_PATH="$HOME/.local/share/cef"
export DYLD_FALLBACK_LIBRARY_PATH="$DYLD_FALLBACK_LIBRARY_PATH:$CEF_PATH:$CEF_PATH/Chromium Embedded Framework.framework/Libraries"
```

### Windows (using PowerShell)

```pwsh
$env:CEF_PATH="$env:USERPROFILE/.local/share/cef"
$env:PATH="$env:PATH;$env:CEF_PATH"
```

**Ninja fix:**

Run all build commands from a MSVC environment, then launch PowerShell from there.

(CEF will refuse to start if **Ninja** is not available in environment.)

## Running the demo

```bash
cargo run --example demo
```

This launches a native window with GPU-rendered canvas demo for accurate benchmarks.

### More demos! (not benchmarks)

```bash
cargo run --example dom_single
cargo run --example dom_multi
cargo run --example server
```


### Dev server override (any example)

```bash
export CEF_DEV_URL=http://localhost:1420
cargo run --example demo
```

### Custom frontend path

```bash
export CEF_APP_PATH=/absolute/path/to/frontend
cargo run --example demo
```

## Development mode

To use a live dev server:

```bash
export CEF_DEV_URL=http://localhost:1420
```

The Rust binary remains unchanged; only the frontend swaps.

## Testing

- `tests/files-cors` contains a minimal Vite based web-app to test extra imported files. 
  `bun install && bun run build` will output the built `dist` into `examples/files-cors` so you can run it using `cargo run --example files-cors`.


## 🚧 Current status

✅ Windowed Chromium app<br>
✅ Local asset loading<br>
✅ GPU acceleration<br>
🔜 Native IPC support<br>
🔜 Packaging helpers<br>
🔜 Template generator<br>
🔜 CI examples<br>
