# Sobr.ia pour étudiant·e / curieux·se

> **Apprendre, comprendre, suivre votre usage IA — en grammes de CO₂,
> en douches, en km de voiture.**

---

## Qui c'est ?

Vous utilisez ChatGPT, Claude, Le Chat ou Gemini pour vos cours, vos
projets ou par curiosité. Vous voulez **comprendre l'impact** de cet
usage avant qu'il devienne une habitude invisible.

Vous n'êtes pas développeur·se, vous n'avez pas envie de configurer
une infrastructure, vous voulez juste **ouvrir l'app et voir des
chiffres parlants**.

## Ce que Sobr.ia résout pour vous

| Question | Réponse Sobr.ia |
|---|---|
| « Combien coûte 1 question à ChatGPT en CO₂ ? » | 1 prompt + 1 clic → estimation en grammes + équivalence « ≈ 2 m voiture » |
| « Quel modèle pollue le moins parmi ceux que j'utilise ? » | Module **Comparer modèles** → benchmark côte-à-côte |
| « Mon usage cette semaine = combien de douches ? » | **Tableau de bord** + équivalences humaines automatiques |

## Top 3 use cases

1. **Estimer mon premier prompt** — choisir un modèle, écrire 50-200
   tokens, lire le résultat en CO₂eq + équivalence humaine.
2. **Suivre mon usage hebdomadaire** — Tableau de bord avec
   agrégation jour/semaine/mois, équivalences (douches, km, kWh
   frigo).
3. **Fixer un budget** — Eco-budget mensuel, alerte locale quand on
   dépasse, encouragements en cas de réussite.

## Modules pertinents

- **Estimer un prompt** — l'atelier de mesure unitaire
- **Comment ça marche** — méthodologie expliquée simplement
- **Simulateur « Et si...? »** — explorer "et si je raccourcis mes
  prompts ?", "et si je passe à un modèle plus petit ?"
- **Tableau de bord** — agrégat de vos usages perso
- **Eco-budget** — objectif mensuel + alerte locale

## Quickstart 5 minutes

```bash
# 1. Téléchargez le binaire Sobr.ia pour votre OS depuis Releases GitHub
#    https://github.com/BkOff-fr/defis-lia-generatif/releases

# 2. Lancez l'app — aucune inscription, aucun compte
./sobria-app        # Linux / macOS
# ou double-cliquez Sobr.ia.exe sur Windows

# 3. Choisissez le persona "Étudiant·e / Curieux·se" dans l'onboarding
# 4. Estimez votre premier prompt — l'atelier s'ouvre, choisissez un
#    modèle (ChatGPT 4o-mini par ex.), tapez votre question, cliquez
#    "Estimer l'impact"
```

**Aucune clé API requise.** Le calcul est local, basé sur les
méthodologies AFNOR SPEC 2314 et EcoLogits.

## Et l'extension navigateur ?

Si vous utilisez beaucoup ChatGPT / Claude / Le Chat dans le
navigateur, installez en plus l'**extension Sobr.ia** (Chrome ou
Firefox) : un badge s'affiche à côté du composer avec votre score
A-F + gCO₂eq, et un pairing 6 chiffres permet de remonter les
estimations dans le Tableau de bord de l'app desktop.

→ [Doc extension](../../crates/sobria-bridge/README.md)

## Pour aller plus loin

- [Comment Sobr.ia calcule l'empreinte](../methodology/)
- [Les sources des chiffres](../sources/CATALOGUE-SOURCES.md)
- [FAQ générale](../../README.md)
