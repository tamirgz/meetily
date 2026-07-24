# Release artifacts

This directory contains locally verified installers for the Hebrew-enabled fork.
Generated installers should also be attached to GitHub Releases so users do not
need to clone the repository.

## Current local artifacts

| Platform | Architecture | Artifact | Status |
| --- | --- | --- | --- |
| macOS | Apple Silicon (`arm64`) | `macos/meetily_0.4.0_aarch64_hebrew.dmg` | Built and verified locally; includes DFN3 enhancement and speaker labels |
| Linux | x64 | `linux/meetily_0.4.0_amd64.deb` | Built in Docker; checksum and Debian metadata verified |
| Linux | x64 | `linux/meetily_0.4.0_amd64.AppImage` | Built in Docker; checksum and AppImage extraction verified |
| Windows | x64 | `.msi`, `-setup.exe` | Pending the native Windows GitHub Actions job |

Windows installers must be built on Windows because the Tauri package depends on
the Microsoft MSVC and WebView2 installer toolchains. The
`build-fork-installers.yml` workflow builds all three platforms on their native
runners and attaches them to a draft GitHub Release when a `v*` tag is pushed.

The current AppImage is about 154 MB, above GitHub's 100 MB limit for a normal
Git object. Publish installers as GitHub Release assets rather than forcing large
binaries into source history. The workflow uploads all installers and their
platform-specific SHA-256 manifests as release assets.

Do not rename an installer without regenerating the matching SHA-256 manifest.
