# ADR-0016 — Politique de visibilité par déploiement (étend ADR-0015)

- **Statut** : Accepted (2026-06-12) — implémenté en C44
- **Décideurs** : Thibault, Cowork
- **Contexte** : décision produit du 2026-06-12 — « voir clairement la
  consommation par employé et par projet ». Étend ADR-0015 sans le
  renverser : le défaut reste protecteur, l'identification intégrale
  devient possible pour les organisations qui en assument le cadre légal.

## Contexte et problème

ADR-0015 fixe un comportement unique (k-anonymat + opt-in salarié). Or les
contextes de déploiement diffèrent : une PME de 6 personnes volontaires,
un grand groupe avec DPO et accord CSE, une collectivité. Imposer un seul
réglage soit sur-protège (frustration de l'opérateur légitime), soit
sous-protège (si on inversait le défaut).

## Décision

Une clé de configuration serveur `visibility_policy`, choisie à
l'initialisation (`init --visibility-policy …`) et modifiable ensuite
(`config set`), trois valeurs :

1. **`anonymous`** — strict : agrégats k-anonymes uniquement. Aucune
   identification individuelle côté admin, même volontaire (les opt-in
   sont ignorés à l'affichage).
2. **`opt_in`** — **défaut** : comportement ADR-0015 inchangé. k-anonymat
   sur les agrégats ; n'apparaissent nommément que les salariés ayant
   activé eux-mêmes le partage (révocable).
3. **`identified`** — nominatif intégral : vues par employé sans
   consentement individuel préalable dans l'outil. **Activation refusée
   sans attestation** : l'opérateur fournit une attestation explicite
   (`--attest "…"`) déclarant que le CSE a été informé/consulté
   (L2312-38) et les salariés informés individuellement (L1222-4).
   L'attestation (texte, date, admin) est stockée en base et affichée
   dans le dashboard. Le grain temporel minimal reste le jour, les
   prompts ne sont jamais collectés (invariants ADR-0013/0015).

Règles transverses :

- Le k-anonymat des agrégats s'applique en `anonymous` et `opt_in` ;
  en `identified` il est sans objet (les données individuelles sont
  visibles par politique) et désactivé.
- La **dimension projet** (C44) suit la même politique : les agrégats
  par projet n'affichent que les projets comptant ≥ k contributeurs
  distincts (modes 1-2), les autres étant fondus dans « autres projets ».
- La politique est affichée en permanence dans le dashboard admin et
  l'espace salarié (transparence : chacun sait sous quel régime il est).
- Le site et la doc présentent toujours `opt_in` comme défaut ; le mode
  `identified` n'est jamais un argument commercial premier.

## Conséquences

- (+) Le besoin opérationnel (« qui consomme quoi, par projet ») est
  servi sans imposer la surveillance par défaut à tous les déployeurs.
- (+) La responsabilité juridique est portée par celui qui la détient
  réellement : l'employeur, via une attestation tracée.
- (−) Trois chemins de code à tester (matrice de politique).
- (−) L'attestation est déclarative : Sobr.ia ne peut pas vérifier la
  réalité de l'information du CSE — assumé et documenté.

## Options rejetées

- **Nominatif par défaut** : renverse ADR-0015, contredit la vitrine et
  la candidature, expose les déployeurs négligents.
- **Consentement « coché par l'admin » au nom des salariés** : le
  consentement individuel n'appartient qu'au salarié ; ici c'est une
  POLITIQUE d'organisation assumée comme telle, pas un pseudo-consentement.
