# Sobr.ia pour Collectivité / Service public

> **Suivre votre empreinte IA territoriale, cadrer vos marchés
> publics frugaux, justifier vos choix sur des sources officielles
> françaises (ADEME, RTE, ARCEP).**

---

## Qui c'est ?

Vous travaillez dans une commune, une métropole, un conseil
départemental ou régional, un ministère, une agence publique ou un
EPCI. Vous lancez des appels d'offres qui intègrent de l'IA
générative et vous devez :

- **Mesurer l'empreinte territoriale** de ces usages (gCO₂eq par
  IRIS, en croisant ComparIA et RTE IRIS).
- **Cadrer vos marchés publics** avec des critères frugaux
  mesurables, en cohérence avec les obligations AGEC.
- **Justifier vos choix de fournisseur** sur des données souveraines
  Etalab 2.0 (ADEME Base Empreinte, ARCEP, RTE).

## Ce que Sobr.ia résout pour vous

| Question | Réponse Sobr.ia |
|---|---|
| « Comment cartographier l'usage IA par IRIS sur mon territoire ? » | **Territoire FR** — IRIS RTE/NaTran/Teréga × ComparIA + Sankey énergétique national |
| « Comment intégrer des critères IA frugale dans un marché public ? » | **Datasheet scientifique** + **Rapport réglementaire (CSRD/AGEC)** — chiffres sourcés Etalab 2.0 |
| « Comment comparer plusieurs candidats sur l'empreinte ? » | **Bibliothèque de modèles** avec encadrés vendor disclosure (Mistral × ADEME en priorité — modèle FR avec ACV publié) |

## Top 3 use cases

1. **Mesurer l'empreinte IA d'un service public** — cartographier
   les sites industriels et datacenters par IRIS, croiser avec les
   modèles utilisés, sortir un bilan territorial.
2. **Cadrer un appel d'offre IA frugale** — utiliser la Datasheet
   scientifique comme template, exiger des fournisseurs qu'ils
   publient leurs disclosures (à l'image de Mistral × ADEME).
3. **Produire un rapport AGEC annuel** — Rapport réglementaire en
   PDF + JSON-LD PROV-O, conforme AFNOR SPEC 2314, traçabilité
   complète.

## Modules pertinents

- **Estimer un prompt** — atelier de mesure
- **Comment ça marche** — méthodologie sourcée Etalab 2.0
- **Datacenters Europe** — voir où sont les datacenters de vos
  fournisseurs
- **Datasheet scientifique** — template reproductibilité pour
  marchés publics
- **Territoire FR** — IRIS RTE + Sankey énergétique
- **Rapport réglementaire (CSRD/AGEC)** — output AGEC

## Quickstart 5 minutes

```bash
# 1. Téléchargez le binaire Sobr.ia depuis Releases GitHub
# 2. Lancez l'app et choisissez "Collectivité / Service public"
#    dans l'onboarding

# 3. Pulls des données officielles ODRÉ / RTE / IRIS (1 fois)
cargo run -p sobria-ingest -- fetch territoire-fr --limit 200
cargo run -p sobria-ingest -- fetch rte-mix --year 2023

# 4. Ouvrez le module Territoire FR — carte IRIS interactive
# 5. Sortez un Rapport réglementaire depuis le module dédié
```

**Toutes les données embarquées sont sous licence Etalab 2.0**
(sources ODRÉ/RTE/AIB) — utilisables dans un cadre de marché
public sans contrainte de licence privée.

## Marchés publics — éléments de cadrage

Sobr.ia v1.0 fournit déjà :

- **Méthodologie publique référencée AFNOR SPEC 2314** — à exiger
  des candidats.
- **Audit chaîné SHA-256** — preuve de non-altération des chiffres
  fournis.
- **Datasheet Gebru** — template scientifique reconnu (NeurIPS, FAccT).
- **JSON-LD PROV-O** — format normalisé pour traçabilité W3C.

Un module dédié **Marchés publics IA frugale** (cahiers des charges
types) est prévu en v1.1+.

## Pour aller plus loin

- [Catalogue sources officielles FR](../sources/CATALOGUE-SOURCES.md)
- [ADR-0009 — Architecture médaillon (lineage Etalab 2.0)](../adr/ADR-0009-medallion-architecture.md)
- [Audit datasets Q3 2026](../sources/AUDIT-2026-Q3.md)
