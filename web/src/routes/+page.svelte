<script lang="ts">
  import {
    History,
    HelpCircle,
    Repeat,
    Car,
    ShowerHead,
    Monitor,
    Smartphone,
    Fingerprint,
    ShieldCheck,
    AlertTriangle,
    Info,
    PlugZap,
    Scale
  } from '@lucide/svelte';
  import {
    estimatePrompt,
    isTauriContext,
    listModels,
    SobriaIpcError,
    type EquivalentDto,
    type EstimationResultDto,
    type IpcErrorCode,
    type ModelPresetDto
  } from '$lib/api';
  import Composer from '$lib/components/Composer.svelte';
  import ResultBlock from '$lib/components/ResultBlock.svelte';
  import HypothesisBlock from '$lib/components/HypothesisBlock.svelte';
  import { tick } from 'svelte';

  // ─── State ───────────────────────────────────────────────────────────
  let models = $state<ModelPresetDto[]>([]);
  let selectedModelId = $state('');
  let prompt = $state(
    'Écris-moi un résumé de 500 mots sur la photosynthèse, accessible à un lycéen, en distinguant la phase claire et la phase sombre.'
  );
  let tokensOut = $state(720);

  let result = $state<EstimationResultDto | null>(null);
  let loading = $state(false);
  let bootstrapping = $state(true);
  let error = $state<{ code: IpcErrorCode; message: string } | null>(null);

  // Ancre pour le scroll smooth post-estimation (cf. submitEstimation).
  let resultAnchor: HTMLDivElement | undefined = $state();

  const tauriAvailable = $derived(isTauriContext());

  // ─── Bootstrap : on charge les modèles via IPC réel ──────────────────
  $effect(() => {
    void (async () => {
      if (!tauriAvailable) {
        bootstrapping = false;
        error = {
          code: 'tauri_unavailable',
          message:
            "L'application doit être lancée via `cargo run -p sobria-app` (ou `cargo tauri dev`). Le contexte Tauri n'est pas disponible dans un navigateur seul."
        };
        return;
      }
      try {
        const list = await listModels();
        models = list.sort((a, b) =>
          a.provider === b.provider
            ? a.display_name.localeCompare(b.display_name)
            : a.provider.localeCompare(b.provider)
        );
        // Préselection : ?model=<id> depuis le Workbench, sinon gpt-4o-mini,
        // sinon premier modèle disponible.
        const urlModel =
          typeof window !== 'undefined'
            ? new URLSearchParams(window.location.search).get('model')
            : null;
        const fromUrl = urlModel ? list.find((m) => m.id === urlModel)?.id : undefined;
        selectedModelId =
          fromUrl ?? list.find((m) => m.id === 'gpt-4o-mini')?.id ?? list[0]?.id ?? '';
      } catch (err) {
        if (err instanceof SobriaIpcError) {
          error = { code: err.code, message: err.message };
        } else {
          error = { code: 'internal', message: 'Échec du chargement des modèles' };
        }
      } finally {
        bootstrapping = false;
      }
    })();
  });

  // ─── Submit : appel IPC réel ─────────────────────────────────────────
  async function submitEstimation() {
    if (!selectedModelId) return;
    loading = true;
    error = null;
    try {
      // Même heuristique que Composer (3,3 chars/token FR). Cf. note dans
      // Composer.svelte — tokenizer réel en v0.3 (chantier outillage).
      const tokensIn = Math.max(1, Math.ceil(prompt.length / 3.3));
      const r = await estimatePrompt({
        model_id: selectedModelId,
        tokens_in: tokensIn,
        tokens_out_estimated: Math.max(1, tokensOut)
      });
      result = r;
      // Scroll smooth vers le bloc résultat, après que le DOM ait été
      // recalculé. Respecte `prefers-reduced-motion` via le param `behavior`.
      await tick();
      const reduced = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
      resultAnchor?.scrollIntoView({
        behavior: reduced ? 'auto' : 'smooth',
        block: 'start'
      });
    } catch (err) {
      result = null;
      if (err instanceof SobriaIpcError) {
        error = { code: err.code, message: err.message };
      } else {
        error = { code: 'internal', message: "Échec de l'estimation" };
      }
    } finally {
      loading = false;
    }
  }

  // ─── Raccourci clavier Ctrl+Enter pour estimer ───────────────────────
  function handleKey(e: KeyboardEvent) {
    if ((e.ctrlKey || e.metaKey) && e.key === 'Enter' && !loading && selectedModelId) {
      e.preventDefault();
      void submitEstimation();
    }
  }

  // ─── Équivalents : mapping label → icône (les labels viennent du Rust) ──
  function equivIcon(label: string): typeof Car {
    const l = label.toLowerCase();
    if (l.includes('voiture')) return Car;
    if (l.includes('douche')) return ShowerHead;
    if (l.includes('écran') || l.includes('ecran')) return Monitor;
    if (l.includes('smartphone') || l.includes('charge')) return Smartphone;
    return Repeat;
  }

  // Format FR avec N chiffres significatifs (cf. note dans ResultBlock.svelte).
  function fmt(value: number, sig = 3): string {
    if (!Number.isFinite(value)) return '—';
    if (value === 0) return '0';
    return new Intl.NumberFormat('fr-FR', {
      maximumSignificantDigits: sig,
      minimumSignificantDigits: 1
    }).format(value);
  }

  // Découpe « 17 km en voiture thermique » → { head: '17 km', tail: 'en voiture …' }
  function splitEquivLabel(e: EquivalentDto): { head: string; tail: string } {
    return { head: fmt(e.value), tail: e.label };
  }

  const errorTone = $derived.by<'info' | 'warn' | 'error'>(() => {
    if (!error) return 'info';
    if (error.code === 'tauri_unavailable') return 'warn';
    if (error.code === 'unknown_model' || error.code === 'invalid_request') return 'warn';
    return 'error';
  });

  // Libellés humains pour les codes d'erreur IPC connus (mirroir simple —
  // source de vérité dans `crates/sobria-app/src/error.rs`).
  const ERROR_LABELS: Record<string, string> = {
    tauri_unavailable: 'Application non lancée via Tauri',
    unknown_model: 'Modèle inconnu',
    invalid_request: 'Requête invalide',
    estimator_error: 'Erreur du moteur Monte-Carlo',
    audit_error: "Erreur du ledger d'audit",
    core_error: 'Erreur interne',
    io_error: 'Erreur disque',
    json_error: 'Erreur de sérialisation',
    internal: 'Erreur interne'
  };
  function errorLabel(code: string): string {
    return ERROR_LABELS[code] ?? 'Erreur';
  }
