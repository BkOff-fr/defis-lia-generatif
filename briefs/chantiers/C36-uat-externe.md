# Chantier C36 — UAT externe 5 testeurs

> **Version cible** : feedback intégré dans v1.0
> **Sprint** : 3-5 jours calendaires (en parallèle de C35 polish final)
> **Pré-requis** : v0.9.0 (C34 catalogue + modalités) + site-v0.1.0 (C33) shippés
> **Type** : chantier **organisationnel + recherche utilisateur**, pas du code (sauf patches express en sortie)
> **Audience** : Thibault (recruteur + facilitateur) + Cowork (protocole + synthèse)

---

## 0. Pourquoi ce chantier

L'audit C32.0 a donné un score moyen 6/10 sur les 5 personas. C32 + C35.4 polishent en interne, mais **rien ne remplace un vrai utilisateur** qui découvre Sobr.ia pour la première fois sans coaching. Bugs UX, malentendus, "où est ce bouton ?" — autant de signaux qu'on ne peut pas générer en interne.

**Objectif UAT** : valider que les 5 personas peuvent **atteindre leur use case principal en moins de 5 minutes** sans assistance, et que les wordings résonnent.

---

## 1. Périmètre

### En périmètre

- Recrutement **5 testeurs externes** (idéalement 1 par persona, sinon 2 Student + 1 chaque autres).
- Préparation **protocole de test** : 3 use cases par persona, scénarios concrets.
- **Sessions** (live visio 45 min OU async vidéo screen-record).
- **Récolte feedback structuré** : scoring SUS (System Usability Scale) + qualitative notes + verbatims clés.
- **Debrief** : synthèse, patches identifiés, priorisation.
- **Patches express** intégrés à C35 ou v1.0.1 selon criticité.

### Hors périmètre

- Tests fonctionnels / régression (couvert par cargo test + Playwright).
- Tests de charge / performance.
- Tests sécurité.
- A/B testing de wordings (pas le moment).

---

## 2. Profils testeurs recherchés

| Persona | Profil cible | Comment recruter |
|---|---|---|
| **Student** | Étudiant·e (lycée terminal / supérieur) curieux·se de l'IA, sans bagage tech | Réseau Thibault (amis, famille), réseaux sociaux (Discord étudiants), aucun pré-requis |
| **Pro Tech** | Dev / ML eng en activité, utilise au moins 1 API LLM régulièrement | Réseau pro Thibault, LinkedIn, communauté Rust FR / Tauri / Svelte |
| **Enterprise** | RSE / DSI / acheteur·euse responsable, organisation 20-500 personnes | Réseau pro, contacts entreprises de la région, idéalement client/prospect actuel ou ancien |
| **Public Sector** | Agent·e collectivité / administration centrale en charge du numérique responsable | LinkedIn (recherche "DPO collectivité" / "numérique responsable mairie"), réseau Etalab si dispo |
| **Researcher** | Doctorant·e / journaliste tech en sciences environnementales ou IA | Twitter académique, réseau labos en informatique frugale (ex: École Polytechnique, Inria) |

**Compensations** : si testeurs externes, prévoir un café/bières offerts ou bon Amazon 20-30€ pour les session de 45 min. À discuter avec Thibault.

---

## 3. Protocole de test (45 min par session)

### Phase 1 — Premier contact (10 min)

- Testeur n'a JAMAIS vu Sobr.ia avant.
- Lui envoyer le lien `https://sobria.brilliantstudio.co/` et lui demander d'**explorer librement 5 minutes** sans coaching.
- Observer (sans intervenir) : où va-t-il/elle ? Lit-il/elle le Hero ? Scroll-il/elle vers le bas ? Clique-t-il/elle "Télécharger" ou "Doc" ?
- Après 5 min, demander en open-ended :
  - *« En 1 phrase, c'est quoi Sobr.ia ? »*
  - *« Pour qui c'est fait selon toi ? »*
  - *« Tu cliquerais sur quoi en premier ? »*

### Phase 2 — Use case persona (20 min)

Donner une **mission concrète** par persona :

