# Site Sobr.ia — Guide de déploiement self-hosted

> **Statut** : v0.1 — doc initiale C33 (2026-05-16)
> **Cible** : déploiement du site `sobria.brilliantstudio.co` sur serveur Thibault `80.11.20.55` (Ubuntu 22.04+, nginx déjà installé).
> **Audience** : Claude Code (exécution SSH) + Thibault (audit / dépannage).

---

## 1. Vue d'ensemble

```
┌──────────────────┐      git push       ┌──────────────────────┐
│   Repo GitHub    │ ──────────────────▶ │  GitHub Actions      │
│  defis-lia-...   │                     │  site-deploy.yml     │
└──────────────────┘                     └──────────┬───────────┘
                                                    │
                                                    │ build Astro
                                                    │ → site/dist/
                                                    │
                                                    │ rsync SSH (deployer)
                                                    ▼
┌────────────────────────────────────────────────────────────────┐
│   Serveur Thibault — 80.11.20.55 (Ubuntu 22.04+, nginx)       │
│   ┌────────────────────────────────────────────────────────┐  │
│   │  /var/www/sobria-site/  (deployer:www-data, 750)       │  │
│   │  ├── index.html                                        │  │
│   │  ├── _astro/...        (assets hashed, cache 1 an)     │  │
│   │  ├── docs/...                                          │  │
│   │  └── ...                                               │  │
│   └────────────────────────────────────────────────────────┘  │
│                                                                 │
│   nginx                                                         │
│   ├── /etc/nginx/sites-enabled/brilliantstudio.co  (existant)  │
│   └── /etc/nginx/sites-enabled/sobria.brilliantstudio.co  ◀── nouveau
│                                                                 │
│   certbot                                                       │
│   └── /etc/letsencrypt/live/sobria.brilliantstudio.co/         │
│       ├── fullchain.pem                                         │
│       └── privkey.pem                                           │
└─────────────────────────────────────────────────────────────────┘
                          ▲
                          │ HTTPS (443)
                          │
                  ┌───────┴───────┐
                  │   Internet    │
                  │  sobria.bri-  │
                  │liantstudio.co │
                  └───────────────┘
```

**Principes de sécurité** :
- ❌ **Jamais** `root` pour le CI. User dédié `deployer` avec sudoers strictement limité.
- ❌ **Jamais** de credentials en clair dans le repo ou les workflows.
- ✅ Clé SSH ed25519 dédiée au déploiement, stockée dans GitHub Secrets.
- ✅ Permissions strictes `/var/www/sobria-site` (750, deployer:www-data).
- ✅ HTTPS uniquement (HTTP → 301 redirect), HSTS + headers de sécurité.
- ✅ Logs nginx minimaux (privacy).

---

## 2. Prérequis

### 2.1 Côté DNS (déjà fait par Thibault)

Record A configuré :

```
sobria.brilliantstudio.co.   IN A   80.11.20.55
```

Vérifier avec :

```bash
dig +short sobria.brilliantstudio.co
# Doit retourner : 80.11.20.55
```

### 2.2 Accès serveur

- IP : `80.11.20.55`
- Accès initial : `ssh root@80.11.20.55` (à utiliser **uniquement** pour le provisioning initial, ensuite on bascule sur `deployer`).
- OS : Ubuntu 22.04 LTS ou plus récent.
- Nginx : déjà installé (vérifier `nginx -v`).

### 2.3 Pré-vérification

Avant le provisioning, valider :

```bash
ssh root@80.11.20.55 << 'EOF'
echo "--- OS info ---"
cat /etc/os-release | grep -E '^(NAME|VERSION)='
echo "--- nginx ---"
nginx -v
systemctl status nginx --no-pager -l | head -10
echo "--- ports en écoute ---"
ss -tlnp | grep -E ':(80|443) '
echo "--- existing sites ---"
ls -la /etc/nginx/sites-enabled/
echo "--- disk free ---"
df -h /var
echo "--- certbot ---"
which certbot && certbot --version || echo "certbot pas installé"
EOF
```

---

## 3. Provisioning serveur (one-shot, à faire en root)

À exécuter **une seule fois**, depuis Claude Code via SSH `root@80.11.20.55`.

