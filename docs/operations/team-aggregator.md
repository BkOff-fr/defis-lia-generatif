# Déploiement `sobria-team-aggregator` (Mode Équipe self-hosted)

> Cible : un admin d'entreprise qui veut déployer le serveur Sobr.ia
> chez lui pour agréger les estimations carbone de ses N employés.
> **Aucun cloud Sobr.ia n'est impliqué.** Voir ADR-0013 Phase 2 et
> `briefs/chantiers/C28-mode-equipe-self-hosted.md`.

## TL;DR — En 5 minutes (C32.3)

**Pour un dirigeant TPE/PME, un gestionnaire bureau, un freelance** :
vous lancez UN exécutable, vous obtenez une URL `https://...:8443`,
vous distribuez 1 code à 12 chiffres par employé. Aucune Kubernetes,
aucune autorité de certification, aucune base de données externe.

**Pour un DSI** : binaire Rust standalone (~15 MB, sans dépendance
runtime), TLS auto-signé via rcgen + ring (pas d'OpenSSL),
SQLite WAL, JWT HS256 24h + refresh 7j Argon2id. Reverse proxy
Caddy / nginx + Let's Encrypt optionnel pour expo Internet. Hardening
systemd fourni. Voir `## Pour les DSI` ci-dessous.

Le binaire `sobria-team-aggregator` est un serveur HTTPS standalone
(~15 MB) qui :

- expose une API REST `/api/v1/*` pour l'extension navigateur + l'app
  Tauri (JWT 24h + refresh 7j, Argon2id partout) ;
- sert un dashboard Svelte (admin + employé) sous `/` ;
- stocke tout dans un seul fichier `team.sqlite` (WAL) ;
- produit des exports CSRD PDF + PROV-O JSON-LD + CSV signés sur
  demande de l'admin.

---

## Table des matières

1. [Quickstart (5 min)](#quickstart-5-min)
2. [Pour les non-IT (TPE/PME, freelances)](#pour-les-non-it-tpepme-freelances)
3. [Pour les DSI](#pour-les-dsi)
4. [Sécurité](#sécurité)
5. [Sauvegardes](#sauvegardes)
6. [Upgrade entre versions](#upgrade-entre-versions)
7. [Troubleshooting](#troubleshooting)

---

## Quickstart (5 min)

### 1. Télécharger le binaire

Sur la page Releases du projet, télécharger l'archive correspondant à
votre OS :

```
sobria-team-aggregator-linux-x86_64
sobria-team-aggregator-macos-arm64
sobria-team-aggregator-windows-x86_64.exe
```

Vérifier le `sha256` publié à côté.

### 2. Initialiser le data dir

```bash
# Linux / macOS
chmod +x sobria-team-aggregator-linux-x86_64
mv sobria-team-aggregator-linux-x86_64 /usr/local/bin/sobria-team-aggregator

sobria-team-aggregator --data-dir ./team-data init \
    --admin-username admin \
    --admin-password 'CHANGE-ME-strong-passphrase-32+chars'
```

Cette commande :

- crée `./team-data/team.sqlite` (schéma v1, WAL activé) ;
- génère un certificat TLS auto-signé (`cert.pem` + `key.pem`,
  validité 10 ans, SANs `localhost`/`127.0.0.1`/`::1`/hostname OS) ;
- pose une JWT signing key 32 octets random ;
- crée l'admin initial (hash Argon2id PHC du mot de passe).

### 3. Lancer le serveur

```bash
sobria-team-aggregator --data-dir ./team-data serve --port 8443
```

Le serveur écoute en HTTPS sur le port 8443. Vérifier :

```bash
curl -k https://localhost:8443/health
# → {"ok":true,"version":"0.7.0"}
```

Ouvrir `https://VOTRE-HOSTNAME:8443/` dans un navigateur, accepter
l'avertissement du certificat auto-signé, et se connecter en tant
qu'admin.

### 4. Distribuer des codes d'enrôlement

Depuis le dashboard admin (`/admin/codes`) ou la CLI :

```bash
sobria-team-aggregator --data-dir ./team-data code create 10 --ttl-days 7
```

Les 10 codes 12 chiffres sont **affichés en clair une seule fois** —
les distribuer à vos employés par un canal sûr (gestionnaire de mots
de passe, mail chiffré). Argon2id PHC en base ensuite, plus rien de
réversible.

### 5. Côté employé

Chaque employé installe l'extension Sobr.ia (Chrome / Firefox /
Edge), va dans **Options → Mode Équipe**, colle l'URL du serveur,
clique « Vérifier », puis « S'enrôler » avec son code à 12 chiffres
et un mot de passe personnel.

Les estimations remontent automatiquement (mode `both` activé par
défaut au premier enrollment : continue de remonter aussi à l'app
desktop perso si pairée).

---

## Pour les non-IT (TPE/PME, freelances)

> Si vous n'avez pas d'équipe IT en interne : le binaire fait _tout_
> tout seul. Suivez juste le Quickstart ci-dessus, vous n'aurez pas
> besoin de toucher au reste de ce document.
>
> Les sections ci-dessous montrent un déploiement type pour 10-100
> employés (auto-start systemd, hardening Linux). Elles sont
> facultatives — pour un usage perso ou TPE, lancer
> `sobria-team-aggregator serve` à la main suffit.

### Architecture conseillée (10 à 100 employés)

```
┌──────────────────────────────┐
│ Poste admin / NAS Synology   │
│  ┌────────────────────────┐  │
│  │ sobria-team-aggregator │  │
│  │   (auto-start systemd) │  │
│  │   :8443 HTTPS auto     │  │
│  └────────────────────────┘  │
│  ┌────────────────────────┐  │
│  │ team-data/             │  │
│  │  team.sqlite (WAL)     │  │
│  │  cert.pem + key.pem    │  │
│  └────────────────────────┘  │
└──────────────────────────────┘
          ↑ LAN ou VPN
          │
┌─────────────────────────────────────┐
│  Postes employés (extensions / app) │
└─────────────────────────────────────┘
```

### Auto-start systemd (Linux)

`/etc/systemd/system/sobria-team-aggregator.service` :

```ini
[Unit]
Description=Sobr.ia Team Aggregator
After=network.target

[Service]
Type=simple
User=sobria
Group=sobria
WorkingDirectory=/var/lib/sobria
ExecStart=/usr/local/bin/sobria-team-aggregator \
    --data-dir /var/lib/sobria/team-data \
    serve --bind 0.0.0.0 --port 8443
Restart=on-failure
RestartSec=5
# Hardening
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/sobria/team-data
PrivateTmp=true
PrivateDevices=true

[Install]
WantedBy=multi-user.target
```

```bash
sudo useradd -r -s /usr/sbin/nologin sobria
sudo mkdir -p /var/lib/sobria/team-data
sudo chown -R sobria:sobria /var/lib/sobria
sudo -u sobria sobria-team-aggregator --data-dir /var/lib/sobria/team-data init --admin-username admin --admin-password 'PASSPHRASE'
sudo systemctl daemon-reload
sudo systemctl enable --now sobria-team-aggregator
sudo systemctl status sobria-team-aggregator
```

### Auto-start launchd (macOS) / Task Scheduler (Windows)

Voir les guides standard de votre OS. Le binaire est sans dépendance
runtime — il suffit d'invoquer `sobria-team-aggregator serve` avec
le bon `--data-dir`.

---

## Pour les DSI

### Reverse proxy + certificat publique (Let's Encrypt)

Si vous exposez le serveur sur Internet (télétravail), mettez un
reverse proxy devant pour un cert reconnu. Exemple `caddy` :

```caddyfile
team.exemple.fr {
    reverse_proxy 127.0.0.1:8443 {
        transport http {
            tls
            tls_insecure_skip_verify   # cert auto-signé interne du binaire
        }
    }
}
```

Avec ça, l'extension et l'app n'ont plus besoin d'accepter manuellement
le cert (Caddy se charge de Let's Encrypt).

Variante `nginx` :

```nginx
server {
    listen 443 ssl http2;
    server_name team.exemple.fr;
    ssl_certificate     /etc/letsencrypt/live/team.exemple.fr/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/team.exemple.fr/privkey.pem;

    location / {
        proxy_pass         https://127.0.0.1:8443;
        proxy_ssl_verify   off;
        proxy_set_header   Host $host;
        proxy_set_header   X-Real-IP $remote_addr;
        proxy_read_timeout 60s;
    }
}
```

### Cert fourni explicitement (PKI interne)

Si vous avez votre PKI, passez les fichiers au `init` (à venir
v0.7.1) ou remplacez `cert.pem` + `key.pem` dans le data dir et
redémarrez le serveur.

### Firewall

- Si LAN-only : autoriser le port 8443 uniquement depuis les
  postes utilisateurs / le VPN.
- Si exposition publique via reverse proxy : autoriser uniquement
  le port 443 entrant ; le 8443 reste en loopback.

### Backup

Le seul état persisté est `team-data/team.sqlite` (et `cert.pem` +
`key.pem`). Voir la section [Sauvegardes](#sauvegardes) ci-dessous.

### Multi-instance (haute dispo)

Non supporté en v0.7.0 — un seul process à la fois pointe sur un
data dir donné (WAL SQLite n'aime pas le partage NFS). Architecture
HA différée à v1.x.

---

## Sécurité

### Inventaire des secrets

Tous stockés dans `team-data/` :

| Élément            | Format                       | Rotation                   |
| ------------------ | ---------------------------- | -------------------------- |
| Mot de passe admin | Argon2id PHC en SQLite       | manuelle                   |
| Mot de passe user  | Argon2id PHC en SQLite       | manuelle                   |
| Enrollment codes   | Argon2id PHC en SQLite       | TTL 7j par défaut          |
| JWT signing key    | 32 octets hex en SQLite      | `--regen-key`\*            |
| Refresh tokens     | Argon2id PHC en SQLite       | rotation à chaque /refresh |
| Clé privée TLS     | PEM ECDSA-P256 dans data dir | `--regen-cert`\*           |

\*Commandes prévues v0.7.1.

### Permissions filesystem

- **Unix** : la clé privée est automatiquement `chmod 600` après
  génération. Le data dir doit appartenir au user qui lance le service
  (et pas être lisible par les autres) :

  ```bash
  chmod 700 /var/lib/sobria/team-data
  chmod 600 /var/lib/sobria/team-data/team.sqlite
  chmod 600 /var/lib/sobria/team-data/key.pem
  ```

- **Windows** : utiliser `icacls` pour restreindre le data dir au
  user de service (typiquement le NetworkService ou un user dédié) :

  ```cmd
  icacls C:\sobria\team-data /inheritance:r
  icacls C:\sobria\team-data /grant:r "%COMPUTERNAME%\sobria_svc:(F)"
  ```

### Pas de tracking vers Sobr.ia

Le binaire ne fait aucun appel sortant vers Sobr.ia. Auditable :

```bash
# Le seul outgoing HTTPS attendu est vers le user-agent qui appelle
# /api/v1/* — sortant de votre LAN sauf si vous exposez en public.
ss -tunap | grep sobria-team-aggregator
```

### TLS

- rustls + ring uniquement (pas d'OpenSSL).
- TLS 1.2 et 1.3 acceptés ; le client (extension/app) négocie.
- Le cert auto-signé est valable 10 ans, mais peut être régénéré à la
  demande via `serve --regen-cert` (voir « Opérations »).

---

## Opérations courantes (v0.7.1, C29)

### Réinitialiser le mot de passe d'un admin

Si un admin oublie son mot de passe, ne **jamais** éditer la SQLite à la main —
utiliser :

```bash
sobria-team-aggregator --data-dir /var/lib/sobria/team-data \
  admin reset-password alice
```

- Prompt interactif (saisie double, sans écho).
- Le nouveau hash est Argon2id PHC.
- **Tous** les tokens admin actifs sont révoqués (un `/login` est nécessaire).

Lister les admins :

```bash
sobria-team-aggregator admin list
```

### Rotation du certificat TLS

Le cert auto-signé est valable 10 ans, mais on peut vouloir le régénérer pour :

- changer l'empreinte (post-incident, rotation de routine) ;
- ajouter de nouveaux SANs si la machine change de hostname.

Procédure :

```bash
sudo systemctl stop sobria-team-aggregator
sobria-team-aggregator --data-dir /var/lib/sobria/team-data \
  serve --regen-cert
```

Le binaire :

1. Sauvegarde l'ancien cert / la clé en `cert.pem.bak.<unix_ts>` et
   `key.pem.bak.<unix_ts>` à côté.
2. Génère un nouveau cert auto-signé (mêmes SANs : `localhost`, `127.0.0.1`,
   `::1` + hostname OS, validité 10 ans).
3. Affiche l'empreinte SHA-256 du nouveau certificat — **à communiquer aux
   utilisateurs** (ils devront re-accepter le fingerprint dans leur
   navigateur ou cocher « accepter les certs auto-signés » dans l'app
   Tauri si non déjà fait).

> ⚠️ Tous les clients qui avaient accepté l'ancien cert devront ré-accepter
> le nouveau. Pour une rotation transparente, prévoir une fenêtre où l'on
> peut prévenir les utilisateurs.

### Configurer les alertes seuils (C29.4)

Les alertes notifient quand la conso `gCO₂eq` d'un utilisateur ou de toute
l'équipe dépasse un plafond sur une période (daily / weekly / monthly UTC).
Voir le dashboard admin « Alertes » (web-team) pour créer / désactiver via UI.

#### Notifications webhook

Aucune config serveur n'est requise — l'URL est définie par seuil.
Format du POST JSON (timeout 5 s) :

```json
{
  "threshold_id": "01HZZZ…",
  "scope": "team",
  "target_id": null,
  "period": "daily",
  "gco2eq_max": 100.0,
  "observed_gco2eq": 127.3,
  "period_start": "2026-05-16T00:00:00+00:00",
  "period_end": "2026-05-16T23:59:59+00:00",
  "triggered_at": "2026-05-16T14:32:01+00:00"
}
```

#### Notifications email (SMTP optionnel)

Pour activer l'envoi email, poser deux clés dans le KV `config` côté SQLite :

```bash
sqlite3 /var/lib/sobria/team-data/team.sqlite <<SQL
INSERT INTO config (key, value) VALUES
  ('smtp_url',  'smtps://user:pass@smtp.example.org:465'),
  ('smtp_from', 'sobria@example.org')
ON CONFLICT(key) DO UPDATE SET value = excluded.value;
SQL
```

- Schémas supportés : `smtp://host:port` (STARTTLS implicite via port) et
  `smtps://user:pass@host:port` (TLS direct). `user:pass@` optionnel.
- Si l'une des deux clés est absente : **fallback automatique en log_only**
  (le trigger est journalisé via `tracing::warn!`, `notify_error` est rempli
  dans `alert_triggers`). Pas de crash.

#### Garanties

- 1 trigger par `(threshold_id, period_start)` (UNIQUE SQL).
- Le trigger est inséré pendant le handler `POST /api/v1/estimations` —
  la notification webhook/email est `tokio::spawn`-ée (non bloquante pour
  l'ack du client).
- `alert_triggers.notified_at` est mis à jour après envoi ;
  `alert_triggers.notify_error` contient l'erreur transport éventuelle.

---

## Sauvegardes

### Backup simple (snapshot quotidien)

Le fichier `team.sqlite` est en mode WAL — pour un snapshot cohérent,
utiliser la commande SQLite officielle :

```bash
sqlite3 /var/lib/sobria/team-data/team.sqlite ".backup '/backup/team-$(date +%Y%m%d).sqlite'"
```

À placer dans `cron` quotidien :

```cron
0 3 * * * sqlite3 /var/lib/sobria/team-data/team.sqlite ".backup '/backup/team-$(date +\%Y\%m\%d).sqlite'" && find /backup/team-*.sqlite -mtime +30 -delete
```

### Restauration

Arrêter le service, remplacer `team.sqlite`, redémarrer :

```bash
sudo systemctl stop sobria-team-aggregator
cp /backup/team-20260516.sqlite /var/lib/sobria/team-data/team.sqlite
sudo systemctl start sobria-team-aggregator
```

### Ce qui n'est PAS sauvegardé par défaut

- Les **prompts en clair** ne sont jamais stockés sur le serveur —
  seules les estimations agrégées (model, tokens, gCO₂eq).
- Les **mots de passe admins/users** sont en Argon2id PHC, irréversibles.
  En cas de perte côté admin : `admin reset-password` (cf. § Opérations
  courantes). Côté user : révoquer l'enrollment code via `code revoke`
  puis en émettre un nouveau.

---

## Upgrade entre versions

### Procédure générale

```bash
sudo systemctl stop sobria-team-aggregator
sqlite3 /var/lib/sobria/team-data/team.sqlite ".backup '/backup/team-pre-upgrade.sqlite'"
sudo cp /chemin/vers/nouveau-binaire /usr/local/bin/sobria-team-aggregator
sudo systemctl start sobria-team-aggregator
journalctl -u sobria-team-aggregator -f -n 30
```

### Migrations SQLite

Le binaire applique automatiquement les migrations à l'ouverture
(`PRAGMA user_version`). En v0.7.0 le schéma est v1 ; les futures
versions ajouteront `v2`, `v3`, etc. sans casser la rétrocompat
sur les colonnes existantes.

### Compatibilité API REST

L'API est versionnée sous `/api/v1/*`. Les futures évolutions
breaking iront sous `/api/v2/*`. Les anciens clients (extensions
et apps non mises à jour) continuent à fonctionner.

---

## Troubleshooting

### « Certificat refusé » par l'extension

Le browser ne fait confiance à un cert auto-signé qu'après acceptation
manuelle par origin. Solution :

1. Ouvrir `https://VOTRE-URL:8443/health` dans un onglet.
2. Cliquer « Avancé » → « Continuer vers le site (dangereux) » ou
   équivalent selon le browser.
3. Revenir dans Options → Mode Équipe et cliquer « Vérifier ».

Solution permanente : mettre Caddy/nginx devant avec Let's Encrypt
(voir [Pour les DSI](#pour-les-dsi)).

### « Code rejeté » au /enroll

Causes possibles :

- Code expiré (TTL par défaut 7j).
- Code déjà utilisé (single-use).
- Code révoqué par l'admin.
- Mot de passe < 8 caractères.
- Fingerprint déjà enrôlé (multi-device différé v0.8+).

Vérifier avec :

```bash
sobria-team-aggregator --data-dir ./team-data code list
```

Re-créer un code si besoin :

```bash
sobria-team-aggregator --data-dir ./team-data code create 1 --ttl-days 7
```

### « Refresh token expiré »

Les refresh tokens vivent 7 jours. Au-delà : l'extension/app
redemande à l'utilisateur de se reconnecter (Options → Mode Équipe →
« Me déconnecter » puis ré-enroll avec un nouveau code).

### Le dashboard montre « Bundle non buildé »

Le binaire embarque `web-team/build/` au compile time. Si vous compilez
depuis source, lancer :

```bash
cd web-team && npm ci && npm run build
cd .. && cargo build -p sobria-team-aggregator --release
```

Les binaires officiels GitHub Releases sont buildés via la CI avec
le frontend pré-compilé.

### Performance : analytics lent au-delà de 1M estimations

Les index `(user_id, ts)`, `(ts)`, `(model_id)` sont déjà posés.
Si vous arrivez à 10M+ d'estimations, envisager :

- Materialized views journalières (prévu v0.8 si la demande arrive).
- Archive des estimations > 1 an vers une autre base.

Pour vider proprement :

```bash
sqlite3 /var/lib/sobria/team-data/team.sqlite \
  "DELETE FROM estimations WHERE ts < datetime('now', '-2 years');"
sqlite3 /var/lib/sobria/team-data/team.sqlite "VACUUM;"
```

### Logs

Niveau par défaut : `info`. Pour plus de verbosité :

```bash
RUST_LOG=sobria_team_aggregator=debug,tower_http=debug \
    sobria-team-aggregator --data-dir /var/lib/sobria/team-data serve
```

Format JSON disponible via `RUST_LOG_FORMAT=json`.

---

## Voir aussi

- `deploiement-equipe.md` — déploiement Docker / Compose (kit PME, C40),
  avec `modeles-communication.md` (emails CSE / salariés).
- ADR-0013 — décision d'architecture (extension + pairing + mode équipe).
- `briefs/chantiers/C28-mode-equipe-self-hosted.md` — brief du chantier.
- README `crates/sobria-team-aggregator/`.
- CLAUDE.md §7 (privacy by design).

## Privacy et conformité (ADR-0015 — C38)

Le serveur applique **côté serveur** (jamais seulement dans l'UI) :

1. **k-anonymat** : les analytics équipe ne sont servis que si le nombre
   d'utilisateurs actifs sur la fenêtre interrogée atteint le seuil
   `k_anonymity_min` (défaut 5, plancher 3). En dessous, le dashboard
   affiche une carte explicative à la place des chiffres.
2. **Identification opt-in** : par défaut, aucun employé n'apparaît
   nommément dans les vues admin. Chaque salarié contrôle son propre
   partage depuis son espace « Mon usage » (toggle « Partage identifié »).
   Aucune commande ni route admin ne peut écrire ce consentement.
3. **Rétention** : les estimations plus anciennes que `retention_days`
   (défaut 730 j, plancher 30) sont purgées au démarrage puis toutes les
   24 h.

### Configuration runtime

```bash
sobria-team-aggregator --data-dir ./team-data config list
sobria-team-aggregator --data-dir ./team-data config get k_anonymity_min
sobria-team-aggregator --data-dir ./team-data config set k_anonymity_min 8
sobria-team-aggregator --data-dir ./team-data config set retention_days 365
```

Les valeurs sous plancher sont refusées ; les clés internes
(`jwt_signing_key`, …) ne sont pas accessibles par cette commande.

### Politique de visibilité (ADR-0016 — C44)

Votre organisation choisit le régime de visibilité des employés :

```bash
# Lire la politique courante (défaut : opt_in)
sobria-team-aggregator --data-dir ./team-data config get visibility_policy

# Anonyme strict : agrégats k-anonymes uniquement, aucune identification
sobria-team-aggregator --data-dir ./team-data config set visibility_policy anonymous

# Opt-in (défaut) : chaque salarié contrôle son identification
sobria-team-aggregator --data-dir ./team-data config set visibility_policy opt_in

# Nominatif : vues par employé — ATTESTATION OBLIGATOIRE
sobria-team-aggregator --data-dir ./team-data config set visibility_policy identified \
  --attest "CSE informé-consulté le 2026-06-10 ; salariés informés par email le 2026-06-11"
```

Le mode `identified` est **refusé sans `--attest`** : l'attestation
(texte, date) est stockée en base et visible dans le dashboard. Les
agrégats par **projet** suivent la même politique (repli « autres
projets » sous le seuil k hors mode nominatif). Le changement est
immédiat (lu à chaque requête).

### Obligations du déployeur (France)

Le serveur minimise par construction, mais l'organisation qui le déploie
reste responsable de :

- l'**information-consultation du CSE** avant mise en service d'un
  dispositif de collecte lié à l'activité des salariés (C. trav.
  L2312-38) ;
- l'**information individuelle préalable** des salariés (L1222-4) —
  l'écran « Mon usage » documente ce qui est collecté et ce qui ne l'est
  jamais, mais ne remplace pas l'information formelle ;
- l'inscription au **registre des traitements** (RGPD art. 30) avec la
  finalité « pilotage budgétaire et environnemental de l'usage IA »
  (jamais l'évaluation individuelle des salariés) ;
- la fixation d'une **durée de rétention** proportionnée
  (`retention_days`).

> Ces éléments sont fournis à titre d'aide opérationnelle et ne
> constituent pas un conseil juridique.
