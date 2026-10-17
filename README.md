# cloudtools

L'écosystème ArtCraft ([github.com/storytold](https://github.com/storytold)) regroupé dans une seule application : **Craft Studio**.

## Installer

Installateurs Windows (`setup.exe`, `.zip`), macOS (`.dmg`, Apple silicon) et Linux (`.deb`, `.tar.gz`) dans les [Releases](https://github.com/maximemarc/cloudtools/releases) ; premier lancement et prérequis dans les notes de chaque version.

## Lancer Craft Studio depuis les sources

Prérequis : Rust 1.95+ ; sous Linux `pkg-config` et `libasound2-dev` (détails par OS dans [docs/craft-studio.md](docs/craft-studio.md)).

```bash
git clone --recurse-submodules --shallow-submodules https://github.com/maximemarc/cloudtools
cd cloudtools
cargo run --release -p craft-studio                     # les 13 apps (premier build : 20–40 min)
cargo run --release -p craft-studio -- --app pdfcraft   # ouvrir directement une app
cargo run --release -p craft-studio -- --list           # identifiants des apps
```

CI ([.github/workflows/craft-studio.yml](.github/workflows/craft-studio.yml)) : build release sur Linux, macOS et Windows, puis `--smoke` (les 13 apps ouvertes tour à tour, une capture chacune) sur les trois OS, en rendu logiciel ou virtualisé (Xvfb + Mesa, Metal paravirtualisé, WARP). Captures dans les artefacts `smoke-<OS>`.

## Publier une version

```bash
# 1. version dans Cargo.toml ([workspace.package] version = "0.2.0"), commit sur main
# 2. optionnel : Actions › Craft Studio › Run workflow, pour essayer les installateurs (artefacts packages-<OS>)
git tag v0.2.0 && git push origin v0.2.0   # build + smoke + paquets sur les 3 OS, puis la release
```

Le job `release` refuse un tag qui ne correspond pas à la version du `Cargo.toml` ; un tag `vX.Y.Z-rc.N` publie une pré-version.

## Layout

```text
apps/craft-studio/        le shell : fenêtre, barre latérale, accueil, bascule entre apps
crates/studio-core/       contrat shell <-> modules
crates/studio-overlay/    réécriture au build des sources des binaires upstream
crates/modules/<app>/     un adaptateur par app (Cargo.toml généré + upstream/studio.rs)
upstream/                 code importé, jamais modifié à la main
  repos.conf              repos, mode d'import, exclusions, licence
  UPSTREAM.lock           commit upstream épinglé par repo
  <repo>/                 snapshot vendorisé (licence MIT / Apache-2.0)
  artcraft-services/      submodule (licence « usage personnel »)
  artcraftx/              submodule (aucune licence)
  cloud-worker/           submodule (aucune licence)
LICENSE-MIT, LICENSE-APACHE  licence du code de ce repo (MIT OR Apache-2.0, comme les apps Craft)
patches/<repo>/           correctifs appliqués après chaque import (vulnérabilités, intégration)
apps/craft-studio/packaging/  installateurs : package.sh, .desktop, Info.plist, script Inno Setup, notes de version
.github/workflows/        CI Craft Studio (build + smoke sur Linux, macOS, Windows ; paquets et release sur tag)
scripts/
  upstream-sync.sh        import / mise à jour depuis l'upstream (+ patches + lockfiles)
  gen-module-manifests.py dépendances des adaptateurs depuis les binaires upstream
  fix-lockfile-vulns.py   montées de version semver-compatibles des crates vulnérables
  osv-check.py            avis GitHub (= Dependabot) sur tous les lockfiles
  third-party-licenses.py notices de licence des crates et repos upstream embarqués (paquets)
  audit-external-calls.py inventaire statique télémétrie / réseau / hosts
  fork-artcraft.sh        fork GitHub des repos upstream
docs/
  craft-studio.md         lancement, architecture, limites
  artcraft/ANALYSIS.md    analyse d'architecture et de licences
  artcraft/EXTERNAL-CALLS.md  audit des sondes et envois externes
  security.md             état des vulnérabilités (corrigées / acceptées)
```

Les repos sans licence permissive restent en submodules : ce repo étant public, leur code n'y est pas redistribué.

## Mettre à jour l'upstream

```bash
./scripts/upstream-sync.sh                     # tout (ou: ./scripts/upstream-sync.sh photocraft pdfcraft)
python3 -I scripts/gen-module-manifests.py
cargo check -p craft-studio && cargo audit
python3 -I scripts/audit-external-calls.py     # puis diff de docs/artcraft/external-calls.inventory.json
git add -f upstream/ .gitmodules               # -f : les .gitignore upstream ne doivent pas filtrer des fichiers suivis
git commit -m "Sync upstream"
```

Un patch qui ne s'applique plus arrête la sync : il faut le rebaser sur le nouvel upstream (`patches/<repo>/`).