### 3.1 Installer certbot si absent

```bash
apt update
apt install -y certbot python3-certbot-nginx
```

### 3.2 Créer le user `deployer`

```bash
# Création user
useradd -m -s /bin/bash deployer
usermod -aG www-data deployer

# Setup SSH directory
mkdir -p /home/deployer/.ssh
chmod 700 /home/deployer/.ssh
chown deployer:deployer /home/deployer/.ssh

# Préparer authorized_keys vide (sera rempli au §3.3)
touch /home/deployer/.ssh/authorized_keys
chmod 600 /home/deployer/.ssh/authorized_keys
chown deployer:deployer /home/deployer/.ssh/authorized_keys
```

### 3.3 Sudoers limité pour `deployer`

```bash
cat > /etc/sudoers.d/deployer << 'EOF'
# deployer peut uniquement reload/restart nginx et lire son status.
# AUCUNE autre commande sudo autorisée.
deployer ALL=(root) NOPASSWD: /bin/systemctl reload nginx
deployer ALL=(root) NOPASSWD: /bin/systemctl restart nginx
deployer ALL=(root) NOPASSWD: /bin/systemctl status nginx
deployer ALL=(root) NOPASSWD: /usr/sbin/nginx -t
EOF

# Validation syntax sudoers
visudo -c -f /etc/sudoers.d/deployer
chmod 0440 /etc/sudoers.d/deployer
```

### 3.4 Générer la clé SSH dédiée déploiement

**Option A — Générer côté serveur, copier privée sur poste Thibault** :

```bash
# Sur le serveur :
ssh-keygen -t ed25519 -f /tmp/sobria_deploy -N "" -C "sobria-site-deploy-github-actions"

# Mettre la clé publique dans authorized_keys de deployer
cat /tmp/sobria_deploy.pub >> /home/deployer/.ssh/authorized_keys

# Afficher la clé privée (à copier dans GitHub Secrets) :
cat /tmp/sobria_deploy

# Nettoyer du serveur (la clé privée ne doit JAMAIS rester sur le serveur)
shred -u /tmp/sobria_deploy
rm /tmp/sobria_deploy.pub
```

**Option B — Générer côté local Thibault et upload publique** (plus sain) :

```bash
# Sur le poste local Thibault :
ssh-keygen -t ed25519 -f ~/.ssh/sobria_deploy -N "" -C "sobria-site-deploy-github-actions"

# Copier publique sur le serveur :
ssh-copy-id -i ~/.ssh/sobria_deploy.pub deployer@80.11.20.55

# La clé privée ~/.ssh/sobria_deploy → à copier dans GitHub Secrets
cat ~/.ssh/sobria_deploy
```

### 3.5 Tester l'accès `deployer`

```bash
# Depuis poste Thibault :
ssh -i ~/.ssh/sobria_deploy deployer@80.11.20.55 'whoami && sudo -n /bin/systemctl status nginx | head -3'
# Doit afficher "deployer" + statut nginx sans demander de mot de passe.
```

### 3.6 Désactiver le login root SSH (recommandé)

Une fois `deployer` opérationnel et testé :

```bash
sed -i 's/^#\?PermitRootLogin.*/PermitRootLogin no/' /etc/ssh/sshd_config
sed -i 's/^#\?PasswordAuthentication.*/PasswordAuthentication no/' /etc/ssh/sshd_config
systemctl reload sshd
```

⚠️ **Avant de couper root**, garder une session active en backup pour pouvoir revert si problème.

### 3.7 Créer le dossier du site

```bash
mkdir -p /var/www/sobria-site
chown -R deployer:www-data /var/www/sobria-site
chmod -R 750 /var/www/sobria-site

# Placeholder pour tester nginx + certbot avant le 1er deploy CI :
cat > /var/www/sobria-site/index.html << 'EOF'
<!doctype html>
<html lang="fr">
<head>
  <meta charset="utf-8">
  <title>Sobr.ia — Bientôt</title>
  <style>
    body { font-family: system-ui; padding: 4rem; text-align: center; background: #fafafa; color: #1a1a1a; }
    h1 { color: #a0e060; }
  </style>
</head>
<body>
  <h1>Sobr.ia</h1>
  <p>Site en cours de déploiement. Reviens dans quelques minutes.</p>
</body>
</html>
EOF
chown deployer:www-data /var/www/sobria-site/index.html
chmod 640 /var/www/sobria-site/index.html
```

