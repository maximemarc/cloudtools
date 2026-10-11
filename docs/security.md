# Dependency vulnerabilities

Status on 2026-10-11. Check again with:

```bash
cargo audit                          # RustSec, every lockfile via .cargo/audit.toml
python3 -I scripts/osv-check.py      # GitHub advisories (what Dependabot reports), every lockfile
```

## Fixed

Dependabot reported 183 alerts after the first import. Fixes live in `patches/` and are replayed
by `scripts/upstream-sync.sh`, which also runs `scripts/fix-lockfile-vulns.py` on every lockfile.

| Advisory                                         | Where                                       | Fix                                                        |
|--------------------------------------------------|---------------------------------------------|------------------------------------------------------------|
| 82 npm advisories (Nx, Vitest, Remix…)           | artcraft Tauri + React frontend             | Not vendored: Craft Studio uses artcraft-native            |
| quick-xml DoS (RUSTSEC-2026-0194/0195)           | deckcraft, designcraft, gridcraft, lightcraft | quick-xml 0.38 → 0.41 (`patches/*/0001-quick-xml-0.41`) |
| rustls-webpki 0.102, rsa 0.9 via rustls-rustcrypto | lightcraft-fetch, filmcraft-speech        | aws-lc-rs provider (`patches/lightcraft/0002`, `patches/filmcraft/0001`) |
| sqlx 0.7 (RUSTSEC-2024-0363), rustls 0.21        | artcraft enums/tokens (`database` feature)  | sqlx 0.8 (`patches/artcraft/0002`)                         |
| ring, rustls, h2, time, quinn-proto, crossbeam, rkyv, bytes, tracing-subscriber, rand, git2, anyhow, rsa 0.9.6 | several lockfiles | Semver-compatible lockfile updates |
| Tauri app crates (sqlx, plist/quick-xml 0.32, tempdir) | artcraft desktop crate                | Not vendored (`patches/artcraft/0001` drops them from the workspace) |

## Accepted

| Advisory                                   | Lockfiles                                                        | Why it stays                                                                 |
|--------------------------------------------|------------------------------------------------------------------|------------------------------------------------------------------------------|
| GHSA-c38w-74pg-36hr / RUSTSEC-2023-0071 (rsa, Marvin attack) | `Cargo.lock`, `upstream/pdfcraft`, `upstream/artcraft` | No fixed release exists. The attack needs RSA *private-key* operations timed over a network; here rsa only runs public-key operations (PdfCraft's signature checks, signing goes through aws-lc-rs; sqlx-mysql's password encryption). Ignored in `.cargo/audit.toml`. |
| GHSA-wrw7-89jp-8q8g (glib 0.18, unsound `VariantStrIter`) | `Cargo.lock`, `upstream/filmcraft`, `upstream/craft-launcher`, `upstream/craft-repo-status-app` | In Craft Studio it is a lockfile entry only: `muda` (macOS menus) is a macOS-only dependency and pulls GTK only on Linux, so glib is never compiled. In the standalone apps it comes with GTK menus/dialogs, which don't use `VariantStrIter`. The fix needs gtk-rs 0.20, an upstream change. |

On GitHub, these alerts can be dismissed as "Vulnerable code is not actually used" (glib) and
"No bandwidth to fix / no fix available" (rsa), with a link to this page.

RustSec notices without a GitHub advisory (unmaintained `paste`, `rustybuzz`, `ttf-parser`,
`instant`, `backoff`…; unsound `git2` calls not used here) are informational and listed by
`cargo audit` as warnings.
