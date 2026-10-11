# Audit des sondes et envois externes

> Analyse statique (aucun build ni exécution) des 21 repos de `upstream/` aux commits de [`upstream/UPSTREAM.lock`](../../upstream/UPSTREAM.lock).
> Inventaire brut : [`external-calls.inventory.json`](./external-calls.inventory.json), régénéré par `python3 -I scripts/audit-external-calls.py`.
> « Statut estimé » = atteignabilité depuis le binaire livré + conditions d'activation (flags, CSP, config, features Cargo). Non confirmé par capture réseau.

> **Mise à jour.** L'app Tauri + React d'`artcraft` (sondes 1 à 5 ci-dessous) n'est plus vendorisée dans ce repo (`upstream/repos.conf`). Les constats restent valables pour l'upstream et pour `artcraftx` (submodule). [Craft Studio](../craft-studio.md) n'embarque d'ArtCraft que `artcraft-native` : aucune sonde, seulement l'API de génération quand un compte est connecté.

## Synthèse

| Périmètre                         | Télémétrie                                                         | Autres envois externes                                      |
|-----------------------------------|--------------------------------------------------------------------|-------------------------------------------------------------|
| **Craft Suite** (12 apps)         | **Aucune.** `designcraft` et `pdfcraft` affichent « No telemetry » | Uniquement sur action explicite (modèles IA, vérif MAJ manuelle, liens) |
| `craft-libs`, `craft-fonts`, `photocraft-corpus` | Aucune                                              | Aucun                                                       |
| `craft-launcher`                  | Aucune sonde dédiée                                                | Vérif MAJ **toutes les heures par défaut** (feed ArtCraft)  |
| **ArtCraft desktop** (`artcraft`, `artcraftx`) | **3 sondes actives** + 2 câblées mais neutralisées    | API ArtCraft + fournisseurs IA (fonctionnel)                |
| `artcraft-services` (backend)     | Observabilité serveur + GA4 sur les sites web                      | Fournisseurs IA, Stripe, SMTP, stockage                     |

## 1. ArtCraft desktop (`artcraft`, `artcraftx`)

Les deux apps partagent le même code (Tauri 2 + webview React). Les chemins ci-dessous sont ceux d'`artcraft`.

### Sondes

| # | Sonde                        | Destination                                         | Données envoyées                                                                                          | Déclenchement                                                                          | Statut estimé |
|---|------------------------------|-----------------------------------------------------|-----------------------------------------------------------------------------------------------------------|----------------------------------------------------------------------------------------|---------------|
| 1 | **Microsoft Clarity**        | `*.clarity.ms` (projet `sixp9uu263`)                | Session replay : DOM, clics, scroll, mouvements, viewport, navigation interne                             | Snippet inline dans `frontend/apps/artcraft/app/index.html`, à chaque lancement ; CSP `script-src`/`connect-src` autorisent `*.clarity.ms` | **Actif** |
| 2 | **Heartbeat d'usage**        | `POST api.storyteller.ai/v1/analytics/active_user_v2` | Token de session aléatoire, nom/version app, OS + version, durée de session, compteurs de générations (image, vidéo, 3D, t2i, i2v…) et de prompts par page | Thread Rust lancé au démarrage (`spawn_storyteller_threads.rs`) ; n'envoie que si un compte ArtCraft est connecté ; cadence imposée par le serveur (`wait_for_retry_millis`) | **Actif si connecté** |
| 3 | **Discord Rich Presence**    | IPC locale → client Discord → serveurs Discord      | App ID `1366596912593113138`, statut « Crafting Arts with ArtCraft » visible des contacts                  | `spawn_discord_presence_thread()` inconditionnel, rafraîchi toutes les 60 s, sans option | **Actif si Discord est ouvert** |
| 4 | Google Analytics 4           | `googletagmanager.com` / `google-analytics.com` (`G-Y4Y8KMLPQ7`) | Page views, events `open_gallery_modal`, `switch_tab`, `open_settings_modal`, `open_billing_settings`, login avec **`user_id` = token utilisateur** | Snippet dans `index.html` + `libs/analytics/google-analytics` | **Bloqué par la CSP** : `googletagmanager.com` absent de `script-src`. Un ajout à la CSP le réactive |
| 5 | PostHog                      | `us.i.posthog.com`                                  | Événements et feature flags (`posthog-js`)                                                                | `posthog.init()` en build prod avec `VITE_POSTHOG_API_KEY`                             | **Inactif** : variable définie nulle part (les `.env` déclarent `REACT_APP_PUBLIC_POSTHOG_KEY`), host absent de `connect-src`, hook de feature flag jamais appelé |

Résiduel, sans effet aujourd'hui :

| Élément                                   | Constat                                                                                 |
|-------------------------------------------|-----------------------------------------------------------------------------------------|
| Google Fonts                              | CSS chargée depuis `fonts.googleapis.com` au lancement (IP/UA vers Google) ; fichiers de police bloqués par `font-src` |
| Gravatar (`ProfileDropdown`)              | Image via hash d'email ; `gravatar.com` absent d'`img-src` → bloquée                     |
| `unpkg.com/detect-gpu` dans la CSP        | Aucun usage dans le code de l'app                                                       |
| `midjourney_client::prompt_session_log`   | Réplique de la télémétrie web de Midjourney ; jamais appelé (code mort)                  |
| `artcraft-native` (app egui)              | Aucune sonde ; API ArtCraft + CDN uniquement                                             |

### Envois fonctionnels

