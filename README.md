# cloudtools

Monorepo regroupant l'écosystème ArtCraft ([github.com/storytold](https://github.com/storytold)) en vue d'une application unifiée.

## Layout

```text
upstream/                 code importé, jamais modifié à la main
  repos.conf              liste des repos, mode d'import, licence
  UPSTREAM.lock           commit upstream épinglé par repo
  <repo>/                 snapshot vendorisé (licence MIT / Apache-2.0)
  artcraft-services/      submodule (licence « usage personnel »)
  artcraftx/              submodule (aucune licence)
  cloud-worker/           submodule (aucune licence)
scripts/
  upstream-sync.sh        import / mise à jour depuis l'upstream
  audit-external-calls.py inventaire statique télémétrie / réseau / hosts
  fork-artcraft.sh        fork GitHub des repos upstream
docs/artcraft/
  ANALYSIS.md             analyse d'architecture et de licences
  EXTERNAL-CALLS.md       audit des sondes et envois externes
  external-calls.inventory.json  sortie brute du scanner
```

Les repos sans licence permissive restent en submodules : ce repo étant public, leur code n'y est pas redistribué.

## Mettre à jour l'upstream

```bash
./scripts/upstream-sync.sh                     # tout
./scripts/upstream-sync.sh photocraft pdfcraft # sélection
git add -f upstream/ .gitmodules               # -f : les .gitignore upstream ne doivent pas filtrer des fichiers suivis
python3 -I scripts/audit-external-calls.py     # puis diff de docs/artcraft/external-calls.inventory.json
git commit -m "Sync upstream photocraft, pdfcraft"
```

Cloner avec les submodules : `git clone --recurse-submodules --shallow-submodules <url>`.

Chaque workspace upstream reste autonome (`cd upstream/photocraft && cargo build`). Il n'y a pas encore de workspace Cargo racine.
