# Modèles de communication — Mode Équipe Sobr.ia (CSE + salariés)

> Deux emails prêts à adapter avant la mise en service du Mode Équipe :
> **(A)** information-consultation du CSE, **(B)** annonce aux salariés.
>
> **Mode d'emploi** : remplacer les champs `[entre crochets]`, vérifier que
> les valeurs annoncées (seuil de k-anonymat, durée de rétention)
> correspondent à votre configuration réelle (`config list`, cf.
> [`deploiement-equipe.md`](deploiement-equipe.md) §7), joindre les annexes.
>
> ⚠️ **Ces modèles sont une aide opérationnelle, pas un conseil juridique :
> à faire valider par votre juriste ou votre DPO avant envoi.** Le cadre de
> référence (L2312-38, L1222-4, registre RGPD art. 30) est rappelé dans
> `team-aggregator.md` § « Privacy et conformité (ADR-0015 — C38) ».

---

## Modèle A — Information-consultation du CSE

**Objet : Information-consultation du CSE — projet de déploiement d'un
outil de mesure de l'empreinte environnementale de l'IA (Sobr.ia Mode
Équipe)**

Mesdames, Messieurs les membres du CSE,

Conformément à l'article L2312-38 du Code du travail, nous vous informons
et vous consultons sur le projet de déploiement de l'outil **Sobr.ia Mode
Équipe** au sein de [Entreprise], envisagé à compter du [date].

**1. Contexte et finalité**

L'usage d'assistants d'IA générative (ChatGPT, Claude, Gemini, etc.) se
développe dans l'entreprise. Sobr.ia permet d'en estimer l'empreinte
environnementale (énergie, gCO₂eq) et le volume d'usage, afin de :

- piloter notre budget et notre trajectoire environnementale (démarche
  RSE, reporting CSRD le cas échéant) ;
- objectiver nos choix d'outils et de modèles d'IA, et sensibiliser à un
  usage plus sobre.

La finalité est exclusivement le **pilotage budgétaire et environnemental
de l'usage de l'IA générative**. L'outil n'est pas un dispositif
d'évaluation des salariés et ne sera pas utilisé à cette fin ; sa
conception même l'empêche (voir garanties ci-dessous).

**2. Fonctionnement**

Chaque salarié volontaire installe une extension de navigateur qui estime
localement, sur son poste, l'empreinte de ses requêtes d'IA. Seules des
métriques techniques sont transmises à un serveur **interne à
l'entreprise** ([URL du serveur]) — aucun éditeur tiers, aucun cloud
externe ne reçoit de données.

**3. Données traitées**

- Collecté : le modèle d'IA utilisé, des compteurs de volume (tokens),
  l'estimation d'énergie et de gCO₂eq associée, l'horodatage, et le compte
  interne du salarié enrôlé.
- **Jamais collecté** : le contenu des conversations (prompts et
  réponses), l'historique de navigation, les frappes clavier, les
  fichiers. Ces données ne quittent jamais le poste de travail.

**4. Garanties techniques (appliquées côté serveur, non contournables
par l'administrateur)**

Ces règles sont documentées publiquement dans la décision d'architecture
ADR-0015 du projet, jointe au présent dossier :

- **Agrégats anonymes (k-anonymat)** : aucune statistique d'équipe n'est
  affichée si moins de [5] salariés sont actifs sur la période consultée ;
- **Anonymat individuel par défaut** : aucun salarié n'apparaît nommément
  dans les vues administrateur. Seul le salarié peut, depuis son espace
  personnel, activer (et désactiver à tout moment) un partage identifié —
  l'administrateur ne dispose d'aucun moyen de l'activer à sa place ;
- **Pas de surveillance fine** : granularité administrateur limitée à la
  journée et à l'équipe ; aucune vue par heure ni par contenu ;
- **Transparence individuelle** : chaque salarié accède à l'intégralité de
  ses propres données depuis son espace « Mon usage » ;
- **Rétention limitée** : suppression automatique des données de plus de
  [730 jours / 24 mois] ;
- **Participation volontaire** : l'enrôlement requiert une action du
  salarié (installation de l'extension et saisie d'un code individuel).

**5. Conformité**

Le traitement sera inscrit au registre des traitements (art. 30 RGPD)
avec la finalité indiquée au point 1. L'information individuelle
préalable des salariés (art. L1222-4 C. trav.) sera assurée par une note
dédiée avant toute mise en service ([projet joint]). Référent :
[DPO / référent données personnelles, contact].

**6. Calendrier et consultation**

Nous proposons l'examen de ce projet lors de la réunion du [date]. Nous
vous remercions de bien vouloir rendre votre avis dans les conditions
prévues à l'article L2312-15 et restons disponibles, en amont, pour toute
question ou démonstration de l'outil.

Pièces jointes :

1. fiche technique de l'outil (`docs/operations/team-aggregator.md`) ;
2. décision d'architecture ADR-0015 (k-anonymat, opt-in, rétention) ;
3. projet de note d'information aux salariés (modèle B ci-dessous).

[Prénom Nom]
[Fonction] — [Entreprise]
[Contact]

---

## Modèle B — Annonce aux salariés

**Objet : Mesurer l'empreinte environnementale de nos usages d'IA —
lancement de Sobr.ia (participation volontaire)**

Bonjour à toutes et à tous,

À partir du [date], [Entreprise] met à disposition **Sobr.ia**, un outil
qui estime l'empreinte environnementale (énergie, CO₂eq) de nos usages
d'assistants d'IA (ChatGPT, Claude, Gemini…). Objectif : disposer de
chiffres d'équipe fiables pour notre démarche [RSE / CSRD / de sobriété
numérique] — pas de surveiller qui que ce soit. Le CSE a été consulté le
[date].