| Destination                                                        | Rôle                                               | Condition                                   |
|--------------------------------------------------------------------|----------------------------------------------------|---------------------------------------------|
| `api.storyteller.ai`, `cdn.storyteller.ai`, `cdn.artcraft.ai`      | Auth, génération, crédits, médias                  | Compte ArtCraft                             |
| `sora.chatgpt.com`, `grok.com` / `x.ai`, `midjourney.com`, `marble.worldlabs.ai`, `securetoken.googleapis.com` | Fournisseurs « consumer » via cookies capturés en webview + empreinte TLS de navigateur (`wreq`) | L'utilisateur connecte son compte fournisseur |
| `queue.fal.run`, `*.fal.media`                                     | FAL en direct (clé API fournie par l'utilisateur)  | Clé `fal.api_key.txt` présente              |
| En-têtes `sentry-trace` / Statsig vers Grok                        | Imitation du client web grok.com (pas de télémétrie ArtCraft) | Usage de Grok                     |

## 2. Craft Suite

Constat commun : pas de SDK d'analytics, pas de crash reporter, pas de heartbeat. Serveur de contrôle JSON et pont MCP **opt-in** (`--control <port>` ou `<APP>_CONTROL_PORT`), liés à `127.0.0.1`. Builds web : `fetch` en même origine uniquement. Liens `getartcraft.com` et `discord.gg` ouverts dans le navigateur au clic.

| App           | Envois externes                                                                                      | Déclenchement                                    | Statut |
|---------------|------------------------------------------------------------------------------------------------------|--------------------------------------------------|--------|
| `photocraft`  | Aucun. `photocraft-cli serve` : TCP avec token, adresse fournie par l'utilisateur                     | Commande explicite                               | Opt-in |
| `lightcraft`  | Modèles IA (masquage SAM, visages) depuis `github.com` / `huggingface.co`, vérifiés (taille, SHA-256 quand épinglé) ; SAM 3 distant via `LIGHTCRAFT_SAM3_REMOTE` (loopback imposé) | Commandes `faces.models.download`, masques IA | Sur action |
| `filmcraft`   | Modèles Whisper / TTS depuis `huggingface.co`                                                        | Feature Cargo `whisper` désactivée par défaut et absente du workflow de release | Désactivé |
| `pdfcraft`    | `api.github.com/repos/storytold/pdfcraft/releases/latest`                                            | Menu Aide › Vérifier les mises à jour, rien d'automatique | Sur action |
| `effectcraft` | Modèles MediaPipe (`storage.googleapis.com`), MobileSAM (`raw.githubusercontent.com`) via un sous-process `curl` | Commande de téléchargement de modèle | Sur action |
| `wordcraft`   | Connecteur Zotero local (`127.0.0.1`)                                                                | Insertion de citation                            | Local |
| `vectorcraft`, `designcraft`, `gridcraft`, `cadcraft`, `deckcraft`, `soundcraft` | Aucun | – | – |

Validation de signatures PDF hors ligne (pas de récupération CRL/OCSP). Modèles OCR de `pdfcraft` embarqués dans les releases.

## 3. Infra partagée

| Repo                    | Envois externes                                                                                         | Statut |
|-------------------------|---------------------------------------------------------------------------------------------------------|--------|
| `craft-launcher`        | `GET api.storyteller.ai/v1/craft_apps/release_info` (repli : API GitHub), UA `ArtCraftLauncher/<version>` ; téléchargement des releases GitHub. Défauts : `check_every = Hour`, `close_to_tray = true`, `auto_update = false` | **Actif par défaut** : pas de donnée personnelle, mais IP + version visibles d'ArtCraft toutes les heures. Désactivable (« Never ») |
| `craft-repo-status-app` | API GitHub (outil interne de suivi des PR)                                                              | Outil dev |
| `cloud-worker`          | Scripts RunPod / Hugging Face                                                                            | Ops dev |

## 4. Backend `artcraft-services` (submodule)

Hors périmètre d'une app desktop, sauf auto-hébergement.

| Élément                                        | Statut                                                        |
|------------------------------------------------|---------------------------------------------------------------|
| Métriques Datadog (`datadog_client`, middleware Actix) | Actif si identifiants Datadog configurés                |
| OpenTelemetry OTLP                             | Uniquement dans `service/job/deprecated/inference_job`        |
| New Relic                                      | Vendorisé mais retiré du workspace (mort)                     |
| GA4 sur `artcraft-webapp` (`G-VKW89949V0`) et `artcraft-website-next` | Actif sur les sites web                 |
| Récepteur du heartbeat `/v1/analytics/active_user*` | Côté serveur de la sonde n° 2                            |
| Stripe, SMTP, Google Sign-In, R2/S3, Redis, Elasticsearch, fournisseurs IA | Fonctionnel                       |

## 5. Pour l'app unifiée

1. **À retirer** : snippets Clarity et GA d'`index.html`, lib `@storyteller/google-analytics`, `posthog-js`, domaines analytics de la CSP Tauri.
2. **Opt-in explicite** : heartbeat d'usage, Discord Rich Presence, vérification de mises à jour.
3. **À garder** : téléchargements de modèles sur action avec checksum (pattern déjà en place dans lightcraft / filmcraft).
4. **Garde-fou d'architecture** : un seul crate `net` autorisé à dépendre de `reqwest` / `ureq` / `wreq` (bans `cargo-deny` + règle de layering `xtask`), une `EgressPolicy` (allowlist, consentement, journal) par laquelle passent tous les appels sortants, et une CSP sans domaine tiers par défaut.
5. **Gate CI** : relancer `scripts/audit-external-calls.py` après chaque `upstream-sync.sh` et faire échouer le job sur toute nouvelle catégorie de télémétrie ou tout nouveau host.
