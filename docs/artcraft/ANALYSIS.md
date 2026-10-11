# Analyse de l'écosystème ArtCraft (github.com/storytold)

> Snapshot du 2026-10-11, clones shallow du `HEAD` de `main`. Métriques brutes : [`metrics.json`](./metrics.json).
> Forks : non créés depuis la session cloud (pas de droits API sur `storytold/*`) → [`scripts/fork-artcraft.sh`](../../scripts/fork-artcraft.sh).

## 1. Périmètre

21 repos retenus sur 47 dans l'org `storytold`. Exclus : héritage FakeYou/Vocodes, plugins Unreal, ML voix (`storyteller-ml`, `vits-finetuning`…), `github-media` (assets README), `placeholder-artcraft` (archivé).

| Groupe                   | Repos                                                                                                                               |
|--------------------------|-------------------------------------------------------------------------------------------------------------------------------------|
| ArtCraft (IA générative) | `artcraft`, `artcraft-services`, `artcraftx`, `cloud-worker`                                                                        |
| Craft App Suite          | `photocraft`, `lightcraft`, `filmcraft`, `pdfcraft`, `vectorcraft`, `effectcraft`, `wordcraft`, `designcraft`, `gridcraft`, `cadcraft`, `deckcraft`, `soundcraft` |
| Infra partagée           | `craft-libs`, `craft-launcher`, `craft-fonts`, `craft-repo-status-app`, `photocraft-corpus`                                         |

Volume total : **~3,5 M LOC Rust**, ~530 k LOC TypeScript, ~46 k tests Rust (`#[test]`).

## 2. Constat principal

- **Code massivement généré par agents IA.** `AGENTS.md`/`CLAUDE.md` partout, roadmaps chiffrées en « Opus 5.5 agent-hours », PhotoCraft : 1 429 commits en 12 jours (pic 366/jour), ~100 % avec trailer `Co-Authored-By` IA. Les 12 apps Craft (2,48 M LOC) ont été créées entre le 30/09 et le 07/10/2026.
- **Discipline d'ingénierie élevée pour du code généré** : layering vérifié au build (`cargo xtask layers`), `unsafe_code = "forbid"` (exceptions isolées aux crates FFI), clippy `deny(unwrap/expect/panic/todo)` hors tests, 0 `todo!()` dans les apps Craft, fuzz « panic_hunt » sur toutes les commandes, oracles CPU vs GPU, corpus réels (PSD).
- **Tous les produits sont en alpha / pré-alpha** (cf. §4). Upstream extrêmement mouvant → un fork divergent devient insynchronisable en quelques jours.

## 3. ArtCraft (IA générative)

| Repo                | Rôle                                                                 | Stack                                                         | Licence                                    | Rust   | TS     |
|---------------------|----------------------------------------------------------------------|---------------------------------------------------------------|--------------------------------------------|--------|--------|
| `artcraft`          | App desktop « IDE pour artistes » : compo 2D/3D + génération image/vidéo/3D | Tauri 2 + React/Nx/Vite/Zustand/Three.js, SQLite (sqlx) ; `artcraft-native` (egui) en parallèle | MIT / Apache-2.0                           | 269 k  | 207 k  |
| `artcraft-services` | Backend `storyteller-web` (API, billing Stripe, crédits, workers), webapps | Actix Web, Tokio, MySQL/SQLx, Redis, Elasticsearch, S3/R2, Nx | **ArtCraft License (WIP) – « fair source »** | 484 k  | 264 k  |
| `artcraftx`         | Variante minimaliste de l'app desktop (expérimentale)               | Tauri 2 + React                                               | **Aucune licence** (= tous droits réservés)  | 245 k  | 59 k   |
| `cloud-worker`      | Scripts RunPod/ComfyUI pour benchmark MiniMax H3                     | Bash, Python                                                  | **Aucune licence**                         | –      | –      |

Points d'architecture :

- **Flux de génération** : client → `storyteller-web` → `artcraft_router` → providers (FAL, GMI Cloud, Seedance, Kling, Veo…) → webhook ou polling worker → stockage objet → CDN. 62 modèles catalogués (image, vidéo, audio, mesh, splats).
- **Duplication par copier-coller entre repos** (pas de package partagé) :

  | Zone                     | `artcraft` vs `artcraft-services`         |
  |--------------------------|-------------------------------------------|
  | `crates/api_clients`     | 1 943 fichiers identiques / 2 039 communs |
  | `crates/schema`          | 230 / 238                                 |
  | `frontend/libs` (TS)     | 1 307 / 1 380                             |

  Conséquence : une partie du code de `artcraft-services` (licence restrictive) existe en clair sous MIT/Apache dans `artcraft`. À vérifier juridiquement avant toute réutilisation.
