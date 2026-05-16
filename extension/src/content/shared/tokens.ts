// Sobr.ia extension — heuristique tokenizer (C27.3).
//
// Pas de BPE en JS côté extension : on utilise une heuristique caractères / 3.3
// alignée sur la médiane des tokenizers GPT-4 / Llama 3 / Mistral sur du texte
// français (cf. discussion C09.E §"Tokenizer FR"). La marge typique est ±15 %,
// suffisante pour l'usage extension (rendu en direct, ordre de grandeur).
//
// Pour `tokensOut`, on ne peut pas connaître la valeur exacte à la soumission
// (la réponse n'a pas commencé). On applique une heuristique conservatrice
// `tokensOut ≈ clamp(tokensIn × 2.5, 200, 1500)` qui couvre le cas typique
// chat (réponse 2-5× la longueur du prompt, plafond ~1500 pour éviter le pire
// scénario sans observer le stream). Le moteur AFNOR/EcoLogits étant linéaire
// en tokens_out, on pourra raffiner sans révolution.

const CHARS_PER_TOKEN_FR = 3.3;
const RATIO_OUT_PER_IN = 2.5;
const MIN_OUT = 200;
const MAX_OUT = 1500;

/**
 * Estime le nombre de tokens d'un texte (heuristique FR).
 *
 * Convention : `Math.max(1, ceil(text.length / 3.3))` pour garantir au moins
 * 1 token pour un texte non vide (sinon l'estimation downstream divise par 0).
 * Texte vide → 0.
 */
export function estimateTokens(text: string): number {
  const len = text.length;
  if (len === 0) return 0;
  return Math.max(1, Math.ceil(len / CHARS_PER_TOKEN_FR));
}

/**
 * Estime le nombre de tokens de sortie attendus, étant donné le prompt.
 *
 * Heuristique : 2.5× la longueur d'entrée, bornée [200, 1500].
 */
export function estimateOutputTokens(tokensIn: number): number {
  if (tokensIn <= 0) return MIN_OUT;
  const raw = Math.round(tokensIn * RATIO_OUT_PER_IN);
  return Math.min(MAX_OUT, Math.max(MIN_OUT, raw));
}
