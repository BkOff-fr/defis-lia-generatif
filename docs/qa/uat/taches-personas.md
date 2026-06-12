# Tâches UAT par persona

> Une section par persona (les 5 de `docs/personas/`). Les missions « fil
> rouge » viennent du brief C36 §3 Phase 2 ; elles sont ici découpées en
> tâches observables et chronométrables.
>
> **Mode d'emploi** :
> - Lire chaque **énoncé tel quel**, à voix haute. Ne jamais prononcer le nom
>   d'un module ou d'un bouton : les énoncés sont des objectifs, pas des chemins.
> - **Seuil indicatif** = temps au-delà duquel on note un dépassement.
>   Au **double du seuil**, on clôt la tâche (« non réussie ») et on passe.
> - **Signaux d'échec** = comportements à consigner mot à mot dans la grille,
>   même si la tâche finit par réussir.
> - Tâches **[cœur]** dans l'ordre ; **[bonus]** seulement s'il reste du temps
>   (la phase 2 dure 20 min, voir `script-session.md`).
>
> **À observer transversalement (toutes sessions)** — demandé par C39 §6 :
> le menu **« Plus »** du rail (chevron sous les 5 essentiels Estimer ·
> Comparer · Suivi · Modèles · Datacenters) est-il découvert sans aide ?
> Noter le moment exact de la première ouverture de « Plus », ou son absence.

---

## 1. Étudiant·e / Curieux·se

**Fil rouge C36** : « Tu utilises ChatGPT pour tes devoirs. Tu veux savoir
combien coûte 1 question en CO₂. Et combien tu consommes par semaine si tu
fais 50 questions. »

**Préparation** : app vierge (onboarding non fait). Aucune autre préparation.

### S1 [cœur] — Premier démarrage
- **Énoncé** : « Vous venez d'installer l'application. Démarrez-la et
  configurez-la pour vous, jusqu'à arriver à l'écran principal. »
- **Réussite observable** : onboarding traversé, persona Étudiant·e (ou autre,
  noter lequel) choisi, arrivée sur l'atelier d'estimation avec le rail visible.
- **Seuil indicatif** : 3 min.
- **Signaux d'échec** : hésitation longue sur le choix de persona (« je suis
  quoi, moi ? ») ; bundle de modules incompris ou décoché au hasard ; clic
  « passer » par dépit ; schéma « 1 prompt = 5 m en voiture » ignoré ou incompris.

### S2 [cœur] — Estimer une question (< 5 min, objectif clé C36)
- **Énoncé** : « Estimez l'empreinte carbone d'une question de votre choix,
  comme celle que vous poseriez à ChatGPT pour un devoir. »
- **Réussite observable** : une estimation s'affiche ET le testeur lit
  correctement à voix haute la valeur en gCO₂eq et une équivalence concrète
  (douche, mètres en voiture…).
- **Seuil indicatif** : 3 min (objectif C36 : use case principal < 5 min).
- **Signaux d'échec** : ne sait pas quel modèle choisir (« c'est quoi
  4o-mini ? ») ; bloqué par les champs tokens (« des tokens ? ») ; lit
  l'intervalle P5-P95 comme une erreur ; ne voit pas l'équivalence.

### S3 [cœur] — Extrapoler à la semaine
- **Énoncé** : « Vous posez environ 50 questions par semaine. Trouvez, avec
  l'application, ce que ça représente par semaine — en CO₂ ou en équivalent
  parlant. »
- **Réussite observable** : donne un ordre de grandeur hebdomadaire cohérent
  (≈ 50 × le résultat unitaire), obtenu par n'importe quel moyen dans l'app
  (simulateur, calcul à partir du résultat…). Noter le moyen choisi.
- **Seuil indicatif** : 4 min.
- **Signaux d'échec** : cherche en vain un bouton « × 50 » ; n'ouvre jamais
  « Plus » (le Simulateur y est rangé) ; sort une calculatrice en soupirant ;
  abandonne (« je sais pas où chercher »).

### S4 [cœur] — Retrouver son historique
- **Énoncé** : « Retrouvez la ou les estimations que vous venez de faire, et
  dites-moi quel modèle a le plus émis depuis le début de votre session. »
- **Réussite observable** : ouvre le suivi (tableau de bord) ou le journal et
  désigne le bon modèle.
- **Seuil indicatif** : 2 min.
- **Signaux d'échec** : confond « Suivi » et « Journal d'audit » ; pense que
  rien n'a été enregistré ; cherche dans l'atelier d'estimation.

### S5 [cœur] — Réduire
- **Énoncé** : « Trouvez comment réduire l'empreinte de votre dernière
  question, et dites-moi de combien vous pourriez la réduire. »
