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

| Capability                   | **rust-cef-runtime**                             | **Tauri**                        | **Electron**     |
| ---------------------------- | ------------------------------------------------ | -------------------------------- | ---------------- |
| Rendering engine             | Chromium                                         | OS WebView                       | Chromium         |
| GPU pipeline                 | Chromium                                         | OS-managed                       | Chromium         |
| VSync control                | **Uncapped on Windows, Linux**                   | OS-locked                        | OS-locked        |
| High-FPS rendering           | **Yes**                                          | Limited                          | Limited          |
| Cross-platform consistency   | Yes                                              | No                               | Yes              |
| Engine-level control         | **Complete**                                     | No                               | Partial          |
| IPC model                    | **Native (CEF / Rust)**                          | JS <-> Rust                      | JS <-> Node      |
| Binary size                  | Compact                                          | **Small**                        | Large            |
| Runtime dependency           | **None**                                         | Tauri runtime                    | Electron runtime |
| Sandbox control              | **Explicit**                                     | OS-defined                       | Limited          |
| Linux GPU reliability        | **Excellent**                                    | VSync-locked (`WebViewGTK`)      | Good             |
| macOS GPU control            | **Untested**                                     | OS-restricted (`WKWebView`)      | Good             |
| Windows GPU stack            | **Excellent**                                    | **Best-in-class**                | Great            |
| Open source                  | Yes                                              | Yes                              | Yes              |
| Opinionated framework        | **No**                                           | Yes                              | Yes              |

> Note: Actual frame pacing depends on GPU drivers and compositor behavior, but the runtime does not enforce OS-level vsync caps like system WebViews.

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

* If you want the smallest possible binary: **use Tauri**
* If your app is standard CRUD UI: use **Tauri or Electron**
* If you want Node.js APIs: **use Electron**
* If you want native OS integration with minimal effort: **use Tauri**

## Architecture overview

```
Rust (CEF)
 ├─ App lifecycle (cef::App)
 ├─ BrowserProcessHandler
 ├─ Native window + browser_view
 ├─ JS <-> Rust IPC (cefQuery) (planned)
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

## Running the examples

### Default GPU demo (recommended)

```bash
cargo run --example demo
```

Launches a native window rendering a **canvas-based animation** designed to accurately reflect GPU-backed rendering performance.

This is the **primary demo** for evaluating rendering behavior and performance.

> **Windows rendering note**
>
> On Windows, rendering behavior is strongly influenced by how Chromium is deployed. WebView-based solutions (such as Tauri on Windows) inherit Chrome's browser-integrated GPU pipelines, including accelerated Canvas2D and a fully sandboxed GPU subprocess, which enables WebGL2. Electron similarly ships dedicated Chromium helper processes that unlock these GPU features.
>
> `rust-cef-runtime` currently prioritizes a **single-binary CEF architecture**, which trades those browser-level privileges for explicit lifecycle control and simpler distribution. As a result, Canvas2D benchmarks on Windows tend to favor WebView-based solutions, and WebGL2 availability is constrained. These limitations are architectural rather than performance regressions, and do not apply on Linux, where full GPU acceleration and WebGL2 are available.

### WASM demo

This project includes a **pure WebAssembly demo** that mirrors the canvas animation example, but moves the **simulation logic into a standalone WASM module**.

#### What this demo demonstrates

* Loading raw `.wasm` binaries via the custom `app://app/` scheme
* JS <-> WASM interop without `wasm-bindgen`
* Canvas rendering driven by WASM logic
* Zero Rust WASM tooling baked into the runtime

> The demo includes both the Rust source and the compiled `.wasm` for reference. In real applications, only the compiled `.wasm` would typically be shipped.

#### Building the WASM module

From the demo directory:

```bash
rustc \
  --target wasm32-unknown-unknown \
  -O \
  --crate-type=cdylib \
  demo.rs \
  -o demo.wasm
```

This produces a standalone `demo.wasm` suitable for direct loading via `fetch()`.

**Note:**

The `wasm32-unknown-unknown` target must be installed:

```bash
rustup target add wasm32-unknown-unknown
```

Once the `.wasm` file is present alongside `index.html`:

```bash
cargo run --example wasm
```

### DOM-based educational demos

The example demonstrate **DOM animation limits** and are **not intended as performance benchmarks**.

```bash
cargo run --example dom
```

Use these to understand:

* Main-thread vs compositor behavior
* CPU-bound DOM animation costs
* Why WebGL/Canvas2D are preferred for high-frequency rendering

### Development server (any example)

Override the frontend with a live dev server (useful for hot reload):

```bash
CEF_START_URL=http://localhost:5173 cargo run --example demo
```

or

```bash
cargo run --example server
```

### Custom frontend directory

Load a custom frontend directly from disk:

```bash
CEF_APP_PATH=/abs/path/to/frontend cargo run --example demo
```

The runtime will load `index.html` from the specified directory.

## Vite-based frontend example

This repository includes a **Vite-built frontend example** used to validate real-world asset loading, module resolution, and import behavior under the custom `app://` scheme.

### Purpose

The Vite example is intentionally minimal, but it exercises features that often break in embedded Chromium runtimes:

* ES module loading
* CSS imports
* Static assets (SVG, images, text, etc.)
* Cross-file imports (`?raw`, nested assets)
* Same-origin behavior under a custom scheme

This makes it a good **integration test** for the runtime rather than a visual demo.

### Location

* Source: `tests/files-cors`
* Build output: `examples/files-cors`

### Building the frontend

From the project root:

```bash
cd tests/files-cors
bun install
bun run build
```

This produces a production-ready build in:

```text
examples/files-cors/
```

### Running the example

Once built, run it via:

```bash
cargo run --example files-cors
```

The runtime will load the built `index.html` using the `app://app/` scheme and serve all assets through the custom CEF resource handler.

> **Note**: This example uses a production Vite build (`vite build`), not the Vite dev server. Dev server usage is supported separately via `CEF_START_URL`.

## Production packaging

`rust-cef-runtime` does not impose a packaging format.

In production, the embedding application is responsible for bundling frontend assets and selecting the startup URL.

### Recommended layout

Place your built frontend in an `content/` directory next to the executable:

### Example (`package.rs`)

```rust
std::env::set_current_dir(&frontend_root)
    .expect("Failed to set frontend root directory");

Runtime::run(CefString::from("app://app/index.html"));
```

You can run:

```bash
cargo build --example package
```

No environment variables are required in production.

## 🚧 Current status

**Implemented**

✅ Cross-platform CEF-based runtime (Rust-native)<br>
✅ Native window creation and lifecycle management<br>
✅ GPU-accelerated rendering via Chromium<br>
✅ File-based and dev-server frontend loading<br>
✅ Linux, Windows, and macOS support (platform-specific init where required)<br>
✅ Modular runtime architecture suitable for reuse<br>

**In progress / planned**

🔜 Native JS <-> Rust IPC (CEF message router / `cefQuery`)<br>
🔜 Examples gallery (Canvas, DOM, WebGL, WASM)<br>
🔜 Packaging & distribution helpers<br>
🔜 CI builds and example verification<br>
🔜 Nominal project scaffolding / starter layout<br>

> *Features are added incrementally. Stability and correctness take priority over convenience abstractions.*