---

## 4. Configuration nginx

### 4.1 Server block

Créer `/etc/nginx/sites-available/sobria.brilliantstudio.co` :

```bash
cat > /etc/nginx/sites-available/sobria.brilliantstudio.co << 'EOF'
# Site Sobr.ia — sobria.brilliantstudio.co
# Configuration générée pour Astro static + Let's Encrypt.

# ─── Redirection HTTP → HTTPS ─────────────────────────────────────────
server {
    listen 80;
    listen [::]:80;
    server_name sobria.brilliantstudio.co;

    # Permettre les challenges ACME (Let's Encrypt) sans redirection
    location /.well-known/acme-challenge/ {
        root /var/www/sobria-site;
        try_files $uri =404;
    }

    # Reste : redirection permanente vers HTTPS
    location / {
        return 301 https://$host$request_uri;
    }
}

# ─── Server HTTPS principal ───────────────────────────────────────────
server {
    listen 443 ssl;
    listen [::]:443 ssl;
    http2 on;
    server_name sobria.brilliantstudio.co;

    # Certificats Let's Encrypt (créés via certbot au §5)
    ssl_certificate /etc/letsencrypt/live/sobria.brilliantstudio.co/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/sobria.brilliantstudio.co/privkey.pem;
    include /etc/letsencrypt/options-ssl-nginx.conf;
    ssl_dhparam /etc/letsencrypt/ssl-dhparams.pem;

    # ─── Racine du site ───────────────────────────────────────────────
    root /var/www/sobria-site;
    index index.html;

    # ─── Security headers ─────────────────────────────────────────────
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-Frame-Options "DENY" always;
    add_header Referrer-Policy "strict-origin-when-cross-origin" always;
    add_header Permissions-Policy "geolocation=(), microphone=(), camera=(), payment=(), usb=()" always;
    # CSP minimal — à durcir si besoin (Astro + Three.js + Pagefind compatible)
    add_header Content-Security-Policy "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data: blob:; font-src 'self' data:; connect-src 'self'; frame-ancestors 'none'; base-uri 'self'; form-action 'self';" always;

    # ─── Compression ──────────────────────────────────────────────────
    gzip on;
    gzip_vary on;
    gzip_min_length 1024;
    gzip_types
        text/plain
        text/css
        text/javascript
        text/xml
        application/json
        application/javascript
        application/xml
        application/xml+rss
        application/wasm
        image/svg+xml;

    # Brotli si module installé (commenté par défaut, à activer si dispo) :
    # brotli on;
    # brotli_comp_level 6;
    # brotli_types text/plain text/css application/json application/javascript text/xml application/xml application/xml+rss text/javascript application/wasm image/svg+xml;

    # ─── Cache assets immutables (Astro hashe les filenames) ──────────
    location /_astro/ {
        expires 1y;
        add_header Cache-Control "public, immutable";
        try_files $uri =404;
    }

    location ~* \.(woff|woff2|ttf|otf|eot)$ {
        expires 1y;
        add_header Cache-Control "public, immutable";
        try_files $uri =404;
    }

    location ~* \.(jpg|jpeg|png|gif|webp|avif|svg|ico)$ {
        expires 30d;
        add_header Cache-Control "public";
        try_files $uri =404;
    }

    # ─── Pages HTML : pas de cache long (révisions fréquentes) ────────
    location ~* \.html$ {
        expires 5m;
        add_header Cache-Control "public, must-revalidate";
        try_files $uri =404;
    }

    # ─── Astro route catch-all + 404 personnalisé ─────────────────────
    location / {
        try_files $uri $uri.html $uri/ /404.html;
    }

    error_page 404 /404.html;
    location = /404.html {
        internal;
    }

    # ─── Logs minimaux (privacy) ──────────────────────────────────────
    access_log /var/log/nginx/sobria-site-access.log combined buffer=64k flush=5m;
    error_log  /var/log/nginx/sobria-site-error.log warn;

    # ─── Limites pour pagefind search ─────────────────────────────────
    client_max_body_size 1m;

    # ─── Bloquer accès aux dotfiles ───────────────────────────────────
    location ~ /\. {
        deny all;
        access_log off;
        log_not_found off;
    }
}
EOF
```

