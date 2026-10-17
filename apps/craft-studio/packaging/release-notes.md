Every Craft app in one window: ArtCraft, PhotoCraft, LightCraft, FilmCraft, EffectCraft, VectorCraft,
DesignCraft, PdfCraft, WordCraft, GridCraft, DeckCraft, CADCraft and SoundCraft.
See the [documentation](https://github.com/maximemarc/cloudtools/blob/main/docs/craft-studio.md).

## Downloads

| System | File | |
|---|---|---|
| Windows 10/11 (x64) | `craft-studio-<version>-windows-x64-setup.exe` | Installer, per-user by default (no administrator rights) |
| | `craft-studio-<version>-windows-x64.zip` | Portable: unzip and run `craft-studio.exe` |
| macOS 11+ (Apple silicon) | `craft-studio-<version>-macos-arm64.dmg` | Drag *Craft Studio* to *Applications* |
| Ubuntu 22.04+, Debian 12+ (x86-64) | `craft-studio_<version>-1_amd64.deb` | `sudo apt install ./craft-studio_<version>-1_amd64.deb` |
| Other Linux (x86-64, glibc 2.35+) | `craft-studio-<version>-linux-x86_64.tar.gz` | Unpack and run `craft-studio`; needs ALSA and a Vulkan or GL driver |

`SHA256SUMS` lists the checksums: `sha256sum -c SHA256SUMS --ignore-missing`.

## First launch

The builds are not signed with a publisher certificate:

- **Windows**: SmartScreen may show "Windows protected your PC". Choose *More info* › *Run anyway*.
- **macOS**: the app is ad-hoc signed, not notarized. Open it once, then allow it in
  *System Settings* › *Privacy & Security* › *Open Anyway*, or run
  `xattr -dr com.apple.quarantine "/Applications/Craft Studio.app"`.

## Network and privacy

No module sends telemetry. ArtCraft needs an ArtCraft account and sends nothing you type or add before you
sign in; the other apps work offline. Details: [EXTERNAL-CALLS.md](https://github.com/maximemarc/cloudtools/blob/main/docs/artcraft/EXTERNAL-CALLS.md).

## Licenses

MIT OR Apache-2.0, like the Craft apps it is built from. Every package carries `LICENSE-MIT`,
`LICENSE-APACHE` and `THIRD-PARTY-LICENSES.txt` (the Craft repos and the 900 crates the build bundles).
