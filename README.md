# "Pure-GPU" HTML Renderer, minus the bullsh*t

A Rust-native Chromium runtime providing a high-performance foundation for GPU-accelerated desktop applications.

`rust-cef-runtime` is a **low-level, low-level Rust runtime built directly on the Chromium Embedded Framework (CEF)** for developers who need **control, performance and consistency** beyond what system WebViews provide.

The project provides a predictable, GPU-accelerated rendering environment for desktop applications that need tighter control than system WebViews allow without bundling Node.js or imposing a framework architecture.

<p align="center">
  <img alt="rust-cef-runtime demo" src="docs/images/output.gif" width="400"><br>
  <b>Chromium goodness. Native Rust. No WebView. No Electron.</b>
</p>

## Motivation

This project started as a "_GPU-accelerated FPS toy demo built with Tauri for the boys_" that performed extremely well on **Windows (WebView2)** out-of-the-box but encountered hard limitations on **Linux**:

* Compositor vsync limits i.e. VSync-locked rendering (~60 FPS)
* Inconsistent GPU paths across OSes
* Limited control over rendering lifecycle

Those constraints are inherent to **system WebViews**, not Tauri itself.

For high-frequency rendering workloads, those constraints become architectural limits. So we pivoted to **CEF**, unlocking:

* Native Chromium rendering everywhere
* Explicit lifecycle and process control
* Reliable GPU acceleration on Linux (and macOS)
* High-frequency rendering where the platform allows

The result is a **clean, reusable Rust + CEF runtime** you can build performant desktop apps on.

## Why choose us

Using Chromium directly solves the rendering problem, but existing options have trade-offs:

* **Electron** bundles Node.js, adds runtime overhead and constrains architecture
* **Custom Chromium builds** are complex, fragile and expensive to maintain

* Tauri uses:

  * **WebView2 on Windows**: Fast, uncapped, GPU-accelerated (usually)
  * **WebKitGTK / WKWebView elsewhere**: Vsync-locked, inconsistent GPU support

* For performance-heavy apps such as:

  * Real-time animations
  * Visualizations
  * WebGL
  * WASM
  * Games
  * High-refresh dashboards

    * Linux/macOS were capped ~60 FPS
    * GPU behavior varied wildly

Our project provides a middle ground:

* Native Chromium GPU pipeline
* Explicit application and window lifecycle
* No embedded Node.js runtime
* Total control over process boundaries and IPC

## How `rust-cef-runtime` compares with the giants

| Capability                   | **rust-cef-runtime**                             | **Tauri**                        | **Electron**     |
| ---------------------------- | ------------------------------------------------ | -------------------------------- | ---------------- |
| Rendering engine             | Chromium                                         | OS WebView                       | Chromium         |
| GPU pipeline                 | Chromium                                         | OS-managed                       | Chromium         |
| VSync control                | **Uncapped on Windows, Linux**                   | OS-locked                        | OS-locked        |
| High-FPS rendering           | **Yes**                                          | Limited                          | Limited          |
| Cross-platform behavior      | Consistent                                       | Platform dependent               | Consistent       |
| Engine-level control         | **Complete**                                     | Limited                          | Partial          |
| IPC model                    | **Native (CEF / Rust)**                          | JS <-> Rust                      | JS <-> Node      |
| Binary size                  | Compact                                          | **Small**                        | Large            |
| Runtime dependency           | **None**                                         | Tauri runtime                    | Electron runtime |
| Sandbox control              | **Explicit**                                     | OS-defined                       | Limited          |
| Linux GPU reliability        | **Excellent**                                    | VSync-locked (`WebViewGTK`)      | Good             |
| macOS GPU control            | **Untested**                                     | OS-restricted (`WKWebView`)      | Good             |
| Windows GPU stack            | **Excellent**                                    | **Best-in-class**                | Great            |
| Intended use                 | Engines / high-performance UIs                   | Apps                             | Apps             |

> Note: Actual frame pacing depends on GPU drivers and compositor behavior, but the runtime does not enforce OS-level vsync caps like system WebViews.

## What this project optimizes for

> `rust-cef-runtime` is not a replacement for Tauri or Electron.

It exists for cases where **engine-level control and rendering behavior matter more than convenience**.

### This runtime is well-suited for:

* High-frequency rendering (render loops, visualization, tooling, engines)
* Developers who want **Chromium without Electron**
* WebGL, Canvas, WASM-heavy workloads
* Identical rendering behavior across platforms
* Rust-first architectures without embedded JS runtimes
* Anyone hitting performance or GPU limitations with OS WebViews
* Anyone who wants **complete control** over rendering & lifecycle
* A base to build **custom shells, engines, or non-standard apps**

## When you should *not* use this project

* If you want the smallest possible binary: **use Tauri**
* If your app is standard CRUD UI: use **Tauri or Electron**
* If you want Node.js APIs: **use Electron**
* If you want native OS integration with minimal effort: **use Tauri**

## Setup

### 1. Install CEF (one time)

The runtime automatically downloads the **exact Chromium build required by the Rust bindings**.

From the project root:

```bash
cargo run -p rust-cef-installer
```

After installation you can simply run the demo:

```bash
cargo run --example demo
```

## Running the examples

### Default GPU demo (recommended)

```bash
cargo run --example demo
```

Launches a native window rendering a **canvas-based animation** designed to accurately reflect GPU-backed rendering performance.

This is the **primary demo** for evaluating rendering behavior and performance.

> **Rendering note**
>
> On Windows, rendering behavior is strongly influenced by how Chromium is deployed. WebView-based solutions (such as Tauri on Windows) inherit Chrome's browser-integrated GPU pipelines, including accelerated Canvas2D and a fully sandboxed GPU subprocess which enables WebGL2. Electron similarly ships dedicated Chromium helper processes that unlock these GPU features.
>
> Our project currently prioritizes a **single-binary CEF architecture** which trades those browser-level privileges for explicit lifecycle control and simpler distribution. These limitations are architectural rather than performance regressions.

### DOM-based educational demos

The example demonstrate **DOM animation limits** and are **not intended as performance benchmarks**.

```bash
cargo run --example dom
```

Use these to understand:

* Main-thread vs compositor behavior
* CPU-bound DOM animation costs
* Why WebGL/Canvas2D are preferred for high-frequency rendering

## Production packaging

`rust-cef-runtime` does not impose a packaging format.

In production, the embedding application is responsible for bundling frontend assets and selecting the startup URL.

You can run:

```bash
cargo build --example package
```

> Note: Place your built frontend in an `content/` directory next to the executable.

## 🚧 Current status

#### Implemented

- [x] Cross-platform CEF-based runtime (Rust-native)
- [x] Native window creation and lifecycle management
- [x] GPU-accelerated rendering via Chromium
- [x] File-based and dev-server frontend loading
- [x] Linux, Windows and macOS support (platform-specific init where required)
- [x] Modular runtime architecture suitable for reuse
- [x] Examples gallery (Canvas, DOM, WebGL, WASM)
- [x] Custom app protocol
- [x] Structured IPC
- [x] Packaging & distribution helpers
- [x] Higher-level application API

#### In progress / planned

* More packaging helpers
* CI builds and example verification
* Nominal project scaffolding / starter layout

## Philosophy

This project is a streamlined CEF runtime giving direct access to the browser model.

You keep control of:

* Window and browser lifecycle
* Multi-process boundaries
* Renderer <-> native communication

Nothing is hidden behind abstractions or opinionated frameworks.