- **Réussite observable** : identifie au moins un levier concret (modèle plus
  petit, prompt plus court…) ET cite un chiffre avant/après (le Simulateur
  « Et si...? » est le chemin attendu, mais tout chemin valide compte).
- **Seuil indicatif** : 4 min.
- **Signaux d'échec** : ne découvre pas « Plus » → ne trouve jamais le
  Simulateur (friction C39 à documenter précisément) ; propose seulement
  « utiliser moins l'IA » sans s'appuyer sur l'app ; manipule les leviers
  sans comprendre le verdict.

### S6 [bonus] — Se fixer un budget
- **Énoncé** : « Vous voulez vous donner une limite mensuelle d'empreinte IA
  et être prévenu·e si vous la dépassez. Mettez ça en place. »
- **Réussite observable** : un éco-budget mensuel est enregistré.
- **Seuil indicatif** : 3 min.
- **Signaux d'échec** : ne sait pas quelle valeur choisir (« c'est quoi, un
  budget raisonnable ? ») ; ne trouve pas le module (derrière « Plus »).

---

## 2. Professionnel·le tech

**Fil rouge C36** : « Ton équipe choisit entre GPT-5, Claude 4.7 et Mistral
Large 3 pour intégrer dans une app. Compare leur empreinte sur un prompt
typique de 500 tokens in / 2000 tokens out. »

**Préparation** : app vierge ; zip de l'extension Chrome téléchargé sur le
poste ; un compte ChatGPT/Claude/Le Chat accessible dans le navigateur de test.

### P1 [cœur] — Onboarding express
- **Énoncé** : « Démarrez l'application et configurez-la pour un usage de
  développeur. »
- **Réussite observable** : onboarding traversé, persona Pro tech choisi.
- **Seuil indicatif** : 2 min.
- **Signaux d'échec** : cherche un mode CLI/API dès l'onboarding ; ironise sur
  le wording (« encore un wizard »).

### P2 [cœur] — Comparer 3 modèles (< 5 min, objectif clé C36)
- **Énoncé** : « Votre équipe hésite entre GPT-5, Claude 4.7 et Mistral
  Large 3 pour une feature de résumé : 500 tokens en entrée, 2000 en sortie.
  Comparez leur empreinte et dites-moi lequel vous recommanderiez. »
- **Réussite observable** : comparaison côte à côte affichée avec les 3 bons
  modèles et les bonnes tailles ; le testeur désigne le moins émetteur et
  mentionne spontanément l'incertitude (P50, intervalle) ou un 2ᵉ indicateur
  (énergie, eau).
- **Seuil indicatif** : 5 min.
- **Signaux d'échec** : ne retrouve pas les modèles dans le sélecteur (noms de
  presets vs noms marketing) ; compare avec des tailles par défaut sans les
  ajuster ; lit uniquement le CO₂ et ignore l'intervalle ; doute des chiffres
  sans trouver la source.

### P3 [cœur] — Associer l'extension navigateur
- **Énoncé** : « Vous voulez que vos vrais usages dans le navigateur remontent
  automatiquement dans l'application. Installez l'extension fournie (zip sur le
  bureau) et associez-la à l'application. Faites ensuite un prompt réel sur
  votre assistant habituel et vérifiez que l'estimation est bien remontée. »
- **Réussite observable** : extension chargée (chrome://extensions, mode
  développeur), code de pairing à 6 chiffres généré dans les paramètres de
  l'app, collé dans l'extension ; badge A-F + gCO₂eq visible près du composer ;
  l'estimation apparaît dans le suivi/journal de l'app.
- **Seuil indicatif** : 6 min.
- **Signaux d'échec** : ne trouve pas où générer le code (paramètres) ;
  confusion avec le code d'enrôlement équipe à 12 chiffres ; « load unpacked »
  inconnu ; badge non remarqué ; remontée non vérifiée (croit sur parole).

### P4 [cœur] — Auditabilité
- **Énoncé** : « Votre tech lead veut une trace vérifiable de ces estimations
  pour le reporting trimestriel. Montrez-moi ce que vous lui donneriez. »
- **Réussite observable** : ouvre le Journal d'audit, vérifie la chaîne
  (intégrité SHA-256) ou exporte (NDJSON / JSON-LD PROV-O), et explique en une
  phrase ce que ça garantit.
- **Seuil indicatif** : 3 min.
- **Signaux d'échec** : ne distingue pas Journal et Suivi ; « chaîne SHA-256 »
  ne lui évoque rien dans l'UI ; export introuvable ou format incompris.

