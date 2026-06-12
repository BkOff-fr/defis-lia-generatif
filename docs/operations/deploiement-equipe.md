# Déploiement du Mode Équipe en Docker (kit PME — C40)

> ⚠️ **Images non testées en CI à ce jour — voir C40.** Les chemins, noms et
> versions de ce kit sont vérifiés contre le repo (rust-embed, binaire,
> `engines` npm), mais aucun build d'image n'est encore exécuté en CI. Au
> premier déploiement, gardez la section [Dépannage](#10-dépannage) sous la main.
>
> **Public** : responsable IT ou RSE d'une PME. **Objectif** : serveur
> opérationnel en ~30 minutes (hors première compilation de l'image).
>
> **Ce que ce guide ne re-documente pas** : le déploiement binaire nu
> (systemd, hardening, alertes, SMTP) est couvert par
> [`team-aggregator.md`](team-aggregator.md). Les règles privacy
> (k-anonymat, opt-in salarié, rétention) sont décrites dans sa section
> **« Privacy et conformité (ADR-0015 — C38) »** : le présent guide s'y
> réfère sans la dupliquer.

---

## Table des matières

1. [Prérequis](#1-prérequis)
2. [Contenu du kit](#2-contenu-du-kit)
3. [Démarrage en 6 étapes](#3-démarrage-en-6-étapes)
4. [Option A — certificat auto-signé (défaut)](#4-option-a--certificat-auto-signé-défaut)
5. [Option B — reverse proxy TLS (Caddy + Let's Encrypt)](#5-option-b--reverse-proxy-tls-caddy--lets-encrypt)
6. [Créer les codes d'enrôlement](#6-créer-les-codes-denrôlement)
7. [Régler la privacy (k-anonymat, rétention)](#7-régler-la-privacy-k-anonymat-rétention)
8. [Sauvegarder le volume (SQLite + WAL)](#8-sauvegarder-le-volume-sqlite--wal)
9. [Mettre à jour l'image](#9-mettre-à-jour-limage)
10. [Dépannage](#10-dépannage)
11. [Voir aussi](#11-voir-aussi)

---

## 1. Prérequis

| Quoi | Détail |
| --- | --- |
| Docker Engine ≥ 24 | BuildKit actif par défaut (requis pour `Dockerfile.dockerignore`) |
| Docker Compose v2 | vérifier : `docker compose version` |
| Le repo cloné | **aucune image n'est publiée sur un registre à ce jour** : build local |
| Machine de build | 2 vCPU / 4 Go RAM ; ~2 Go de disque pour les images |
| Machine d'exécution | très modeste : < 256 Mo RAM pour 10-100 salariés |
| Réseau | port `8443` libre (Option A) ou `80` + `443` (Option B) |

> ⏱️ **Première compilation : 10 à 25 minutes** (profil release frugal du
> workspace : `opt-level = "z"`, `lto = true`). Les builds suivants
> réutilisent le cache Docker et sont bien plus rapides.

## 2. Contenu du kit

| Fichier | Rôle |
| --- | --- |
| `deploy/team/Dockerfile` | image multi-stage : build SvelteKit (`web-team/`) → build Rust (le dashboard est **embarqué dans le binaire** via rust-embed) → runtime `debian:stable-slim` non-root |
| `deploy/team/Dockerfile.dockerignore` | réduit le contexte de build (BuildKit) |
| `deploy/team/entrypoint.sh` | `init` automatique au premier démarrage, puis `serve` |
| `deploy/team/docker-compose.yml` | service unique + volume nommé `data` |
| `docs/operations/modeles-communication.md` | modèles d'emails CSE + salariés, **à envoyer avant la mise en service** |

Tout l'état du serveur (base SQLite, certificat TLS, clé privée) vit dans le
volume nommé `sobria-team_data`, monté sur `/data` dans le conteneur.

## 3. Démarrage en 6 étapes

```bash
# 1. Cloner le repo et se placer dans le dossier du kit.
git clone <URL-du-repo> sobria && cd sobria/deploy/team

# 2. Créer le fichier .env (mot de passe admin initial, jamais commité).
cat > .env <<'EOF'
SOBRIA_TEAM_ADMIN_PASSWORD=remplacez-par-une-passphrase-32-caracteres-mini
# SOBRIA_TEAM_ADMIN_USERNAME=admin
# Nom sous lequel vos salariés joindront le serveur (inscrit dans les SANs
# du certificat auto-signé généré au premier démarrage) :
# SOBRIA_TEAM_HOSTNAME=sobria.interne.exemple.fr
EOF
chmod 600 .env

# 3. Builder et démarrer (1ʳᵉ fois : comptez 10-25 min de compilation).
docker compose up -d --build

# 4. Suivre l'init puis le démarrage.
docker compose logs -f sobria-team
#   attendu : « [entrypoint] Premier démarrage : init… » puis
#             « [entrypoint] Lancement : serve --bind 0.0.0.0 --port 8443 »
```

```bash
# 5. Vérifier la santé (l'option -k accepte le certificat auto-signé).
curl -k https://localhost:8443/health
#   → {"ok":true,"version":"0.9.0"}
```

Ouvrir ensuite `https://<hôte>:8443/` dans un navigateur, **accepter
l'avertissement de certificat** (voir [Option A](#4-option-a--certificat-auto-signé-défaut)),
et se connecter avec l'admin créé à l'étape 2.

```bash
# 6. L'init est faite : retirer le secret du .env (il ne sert plus),
#    puis recréer le conteneur sans la variable.
sed -i 's/^SOBRIA_TEAM_ADMIN_PASSWORD=.*/# mot de passe retiré apres init/' .env
docker compose up -d
```

> Le conteneur tourne non-root (UID 10001), avec
> `no-new-privileges`, et redémarre seul (`restart: unless-stopped`),
> y compris après un reboot de la machine (si le démon Docker est activé).

## 4. Option A — certificat auto-signé (défaut)

Au premier démarrage, `init` génère un certificat TLS auto-signé
(validité 10 ans) dont les SANs couvrent `localhost`, `127.0.0.1`, `::1`
et le **hostname du conteneur** — d'où l'intérêt de poser
`SOBRIA_TEAM_HOSTNAME` dans le `.env` *avant* le premier démarrage.

### Accepter l'empreinte (chaque poste salarié)

1. Ouvrir `https://<hôte>:8443/health` dans le navigateur.
2. « Avancé » → « Continuer vers le site » (formulation selon navigateur).
3. Dans l'extension Sobr.ia : Options → Mode Équipe → « Vérifier ».

Détails et cas d'erreur : `team-aggregator.md` § Troubleshooting
(« Certificat refusé par l'extension »).

### Vérifier l'empreinte (recommandé)

Pour écarter une interception, comparez l'empreinte vue par le salarié
(détails du certificat dans le navigateur) avec celle du serveur :

```bash
docker compose cp sobria-team:/data/cert.pem .
openssl x509 -in cert.pem -noout -fingerprint -sha256 && rm cert.pem
```

### Limites assumées de l'Option A

- Avertissement à accepter **sur chaque poste et chaque navigateur** ;
- le nom DNS utilisé doit correspondre au hostname inscrit dans le cert,
  sinon avertissement supplémentaire de non-correspondance ;
- à chaque rotation du certificat, tous les clients ré-acceptent ;
- réservée au **LAN / VPN**. Pour une exposition Internet (télétravail
  sans VPN) ou pour supprimer tout avertissement : Option B.

### Rotation du certificat

```bash
docker compose stop sobria-team
docker compose run --rm sobria-team serve --regen-cert
#   → notez l'empreinte SHA-256 affichée, puis Ctrl+C
#     (ce conteneur éphémère n'expose aucun port).
docker compose up -d
```

L'ancien cert/clé est sauvegardé en `*.pem.bak.<timestamp>` dans `/data`.
Prévenez les utilisateurs : ils devront ré-accepter la nouvelle empreinte.

## 5. Option B — reverse proxy TLS (Caddy + Let's Encrypt)

Recommandée si le serveur est joignable depuis Internet, ou pour offrir un
certificat reconnu sans aucune manipulation côté salariés.

**Prérequis Let's Encrypt** : un nom DNS public (ex. `team.exemple.fr`)
pointant vers la machine, ports `80` et `443` ouverts depuis Internet.
Caddy obtient et **renouvelle automatiquement** le certificat.

### `deploy/team/Caddyfile` (deux blocs)

```caddyfile
{
	# Bloc global — email de contact ACME (rappels d'expiration Let's Encrypt).
	email admin@exemple.fr
}

# Bloc site — Caddy termine le TLS public et proxifie vers l'aggregator
# via le réseau interne Compose (le port 8443 n'est plus publié).
team.exemple.fr {
	reverse_proxy https://sobria-team:8443 {
		transport http {
			tls
			tls_insecure_skip_verify  # cert auto-signé interne de l'aggregator
		}
	}
}
```

> `tls_insecure_skip_verify` : le flux Caddy → aggregator reste chiffré mais
> n'est pas authentifié. Acceptable ici : les deux conteneurs partagent un
> réseau bridge privé sur la même machine (même compromis que dans
> `team-aggregator.md` § Pour les DSI).

### Adapter `docker-compose.yml`

```yaml
services:
  sobria-team:
    # … tout reste identique, mais SUPPRIMEZ la section `ports:` :
    # l'aggregator n'est plus joignable que par Caddy, en interne.

  caddy:
    image: caddy:2
    restart: unless-stopped
    ports:
      - "80:80"    # challenge ACME + redirection HTTPS
      - "443:443"
    volumes:
      - ./Caddyfile:/etc/caddy/Caddyfile:ro
      - caddy-data:/data      # certificats Let's Encrypt persistés
      - caddy-config:/config

volumes:
  data:
  caddy-data:
  caddy-config:
```

Les salariés utilisent alors `https://team.exemple.fr` directement —
aucune empreinte à accepter, ni dans le navigateur ni dans l'extension.
Variante nginx : voir `team-aggregator.md` § Pour les DSI.

## 6. Créer les codes d'enrôlement

Chaque salarié s'enrôle avec un **code à 12 chiffres à usage unique**
(TTL 7 jours par défaut, hashé Argon2id en base après affichage).

### Via l'UI

`https://<hôte>:8443/admin/codes` (connexion admin) : création par lot,
liste des états (actif / utilisé / expiré / révoqué), révocation.

### Via la CLI (conteneur en marche)

```bash
docker compose exec sobria-team \
  sobria-team-aggregator --data-dir /data code create 10 --ttl-days 7
```

Les codes sont **affichés une seule fois** : distribuez-les par un canal
sûr (gestionnaire de mots de passe, message direct — pas de liste en
clair sur un wiki). Gestion :

```bash
docker compose exec sobria-team sobria-team-aggregator --data-dir /data code list
docker compose exec sobria-team sobria-team-aggregator --data-dir /data code revoke <ULID>
```

> Conteneur arrêté ? `docker compose run --rm sobria-team code create 10 --ttl-days 7`
> (l'entrypoint relaie toute sous-commande au binaire avec `--data-dir /data`).
> Si votre admin initial ne s'appelle pas `admin`, ajoutez `--admin <username>`.

Côté salarié (installation de l'extension, enrôlement) : voir
`team-aggregator.md` § Quickstart étape 5, et le modèle d'annonce prêt à
envoyer dans [`modeles-communication.md`](modeles-communication.md).

## 7. Régler la privacy (k-anonymat, rétention)

Les garanties (k-anonymat des agrégats, identification opt-in contrôlée
par le salarié, purge de rétention) sont appliquées **côté serveur** et
documentées dans `team-aggregator.md` § « Privacy et conformité
(ADR-0015 — C38) » — à lire avant la mise en service, avec l'ADR-0015.

Réglage par la CLI `config` :

```bash
# Voir les clés, valeurs effectives, défauts et planchers.
docker compose exec sobria-team sobria-team-aggregator --data-dir /data config list

# Relever le seuil de k-anonymat (défaut 5, plancher 3).
docker compose exec sobria-team sobria-team-aggregator --data-dir /data \
  config set k_anonymity_min 8

# Abaisser la rétention à 1 an (défaut 730 j, plancher 30).
docker compose exec sobria-team sobria-team-aggregator --data-dir /data \
  config set retention_days 365
```

Prise d'effet :

- `k_anonymity_min` est lu **à chaque requête** d'analytics → effet immédiat ;
- `retention_days` s'applique au **prochain cycle de purge** (au démarrage,
  puis toutes les 24 h) — `docker compose restart sobria-team` pour purger
  immédiatement.

Les valeurs sous plancher sont refusées par le binaire.

> 📣 **Avant d'ouvrir le service aux salariés** : information-consultation
> du CSE et information individuelle préalable — modèles d'emails prêts à
> adapter dans [`modeles-communication.md`](modeles-communication.md).

## 8. Sauvegarder le volume (SQLite + WAL)

À sauvegarder : `team.sqlite` (la base est en mode WAL : les fichiers
`team.sqlite-wal` / `team.sqlite-shm` sont transitoires et **ne se copient
pas à la volée**) ainsi que `cert.pem` + `key.pem` (sinon, après une
restauration à neuf, nouvelle empreinte à faire ré-accepter partout).

### À chaud (recommandé — le serveur reste en ligne)

`sqlite3` est inclus dans l'image précisément pour la commande `.backup`,
qui produit un snapshot **cohérent** même pendant les écritures :

```bash
docker compose exec sobria-team \
  sqlite3 /data/team.sqlite ".backup '/data/backup-team.sqlite'"
docker compose cp sobria-team:/data/backup-team.sqlite "./team-$(date +%Y%m%d).sqlite"
docker compose exec sobria-team rm /data/backup-team.sqlite
```

Une fois (puis après chaque rotation de cert) :

```bash
docker compose cp sobria-team:/data/cert.pem ./sauvegarde-cert.pem
docker compose cp sobria-team:/data/key.pem  ./sauvegarde-key.pem
chmod 600 sauvegarde-key.pem   # la clé privée TLS est un secret
```

Automatisation : placer les trois commandes du snapshot dans un script
appelé par `cron` sur l'hôte (quotidien, rétention 30 j — même logique que
`team-aggregator.md` § Sauvegardes). Stockez les sauvegardes chiffrées et
hors de la machine.

### À froid (volume complet, serveur arrêté)

```bash
docker compose stop sobria-team
docker run --rm -v sobria-team_data:/data -v "$PWD":/backup debian:stable-slim \
  tar czf "/backup/sobria-team-data-$(date +%Y%m%d).tar.gz" -C / data
docker compose start sobria-team
```

(Le nom exact du volume se vérifie avec `docker volume ls | grep sobria`.)

### Restauration

```bash
docker compose stop sobria-team
docker compose cp ./team-20260612.sqlite sobria-team:/data/team.sqlite
# Purger les fichiers WAL périmés de l'ancienne base (sinon corruption) :
docker compose run --rm --entrypoint /bin/sh sobria-team \
  -c "rm -f /data/team.sqlite-wal /data/team.sqlite-shm"
docker compose start sobria-team
```

## 9. Mettre à jour l'image

```bash
cd deploy/team

# 1. Sauvegarde préalable — toujours (cf. §8).
docker compose exec sobria-team \
  sqlite3 /data/team.sqlite ".backup '/data/backup-pre-upgrade.sqlite'"

# 2. Récupérer le nouveau code et rebuilder l'image.
git -C ../.. pull
docker compose build --pull

# 3. Si la version du workspace a changé : bumper le tag `image:`
#    dans docker-compose.yml (aligné sur Cargo.toml).

# 4. Recréer le conteneur sur la nouvelle image, surveiller le démarrage.
docker compose up -d
docker compose logs -f sobria-team

# 5. Ménage des couches orphelines.
docker image prune -f
```

Les migrations SQLite sont appliquées automatiquement à l'ouverture
(`PRAGMA user_version`) et l'API est versionnée `/api/v1/*` — les
extensions non mises à jour continuent de fonctionner
(cf. `team-aggregator.md` § Upgrade entre versions).

## 10. Dépannage

### Voir les logs

```bash
docker compose logs -f --tail 100 sobria-team
```

Verbosité : décommenter `RUST_LOG` dans `docker-compose.yml` (ou le poser
dans `.env`) puis `docker compose up -d`. Format JSON : `RUST_LOG_FORMAT=json`.

### Vérifier la santé

```bash
curl -k https://localhost:8443/health                # {"ok":true,...}
docker compose ps                                    # colonne STATUS : (healthy)
docker inspect --format '{{json .State.Health}}' sobria-team-aggregator
```

### Symptômes fréquents

| Symptôme | Cause probable → remède |
| --- | --- |
| Le conteneur redémarre en boucle au 1ᵉʳ lancement | `SOBRIA_TEAM_ADMIN_PASSWORD` absente du `.env` (message `[entrypoint] ERREUR` dans les logs) ou < 8 caractères (refusé par le binaire) |
| `port is already allocated` | 8443 occupé sur l'hôte → mapper `"8444:8443"` dans `ports:` |
| `unhealthy` alors que les logs semblent sains | port interne changé sans poser `SOBRIA_TEAM_PORT` (le healthcheck suit cette variable) |
| Build : `folder '../../web-team/build' does not exist` | ordre des `COPY` du Dockerfile modifié — le bundle web doit être copié **avant** `cargo build` (rust-embed) |
| `GLIBC_x.y not found` au démarrage | stages build/runtime sur des releases Debian différentes → épingler la même (commentaire en tête du Dockerfile) |
| `init` refuse : data dir déjà initialisé (cert présent sans `team.sqlite`) | 1ᵉʳ init interrompu. Sans donnée à préserver : `docker compose down -v` (détruit le volume !) puis `up -d`. Sinon : restaurer une sauvegarde (§8) |
| Permissions refusées sur `/data` | bind mount utilisé à la place du volume nommé → `chown -R 10001:10001 <dossier hôte>` |
| « Certificat refusé » par l'extension, « Code rejeté », « Bundle non buildé » | voir `team-aggregator.md` § Troubleshooting |

## 11. Voir aussi

- [`team-aggregator.md`](team-aggregator.md) — référence opérateur complète
  (binaire nu, systemd, sécurité, alertes, **privacy ADR-0015**).
- [`modeles-communication.md`](modeles-communication.md) — emails CSE +
  salariés prêts à adapter.
- `docs/adr/ADR-0015-privacy-mode-equipe.md` — k-anonymat, opt-in, rétention.
- `docs/adr/ADR-0013-extension-pairing-team-mode.md` — architecture deux étages.
- `deploy/team/` — Dockerfile, compose, entrypoint (commentés).
