# Catalogue des 28 datacenters européens — Sobr.ia

> **Version** : 1.0 — 13 mai 2026.
> **Dataset source** : `crates/sobria-geoloc/data/datacenters.json`.
> **Licence dataset** : CC-BY-4.0 (compilation), sources individuelles
> sous licence propriétaire des opérateurs (rapports publics) ou Etalab 2.0
> (Electricity Maps).

## 1. Critères de sélection

Cf. brief `C12-datacenters-europe.md` §1. Sélection orientée représentativité,
pas exhaustivité. Pull request ouverte pour ajouter des DC manquants en v1.1.

## 2. Sources de référence

| Donnée | Source primaire | Fréquence MAJ | Licence |
|---|---|---|---|
| PUE / WUE annuel par DC | Rapports Sustainability des opérateurs | Annuelle | Propriétaire (cité) |
| IF élec moyen pays (gCO₂eq/kWh) | Electricity Maps + AIB European Residual Mix | Annuelle | CC-BY-4.0 |
| Profils horaires charge pays | **v1.0** : courbe typique modélisée (forme demande UE moyenne). **v1.1** : pull ENTSO-E Transparency Platform (TOTAL LOAD A65, mensuel, CC-BY-4.0) avec clé API utilisateur. | À l'usage | CC-BY-4.0 |
| Coordonnées DC | Sites publics opérateurs / OpenStreetMap | À la demande | ODbL |

## 3. Tableau complet (28 DC, 13 pays)

### 🇫🇷 France (4 DC, mix 56 g/kWh)

| ID | Opérateur | Ville | PUE | WUE | Capacité | Source |
|---|---|---|---|---|---|---|
| `ovh-rbx-roubaix` | OVHcloud | Roubaix | 1.09 | 0.26 | 30 MW | OVH Sustainability 2023 |
| `ovh-gra-gravelines` | OVHcloud | Gravelines | 1.05 | 0.18 | 100 MW | OVH Sustainability 2023 |
| `scaleway-dc5-paris` | Scaleway | St-Ouen-l'Aumône | 1.15 | 0.0 | 8.5 MW | Scaleway Q4 2023 disclosure |
| `aws-eu-west-3-paris` | AWS | Paris | 1.15 | n/a | n/a | AWS Sustainability Report 2024 |

### 🇩🇪 Allemagne (4 DC, mix 386 g/kWh)

| ID | Opérateur | Ville | PUE | WUE | Capacité | Source |
|---|---|---|---|---|---|---|
| `aws-eu-central-1-frankfurt` | AWS | Frankfurt | 1.15 | n/a | n/a | AWS 2024 |
| `gcp-europe-west3-frankfurt` | Google Cloud | Frankfurt | 1.10 | 1.10 | n/a | Google Env Report 2024 |
| `azure-germany-west-central-frankfurt` | Azure | Frankfurt | 1.18 | 0.49 | n/a | Microsoft Sustainability 2024 |
| `interxion-fra15-frankfurt` | Digital Realty | Frankfurt | 1.30 | n/a | 30 MW | Digital Realty ESG 2023 |

### 🇮🇪 Irlande (3 DC, mix 296 g/kWh)

| ID | Opérateur | Ville | PUE | WUE | Capacité | Source |
|---|---|---|---|---|---|---|
| `aws-eu-west-1-dublin` | AWS | Dublin | 1.15 | n/a | n/a | AWS 2024 |
| `azure-north-europe-dublin` | Azure | Dublin | 1.20 | 0.49 | n/a | Microsoft 2024 |
| `equinix-db1-dublin` | Equinix | Dublin | 1.30 | n/a | 12 MW | Equinix Sustainability 2023 |

### 🇳🇱 Pays-Bas (3 DC, mix 314 g/kWh)

| ID | Opérateur | Ville | PUE | WUE | Capacité | Source |
|---|---|---|---|---|---|---|
| `gcp-europe-west4-eemshaven` | Google Cloud | Eemshaven | 1.10 | 1.10 | n/a | Google 2024 |
| `equinix-am3-amsterdam` | Equinix | Amsterdam | 1.30 | n/a | 18 MW | Equinix 2023 |
| `azure-west-europe-amsterdam` | Azure | Amsterdam | 1.18 | 0.49 | n/a | Microsoft 2024 |

### 🇬🇧 Royaume-Uni (3 DC, mix 245 g/kWh)

