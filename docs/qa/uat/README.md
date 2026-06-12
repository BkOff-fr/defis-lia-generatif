# Kit UAT Sobr.ia — Protocole de tests utilisateurs externes (C36)

> **Statut** : kit prêt à dérouler — version 1.0, 2026-06-12.
> **Origine** : ce kit opérationnalise le brief `briefs/chantiers/C36-uat-externe.md`.
> Le protocole (phases, missions, SUS, DoD) vient du brief ; ce dossier le rend
> exécutable sans préparation supplémentaire.
> **Version testée** : app desktop v0.9.x + site/démo web (C37) + rail simplifié (C39).
> **Animateur** : Thibault. **Synthèse** : Cowork.

---

## 1. Objectif (rappel C36)

Valider que **chacun des 5 personas atteint son use case principal en moins de
5 minutes sans assistance**, et que les wordings résonnent. L'audit interne C32.0
a donné 6/10 en moyenne : rien ne remplace un vrai utilisateur qui découvre
Sobr.ia pour la première fois sans coaching.

Critères de succès globaux (Definition of Done C36) :

- [ ] 5 sessions réalisées (1 par persona ; repli : 4 personas + 2 Étudiants).
- [ ] 1 grille d'observation remplie par session (`sessions/uat-session-<persona>-<date>.md`).
- [ ] 1 synthèse remplie (`synthese-template.md` → `uat-synthese-v1.0.md`).
- [ ] Score SUS global ≥ 70 (référence industrie : 68 = moyen).
- [ ] Top 3 frictions identifiées **et patchées** avant le tag v1.0.
- [ ] 5 verbatims positifs extraits pour le dossier de candidature data.gouv.fr.

Si SUS < 70 ou friction bloquante : **on patche avant de tagger v1.0** (règle C36 §8).

## 2. Ordre des documents du kit

1. `README.md` (ce fichier) — à lire avant le recrutement. Préparer le matériel (§5).
2. Consentement RGPD (§6) — à imprimer/copier, **signer avant tout enregistrement**.
3. `script-session.md` — le déroulé minute par minute, à garder sous les yeux pendant la session.
4. `taches-personas.md` — ouvrir la section du persona du jour ; lire les énoncés à voix haute.
5. `grille-observation.md` — copier en `sessions/uat-session-<persona>-<date>.md` avant chaque session, remplir pendant.
6. `synthese-template.md` — copier en `uat-synthese-v1.0.md`, remplir après les 5 sessions.

## 3. Recrutement

**Combien** : 5 testeurs externes, idéalement 1 par persona. Repli (C36 §7) : si un
persona est introuvable, faire 4 sur 5 + 2 Étudiants. Mieux que zéro.

**Condition absolue** : le testeur n'a **jamais vu Sobr.ia** (ni l'app, ni le site, ni une démo).

| Persona | Profil cible | Canaux de recrutement |
|---|---|---|
| Étudiant·e | Lycée terminale / supérieur, curieux·se de l'IA, sans bagage tech | Réseau perso Thibault, Discord étudiants, réseaux sociaux |
| Pro tech | Dev / ML eng en activité, utilise au moins 1 API LLM régulièrement | Réseau pro, LinkedIn, communautés Rust FR / Tauri / Svelte |
| Entreprise | RSE / DSI / acheteur·euse responsable, organisation 20-500 personnes | Réseau pro, contacts entreprises de la région, client/prospect |
| Collectivité | Agent·e collectivité ou administration, numérique responsable | LinkedIn (« DPO collectivité », « numérique responsable mairie »), réseau Etalab |
| Chercheur·se | Doctorant·e / journaliste tech (environnement ou IA) | Réseaux académiques, labos informatique frugale (Inria, Polytechnique) |

**Compensation** (C36 §2) : café/bières offerts ou bon d'achat 20-30 € par session
de 45-60 min. À confirmer avec Thibault avant l'invitation.

**Message d'invitation type** : « Je cherche 5 personnes pour tester une application
qui mesure l'impact environnemental de l'IA (45-60 min, visio ou sur place,
défrayé). Aucune préparation : c'est l'application qu'on teste, pas vous. »
Ne pas en dire plus — le premier contact fait partie du test.

## 4. Format et durée

- **Durée réservée : 60 min** par session, dont **45 min de test effectif**
  (conforme C36 : 10 min premier contact + 20 min tâches + 15 min débrief),
  encadrées par l'accueil et la clôture. Détail dans `script-session.md`.
- **Format privilégié : live** (visio avec partage d'écran + contrôle du poste de
  test, ou en présentiel sur le poste préparé).
- **Format de repli : asynchrone** — le testeur s'enregistre (écran + voix) en
  déroulant le script qu'on lui envoie. Adaptations en fin de `script-session.md`.
- **1 à 2 sessions par jour maximum** (J2-J4 du planning C36 §6) : prendre le temps
  de compléter la grille à chaud après chaque session.

## 5. Matériel — checklist de préparation

À vérifier **avant chaque session** (15 min de pré-vol, détail dans `script-session.md`) :

- [ ] **Poste de test** avec Sobr.ia desktop v0.9.x installée et lancée une fois,
  puis **base locale réinitialisée** + localStorage WebView vidé (procédure exacte :
  `docs/qa/smoke-test-v0.8.0-2026-05.md` § Pré-vol). Le testeur doit voir l'onboarding vierge.
