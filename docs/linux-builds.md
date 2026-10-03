# Linux builds

The Linux port targets x86_64 Arch Linux and Omarchy first. It uses Tauri and
the system WebKitGTK 4.1 runtime. iCloud, Apple Calendar, and Apple Contacts
are unavailable. Local notes, Git sync, AI, local embeddings, browser capture,
and the agent CLI use the shared desktop implementation.

## Build

Use Node 24, the pnpm version pinned in `package.json`, and stable Rust.
Install the [Tauri Linux prerequisites](https://v2.tauri.app/start/prerequisites/).
On Arch, also provide GStreamer's base, good, and bad plugins for audio
recording and playback. A Secret Service implementation such as GNOME Keyring
is required to save AI and GitHub credentials; Reflect never falls back to a file.

```sh
pnpm install --frozen-lockfile
pnpm release:linux build
```

The helper stages both sidecars, builds the app, and produces a native archive,
its SHA-256 checksum, and a matching `PKGBUILD` in `target/linux-artifacts/`.
The version comes from `apps/desktop/package.json`; prereleases are translated
to Arch's version syntax without modifying the app version.

On Arch, turn those artifacts into an installable package:

```sh
cd target/linux-artifacts
makepkg
sudo pacman -U reflect-open-linux-*.pkg.tar.zst
```

`makepkg` may request missing package dependencies. Review and install them
with pacman before retrying. Launch **Reflect** from the desktop menu or run
`reflect-open`. The `reflect` command exposes the read/discovery CLI.

The archive contains a `/usr` tree for packaging; it is not an AppImage.
Build locally for your distribution, or use a compatible CI artifact.
The Linux workflow currently builds on Ubuntu 24.04 and uploads artifacts;
it does not publish releases or claim support for every distribution.

## Integration behavior

- Browser capture registers manifests under the XDG config directory for
  detected native Chrome, Chromium, Brave, Edge, Vivaldi, and Opera profiles.
  Open the graph once after installing the extension or a new browser.
  Snap/Flatpak browsers and custom browser profile roots are not covered.
- The app, CLI, and capture host live together under `/usr/lib/reflect-open`.
  This keeps browser capture and agent skills usable while Reflect is closed.
- Linux preview images use public OpenGraph/Twitter metadata. Fetches retain
  the existing redirect, address, byte-size, and image-decoding limits.
- Microphone access requires an explicit session consent dialog. Cancelling
  does not acquire a device. Audio stays local until the existing transcription
  pipeline sends it to the configured provider.
- logind resume notifications request an index reconciliation. File watching
  remains active if logind is unavailable, and watcher errors also request a scan.
  Linux read/open notifications are filtered before debouncing so indexing
  cannot trigger itself by reading notes or walking directories.
- Updates are installed through the package manager. Linux builds neither
  check nor install updates from upstream Reflect's macOS feed.

## Validation

```sh
pnpm check
pnpm build
pnpm test --run apps/desktop/scripts/release-linux.test.mjs \
  apps/desktop/src/lib/desktop-capabilities.test.ts \
  apps/desktop/src/hooks/use-audio-recorder.test.tsx
pnpm --filter @reflect/desktop sidecar
cargo test -p reflect-open capture::tests
cargo test -p reflect-open link_preview::platform::tests
cargo test -p reflect-open watcher::tests
```

Native checks should use a disposable graph and isolated XDG config/data/cache
directories. Verify editing and reopening, Unicode input, paste/drop, attachments,
Ctrl shortcuts, multiple windows, clipboard links, OS trash, external edits,
embedding download/reuse, and microphone recording/playback. Live provider and
remote-sync tests require user-provided credentials. Confirm those separately;
browser tests and a successful build do not establish device-level readiness.

### Initial validation on Omarchy

The first x86_64 package was built and exercised on Omarchy 4.0.4 with
WebKitGTK 2.52.6. Native checks covered graph reopening, Unicode Markdown
round-tripping, indexed CLI search, queued capture while the app was closed,
browser-manifest registration, and a disposable Secret Service credential
round-trip. Synthetic Opus/WebM recording decoded successfully in the actual
webview; cancelling the native microphone dialog returned denial.

The initial unfiltered watcher grew to roughly 13 GB while idle. With read
events filtered before debouncing, the same scratch graph stayed near
260–280 MB during the initial native checks. This is a smoke-test observation,
not a general memory benchmark. Physical microphone quality, real provider
requests, authenticated remote sync, and suspend/resume still need user-session
validation.