| ID | Opérateur | Ville | PUE | WUE | Capacité | Source |
|---|---|---|---|---|---|---|
| `aws-eu-west-2-london` | AWS | London | 1.18 | n/a | n/a | AWS 2024 |
| `gcp-europe-west2-london` | Google Cloud | London | 1.10 | 1.10 | n/a | Google 2024 |
| `azure-uk-south-london` | Azure | London | 1.20 | 0.49 | n/a | Microsoft 2024 |

### 🇸🇪 Suède (2 DC, mix 41 g/kWh — meilleur cas)

| ID | Opérateur | Ville | PUE | WUE | Capacité | Source |
|---|---|---|---|---|---|---|
| `aws-eu-north-1-stockholm` | AWS | Stockholm | 1.10 | n/a | n/a | AWS 2024 |
| `bahnhof-pionen-stockholm` | Bahnhof | Stockholm | 1.15 | n/a | 1.5 MW | Bahnhof rapport 2023 |

### 🇫🇮 Finlande (2 DC, mix 132 g/kWh)

| ID | Opérateur | Ville | PUE | WUE | Capacité | Source |
|---|---|---|---|---|---|---|
| `gcp-europe-north1-hamina` | Google Cloud | Hamina | 1.10 | 1.10 | n/a | Google 2024 |
| `telia-helsinki` | Telia | Helsinki | 1.20 | n/a | 24 MW | Telia Sustainability 2023 |

### 🇪🇸 Espagne (2 DC, mix 143 g/kWh)

| ID | Opérateur | Ville | PUE | WUE | Capacité | Source |
|---|---|---|---|---|---|---|
| `aws-eu-south-2-madrid` | AWS | Madrid | 1.15 | n/a | n/a | AWS 2024 |
| `azure-spain-central-madrid` | Azure | Madrid | 1.20 | 0.49 | n/a | Microsoft 2024 |

### 🇮🇹 Italie (1 DC, mix 354 g/kWh)

| ID | Opérateur | Ville | PUE | WUE | Capacité | Source |
|---|---|---|---|---|---|---|
| `aws-eu-south-1-milan` | AWS | Milan | 1.18 | n/a | n/a | AWS 2024 |

### 🇵🇱 Pologne (1 DC, mix 633 g/kWh — pire cas EU)

| ID | Opérateur | Ville | PUE | WUE | Capacité | Source |
|---|---|---|---|---|---|---|
| `google-warsaw` | Google Cloud | Warsaw | 1.10 | 1.10 | n/a | Google 2024 |

### 🇨🇭 Suisse (1 DC, mix 47 g/kWh)

| ID | Opérateur | Ville | PUE | WUE | Capacité | Source |
|---|---|---|---|---|---|---|
| `equinix-zh4-zurich` | Equinix | Zurich | 1.30 | n/a | 10 MW | Equinix 2023 |

### 🇦🇹 Autriche (1 DC, mix 89 g/kWh)

| ID | Opérateur | Ville | PUE | WUE | Capacité | Source |
|---|---|---|---|---|---|---|
| `interxion-vie2-vienna` | Digital Realty | Vienna | 1.30 | n/a | 8 MW | Digital Realty ESG 2023 |

### 🇩🇰 Danemark (1 DC, mix 151 g/kWh)

| ID | Opérateur | Ville | PUE | WUE | Capacité | Source |
|---|---|---|---|---|---|---|
| `interxion-cph2-copenhagen` | Digital Realty | Copenhagen | 1.25 | n/a | 9 MW | Digital Realty ESG 2023 |

## 4. Notes méthodologiques

- **IF national vs IF horaire** : on prend la moyenne annuelle pays. Sur 24h
  le mix varie (plus de charbon la nuit en DE, par exemple). La granularité
  horaire est livrée par M20 Territoire FR (RTE eco2mix) en v1.0 ; pour les
  autres pays on resterait sur la moyenne annuelle en v1.0 (raffinement v1.1
  via ENTSO-E live).
- **PUE déclaratif** : on prend les chiffres officiels des rapports
  sustainability. Méthodologies de mesure variables entre opérateurs (par
  exemple AWS calcule par région entière, OVH par site). Comparaisons à
  prendre avec une marge d'incertitude ±0.05.
- **WUE manquant** : signifie "non publié par l'opérateur" — pas zéro.
  L'UI doit afficher "n/a" et pas "0.0".

## 5. Mise à jour

Pour ajouter ou modifier un DC :
1. Éditer `crates/sobria-geoloc/data/datacenters.json` (PR sur GitHub).
2. Tous les tests `sobria-geoloc` doivent rester verts (28 ± nouveaux
   ajouts cohérents).
3. Mettre à jour ce catalogue avec les nouvelles sources.
4. Bumper la version du dataset dans `_meta.version` (semver).