- **Intégrations « consumer »** (Sora, Grok, Midjourney, Kinovi) : capture des cookies de session de l'utilisateur via webview Tauri + usurpation d'empreinte TLS/HTTP2 navigateur (`browser_emulation` sur `wreq`, `cloudflare_mitigation`, gestion `cf_clearance`). **Risque CGU élevé** pour tout redistributeur.
- **CI** : publication macOS/Windows et image Docker uniquement ; aucun job `cargo test` en CI publique sur `artcraft` ni `artcraft-services`.

Licence `artcraft-services` (extraits) : usage personnel uniquement ; interdit de vendre le logiciel, de développer un produit concurrent (outils IA image/vidéo), de forker en retirant liens communauté/dons/services payants, d'utiliser nom/logo.

## 4. Craft App Suite

Architecture commune à toutes les apps (gabarit identique) :

```text
crates/   L0 geom/color/codecs…  → L1 doc (modèle pur)  → L2 ops/algos → L3 compose/gpu
          → L4 io/plugins → L5 engine (Session + registre de commandes) → L6 ui-egui, automation/mcp
apps/     <app> (eframe + wgpu, canal de contrôle TCP) · <app>-cli (headless, batch, MCP) · <app>-web (wasm, trunk)
xtask/    layers | wasm | ci | corpus | parity | perf | scorecard
```

- « **Everything is a command** » : UI, CLI, canal JSON et serveur MCP appellent le même registre (500+ commandes sur PhotoCraft). Pilotage par agents natif.
- Rust edition 2024, MSRV 1.90–1.95, egui 0.36, wgpu, rayon ; pas de Tauri/webview (interdit par `AGENTS.md`).
- Distribution : macOS (DMG notarisé), Windows (MSI, ARM64), Linux (AppImage/deb/rpm/Flatpak), FreeBSD, Web.

| App           | Cible clean-room   | Version | Stade      | « Ready for real work » | Rust LOC | Crates | Tests | Tests/kLOC |
|---------------|--------------------|---------|------------|-------------------------|----------|--------|-------|------------|
| `photocraft`  | Photoshop          | 0.7.0   | alpha      | ~45 %                   | 436 k    | 36     | 6 202 | 14,2       |
| `vectorcraft` | Illustrator        | 0.9.0   | alpha      | ~62 %                   | 377 k    | 30     | 5 321 | 14,1       |
| `filmcraft`   | Premiere Pro       | 0.6.0   | alpha      | ~55 %                   | 360 k    | 50     | 2 792 | 7,8        |
| `effectcraft` | After Effects      | 0.8.0   | alpha      | ~45 %                   | 338 k    | 38     | 2 854 | 8,4        |
| `lightcraft`  | Lightroom          | 0.6.0   | alpha      | ~55 %                   | 196 k    | 29     | 2 517 | 12,9       |
| `pdfcraft`    | Acrobat            | 0.6.0   | alpha      | 49 %                    | 173 k    | 36     | 2 161 | 12,5       |
| `designcraft` | InDesign           | 0.6.0   | pré-alpha  | ~47 %                   | 128 k    | 21     | 1 156 | 9,0        |
| `deckcraft`   | PowerPoint         | 0.5.0   | alpha      | ~55 %                   | 117 k    | 28     | 506   | 4,3        |
| `wordcraft`   | Word               | 0.5.0   | alpha      | 60 %                    | 114 k    | 21     | 1 097 | 9,6        |
| `cadcraft`    | AutoCAD            | 0.5.0   | pré-alpha  | ~38 %                   | 91 k     | 17     | 932   | 10,3       |
| `gridcraft`   | Excel              | 0.5.0   | alpha      | ~50 %                   | 82 k     | 17     | 590   | 7,2        |
| `soundcraft`  | Pro Tools          | 0.5.0   | pré-alpha  | ~40 %                   | 72 k     | 21     | 549   | 7,6        |

(« Ready » = auto-évaluation upstream dans `ROADMAP.md` / `docs/target-app-parity.md`.)

Qualité / sécurité :

- `unsafe` confiné : FFI décodage matériel (`filmcraft/crates/platform` : VideoToolbox, VAAPI, NVDEC, Media Foundation), hôtes de plugins audio (`soundcraft` : VST3, CLAP, AU), tablette macOS (`photocraft-tablet`).
- Tests CI : seuls `photocraft`, `pdfcraft`, `deckcraft` ont un `ci.yml` exécutant les tests ; les autres ne testent qu'en FreeBSD / Windows ARM64 dans les workflows publics.
- Fichiers très gros côté UI (`photocraft/crates/ui-egui/src/canvas.rs` 6,6 k lignes, `designcraft/.../dialogs.rs` 4,6 k) : dette de découpage.