### 4.2 Activer le site

```bash
ln -s /etc/nginx/sites-available/sobria.brilliantstudio.co /etc/nginx/sites-enabled/

# Tester la config
nginx -t

# Recharger si OK
systemctl reload nginx
```

### 4.3 Tester l'accès HTTP (avant SSL)

```bash
curl -I http://sobria.brilliantstudio.co/
# Doit retourner 301 → https://...
# Ou 200 si on a un placeholder
```

---

## 5. Setup Let's Encrypt (certbot)

### 5.1 Obtention du certificat

```bash
certbot --nginx -d sobria.brilliantstudio.co --non-interactive --agree-tos --email thibault@brilliantstudio.co
```

Certbot va :
1. Vérifier le challenge ACME via `/.well-known/acme-challenge/`.
2. Obtenir le certificat Let's Encrypt.
3. Modifier automatiquement le server block pour pointer vers les bons certificats (déjà fait dans notre config §4.1, certbot vérifie juste la cohérence).
4. Tester avec `nginx -t` et recharger.

### 5.2 Vérifier le renouvellement automatique

Certbot crée un cron job ou systemd timer pour renouvellement automatique. Vérifier :

```bash
systemctl status certbot.timer
# OU
crontab -l | grep certbot

# Test de renouvellement (dry-run, ne change rien) :
certbot renew --dry-run
```

### 5.3 Test HTTPS

```bash
curl -I https://sobria.brilliantstudio.co/
# Doit retourner 200 OK + headers de sécurité (HSTS, etc.)

# Test SSL avec testssl.sh ou ssllabs.com
# Cible : note A ou A+
```

---

## 6. Workflow GitHub Actions

### 6.1 Secrets à configurer

Dans GitHub repo Settings → Secrets and variables → Actions, créer :

| Nom | Valeur |
|---|---|
| `SOBRIA_DEPLOY_SSH_KEY` | Contenu complet de la clé privée ed25519 (incluant `-----BEGIN OPENSSH PRIVATE KEY-----` et `-----END OPENSSH PRIVATE KEY-----`) |
| `SOBRIA_DEPLOY_HOST` | `80.11.20.55` |
| `SOBRIA_DEPLOY_USER` | `deployer` |
| `SOBRIA_DEPLOY_PATH` | `/var/www/sobria-site/` |
| `SOBRIA_DEPLOY_KNOWN_HOSTS` | Sortie de `ssh-keyscan -t ed25519 80.11.20.55` (1 ligne) |

### 6.2 Workflow `.github/workflows/site-deploy.yml`

```yaml
name: site-deploy

on:
  push:
    branches: [main]
    paths:
      - 'site/**'
      - 'docs/**'
      - 'scripts/sync-docs.sh'
      - '.github/workflows/site-deploy.yml'
  workflow_dispatch:

concurrency:
  group: site-deploy
  cancel-in-progress: false

jobs:
  build-and-deploy:
    runs-on: ubuntu-latest
    timeout-minutes: 15
    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Setup Node 20
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'
          cache-dependency-path: site/package-lock.json

      - name: Sync docs into site/src/content/
        run: bash scripts/sync-docs.sh

      - name: Install dependencies
        run: |
          cd site
          npm ci

      - name: Lint + type check
        run: |
          cd site
          npm run check
          npm run lint

      - name: Build Astro
        run: |
          cd site
          npm run build

      - name: Setup SSH
        run: |
          mkdir -p ~/.ssh
          echo "${{ secrets.SOBRIA_DEPLOY_SSH_KEY }}" > ~/.ssh/id_ed25519
          chmod 600 ~/.ssh/id_ed25519
          echo "${{ secrets.SOBRIA_DEPLOY_KNOWN_HOSTS }}" > ~/.ssh/known_hosts
          chmod 644 ~/.ssh/known_hosts

      - name: Deploy via rsync
        run: |
          rsync -avz --delete \
            -e "ssh -i ~/.ssh/id_ed25519 -o StrictHostKeyChecking=yes" \
            site/dist/ \
            ${{ secrets.SOBRIA_DEPLOY_USER }}@${{ secrets.SOBRIA_DEPLOY_HOST }}:${{ secrets.SOBRIA_DEPLOY_PATH }}

      - name: Post-deploy smoke test
        run: |
          sleep 5
          curl -sfI https://sobria.brilliantstudio.co/ | head -5
          echo "✓ Site répond"
```

