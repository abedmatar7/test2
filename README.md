# MedPulse — Offline Rust + Slint Medical Study App (Android & Cross-Platform)

MedPulse is a 100% offline, privacy-first medical study and clinical reasoning application written in **Rust** with a declarative **[Slint](https://docs.slint.dev/latest/docs/slint/guide/platforms/mobile/android/)** UI. It is engineered specifically for AMOLED Android smartphones (such as Qualcomm Snapdragon 870 `aarch64-linux-android` devices) while keeping 100% of domain logic in a platform-independent `medpulse-core` crate so Windows, Linux, and macOS builds compile from the exact same codebase without a rewrite.

## Why Slint Over Dioxus for Android OLED Study?

| Criterion | [Slint 1.18 (`android-activity`)](https://docs.slint.dev/latest/docs/rust/slint/android/) | [Dioxus Mobile (`wry` / `tao`)](https://www.reddit.com/r/rust/comments/11y8swp/personal_review_for_rust_gui_frameworks/) |
| :--- | :--- | :--- |
| **Rendering Pipeline** | Direct GPU OpenGL ES / Skia / FemtoVG surface via Android `NativeActivity` | Wraps Android System `WebView` (`android.webkit.WebView`) + DOM/CSS compositor |
| **True-Black (`#000000`) AMOLED Control** | Deterministic `(0, 0, 0)` pixel shutoff with zero WebView compositor backlight/halo bleed | Subject to Android WebView dark-mode overrides and compositor layer blending |
| **60-Second Pixel-Shift Burn-in Guard** | Native reactive property bindings (`x`/`y` offsets) with zero DOM layout reflow overhead | Triggers periodic CSS/DOM layout recalculations inside Chromium WebView |
| **APK Footprint & RAM** | Compact native `.so` binary (~8–14 MB APK) with minimal background memory | Higher memory footprint from Chromium WebView runtime |

## Workspace Architecture

```text
medpulse/
├── Cargo.toml                     # Cargo workspace root (resolver = "2")
├── README.md                      # Architecture, Android NDK/APK build steps, desktop targets
├── assets/                        # 100% offline bundled assets (zero network calls)
│   ├── quotes.json                # 160 medical motivational quotes (verified + marked unattributed)
│   ├── i18n/
│   │   └── en.json                # Centralized UI strings for future multi-language expansion
│   ├── icons/                     # 22 clean, restyleable SVG line-art anatomical icons
│   │   ├── heart.svg, lungs.svg, brain.svg, liver.svg, kidneys.svg, stomach.svg ...
│   │   └── neuron.svg, rbc.svg, dna.svg, nephron.svg, alveolus.svg
│   ├── cards/                     # Diagnostic SVG diagrams for image flashcards
│   │   ├── ecg_stemi.svg
│   │   ├── histology_reed_sternberg.svg
│   │   ├── xray_tension_pneumothorax.svg
│   │   └── anatomy_circle_of_willis.svg
│   └── seed/
│       └── seed_data.json         # 13 specialties, flashcards, vignettes, Goodpasture case, OSCE station
├── core/                          # Platform-independent Rust library (`medpulse-core`)
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── models.rs              # Domain models,BackupPayload, implementation intentions, mood logs
│   │   ├── i18n.rs                # Localization catalog loader (`UiStrings`)
│   │   ├── timer.rs               # 25/5, 50/10, Custom timer, 60s pixel shift, 5-min organ rotator
│   │   ├── srs.rs                 # FSRS-4.5 & SM-2 schedulers + {{c1::...::hint}} Cloze engine
│   │   ├── quotes.rs              # Non-repeating 160-quote deck cycle + study/break boosters
│   │   ├── planner.rs             # Smart daily plan, rest-day grace streak, weak-topic heatmap
│   │   └── storage.rs             # Offline SQLite (`rusqlite` bundled) + JSON export/import
│   └── tests/
│       └── integration_tests.rs   # Unit & integration tests for all core subsystems
└── app-android/                   # Slint UI + Android `NativeActivity` & Desktop entrypoints
    ├── Cargo.toml                 # Configured for `cargo-apk2` / `cargo-ndk` + `medpulse-desktop` bin
    ├── build.rs                   # Compiles `ui/app.slint`
    ├── ui/
    │   └── app.slint              # Pure-Black OLED ambient panel + 6-tab medical study UI
    ├── src/
    │   ├── lib.rs                 # `android_main` + shared UI controller bindings
    │   └── main_desktop.rs        # Native Windows/Linux/macOS binary entry point
    └── android/
        ├── AndroidManifest.xml    # Zero INTERNET permission; WAKE_LOCK + ForegroundService
        └── src/main/java/dev/medpulse/app/
            └── MedPulseTimerService.java # Screen-off-safe ForegroundService + alarm/vibration
```

## Exact Android Build & ADB Deployment Instructions

### 1. Prerequisites (Rust Toolchain & Android NDK)

Install the Rust Android targets and Slint's maintained APK packager [`cargo-apk2`](https://crates.io/crates/cargo-apk2) (or [`cargo-ndk`](https://github.com/bbqsrc/cargo-ndk)):

```bash
# Add Android targets (ARM64 for Snapdragon 870 / modern phones, plus ARMv7 and x86_64 emulators)
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android

# Install cargo-apk2 (maintained by the Slint team, supports compiling Java ForegroundService into DEX)
cargo install cargo-apk2

# Optional: install cargo-ndk if integrating into an external Gradle wrapper
cargo install cargo-ndk
```

Ensure `ANDROID_HOME` (or `ANDROID_SDK_ROOT`) and `ANDROID_NDK_ROOT` point to your Android SDK (API 35) and NDK (r26d / r27+):

```bash
export ANDROID_HOME="$HOME/Android/Sdk"
export ANDROID_NDK_ROOT="$ANDROID_HOME/ndk/27.0.12077973"
export PATH="$ANDROID_HOME/platform-tools:$PATH"
```

### 2. Run Platform-Independent Core Unit Tests on Host

Because `medpulse-core` has zero UI or Android dependencies and bundles SQLite C sources via [`rusqlite` (`features = ["bundled"]`)](https://docs.rs/crate/rusqlite/latest), you can run the full test suite directly on your host machine:

```bash
cargo test -p medpulse-core
```

### 3. Build the Debug APK & Install via `adb`

```bash
cd app-android

# Build debug APK for 64-bit ARM Android (e.g., Xiaomi 12X / Snapdragon 870)
cargo apk2 build --target aarch64-linux-android --lib

# Verify that android.permission.INTERNET is completely absent from the packaged APK
aapt dump permissions ../target/debug/apk/MedPulseStudy.apk

# Install and launch on a connected device with USB debugging enabled
adb install -r ../target/debug/apk/MedPulseStudy.apk
adb shell am start -n dev.medpulse.app/android.app.NativeActivity
```

Or build, deploy, and stream logs in a single command:

```bash
cargo apk2 run --target aarch64-linux-android --lib
```

### 4. Running on Windows, Linux, or macOS (No Rewrite Needed)

To run the exact same Slint UI natively on desktop:

```bash
cargo run -p medpulse-app-android --bin medpulse-desktop
```