### P5 [bonus] — Croiser deux méthodologies
- **Énoncé** : « Vos reviewers demandent si vos chiffres dépendent de la
  méthode de calcul. Évaluez un même prompt selon deux méthodologies et
  dites-moi ce que vous concluez de l'écart. »
- **Réussite observable** : active/voit AFNOR SPEC 2314 et EcoLogits sur la
  même estimation, constate l'écart et le formule (« même ordre de grandeur »,
  « X % d'écart car hypothèses différentes »).
- **Seuil indicatif** : 4 min.
- **Signaux d'échec** : ne trouve pas le choix de méthodologies (derrière
  « Plus ») ; interprète l'écart comme un bug.

### P6 [bonus] — Sourcer un chiffre
- **Énoncé** : « On vous demande d'où sort le chiffre pour Mistral Large.
  Trouvez la source dans l'application. »
- **Réussite observable** : ouvre la fiche modèle dans la Bibliothèque et cite
  l'encadré vendor (Mistral × ADEME) ou la source documentée.
- **Seuil indicatif** : 3 min.
- **Signaux d'échec** : cherche dans « Comment ça marche » sans aboutir ;
  conclut « c'est pas sourcé ».

---

## 3. Entreprise (DSI / RSE)

**Fil rouge C36** : « Tu es DSI dans une PME de 50 personnes. Tu veux savoir
comment déployer un suivi d'usage IA pour toute ton équipe. »

**Préparation** : app vierge ; binaire `sobria-team-aggregator` téléchargé sur
une machine/terminal accessible ; `docs/operations/team-aggregator.md`
ouvrable ; optionnel : 2ᵉ instance pré-alimentée (≥ 5 utilisateurs fictifs).
**Variante non technique** (si aise terminal ≤ 2/5 au pré-test) : l'animateur
exécute E2 en suivant les instructions DICTÉES par le testeur, doc à l'appui.

### E1 [cœur] — Comprendre l'offre équipe
- **Énoncé** : « Vos 50 collaborateurs utilisent ChatGPT et Copilot. Trouvez,
  avec l'application ou sa documentation, comment suivre cet usage au niveau
  de l'entreprise — et dites-moi où iraient les données. »
- **Réussite observable** : identifie le Mode Équipe self-hosted et verbalise
  que le serveur est hébergé PAR l'entreprise (aucun cloud Sobr.ia).
- **Seuil indicatif** : 4 min.
- **Signaux d'échec** : cherche un « compte entreprise » / SSO cloud ; conclut
  que ça n'existe pas ; inquiétude RGPD non levée par ce qu'il lit.

### E2 [cœur] — Déployer le serveur d'équipe
- **Énoncé** : « Déployez ce suivi d'équipe : mettez le serveur en route et
  ouvrez son interface d'administration. Le binaire est déjà téléchargé,
  la documentation est disponible. »
