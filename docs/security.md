# Dependency vulnerabilities

Status on 2026-10-17. Check again with:

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
| GHSA-wrw7-89jp-8q8g (glib 0.18, unsound)         | `Cargo.lock`, filmcraft                     | Out of the lockfiles: muda (macOS menus) without its default `gtk3` feature, which only adds the Linux backend (`scripts/gen-module-manifests.py`, `patches/filmcraft/0002`) |
| rustls 0.23.45 (GHSA-c8m5-c952-6cm4, GHSA-vp88-798v-h67f, GHSA-h2v8-394m-p7vv, GHSA-w44r-93j7-qv28) | `Cargo.lock` and 6 upstream lockfiles | rustls 0.23.46 |

## Accepted

| Advisory                                   | Lockfiles                                                        | Why it stays                                                                 |
|--------------------------------------------|------------------------------------------------------------------|------------------------------------------------------------------------------|
| GHSA-c38w-74pg-36hr / RUSTSEC-2023-0071 (rsa, Marvin attack) | `Cargo.lock`, `upstream/pdfcraft`, `upstream/artcraft` | No fixed release exists. The attack needs RSA *private-key* operations timed over a network; here rsa only runs public-key operations (PdfCraft's signature checks, signing goes through aws-lc-rs; sqlx-mysql's password encryption). Ignored in `.cargo/audit.toml`. |
| GHSA-wrw7-89jp-8q8g (glib 0.18, unsound `VariantStrIter`) | `upstream/craft-launcher`, `upstream/craft-repo-status-app` | Not part of Craft Studio. These two standalone apps use GTK 3 on Linux for real (tray icon, Tauri's webview), and gtk-rs stopped at glib 0.18 for GTK 3; neither calls `VariantStrIter`. The fix is a move to GTK 4, an upstream change. |
| GHSA-vv5m-jxxh-4627, GHSA-4v9h-p7m9-rh26, GHSA-4684-h2q9-2vcp (zip < 9, extraction to disk) | `Cargo.lock`, deckcraft, designcraft, gridcraft, wordcraft, filmcraft, lightcraft | The three are about `ZipArchive::extract` / `ZipStreamReader::extract` writing an archive to disk. Nothing here extracts: the office formats and candle-core read entries in memory (`by_name`, `by_index`). The fix is zip 9, a major version candle-core 0.9 does not take. Ignored in `.cargo/audit.toml`. |

On GitHub, these alerts can be dismissed as "Vulnerable code is not actually used" (glib, zip)
and "No bandwidth to fix / no fix available" (rsa), with a link to this page.

## Secret scanning

GitHub reports two Google API keys in `upstream/artcraft/crates/api_clients/`: `midjourney_client`
and `worldlabs_consumer_client`. They are the public Firebase Web API keys of Midjourney's and
WorldLabs' own web apps (Firebase keys identify a project and ship in every browser bundle; access
is gated by the user's token), copied by the ArtCraft authors for ArtCraft's "connect your
Midjourney / WorldLabs account" integrations. They are not credentials of this repository, cannot
be rotated from here, and Craft Studio does not compile those crates. The alerts can be closed as
"False positive" with a link to this page.

RustSec notices without a GitHub advisory (unmaintained `paste`, `rustybuzz`, `ttf-parser`,
`instant`, `backoff`…; unsound `git2` calls not used here) are informational and listed by
`cargo audit` as warnings.