**Ce qui est mesuré**

- le modèle d'IA utilisé, le volume des échanges (tokens) ;
- l'estimation d'énergie et de gCO₂eq correspondante, avec l'horodatage.

**Ce qui n'est jamais mesuré, ni transmis**

- le **contenu de vos conversations** (questions et réponses) — il ne
  quitte jamais votre poste ;
- votre historique de navigation, vos frappes clavier, vos fichiers.

Les données partent uniquement vers un serveur **interne à l'entreprise**
([URL du serveur]) — aucun prestataire externe.

**Vos chiffres restent anonymes par défaut**

- Les tableaux de bord d'équipe n'affichent que des **agrégats anonymes**,
  et uniquement si au moins [5] personnes sont actives sur la période —
  en dessous, rien n'est affiché du tout.
- Personne (pas même l'administrateur) ne voit vos chiffres individuels.
  Si vous le souhaitez, vous pouvez activer un **« Partage identifié »**
  depuis votre espace personnel : ce choix est **désactivé par défaut**,
  vous appartient en propre, et reste **réversible à tout moment**.
- Vous voyez l'intégralité de vos propres données dans votre espace
  « Mon usage ». Tout est automatiquement supprimé au bout de
  [24 mois].

**Comment participer (≈ 3 minutes, volontaire)**

1. Installez l'extension Sobr.ia ([Chrome / Firefox / Edge — lien interne]).
2. Ouvrez `[URL du serveur]` dans votre navigateur et acceptez le
   certificat interne [Option B : cette étape disparaît si vous utilisez
   un certificat public — supprimer la mention].
3. Dans l'extension : Options → **Mode Équipe** → collez `[URL du
   serveur]` → « Vérifier » → « S'enrôler ».
4. Saisissez le **code à 12 chiffres** que [contact / votre manager] vous
   transmettra individuellement, et choisissez un mot de passe personnel.

La participation est volontaire : sans enrôlement de votre part, rien
n'est mesuré sur votre poste.

**Questions ?**

- Technique (installation, code, certificat) : [référent IT, contact].
- Données personnelles : [DPO / référent, contact] — et la documentation
  de l'outil est consultable ici : [lien interne vers la doc].

Merci de votre participation — chaque mesure rend nos chiffres d'équipe
plus justes et nos décisions plus sobres.

[Prénom Nom]
[Fonction] — [Entreprise]

---

## Check-list avant envoi

- [ ] Valeurs `[5]` (k-anonymat) et `[730 jours / 24 mois]` (rétention)
      alignées sur `config list` (sinon, ajuster l'un ou l'autre).
- [ ] URL du serveur définitive (Option A : hostname du cert ; Option B :
      nom public — cf. [`deploiement-equipe.md`](deploiement-equipe.md)).
- [ ] Registre des traitements mis à jour (art. 30 RGPD) — voir
      `team-aggregator.md` § « Obligations du déployeur (France) ».
- [ ] Relecture juriste / DPO effectuée.
- [ ] Avis du CSE recueilli **avant** l'envoi du modèle B.
- [ ] Canal de distribution individuel des codes choisi (jamais de liste
      de codes en clair sur un espace partagé).
