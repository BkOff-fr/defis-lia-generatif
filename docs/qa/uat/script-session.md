# Script de session UAT — déroulé minute par minute

> 60 min réservées, 45 min de test effectif (protocole C36 §3).
> Garder ce script sous les yeux ; les énoncés de tâches sont dans
> `taches-personas.md` (section du persona du jour) ; tout se note dans la
> copie de `grille-observation.md`.

---

## T-15 min — Pré-vol (avant l'arrivée du testeur)

- [ ] Poste de test : app desktop lancée puis fermée, base réinitialisée,
  localStorage WebView vidé → l'onboarding doit repartir de zéro
  (procédure : `docs/qa/smoke-test-v0.8.0-2026-05.md` § Pré-vol).
- [ ] Navigateur propre, onglet vierge (ne PAS pré-ouvrir le site).
- [ ] Préparation spécifique persona faite (voir `taches-personas.md` § Préparation).
- [ ] Enregistrement testé (micro + écran). Chronomètre prêt.
- [ ] Grille copiée : `sessions/uat-session-<persona>-<date>.md`, en-tête rempli.
- [ ] Consentement prêt à signer.

---

## 00:00 → 05:00 — Accueil et consentement (5 min)

À dire (l'esprit, pas mot à mot) :

> « Merci d'être là. On teste une application, pas vous : il n'y a aucune
> bonne ou mauvaise réponse, et si quelque chose ne marche pas, c'est une
> information précieuse pour nous, jamais une erreur de votre part.
> Je vais vous demander de **penser à voix haute** en continu : dites-moi ce
> que vous cherchez, ce que vous comprenez, ce qui vous agace.
> Pendant les exercices, **je ne pourrai pas vous aider** — c'est justement
> ce qu'on mesure. Si vous bloquez, dites-le et on continue, ce n'est pas grave.
> Vous pouvez arrêter à tout moment. »

- [ ] Faire signer le consentement (modèle dans `README.md` §6).
- [ ] Démarrer l'enregistrement et **annoncer à voix haute** : « il est [heure],
  session T[n], persona [x], le consentement est signé ».

## 05:00 → 10:00 — Pré-test : contexte d'usage IA du participant (5 min)

Poser, noter dans la grille (§ Pré-test) :

1. « Quels assistants IA utilisez-vous ? (ChatGPT, Claude, Le Chat, Gemini,
   Copilot…) Version gratuite ou payante ? »
2. « À quelle fréquence, et pour faire quoi ? » (études, code, rédaction, perso…)
3. « Avez-vous déjà entendu parler de l'impact environnemental de l'IA ?
   Qu'est-ce que vous en savez, en une phrase ? »
4. « Sur une échelle de 1 à 5, à l'aise avec l'informatique ? Et avec un
   terminal / la ligne de commande ? » (le 2ᵉ point conditionne la variante
   de la tâche admin pour le persona Entreprise)
5. « Quel OS et quel navigateur au quotidien ? »

Ne rien expliquer de Sobr.ia à ce stade.

## 10:00 → 20:00 — Phase 1 : premier contact avec le site (10 min)

Protocole C36 §3 Phase 1 — le testeur n'a jamais vu Sobr.ia.

1. Donner le lien `https://sobria.brilliantstudio.co/` :
   > « Voici un lien. Explorez librement pendant 5 minutes, comme si vous
   > étiez tombé dessus depuis un moteur de recherche. À voix haute. »
2. **5 min d'exploration, zéro intervention.** Observer et noter : où va le
   regard ? Lit-il le hero ? Scrolle-t-il ? Clique-t-il « Télécharger », « Doc » ?
   Entre-t-il dans la démo web ? Remarque-t-il la bannière « DÉMO » ?
3. À 5 min, poser les 3 questions C36 (réponses mot à mot dans la grille) :
   - « En 1 phrase, c'est quoi Sobr.ia ? »
   - « Pour qui c'est fait, selon vous ? »
   - « Vous cliqueriez sur quoi en premier ? »

Note animateur : la démo web est volontairement limitée (bannière ambre,
certaines actions répondent « Application de bureau requise ») — si le testeur
bute dessus, noter sa réaction, ne pas justifier.

## 20:00 → 40:00 — Phase 2 : tâches du persona (20 min)

Basculer sur le **poste de test** (app desktop, état vierge).

> « On passe sur l'application installée. Je vais vous confier des missions,
> une par une. Faites comme chez vous, à voix haute. »

Pour chaque tâche de `taches-personas.md` (section du persona) :

1. Lire l'énoncé **tel quel**, le répéter si demandé, ne rien reformuler d'autre.
2. Démarrer le chrono à la fin de l'énoncé ; noter l'heure de début.
3. Observer en silence. Relancer le think-aloud si silence > 30 s.
4. **Règle des 2 min** : stagnation > 2 min = friction à noter, sans intervenir.
5. Arrêter à la réussite (critère observable atteint), à l'abandon, ou au
   **double du seuil indicatif** → « parfait, on passe à la suite ».
6. Remplir la ligne de grille à chaud : réussite (oui / avec aide / non),
   temps, verbatims, frictions, gravité.

Faire les tâches **cœur** dans l'ordre ; les tâches **bonus** seulement s'il
reste du temps avant 40:00. À 40:00, couper proprement même en cours de tâche
(noter « non terminée — temps de session »).

## 40:00 → 52:00 — Phase 3 : débrief (12 min)

### SUS — System Usability Scale (5-6 min)

> « Dix affirmations ; pour chacune, donnez une note de 1 "pas du tout
> d'accord" à 5 "tout à fait d'accord". En pensant uniquement à
> l'application que vous venez d'utiliser. Sans trop réfléchir. »

1. Je pense que j'utiliserais Sobr.ia fréquemment.
2. J'ai trouvé Sobr.ia inutilement complexe.
3. J'ai trouvé Sobr.ia facile à utiliser.
4. J'aurais besoin de l'aide d'un technicien pour utiliser Sobr.ia.
5. Les différentes fonctions de Sobr.ia sont bien intégrées.
6. Il y a trop d'incohérences dans Sobr.ia.
7. La plupart des gens apprendraient très vite à utiliser Sobr.ia.
8. J'ai trouvé Sobr.ia très lourd à utiliser.
9. Je me suis senti·e en confiance en utilisant Sobr.ia.
10. J'ai dû beaucoup apprendre avant de me sentir à l'aise avec Sobr.ia.

Noter les 10 réponses brutes (1-5) dans la grille. **Calcul** (après la
session) : items impairs → note − 1 ; items pairs → 5 − note ; somme des 10
contributions × 2,5 = score sur 100. Cible C36 : moyenne ≥ 70.

### 5 questions ouvertes C36 (5-6 min)

1. « Quel mot décrit Sobr.ia ? »
2. « Qu'est-ce qui vous a surpris, en bien ou en mal ? »
3. « Vous le recommanderiez à un·e ami·e ? Pour faire quoi ? »
4. « Si vous pouviez ajouter UNE chose ? »
5. « Si vous pouviez retirer UNE chose ? »

Relance unique autorisée : « pourquoi ? ». Verbatims mot à mot.

## 52:00 → 55:00 — Clôture (3 min)

- Remercier ; remettre la compensation.
- Répondre maintenant (et seulement maintenant) aux questions restées en
  suspens pendant le test.
- Demander : « d'accord pour être recontacté·e quand la v1.0 sort ? »
- Arrêter l'enregistrement. Annoncer l'heure de fin.

## 55:00 → +15 min — À chaud (animateur seul)

- Compléter la grille pendant que c'est frais ; calculer le SUS.
- Surligner : 3 pires moments, meilleur verbatim, 1 surprise.
- Ranger l'enregistrement (nommage : `uat-T<n>-<persona>-<date>`).
- Réinitialiser le poste de test pour la session suivante.

---

## Variante asynchrone (format de repli C36)

Si la session live est impossible : envoyer au testeur (1) le lien du site,
(2) le lien de téléchargement de l'app, (3) la liste des tâches cœur de son
persona recopiée SANS les critères de réussite ni les seuils, (4) la consigne
de s'enregistrer (écran + voix, outil au choix) en pensant à voix haute,
(5) le questionnaire SUS + 5 questions ouvertes dans un document à remplir,
(6) le consentement à retourner signé AVANT l'envoi de la vidéo.
Au dépouillement : remplir une grille d'observation normalement, en notant
`format: async` dans l'en-tête. Limite connue : pas de règle des 2 min ni de
relance think-aloud — gravités à apprécier avec prudence.