- **Réussite observable** : `init` (admin + mot de passe) puis `serve`
  exécutés ; interface admin ouverte dans le navigateur (https://…:8443/admin)
  malgré l'avertissement TLS auto-signé ; connexion admin réussie.
- **Seuil indicatif** : 8 min (avec doc).
- **Signaux d'échec** : avertissement navigateur « connexion non privée »
  vécu comme bloquant ou louche (noter la réaction mot à mot) ; confusion
  init/serve ; mot de passe exemple `CHANGE-ME` conservé tel quel ; doc non
  trouvée ou non lue.

### E3 [cœur] — Enrôler un collaborateur
- **Énoncé** : « Faites entrer votre premier collaborateur : générez-lui un
  accès, puis associez l'application de ce poste (jouez le collaborateur). »
- **Réussite observable** : code d'enrôlement à 12 chiffres créé dans l'admin,
  saisi dans les paramètres de l'app desktop ; le poste apparaît côté serveur.
- **Seuil indicatif** : 5 min.
- **Signaux d'échec** : confusion code 12 chiffres (équipe) / code 6 chiffres
  (extension) ; ne sait pas où coller le code dans l'app ; doute que ça ait
  marché (pas de feedback perçu).

### E4 [cœur] — Ce qu'un admin voit (et ne voit pas)
- **Énoncé** : « Vous êtes maintenant l'admin. Regardez le tableau de bord
  d'équipe et dites-moi : que pouvez-vous voir de l'activité de vos
  collaborateurs, et que ne pouvez-vous PAS voir ? »
- **Réussite observable** : verbalise correctement le modèle « sans
  surveillance » : agrégats bloqués tant que moins de 5 actifs (k-anonymat),
  pas de totaux individuels sans opt-in explicite du salarié, mention
  « partage non activé ». Avec l'instance pré-alimentée : nomme la différence
  entre participants opt-in et agrégat anonyme.
- **Seuil indicatif** : 4 min.
- **Signaux d'échec — les plus précieux de la session** : interprète le
  k-anonymat comme une panne (« c'est vide, c'est cassé ») ; cherche le
  classement individuel et s'agace de ne pas l'avoir ; à l'inverse, ne croit
  pas à la protection (« ils doivent bien voir quelque part ») ; ne comprend
  pas pourquoi 5.
- **Note animateur** : ne JAMAIS expliquer le k-anonymat avant ou pendant la
  tâche — c'est l'UI qui doit le faire.

### E5 [bonus] — Rapport réglementaire
- **Énoncé** : « Votre comex veut un rapport CSRD sur l'usage IA. Produisez-le
  depuis l'application. »
- **Réussite observable** : génère le rapport (PDF, avec JSON-LD PROV-O) depuis
  le module Rapport réglementaire et identifie à qui il est destiné.
- **Seuil indicatif** : 4 min.
- **Signaux d'échec** : module non trouvé (derrière « Plus ») ; champs du
  formulaire incompris ; doute sur la recevabilité (« je peux vraiment mettre
  ça dans mon rapport ? » — noter pourquoi).

---

## 4. Collectivité / Service public

**Fil rouge C36** : « Tu es responsable développement durable d'une mairie de
30 000 habitants. Tu veux un rapport CSRD-compatible sur l'usage IA d'une
équipe pilote. »

**Préparation** : app vierge ; données Territoire FR pré-ingérées (fetch
territoire-fr + rte-mix, cf. quickstart persona) ; vérifier la carte IRIS.

### C1 [cœur] — Prise en main
- **Énoncé** : « Configurez l'application pour votre collectivité, puis
  estimez l'empreinte d'un prompt type de votre équipe pilote — par exemple
  une demande de rédaction de courrier. »
- **Réussite observable** : onboarding (persona Collectivité) + une estimation
  lue correctement (valeur + équivalence).
- **Seuil indicatif** : 4 min.
- **Signaux d'échec** : mêmes signaux que S1/S2 ; cherche d'emblée une entrée
  « collectivités » dédiée et ne la voit pas dans l'app.

### C2 [cœur] — Empreinte territoriale
- **Énoncé** : « Vous préparez une note pour vos élus : trouvez ce que
  l'application sait de la consommation énergétique industrielle autour de
  votre territoire, et donnez-moi une valeur pour une zone proche de chez vous. »
- **Réussite observable** : ouvre Territoire France (IRIS), navigue/zoome vers
  sa région, lit une valeur de consommation (élec/gaz) pour une maille IRIS.
- **Seuil indicatif** : 5 min.
- **Signaux d'échec** : module non découvert (derrière « Plus ») ; « IRIS » non
  compris (jargon INSEE) ; carte jugée illisible ; ne fait pas le lien entre
  ces données territoriales et l'IA (noter sa formulation exacte).

### C3 [cœur] — Où tournent les modèles
- **Énoncé** : « Un élu vous demande : "nos requêtes IA, elles tournent où,
  physiquement ?" Trouvez de quoi lui répondre pour un fournisseur de votre
  choix. »
- **Réussite observable** : ouvre Datacenters Europe, identifie le ou les
  datacenters d'un fournisseur et formule une réponse plausible (pays/ville,
  prudence sur l'incertitude acceptée).
- **Seuil indicatif** : 3 min.
- **Signaux d'échec** : confond Datacenters Europe et Territoire FR ; prend la
  localisation pour une certitude absolue ; n'arrive pas à filtrer par
  fournisseur.

### C4 [cœur] — Rapport pour l'équipe pilote (< 5 min, objectif clé C36)
- **Énoncé** : « Produisez le rapport réglementaire (type CSRD/AGEC) que vous
  joindriez à votre note, à partir des estimations de votre session. »
- **Réussite observable** : rapport généré (PDF) ; le testeur cite au moins un
  élément de crédibilité (méthodologie AFNOR SPEC 2314, sources, traçabilité).
- **Seuil indicatif** : 5 min.
- **Signaux d'échec** : module non trouvé ; rapport jugé « pas assez officiel »
  ou trop technique pour des élus (noter les termes exacts) ; ne sait pas quoi
  mettre dans les champs.

### C5 [bonus] — Sourcer pour un marché public
- **Énoncé** : « Vous rédigez un appel d'offres "IA frugale". Trouvez dans
  l'application de quoi exiger des candidats des chiffres comparables aux
  vôtres : la méthode utilisée et la licence des données. »
- **Réussite observable** : cite la méthodologie (AFNOR SPEC 2314 /
  EcoLogits) via « Comment ça marche » ou la Datasheet, ET la licence
  Etalab 2.0 des données embarquées.
- **Seuil indicatif** : 4 min.
- **Signaux d'échec** : information éparpillée (navigue partout sans
  conclure) ; licence introuvable ; « Datasheet » non compris.

