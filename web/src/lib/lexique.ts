// C41 — Lexique inline : définitions courtes des termes techniques,
// affichées en tooltip (composant `Term.svelte`) et reliées au glossaire
// canonique (`/methodo#glossaire`, source : docs/methodology/GLOSSAIRE.md).
// Une à deux phrases maximum : la pédagogie complète vit dans M8.

export type TermKey =
  | 'p50'
  | 'p5p95'
  | 'token'
  | 'tokens_sortie'
  | 'prefill'
  | 'decode'
  | 'pue'
  | 'wue'
  | 'if_elec'
  | 'monte_carlo';

export const LEXIQUE: Record<TermKey, string> = {
  p50: 'Médiane : la moitié des 10 000 tirages Monte-Carlo donne moins, l’autre moitié donne plus. C’est la valeur centrale, pas un maximum.',
  p5p95:
    'Intervalle d’incertitude : 90 % des tirages Monte-Carlo tombent entre ces deux bornes. Plus il est étroit, plus l’estimation est sûre.',
  token:
    'Unité de découpage du texte par les LLM (~¾ de mot en français). L’énergie consommée croît avec le nombre de tokens traités.',
  tokens_sortie:
    'Tokens que le modèle va générer en réponse — estimés avant l’envoi, car la longueur réelle n’est pas connue à l’avance.',
  prefill:
    'Phase de lecture du prompt par le modèle (tokens d’entrée). Moins coûteuse par token que la génération.',
  decode:
    'Phase de génération de la réponse, token par token. C’est généralement le poste d’énergie dominant.',
  pue: 'Power Usage Effectiveness : énergie totale du datacenter ÷ énergie des serveurs. 1,2 = 20 % d’overhead (refroidissement, pertes).',
  wue: 'Water Usage Effectiveness : litres d’eau consommés par kWh informatique, principalement pour le refroidissement.',
  if_elec:
    'Intensité carbone de l’électricité locale, en gCO₂eq par kWh. ~56 en France (nucléaire), >350 dans les mix charbon/gaz.',
  monte_carlo:
    'Méthode statistique : on recalcule 10 000 fois avec des paramètres tirés dans leurs plages d’incertitude, pour obtenir une distribution plutôt qu’un chiffre unique.'
};
