<script lang="ts" module>
  // C32.3 — Équivalences carbone, eau, énergie « humaines ».
  //
  // Composant réutilisable dans M1 (sous résultat), M15 (cards totaux),
  // M25 (jauge atteinte). Répond au finding #7 de l'audit produit C32.0 :
  // « pas d'équivalences humaines (douches, km voiture, kWh frigo) ».
  //
  // Source des facteurs :
  //   - CO₂eq voiture thermique : ADEME Base Empreinte (mix moyen 2025) —
  //     ~200 gCO₂eq / km soit 1 g CO₂eq ≈ 5 m.
  //   - CO₂eq streaming vidéo SD : The Shift Project « Lean ICT » 2019 —
  //     ~15 g CO₂eq / heure soit 1 g CO₂eq ≈ 4 min.
  //   - Eau douche éco : ADEME / Centre d'Information sur l'Eau —
  //     douche courte ~8 L soit 1 L ≈ 1/8 douche.
  //   - Énergie LED 60W : 1 Wh ≈ 1 minute d'éclairage 60 W
  //     (60 W × 1 min = 1 Wh).
  //
  // Volontairement « ordre de grandeur ». Le résultat scientifique
  // précis reste dans ResultBlock (P5/P50/P95) — ce composant traduit
  // pour le grand public.

  /** g CO₂eq par mètre de voiture thermique (ADEME, 2025 mix moyen). */
  export const CAR_GCO2EQ_PER_M = 0.2;
  /** g CO₂eq par minute de streaming vidéo SD (Shift Project, Lean ICT 2019). */
  export const STREAMING_GCO2EQ_PER_MIN = 0.25;
  /** L d'eau par douche éco / courte (ADEME / CIEau). */
  export const SHOWER_L = 8;
  /** Wh consommé par une LED 60 W pendant 1 minute. */
  export const LED_WH_PER_MIN = 1;
</script>