### 6.3 Premier déploiement manuel

Avant de pusher sur `main`, faire un test manuel :

```bash
# Sur poste Thibault, build local :
cd site
npm ci
npm run build

# Upload manuel (test) :
rsync -avz --delete -e "ssh -i ~/.ssh/sobria_deploy" \
  dist/ deployer@80.11.20.55:/var/www/sobria-site/

# Vérifier :
curl -sfI https://sobria.brilliantstudio.co/
```

---

## 7. Rotation de la clé SSH

Bonne pratique annuelle ou en cas de suspicion compromission.

```bash
# 1. Générer nouvelle clé locale
ssh-keygen -t ed25519 -f ~/.ssh/sobria_deploy_new -N "" -C "sobria-site-deploy-2027"

# 2. Ajouter nouvelle publique sur serveur
ssh-copy-id -i ~/.ssh/sobria_deploy_new.pub deployer@80.11.20.55

# 3. Tester nouvelle clé
ssh -i ~/.ssh/sobria_deploy_new deployer@80.11.20.55 'whoami'

# 4. Mettre à jour GitHub Secret SOBRIA_DEPLOY_SSH_KEY avec contenu de ~/.ssh/sobria_deploy_new

# 5. Vérifier qu'un workflow trigger sait deploy

# 6. Retirer ancienne publique du serveur
ssh -i ~/.ssh/sobria_deploy_new deployer@80.11.20.55 'nano ~/.ssh/authorized_keys'  # supprimer ligne ancienne

# 7. Supprimer ancienne locale
shred -u ~/.ssh/sobria_deploy

# 8. Renommer nouvelle
mv ~/.ssh/sobria_deploy_new ~/.ssh/sobria_deploy
mv ~/.ssh/sobria_deploy_new.pub ~/.ssh/sobria_deploy.pub
```

---

## 8. Troubleshooting

### 8.1 Le DNS ne résout pas

```bash
dig +short sobria.brilliantstudio.co
# Si pas de retour : vérifier record A côté registrar brilliantstudio.co
# Attendre propagation DNS (jusqu'à 24h max, généralement < 1h)
```

### 8.2 certbot échoue avec "challenge failed"