- **Student** : *« Tu utilises ChatGPT pour tes devoirs. Tu veux savoir combien coûte 1 question en CO₂. Et combien tu consommes par semaine si tu fais 50 questions. »*
- **Pro Tech** : *« Ton équipe choisit entre GPT-5, Claude 4.7 et Mistral Large 3 pour intégrer dans une app. Compare leur empreinte sur un prompt typique de 500 tokens in / 2000 tokens out. »*
- **Enterprise** : *« Tu es DSI dans une PME de 50 personnes. Tu veux savoir comment déployer un suivi d'usage IA pour toute ton équipe. »*
- **Public Sector** : *« Tu es responsable développement durable mairie 30k habitants. Tu veux un rapport CSRD-compatible sur l'usage IA d'une équipe pilote. »*
- **Researcher** : *« Tu prépares un papier sur l'empreinte IA. Tu veux reproduire les chiffres Sobr.ia et citer le projet avec DOI. »*

Observer ce qui marche et ce qui bloque. Si le testeur stagne > 2 min, **prendre note** mais essayer de ne pas aider immédiatement (sauf si abandon imminent).

### Phase 3 — Wrap-up (15 min)

Questionnaire SUS (10 questions standard, score 0-100). Plus 5 questions ouvertes :

1. Quel mot décrit Sobr.ia ?
2. Qu'est-ce qui t'a surpris (positivement ou négativement) ?
3. Tu recommandes à un·e ami·e ? Pour quoi faire ?
4. Si tu pouvais ajouter UNE chose, ce serait quoi ?
5. Si tu pouvais retirer UNE chose, ce serait quoi ?

Enregistrer la session (audio + screen) avec consentement explicite. Notes complètes par Thibault ou Cowork dans `docs/qa/uat-session-<persona>-<date>.md`.

---

## 4. Livrables

- `docs/qa/uat-protocole-v1.0.md` (ce brief opérationnalisé).
- `docs/qa/uat-session-<persona>-<date>.md` × 5 (1 par session).
- `docs/qa/uat-synthese-v1.0.md` : synthèse cross-personas avec :
  - Score SUS moyen + par persona.
  - Top 5 frictions identifiées (priorisées par fréquence × criticité).
  - Top 5 verbatims positifs (pour le pitch candidature).
  - Recommandations patches express pré-v1.0.
- Si patches identifiés bloquants : ajouter à C35 ou ouvrir patches v0.9.x au fil de l'eau.

---

## 5. Definition of Done

- [ ] 5 sessions UAT réalisées.
- [ ] 5 docs `uat-session-*.md` rédigés.
- [ ] 1 doc synthèse `uat-synthese-v1.0.md`.
- [ ] Score SUS global ≥ 70 (industrie : 68 = moyen).
- [ ] Top 3 frictions identifiées et patchées avant tag v1.0.
- [ ] 5 verbatims positifs extraits pour intégration dossier candidature data.gouv.fr.

---

## 6. Découpage temporel

| Jour | Activité |
|---|---|
| J1 | Préparation protocole final + recrutement (5 testeurs confirmés) |
| J2-J4 | 5 sessions UAT (1-2 par jour selon dispo testeurs) |
| J5 | Synthèse + patches express bloquants identifiés |

Total : **3-5 jours calendaires** selon dispo des testeurs.

---

## 7. Risques + mitigations

| Risque | Mitigation |
|---|---|
| Pas de testeur disponible pour 1 persona | Faire avec 4 sur 5 + 2 Student. Mieux que zéro. |
| Testeur abandonne en cours de session | Noter le point d'abandon = friction critique. |
| Bugs critiques découverts | Patches express dans v0.9.x avant v1.0. |
| Score SUS < 70 | Reporter v1.0 candidature de 1-2 semaines pour intégrer findings. |

---

## 8. Lien avec C35

C36 tourne **en parallèle** de C35. Sortie C36 = patches identifiés à intégrer dans C35.4 (polish personas) ou dans patches v0.9.x express.

Si on doit choisir entre tagger v1.0 ou patcher des findings UAT : **on patche**. v1.0 candidature doit être solide.