<script lang="ts">
  import { Info } from '@lucide/svelte';

  type Props = {
    /** Empreinte carbone en grammes de CO₂eq. */
    gco2eq: number;
    /** (Optionnel) Eau consommée en millilitres. */
    waterMl?: number;
    /** (Optionnel) Énergie consommée en watt-heures. */
    energyWh?: number;
    /** Mode compact (utilisé dans des cards étroites). */
    compact?: boolean;
  };
  let { gco2eq, waterMl = 0, energyWh = 0, compact = false }: Props = $props();

  // ─── Calcul des équivalents bruts ───────────────────────────────────────
  // Chaque équivalent est calculé même si la valeur est ridiculement petite ;
  // c'est `pickRelevant()` qui filtre ensuite selon la magnitude pour
  // n'afficher que ce qui est parlant.

  type Equivalent = {
    /** Texte court (ex « ≈ 2 m voiture »). */
    label: string;
    /** Magnitude relative (sert au filtrage). Plus grand = plus pertinent. */
    weight: number;
    /** Source ADEME / Shift Project / etc. — pour le tooltip. */
    source: string;
  };

  function fmt(value: number, digits = 1): string {
    if (!Number.isFinite(value)) return '—';
    return new Intl.NumberFormat('fr-FR', {
      maximumFractionDigits: digits,
      minimumFractionDigits: 0
    }).format(value);
  }

  // CO₂eq → km voiture (auto-scale m vs km).
  function carEquiv(g: number): Equivalent | null {
    if (g <= 0) return null;
    const meters = g / CAR_GCO2EQ_PER_M;
    if (meters < 1) {
      return {
        label: `≈ ${fmt(meters * 100, 0)} cm voiture`,
        weight: meters,
        source: 'ADEME Base Empreinte 2025'
      };
    }
    if (meters < 1000) {
      return {
        label: `≈ ${fmt(meters, meters < 10 ? 1 : 0)} m voiture`,
        weight: meters,
        source: 'ADEME Base Empreinte 2025'
      };
    }
    return {
      label: `≈ ${fmt(meters / 1000, 1)} km voiture`,
      weight: meters,
      source: 'ADEME Base Empreinte 2025'
    };
  }

  // CO₂eq → streaming vidéo SD.
  function streamingEquiv(g: number): Equivalent | null {
    if (g <= 0) return null;
    const minutes = g / STREAMING_GCO2EQ_PER_MIN;
    if (minutes < 1) {
      return {
        label: `≈ ${fmt(minutes * 60, 0)} s streaming SD`,
        weight: minutes,
        source: 'Shift Project · Lean ICT 2019'
      };
    }
    if (minutes < 60) {
      return {
        label: `≈ ${fmt(minutes, minutes < 10 ? 1 : 0)} min streaming SD`,
        weight: minutes,
        source: 'Shift Project · Lean ICT 2019'
      };
    }
    return {
      label: `≈ ${fmt(minutes / 60, 1)} h streaming SD`,
      weight: minutes,
      source: 'Shift Project · Lean ICT 2019'
    };
  }

  // Eau → douches éco.
  function showerEquiv(ml: number): Equivalent | null {
    if (ml <= 0) return null;
    const liters = ml / 1000;
    const showers = liters / SHOWER_L;
    if (showers < 0.05) {
      // < 5 % d'une douche → afficher en gouttes / mL pour rester parlant.
      return {
        label: `≈ ${fmt(ml, ml < 10 ? 1 : 0)} mL d'eau`,
        weight: showers,
        source: 'ADEME · douche éco 8 L'
      };
    }
    if (showers < 1) {
      return {
        label: `≈ ${fmt(showers * 100, 0)} % d'une douche éco`,
        weight: showers,
        source: 'ADEME · douche éco 8 L'
      };
    }
    return {
      label: `≈ ${fmt(showers, showers < 10 ? 1 : 0)} douche${showers >= 2 ? 's' : ''} éco`,
      weight: showers,
      source: 'ADEME · douche éco 8 L'
    };
  }

  // Énergie → minutes LED 60W.
  function ledEquiv(wh: number): Equivalent | null {
    if (wh <= 0) return null;
    const minutes = wh / LED_WH_PER_MIN;
    if (minutes < 1) {
      return {
        label: `≈ ${fmt(minutes * 60, 0)} s LED 60 W`,
        weight: minutes,
        source: '1 Wh = 1 min LED 60 W'
      };
    }
    if (minutes < 60) {
      return {
        label: `≈ ${fmt(minutes, minutes < 10 ? 1 : 0)} min LED 60 W`,
        weight: minutes,
        source: '1 Wh = 1 min LED 60 W'
      };
    }
    return {
      label: `≈ ${fmt(minutes / 60, 1)} h LED 60 W`,
      weight: minutes,
      source: '1 Wh = 1 min LED 60 W'
    };
  }

  // ─── Sélection adaptative : 2-3 équivalents les plus parlants ─────────
  // Stratégie : on calcule tous les équivalents disponibles, on retire ceux
  // dont `weight < 0.01` (vraiment négligeable), puis on prend les 2-3
  // premiers dans l'ordre voiture → streaming → douche → LED. Cet ordre
  // suit l'audit produit C32.0 (voiture en premier, c'est l'image la
  // plus universelle).

  const equivalents = $derived.by<Equivalent[]>(() => {
    const all = [
      carEquiv(gco2eq),
      streamingEquiv(gco2eq),
      showerEquiv(waterMl),
      ledEquiv(energyWh)
    ];
    const filtered = all.filter((e): e is Equivalent => e !== null && e.weight >= 0.01);
    const max = compact ? 2 : 3;
    return filtered.slice(0, max);
  });
</script>

{#if equivalents.length > 0}
  <div class="equiv-carbon" class:compact data-testid="equivalence-carbon">
    <span class="equiv-items">
      {#each equivalents as eq, i (eq.label)}
        <span class="equiv-item" title="Source : {eq.source}">
          {eq.label}
        </span>
        {#if i < equivalents.length - 1}
          <span class="equiv-sep" aria-hidden="true">·</span>
        {/if}
      {/each}
    </span>
    <span
      class="equiv-info"
      title="Ordres de grandeur basés sur des facteurs ADEME Base Empreinte et The Shift Project. Pour les chiffres scientifiques précis (P5/P50/P95), voir le bloc résultat principal."
      aria-label="Ordre de grandeur · cliquez pour voir les sources"
    >
      <Info size={11} strokeWidth={1.8} />
    </span>
  </div>
{/if}

<style>
  .equiv-carbon {
    display: inline-flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 6px;
    font: 400 12px/1.4 var(--font-ui);
    color: var(--ivory-3);
  }
  .equiv-carbon.compact {
    font-size: 12px;
    gap: 4px;
  }
  .equiv-items {
    display: inline-flex;
    align-items: baseline;
    flex-wrap: wrap;
    gap: 6px;
  }
  .equiv-carbon.compact .equiv-items {
    gap: 4px;
  }
  .equiv-item {
    color: var(--ivory-2);
    white-space: nowrap;
  }
  .equiv-sep {
    color: var(--ivory-4);
    font-weight: 600;
  }
  .equiv-info {
    display: inline-flex;
    align-items: center;
    color: var(--ivory-4);
    cursor: help;
    padding: 2px;
    border-radius: 50%;
    transition: color var(--dur-base) var(--ease);
  }
  .equiv-info:hover {
    color: var(--lime);
  }
</style>