- Vérifier que `http://sobria.brilliantstudio.co/` répond bien en HTTP avant SSL.
- Vérifier `/.well-known/acme-challenge/` accessible et pas redirigé HTTPS.
- Vérifier firewall (port 80 ouvert depuis l'extérieur).

```bash
# Test depuis l'extérieur (depuis poste local) :
curl -I http://sobria.brilliantstudio.co/.well-known/acme-challenge/test
# Doit retourner 404 (et pas 301 / 502 / etc.)
```

### 8.3 rsync échoue : "Permission denied"

- Vérifier que la clé publique deployer est bien dans `/home/deployer/.ssh/authorized_keys` et avec bonnes permissions (600).
- Vérifier que `/var/www/sobria-site` appartient à `deployer:www-data` avec mode 750.
- Vérifier que `deployer` n'est pas verrouillé : `passwd -S deployer`.

### 8.4 nginx -t échoue

```bash
nginx -t
# Lit le message d'erreur, corrige le fichier mentionné.
# Si SSL manquant : vérifier que certbot a bien tourné.
# Si syntaxe : vérifier brackets et points-virgules.
```

### 8.5 Site répond mais blanc / erreur 404

- Vérifier que `index.html` est bien présent : `ls -la /var/www/sobria-site/`
- Vérifier permissions : nginx doit pouvoir lire (groupe www-data).
- Vérifier nginx error log : `tail -50 /var/log/nginx/sobria-site-error.log`

### 8.6 Score Lighthouse bas

- Vérifier Brotli activé (gain ~15% sur perf).
- Vérifier cache `_astro/` actif (`curl -I` doit montrer `Cache-Control: public, immutable`).
- Vérifier HTTP/2 actif (`curl -I --http2` doit retourner `HTTP/2 200`).
- Vérifier headers de sécurité tous présents (HSTS, CSP, X-Frame-Options).

---

## 9. Monitoring

### 9.1 Logs nginx

```bash
# Accès récent
tail -f /var/log/nginx/sobria-site-access.log

# Erreurs
tail -f /var/log/nginx/sobria-site-error.log

# Statistiques rapides
awk '{print $1}' /var/log/nginx/sobria-site-access.log | sort | uniq -c | sort -rn | head -20
```

### 9.2 Rotation des logs

Logrotate gère automatiquement la rotation (config par défaut Ubuntu). Vérifier :

```bash
cat /etc/logrotate.d/nginx
```

### 9.3 Test certificat SSL avant expiration

```bash
echo | openssl s_client -servername sobria.brilliantstudio.co -connect sobria.brilliantstudio.co:443 2>/dev/null | openssl x509 -noout -dates
```

Renouvellement automatique normalement à 30 jours avant expiration. Si vrai problème :

```bash
certbot renew
systemctl reload nginx
```

---

## 10. Checklist de validation initiale

À cocher après provisioning :

- [ ] DNS `sobria.brilliantstudio.co` résout vers `80.11.20.55`.
- [ ] User `deployer` créé, login SSH OK depuis poste Thibault.
- [ ] Sudoers `deployer` configuré (uniquement reload/restart/status nginx).
- [ ] Dossier `/var/www/sobria-site` créé, permissions `deployer:www-data 750`.
- [ ] Server block nginx `sobria.brilliantstudio.co` activé et `nginx -t` OK.
- [ ] HTTP `http://sobria.brilliantstudio.co/` répond (placeholder).
- [ ] Certbot exécuté avec succès, certificat délivré.
- [ ] HTTPS `https://sobria.brilliantstudio.co/` répond, certificat valide.
- [ ] Renouvellement certbot automatique configuré (systemd timer ou cron).
- [ ] GitHub Secrets configurés (5 secrets).
- [ ] Premier deploy manuel via rsync OK.
- [ ] Workflow `site-deploy.yml` testé en `workflow_dispatch`.
- [ ] Post-deploy smoke test (curl) retourne 200.
- [ ] (Optionnel) Login root SSH désactivé (`PermitRootLogin no`).

---

## 11. Backup / disaster recovery

### 11.1 Backup serveur

Le site est entièrement reconstructible depuis le repo Git. Pas besoin de backup du site lui-même. **Mais** sauvegarder :

- `/etc/nginx/sites-available/sobria.brilliantstudio.co` (config nginx) → versionner dans `docs/operations/nginx-snapshots/`.
- `/etc/letsencrypt/live/sobria.brilliantstudio.co/` (cert + key) → backup chiffré côté Thibault.
- Liste GitHub Secrets → documenter (sans révéler valeurs) dans cette doc.

### 11.2 Migration vers nouveau serveur

Si Thibault change de VPS :

1. Provisionner nouveau serveur (refaire §3 à §5).
2. Mettre à jour `SOBRIA_DEPLOY_HOST` dans GitHub Secrets.
3. Mettre à jour record DNS A → nouvelle IP.
4. Attendre propagation DNS.
5. Trigger workflow `site-deploy.yml` manuellement.
6. Vérifier accès HTTPS.
7. Décommissionner ancien serveur après 48h.

---

## 12. Liens utiles

- Brief chantier : `briefs/chantiers/C33-site-internet.md`
- Prompt Claude Code : `briefs/chantiers/C33-PROMPT-CLAUDE-CODE.md`
- ADR souveraineté : `docs/adr/ADR-0014-dual-track-local-cloud.md`
- Documentation nginx : <https://nginx.org/en/docs/>
- Documentation certbot : <https://certbot.eff.org/instructions>
- Documentation Astro static : <https://docs.astro.build/en/guides/deploy/>