---

## 5. Chercheur·se / Journaliste

**Fil rouge C36** : « Tu prépares un papier sur l'empreinte IA. Tu veux
reproduire les chiffres Sobr.ia et citer le projet avec DOI. »

**Préparation** : app vierge. Prévoir l'accès au README GitHub du projet
(la citation DOI y est — observer si le testeur l'atteint depuis l'app).

### R1 [cœur] — Estimation et incertitude
- **Énoncé** : « Estimez l'empreinte d'un prompt de votre choix, puis
  expliquez-moi ce que signifient exactement les chiffres affichés — comme si
  vous l'écriviez dans votre papier. »
- **Réussite observable** : estimation faite ; explication correcte de la
  médiane et de l'intervalle P5-P95 (Monte-Carlo), sans confondre intervalle
  d'incertitude et marge d'erreur de mesure.
- **Seuil indicatif** : 3 min.
- **Signaux d'échec** : survole l'intervalle ; demande où sont les hypothèses
  sans les trouver ; vocabulaire UI jugé flou (noter les termes incriminés).

### R2 [cœur] — Comparer sous deux méthodologies
- **Énoncé** : « Pour votre papier, comparez 4 ou 5 modèles de votre choix sur
  un même prompt, selon deux méthodologies différentes, et dites-moi si vos
  conclusions tiendraient face à un reviewer. »
- **Réussite observable** : comparaison multi-modèles affichée ; AFNOR +
  EcoLogits croisées sur au moins un modèle ; écart constaté et commenté.
- **Seuil indicatif** : 5 min.
- **Signaux d'échec** : choix de méthodologies non découvert ; écart
  inter-méthodos pris pour une incohérence du produit ; presets de modèles
  jugés datés ou ambigus (noter lesquels).

### R3 [cœur] — Reproductibilité
- **Énoncé** : « Votre papier doit être reproductible. Trouvez comment un
  tiers pourrait reproduire exactement vos chiffres, et qu'est-ce que vous
  mettriez dans la section "Méthodes". »
- **Réussite observable** : cite au moins deux éléments parmi : seed
  déterministe (SOBRIA_SEED=42), notebook Quarto de validation, méthodologie
  versionnée, hash du référentiel / ledger SHA-256.
- **Seuil indicatif** : 5 min.
- **Signaux d'échec** : ne trouve rien dans l'app et suppose que c'est « dans
  le code quelque part » ; ne fait pas le lien app ↔ notebook ; « Comment ça
  marche » lu mais jugé insuffisant pour une section Méthodes.

### R4 [cœur] — Citer le projet
- **Énoncé** : « Vous citez Sobr.ia dans votre bibliographie. Trouvez comment
  le projet demande à être cité. »
- **Réussite observable** : trouve le DOI Zenodo (via À propos, ou en
  rejoignant le README §Citation depuis l'app/le site).
- **Seuil indicatif** : 3 min.
- **Signaux d'échec attendus** : cherche dans l'app sans aboutir (le DOI vit
  dans le README — friction probable, la documenter précisément : où a-t-il
  cherché, dans quel ordre ?).

### R5 [bonus] — Exporter des données auditables
- **Énoncé** : « Exportez les estimations de votre session dans un format que
  vous pourriez publier en données supplémentaires de votre papier. »
- **Réussite observable** : export obtenu (JSON-LD PROV-O / NDJSON depuis le
  Journal, ou datasheet du module Datasheet scientifique) ; sait dire ce que
  le format garantit.
- **Seuil indicatif** : 4 min.
- **Signaux d'échec** : hésite entre Journal / Datasheet / Rapport sans
  comprendre leurs rôles ; format inconnu (« c'est quoi PROV-O ? ») sans
  explication accessible.

---

## Récapitulatif des objectifs « < 5 min » (à reporter en synthèse)

| Persona | Tâche clé C36 | Réussie < 5 min sans aide ? |
|---|---|---|
| Étudiant·e | S2 (estimer une question) | oui / non |
| Pro tech | P2 (comparer 3 modèles) | oui / non |
| Entreprise | E1+E2 (comprendre puis déployer le suivi équipe) | oui / non (E1 seule < 5 min) |
| Collectivité | C4 (rapport réglementaire) | oui / non |
| Chercheur·se | R3 (reproductibilité) | oui / non |