- [ ] **Navigateur propre** (profil Chrome dédié) avec accès au site
  `https://sobria.brilliantstudio.co/` (phase 1) et l'archive de l'extension
  `sobria-extension-chrome-v0.6.x.zip` déjà téléchargée (persona Pro tech).
- [ ] **Persona Entreprise uniquement** : binaire `sobria-team-aggregator` téléchargé
  sur une machine/VM accessible + terminal prêt + `docs/operations/team-aggregator.md`
  ouvrable. Optionnel : une seconde instance pré-alimentée (≥ 5 utilisateurs fictifs)
  pour montrer le dashboard débloqué.
- [ ] **Persona Collectivité uniquement** : données Territoire FR pré-ingérées
  (`cargo run -p sobria-ingest -- fetch territoire-fr --limit 200` et `fetch rte-mix`),
  vérifier que la carte IRIS s'affiche.
- [ ] **Enregistrement** : visio avec enregistrement (audio + écran) ou OBS en local.
  Tester micro + capture avant l'arrivée du testeur.
- [ ] **Documents** : consentement (2 exemplaires), grille d'observation copiée et
  renommée, section persona de `taches-personas.md` relue, chronomètre.
- [ ] **Compensation** prête.

## 6. Consentement RGPD — modèle court

À faire signer (papier ou copie numérique datée) **avant** de lancer l'enregistrement.
En visio : envoyer avant la session, recueillir le « oui » oral enregistré en début
de captation + retour signé par e-mail.

```
CONSENTEMENT — Test utilisateur Sobr.ia

Responsable du traitement : Thibault, projet Sobr.ia (candidature défi
data.gouv.fr « Impact environnemental de l'IA générative »).
Contact : [adresse e-mail de contact — à compléter avant impression].

Finalité : améliorer l'application Sobr.ia. La session est un test du
LOGICIEL, pas de la personne. Aucune bonne ou mauvaise réponse.

Données collectées : enregistrement audio et capture d'écran de la session,
notes d'observation, réponses aux questionnaires. Aucune donnée n'est
collectée sur votre matériel personnel. Les prompts saisis pendant le test
restent locaux à la machine de test (l'application n'envoie rien en ligne).

Base légale : votre consentement (art. 6.1.a RGPD).

Conservation : enregistrements bruts supprimés au plus tard 6 mois après la
session. Les citations utilisées dans les documents du projet sont
anonymisées (identifiant T1 à T5).

Destinataires : équipe projet Sobr.ia uniquement. Aucune diffusion publique
de l'enregistrement. Des citations anonymisées peuvent figurer dans le
dossier de candidature data.gouv.fr.

Vos droits : accès, rectification, effacement, retrait du consentement à
tout moment (y compris en cours de session), sans justification et sans
conséquence. Réclamation possible auprès de la CNIL (cnil.fr).

[ ] J'accepte l'enregistrement audio + écran de la session.
[ ] J'accepte que des citations anonymisées soient réutilisées.

Date :            Nom :                  Signature :
```

Si le testeur refuse l'enregistrement : faire la session quand même, notes
manuscrites uniquement (le noter dans la grille).

## 7. Rôle de l'animateur

Règles d'or, dans l'ordre où elles sauvent une session :

1. **Faire verbaliser (think-aloud)** : demander au testeur de penser à voix haute
   en continu (« dites-moi ce que vous cherchez, ce que vous comprenez, ce qui vous
   étonne »). S'il se tait > 30 s : « qu'est-ce que vous regardez, là ? ».
2. **Ne pas aider. Jamais spontanément.** Répondre aux questions par des questions
   (« vous, vous feriez quoi ? », « qu'est-ce que vous attendriez ici ? »).
3. **Règle des 2 minutes (C36 §3)** : si le testeur stagne plus de 2 min sur un
   point, **noter la friction** mais ne pas intervenir, sauf abandon imminent.
   Si on doit débloquer pour continuer : donner l'indice minimal, et marquer la
   tâche « Avec aide » dans la grille.
4. **Passer à la tâche suivante** quand le double du seuil indicatif est dépassé
   (les seuils sont dans `taches-personas.md`). Dire : « ce n'est pas vous, c'est
   exactement ce qu'on cherchait à savoir — on passe à la suite ».
5. **Neutralité** : ne pas vendre le produit, ne pas se justifier (« oui c'est prévu
   en v1.1 » = interdit pendant le test ; garder ça pour la clôture).
6. **Noter les verbatims mot à mot**, avec les guillemets, à la volée — c'est la
   matière première de la synthèse et de la candidature.
7. **Horodater** le début de chaque tâche (l'enregistrement permet de recouper).

## 8. Après chaque session, puis après la campagne

- À chaud (15 min) : compléter la grille, calculer le score SUS, surligner les
  3 moments les plus durs et le meilleur verbatim.
- Après les 5 sessions : remplir `synthese-template.md`, prioriser les frictions
  (gravité × fréquence), trancher : patch express pré-v1.0 (→ C35 / v0.9.x) ou backlog.

## 9. Correspondance avec les livrables C36

| Livrable C36 §4 | Dans ce kit |
|---|---|
| `uat-protocole-v1.0.md` | `README.md` + `script-session.md` + `taches-personas.md` |
| `uat-session-<persona>-<date>.md` × 5 | copies remplies de `grille-observation.md` dans `sessions/` |
| `uat-synthese-v1.0.md` | `synthese-template.md` rempli |
| Patches express | section Décisions de la synthèse → C35 / v0.9.x |