### Duplication inter-apps (pas de noyau partagé)

`craft-libs` ne contient qu'**un** crate (`craft-ui`, 5,7 k LOC). Tout le reste est réimplémenté ou copié-divergé :

| Crate        | Présent dans | Remarque                                                        |
|--------------|--------------|-----------------------------------------------------------------|
| `engine`     | 12 apps      | même pattern (Session + registre de commandes), code distinct   |
| `ui-egui`    | 12 apps      | quasiment aucun fichier identique                               |
| `geom`       | 10 apps      | réimplémenté 10 fois                                            |
| `color`      | 8 apps       | idem                                                            |
| `mcp`        | 7 apps       | serveur MCP réécrit par app                                     |
| `av1`, `h264`, `hevc`, `vp9`, `opus`, `isobmff`, `matroska` | `filmcraft` + `deckcraft` | copiés puis divergés (ex. `av1` : 12/21 fichiers encore identiques) |

## 5. Risques pour un fork

| Risque                          | Niveau  | Détail                                                                                                     |
|---------------------------------|---------|------------------------------------------------------------------------------------------------------------|
| Licence `artcraft-services`     | Bloquant | Non-concurrence + usage commercial interdit                                                               |
| Absence de licence              | Bloquant | `artcraftx`, `cloud-worker` : aucun droit de réutilisation ; `craft-repo-status-app` : `MIT` dans Cargo.toml seulement |
| CGU providers                   | Élevé   | Clients « consumer » Sora/Grok/Midjourney avec usurpation d'empreinte navigateur                           |
| Vélocité upstream               | Élevé   | 100–370 commits/jour → un fork modifié diverge en jours                                                    |
| Standards privés                | Moyen   | `AGENTS.md` renvoie à `../craftrules` (repo privé) : règles de contribution partiellement inaccessibles   |
| Marques                         | Moyen   | Logos ArtCraft hors licence (`docs/brand/LICENSE-brand.txt`) ; mentions Adobe/Microsoft/Autodesk en usage nominatif |
| Maturité                        | Moyen   | Alpha/pré-alpha ; parité auto-déclarée par les agents                                                      |
| Toolchain                       | Faible  | Rust ≥ 1.95 pour 6 apps                                                                                   |

## 6. Ce qui est réutilisable (MIT/Apache, sans risque licence)

- **Crates autonomes, sans dépendance workspace** (publiables) : `photocraft-psd` (PSD/PSB, round-trip 307/309 sur psd-tools), `photocraft-codecs`, `photocraft-cms` (ICC pur Rust).
- **Autres crates « format » à fort intérêt** (dépendances internes à vérifier crate par crate) : `affinity` (lecture .af*), `pdfcraft/{forms,sign,redact,ocr,preflight}`, `gridcraft/{formula,functions,xlsx}`, `wordcraft/{docx,layout}`, `cadcraft/{dxf,dwg}`.
- **CLI headless + MCP** de chaque app : intégrables côté serveur sans toucher au code.
- **Pattern d'architecture** : moteur de commandes unique exposé UI/CLI/MCP, layering vérifié par `xtask`, garde-fous « never crash ».

## 7. Pistes (à arbitrer)

1. **Forks « miroir »** : fork sans modifier `main`, travail sur branches, sync quotidien (`gh repo sync` ou workflow planifié). Seule stratégie viable vu la vélocité.
2. **Consommer plutôt que forker** : dépendre des crates par `git = …, rev = …` (comme upstream le fait pour `craft-libs`) et n'en forker qu'un sous-ensemble.
3. **Intégration dans une stack Symfony / React** :
   - Backend : worker Symfony Messenger qui pilote `<app>-cli` (`Process`) pour conversion / batch (PSD → PNG, PDF redact/sign, XLSX calc) ; ou client MCP.
   - Front React 19 : embarquer les builds `<app>-web` (wasm) en iframe / web component.
4. **Contribution upstream à fort levier** : extraire `geom`, `color`, `mcp`, codecs vidéo dans `craft-libs` (dédoublonnage 8–12×).
5. **Ne pas réutiliser** `artcraft-services`, `artcraftx`, `cloud-worker` ni les clients « consumer » sans validation juridique.

## Annexe : forker

```bash
# depuis un poste avec gh authentifié
./scripts/fork-artcraft.sh                 # vers ton compte
./scripts/fork-artcraft.sh --org futur-code
DRY_RUN=1 ./scripts/fork-artcraft.sh       # simulation
```

Une fois les forks créés sous `maximemarc/*`, ils peuvent être attachés à une session Claude Code (droits push) pour y travailler directement.
