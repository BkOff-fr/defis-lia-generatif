<script lang="ts">
  import { FlaskConical, ExternalLink, ArrowUpRight } from '@lucide/svelte';
  import type { HypothesisDto } from '$lib/api';

  type Props = { hypotheses: HypothesisDto[] };
  const { hypotheses }: Props = $props();

  // Met en forme une hypothèse `value: unknown` (cf. dto.rs : c'est un
  // `serde_json::Value` côté Rust, donc on peut tomber sur number, string,
  // objet, tableau, bool, null). Pour les nombres on garde 3 chiffres
  // significatifs (mêmes valeurs scientifiques que ailleurs dans l'app).
  function fmtValue(v: unknown): string {
    if (v === null || v === undefined) return '—';
    if (typeof v === 'number') {
      if (!Number.isFinite(v)) return '—';
      if (v === 0) return '0';
      return new Intl.NumberFormat('fr-FR', {
        maximumSignificantDigits: 3,
        minimumSignificantDigits: 1
      }).format(v);
    }
    if (typeof v === 'string') return v;
    if (typeof v === 'boolean') return v ? 'oui' : 'non';
    return JSON.stringify(v);
  }

  function isUrl(source: string): boolean {
    return /^https?:\/\//i.test(source);
  }

  // Nettoie l'URL pour affichage : retire `https://` et tronque si trop long.
  function prettySource(source: string, max = 40): string {
    const stripped = source.replace(/^https?:\/\//i, '');
    return stripped.length > max ? stripped.slice(0, max - 1) + '…' : stripped;
  }
</script>

<section class="hyp-block" aria-label="Hypothèses utilisées">
  <div class="hh">
    <FlaskConical size={18} strokeWidth={1.6} />
    <div class="t">Hypothèses utilisées</div>
    <div class="spc"></div>
    <a class="btn-ghost-mini" href="/methodo">
      <ExternalLink size={14} strokeWidth={1.8} />
      Voir la méthodologie complète
    </a>
  </div>

  {#if hypotheses.length === 0}
    <p class="empty">Aucune hypothèse exportée par le moteur pour cette estimation.</p>
  {:else}
    <ul class="hyp-grid" role="list">
      {#each hypotheses as h (h.key)}
        <li class="hyp-card">
          <header class="hyp-head">
            <span class="sym" title={h.key}>{h.key}</span>
            {#if isUrl(h.source)}
              <a
                class="src"
                href={h.source}
                target="_blank"
                rel="noopener noreferrer"
                title={h.source}
              >
                <span class="src-text">{prettySource(h.source)}</span>
                <ArrowUpRight size={11} strokeWidth={2} />
              </a>
            {:else}
              <span class="src" title={h.source}>
                <span class="src-text">{h.source}</span>
              </span>
            {/if}
          </header>
          <div class="val" title={String(h.value)}>{fmtValue(h.value)}</div>
        </li>
      {/each}
    </ul>
  {/if}
</section>

<style>
  .hyp-block {
    margin-top: 28px;
    padding: 28px 32px;
    background: var(--ink-2);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
  }
  .hh {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-bottom: 20px;
    padding-bottom: 16px;
    border-bottom: 1px solid var(--border);
  }
  .hh :global(svg) {
    color: var(--lime);
    flex-shrink: 0;
  }
  .hh .t {
    font: 400 22px/1 var(--font-display);
    font-style: italic;
    color: var(--ivory);
  }
  .hh .spc {
    flex: 1;
  }

  .btn-ghost-mini {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 32px;
    padding: 0 12px;
    background: transparent;
    color: var(--ivory-2);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    font: 500 12px/1 var(--font-ui);
    text-decoration: none;
    transition: all var(--dur-base) var(--ease);
  }
  .btn-ghost-mini:hover {
    border-color: var(--border-hi);
    color: var(--ivory);
  }

  /* Grid 2 colonnes de cartes ; chaque carte est autonome — le value se
     pose en dessous du head et est libre de wrapper sans bousculer la
     ligne sœur. */
  .hyp-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 10px;
    padding: 0;
    margin: 0;
    list-style: none;
  }
  .hyp-card {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 12px 14px;
    background: rgba(255, 255, 255, 0.02);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    transition:
      border-color var(--dur-base) var(--ease),
      background var(--dur-base) var(--ease);
    min-width: 0;
  }
  .hyp-card:hover {
    border-color: var(--border-hi);
    background: rgba(255, 255, 255, 0.035);
  }
  .hyp-head {
    display: flex;
    align-items: baseline;
    gap: 10px;
    min-width: 0;
  }
  .sym {
    font: 400 13px/1.2 var(--font-mono);
    color: var(--lime);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    flex-shrink: 0;
    max-width: 50%;
  }
  .src {
    font: 400 11px/1.2 var(--font-ui);
    color: var(--ivory-3);
    display: inline-flex;
    align-items: center;
    gap: 4px;
    margin-left: auto;
    min-width: 0;
    border-bottom: none;
    text-decoration: none;
    transition: color var(--dur-base) var(--ease);
  }
  .src-text {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .src:hover {
    color: var(--blue);
  }
  .val {
    font: 500 14px/1.4 var(--font-mono);
    color: var(--ivory);
    /* `word-break: break-word` autorise les longs identifiants (URLs,
       UUIDs, expressions math) à se couper proprement sans déborder. */
    overflow-wrap: anywhere;
    word-break: break-word;
  }

  .empty {
    font: 400 13px/1.5 var(--font-ui);
    color: var(--ivory-3);
    font-style: italic;
  }

  @media (max-width: 720px) {
    .hyp-grid {
      grid-template-columns: 1fr;
    }
    .hyp-block {
      padding: 22px 18px;
    }
  }
</style>