</script>

<svelte:window onkeydown={handleKey} />

<svelte:head>
  <title>Sobr.ia · Estimer un prompt</title>
</svelte:head>

<div class="canvas-inner">
  <!-- ─── Top bar ───────────────────────────────────────────────── -->
  <div class="topbar">
    <nav class="breadcrumb" aria-label="Fil d'Ariane">
      Atelier <span class="sep">/</span>
      <span class="current">Estimer un prompt</span>
    </nav>
    <div class="spacer"></div>
    <span class="local-pill" title="Aucune donnée envoyée vers un service externe">
      <span class="dot" aria-hidden="true"></span>
      100 % local · aucune donnée envoyée
    </span>
    <button class="icon-btn" type="button" disabled aria-label="Historique (à venir)">
      <History size={16} strokeWidth={1.6} />
    </button>
    <a class="icon-btn" href="/methodo" aria-label="Aide & méthodologie">
      <HelpCircle size={16} strokeWidth={1.6} />
    </a>
  </div>

  <!-- ─── Hero éditorial ───────────────────────────────────────── -->
  <section class="hero">
    <div class="hero-eyebrow">
      <span class="pulse" aria-hidden="true"></span>
      Atelier d'estimation · Module M2
    </div>
    <h1 class="hero-h1">
      Quel est le poids carbone, <em>réel</em>, d'une seule requête à votre LLM ?
    </h1>
    <p class="hero-sub">
      Saisissez votre prompt, choisissez un modèle. Sobr.ia simule 10 000 trajectoires Monte-Carlo
      pour estimer l'énergie, le CO₂, l'eau et les métaux — avec un intervalle d'incertitude P5–P95.
    </p>
  </section>

  <!-- ─── Bannière erreur globale ──────────────────────────────── -->
  {#if error}
    <div class="banner" data-tone={errorTone} role="alert">
      <span class="banner-ico" aria-hidden="true">
        {#if errorTone === 'warn'}
          <AlertTriangle size={18} strokeWidth={1.8} />
        {:else if errorTone === 'error'}
          <PlugZap size={18} strokeWidth={1.8} />
        {:else}
          <Info size={18} strokeWidth={1.8} />
        {/if}
      </span>
      <div class="banner-body">
        <strong>{errorLabel(error.code)}</strong>
        <span>{error.message}</span>
      </div>
    </div>
  {/if}

  <!-- ─── Composer (form) ──────────────────────────────────────── -->
  {#if bootstrapping}
    <div class="composer-skel" aria-busy="true">Chargement du référentiel…</div>
  {:else if tauriAvailable && models.length > 0}
    <Composer
      {models}
      bind:selectedModelId
      bind:prompt
      bind:tokensOut
      estimating={loading}
      onsubmit={submitEstimation}
    />
  {/if}

  <!-- ─── Résultat ─────────────────────────────────────────────── -->
  {#if result}
    <div bind:this={resultAnchor} style="scroll-margin-top: 24px"></div>
    <ResultBlock {result} />

    <!-- Équivalents -->
    {#if result.equivalents.length > 0}
      <section class="equiv-strip" aria-label="Mises en perspective">
        <div class="equiv-label">
          <Repeat size={14} strokeWidth={1.8} />
          Pour mettre cela en perspective
        </div>
        <div class="equiv-grid" data-count={result.equivalents.length}>
          {#each result.equivalents as eq (eq.label)}
            {@const Icon = equivIcon(eq.label)}
            {@const parts = splitEquivLabel(eq)}
            <div class="equiv-tile" title={eq.source}>
              <div class="ico"><Icon size={22} strokeWidth={1.4} /></div>
              <div class="col">
                <div class="v">{parts.head}</div>
                <div class="t">{parts.tail}</div>
              </div>
            </div>
          {/each}
        </div>
      </section>
    {/if}

    <!-- Cross-screen CTA : prolonger vers le comparateur avec le même prompt -->
    <a
      class="cross-cta"
      href={`/comparer?prompt=${encodeURIComponent(prompt)}&tokensOut=${tokensOut}&model=${encodeURIComponent(selectedModelId)}`}
    >
      <span class="cross-ico" aria-hidden="true">
        <Scale size={18} strokeWidth={1.6} />
      </span>
      <span class="cross-body">
        <span class="cross-title">Et avec d'autres modèles&nbsp;?</span>
        <span class="cross-sub">
          Comparer ce prompt sur 2 à 8 modèles côte-à-côte (matrice + score composite).
        </span>
      </span>
      <span class="cross-arrow" aria-hidden="true">→</span>
    </a>

    <!-- Hypothèses -->
    <HypothesisBlock hypotheses={result.hypotheses} />

    <!-- Signature ledger -->
    <div class="signature">
      <span class="ico" aria-hidden="true"><Fingerprint size={14} strokeWidth={1.6} /></span>
      <span>Estimation journalisée dans le ledger d'audit · entrée</span>
      <a class="hash" href={`/journal?focus=${result.audit_id}`}>
        #{result.audit_id} · seed {result.seed}
      </a>
      <div class="spacer"></div>
      <span class="verify">
        <ShieldCheck size={12} strokeWidth={1.8} />
        Chaîne intègre
      </span>
    </div>
  {/if}
</div>

<style>
  .canvas-inner {
    max-width: 1240px;
    margin: 0 auto;
    padding: 40px 56px 80px;
  }

  /* ─── Top bar ──────────────────────────────────────────────── */
  .topbar {
    display: flex;
    align-items: center;
    gap: 16px;
    margin-bottom: 32px;
  }
  .breadcrumb {
    font: 400 13px/1 var(--font-ui);
    color: var(--ivory-3);
    letter-spacing: 0.02em;
  }
  .breadcrumb .sep {
    color: var(--ivory-4);
    margin: 0 8px;
  }
  .breadcrumb .current {
    color: var(--ivory-2);
  }
  .spacer {
    flex: 1;
  }

  .local-pill {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    height: 32px;
    padding: 0 14px;
    background: var(--lime-soft);
    border: 1px solid rgba(197, 240, 74, 0.25);
    border-radius: 999px;
    font: 500 12px/1 var(--font-ui);
    color: var(--lime);
    letter-spacing: 0.01em;
  }
  .local-pill .dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--lime);
    box-shadow: 0 0 8px var(--lime);
    animation: pill-pulse 2.4s ease-in-out infinite;
    position: relative;
  }
  .local-pill .dot::after {
    content: '';
    position: absolute;
    inset: -3px;
    border-radius: 50%;
    border: 1px solid var(--lime);
    opacity: 0;
    animation: ripple 2.4s ease-out infinite;
  }
  @keyframes pill-pulse {
    0%,
    100% {
      opacity: 1;
    }
    50% {
      opacity: 0.4;
    }
  }
  @keyframes ripple {
    0% {
      transform: scale(0.8);
      opacity: 0.6;
    }
    100% {
      transform: scale(2.6);
      opacity: 0;
    }
  }

  .icon-btn {
    width: 32px;
    height: 32px;
    display: grid;
    place-items: center;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    color: var(--ivory-2);
    cursor: pointer;
    transition: all var(--dur-base) var(--ease);
    text-decoration: none;
  }
  .icon-btn:hover {
    background: var(--surface-hi);
    border-color: var(--border-hi);
    color: var(--ivory);
  }
  .icon-btn:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }

  /* ─── Hero ─────────────────────────────────────────────────── */
  .hero {
    position: relative;
    padding: 48px 0 36px;
    margin-bottom: 32px;
    border-bottom: 1px solid var(--border);
    animation: rise 600ms var(--ease) backwards;
  }
  .hero-eyebrow {
    font: 500 11px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.16em;
    color: var(--ivory-3);
    margin-bottom: 14px;
    display: inline-flex;
    align-items: center;
    gap: 8px;
  }
  .hero-eyebrow .pulse {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--lime);
    box-shadow: 0 0 10px var(--lime);
  }
  .hero-h1 {
    font: 400 56px/1.05 var(--font-display);
    font-style: italic;
    color: var(--ivory);
    letter-spacing: -0.02em;
    max-width: 720px;
    margin: 0 0 8px;
  }
  .hero-h1 em {
    font-style: normal;
    color: var(--lime);
    background: linear-gradient(90deg, #c5f04a 0%, #a8d83a 50%, #c5f04a 100%);
    background-size: 200% 100%;
    -webkit-background-clip: text;
    background-clip: text;
    -webkit-text-fill-color: transparent;
    animation: shimmer 4s ease-in-out infinite;
  }
  @keyframes shimmer {
    0%,
    100% {
      background-position: 0% 50%;
    }
    50% {
      background-position: 100% 50%;
    }
  }
  .hero-sub {
    font: 400 16px/1.5 var(--font-ui);
    color: var(--ivory-2);
    max-width: 560px;
    margin: 0;
  }

  /* ─── Banner erreur ────────────────────────────────────────── */
  .banner {
    display: flex;
    gap: 14px;
    align-items: flex-start;
    padding: 16px 20px;
    border-radius: var(--radius-md);
    border: 1px solid var(--border-hi);
    background: rgba(245, 183, 105, 0.08);
    margin-bottom: 20px;
  }
  .banner[data-tone='warn'] {
    background: rgba(245, 183, 105, 0.08);
    border-color: rgba(245, 183, 105, 0.25);
    color: var(--amber);
  }
  .banner[data-tone='error'] {
    background: rgba(240, 108, 90, 0.08);
    border-color: rgba(240, 108, 90, 0.3);
    color: var(--coral);
  }
  .banner[data-tone='info'] {
    background: rgba(126, 182, 255, 0.08);
    border-color: rgba(126, 182, 255, 0.25);
    color: var(--blue);
  }
  .banner-ico {
    display: inline-flex;
    align-items: center;
    flex-shrink: 0;
    padding-top: 2px;
  }
  .banner-body {
    display: flex;
    flex-direction: column;
    gap: 4px;
    font: 400 13px/1.4 var(--font-ui);
    color: var(--ivory);
  }
  .banner-body strong {
    font-weight: 600;
  }
  .banner-body span {
    color: var(--ivory-2);
  }

  /* ─── Composer skeleton ────────────────────────────────────── */
  .composer-skel {
    height: 320px;
    border: 1px dashed var(--border);
    border-radius: var(--radius-xl);
    display: grid;
    place-items: center;
    color: var(--ivory-3);
    font: 400 13px/1 var(--font-mono);
    margin-bottom: 24px;
  }

  /* ─── Équivalents ──────────────────────────────────────────── */
  .equiv-strip {
    margin-top: 32px;
    padding: 24px 28px;
    background: rgba(255, 255, 255, 0.02);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
  }
  .equiv-label {
    display: flex;
    align-items: center;
    gap: 10px;
    font: 500 11px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.14em;
    color: var(--ivory-3);
    margin-bottom: 18px;
  }
  .equiv-label :global(svg) {
    color: var(--lime);
  }
  .equiv-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(190px, 1fr));
    gap: 16px;
  }
  .equiv-tile {
    position: relative;
    padding: 18px;
    background: linear-gradient(180deg, rgba(255, 255, 255, 0.03), transparent);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    display: flex;
    gap: 14px;
    align-items: center;
    transition: all 300ms var(--ease-spring);
  }
  .equiv-tile:hover {
    transform: translateY(-4px) scale(1.02);
    border-color: rgba(197, 240, 74, 0.3);
    background: linear-gradient(180deg, rgba(197, 240, 74, 0.06), rgba(255, 255, 255, 0.02));
  }
  .equiv-tile .ico {
    width: 38px;
    height: 38px;
    flex-shrink: 0;
    display: grid;
    place-items: center;
    color: var(--lime);
    transition: transform 500ms var(--ease-spring);
  }
  .equiv-tile:hover .ico {
    transform: rotate(-12deg) scale(1.15);
  }
  .equiv-tile .v {
    font: 400 22px/1 var(--font-display);
    font-style: italic;
    color: var(--ivory);
    letter-spacing: -0.01em;
    transition: color var(--dur-slow) var(--ease);
  }
  .equiv-tile:hover .v {
    color: var(--lime);
  }
  .equiv-tile .t {
    font: 400 12px/1.3 var(--font-ui);
    color: var(--ivory-3);
    margin-top: 4px;
  }

  /* ─── Cross-screen CTA vers Comparer ───────────────────────── */
  .cross-cta {
    display: flex;
    align-items: center;
    gap: 16px;
    margin-top: 24px;
    padding: 16px 22px;
    background: linear-gradient(90deg, rgba(197, 240, 74, 0.05), rgba(126, 182, 255, 0.04));
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    text-decoration: none;
    border-bottom: none;
    transition: all var(--dur-base) var(--ease);
  }
  .cross-cta:hover {
    border-color: rgba(197, 240, 74, 0.3);
    background: linear-gradient(90deg, rgba(197, 240, 74, 0.08), rgba(126, 182, 255, 0.06));
    transform: translateY(-1px);
  }
  .cross-ico {
    display: grid;
    place-items: center;
    width: 38px;
    height: 38px;
    flex-shrink: 0;
    background: var(--lime-soft);
    border: 1px solid rgba(197, 240, 74, 0.3);
    border-radius: var(--radius-md);
    color: var(--lime);
  }
  .cross-body {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }
  .cross-title {
    font: 500 14px/1.3 var(--font-ui);
    color: var(--ivory);
  }
  .cross-sub {
    font: 400 12px/1.4 var(--font-ui);
    color: var(--ivory-3);
  }
  .cross-arrow {
    font: 400 22px/1 var(--font-display);
    font-style: italic;
    color: var(--ivory-3);
    transition: transform var(--dur-base) var(--ease);
  }
  .cross-cta:hover .cross-arrow {
    color: var(--lime);
    transform: translateX(3px);
  }

  /* ─── Signature ────────────────────────────────────────────── */
  .signature {
    display: flex;
    align-items: center;
    gap: 14px;
    margin-top: 32px;
    padding: 18px 24px;
    background: rgba(0, 0, 0, 0.3);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    font: 400 12px/1 var(--font-mono);
    color: var(--ivory-3);
    flex-wrap: wrap;
  }
  .signature .ico {
    color: var(--lime);
    display: inline-flex;
    align-items: center;
  }
  .signature .hash {
    color: var(--ivory-2);
    text-decoration: none;
    border-bottom: 1px dashed rgba(255, 255, 255, 0.15);
    transition: color var(--dur-base) var(--ease);
  }
  .signature .hash:hover {
    color: var(--lime);
    border-bottom-color: var(--lime);
  }
  .signature .verify {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    color: var(--lime);
    font-weight: 500;
  }

  @keyframes rise {
    from {
      opacity: 0;
      transform: translateY(12px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  @media (max-width: 720px) {
    .canvas-inner {
      padding: 24px 20px 60px;
    }
    .hero-h1 {
      font-size: 36px;
    }
  }
</style>
