<script lang="ts">
  import {
    Settings2,
    HelpCircle,
    Lock,
    Info,
    AlertTriangle,
    PlugZap,
    Copy,
    Check,
    Cpu,
    Dice5,
    Folder,
    Hammer
  } from '@lucide/svelte';
  import { isTauriContext, metaInfo, SobriaIpcError, type MetaInfo } from '$lib/api';

  let meta = $state<MetaInfo | null>(null);
  let bootstrapping = $state(true);
  let loadError = $state<{ code: string; message: string } | null>(null);
  let copied = $state<string | null>(null);

  const tauriAvailable = $derived(isTauriContext());

  $effect(() => {
    void (async () => {
      if (!tauriAvailable) {
        bootstrapping = false;
        loadError = {
          code: 'tauri_unavailable',
          message:
            "L'application doit être lancée via `cargo run -p sobria-app`. Les paramètres runtime ne sont disponibles que dans le contexte Tauri."
        };
        return;
      }
      try {
        meta = await metaInfo();
      } catch (err) {
        if (err instanceof SobriaIpcError) {
          loadError = { code: err.code, message: err.message };
        } else {
          loadError = { code: 'internal', message: 'Échec du chargement des paramètres.' };
        }
      } finally {
        bootstrapping = false;
      }
    })();
  });

  async function copy(label: string, text: string) {
    if (typeof navigator === 'undefined' || !navigator.clipboard) return;
    try {
      await navigator.clipboard.writeText(text);
      copied = label;
      setTimeout(() => {
        if (copied === label) copied = null;
      }, 1500);
    } catch {
      // Pas d'autorisation clipboard — silencieux.
    }
  }

  const errorTone = $derived.by<'info' | 'warn' | 'error'>(() => {
    if (!loadError) return 'info';
    if (loadError.code === 'tauri_unavailable') return 'warn';
    return 'error';
  });

  // Paramètres « futurs » — placeholders pour montrer le cap.
  const upcomingPrefs = [
    {
      icon: 'theme',
      label: 'Thème',
      hint: 'Sombre (par défaut) / Clair / Système',
      chantier: 'C10'
    },
    {
      icon: 'lang',
      label: 'Langue',
      hint: 'FR (par défaut) / EN — `@inlang/paraglide` à brancher',
      chantier: 'C10'
    },
    {
      icon: 'seed',
      label: 'Seed Monte-Carlo personnalisé',
      hint: "Override de SOBRIA_SEED depuis l'UI pour reproductibilité ad hoc",
      chantier: 'C11'
    },
    {
      icon: 'opt',
      label: 'Télémétrie opt-in',
      hint: "Partage anonyme du nb d'estimations + hash agrégé (off par défaut)",
      chantier: 'C12'
    }
  ];
</script>

<svelte:head>
  <title>Sobr.ia · Paramètres</title>
</svelte:head>

<div class="canvas-inner">
  <!-- ─── TopBar ─────────────────────────────────────────────── -->
  <div class="topbar">
    <nav class="breadcrumb" aria-label="Fil d'Ariane">
      Atelier <span class="sep">/</span>
      <span class="current">Paramètres</span>
    </nav>
    <div class="spacer"></div>
    <span class="local-pill" title="Tout traitement local">
      <Lock size={12} strokeWidth={1.8} />
      100 % local
    </span>
    <a class="icon-btn" href="/methodo" aria-label="Méthodologie">
      <HelpCircle size={16} strokeWidth={1.6} />
    </a>
  </div>

  <!-- ─── Hero ───────────────────────────────────────────────── -->
  <section class="hero">
    <div class="hero-eyebrow">
      <span class="pulse" aria-hidden="true"></span>
      Paramètres · runtime &amp; préférences
    </div>
    <h1 class="hero-h1">
      Vos <em>paramètres</em> et l'état du moteur Sobr.ia.
    </h1>
    <p class="hero-sub">
      Tout est local. Les chemins ci-dessous pointent vers votre disque ; aucune donnée n'est
      envoyée vers un serveur distant. Les préférences utilisateur arrivent dans les chantiers
      suivants — pour l'instant, la configuration est pilotée par variables d'environnement (cf. <a
        href="/methodo">méthodologie</a
      >).
    </p>
  </section>

  <!-- ─── Bannière erreur ────────────────────────────────────── -->
  {#if loadError}
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
        <strong
          >{loadError.code === 'tauri_unavailable'
            ? 'Application non lancée via Tauri'
            : 'Erreur'}</strong
        >
        <span>{loadError.message}</span>
      </div>
    </div>
  {/if}

  <!-- ─── Runtime (lecture seule via meta_info) ────────────── -->
  <section class="section">
    <header class="section-head">
      <Settings2 size={16} strokeWidth={1.8} />
      <h2>Runtime</h2>
      <span class="section-hint mono">lecture seule · `meta_info` IPC</span>
    </header>

    {#if bootstrapping}
      <div class="runtime-skel">Chargement…</div>
    {:else if meta}
      <dl class="runtime-grid">
        <div class="runtime-row">
          <dt>
            <Cpu size={12} strokeWidth={1.8} />
            Version d'application
          </dt>
          <dd class="mono">{meta.app_version}</dd>
        </div>
        <div class="runtime-row">
          <dt>
            <Dice5 size={12} strokeWidth={1.8} />
            Seed Monte-Carlo
          </dt>
          <dd class="mono">
            {meta.estimator_seed}
            <button
              type="button"
              class="copy-btn"
              onclick={() => copy('seed', String(meta?.estimator_seed))}
              aria-label="Copier le seed"
            >
              {#if copied === 'seed'}
                <Check size={11} strokeWidth={2} />
              {:else}
                <Copy size={11} strokeWidth={1.8} />
              {/if}
            </button>
          </dd>
        </div>
        <div class="runtime-row">
          <dt>
            <Dice5 size={12} strokeWidth={1.8} />
            Tirages Monte-Carlo (N)
          </dt>
          <dd class="mono">
            {new Intl.NumberFormat('fr-FR').format(meta.estimator_n)}
          </dd>
        </div>
        <div class="runtime-row">
          <dt>
            <Folder size={12} strokeWidth={1.8} />
            Ledger d'audit
          </dt>
          <dd class="mono break">
            {meta.audit_path}
            <button
              type="button"
              class="copy-btn"
              onclick={() => copy('audit', meta?.audit_path ?? '')}
              aria-label="Copier le chemin du ledger"
            >
              {#if copied === 'audit'}
                <Check size={11} strokeWidth={2} />
              {:else}
                <Copy size={11} strokeWidth={1.8} />
              {/if}
            </button>
          </dd>
        </div>
        <div class="runtime-row">
          <dt>
            <Folder size={12} strokeWidth={1.8} />
            Racine des données
          </dt>
          <dd class="mono break">
            {meta.data_root}
            <button
              type="button"
              class="copy-btn"
              onclick={() => copy('data', meta?.data_root ?? '')}
              aria-label="Copier la racine des données"
            >
              {#if copied === 'data'}
                <Check size={11} strokeWidth={2} />
              {:else}
                <Copy size={11} strokeWidth={1.8} />
              {/if}
            </button>
          </dd>
        </div>
      </dl>

      <p class="section-foot">
        <Info size={11} strokeWidth={1.8} />
        Pour modifier le seed Monte-Carlo : variable d'environnement
        <code>SOBRIA_SEED=42</code> au lancement de <code>cargo run -p sobria-app</code>.
        Reproductibilité garantie quand le seed et N sont identiques.
      </p>
    {/if}
  </section>

  <!-- ─── Préférences (à venir) ───────────────────────────── -->
  <section class="section">
    <header class="section-head">
      <Hammer size={16} strokeWidth={1.8} />
      <h2>Préférences utilisateur · à venir</h2>
      <span class="section-hint mono">chantiers C10 / C11 / C12</span>
    </header>

    <ul class="prefs-grid">
      {#each upcomingPrefs as p (p.label)}
        <li class="pref-card">
          <div class="pref-label">{p.label}</div>
          <div class="pref-hint">{p.hint}</div>
          <span class="pref-chantier mono">{p.chantier}</span>
        </li>
      {/each}
    </ul>
  </section>
</div>

<style>
  .canvas-inner {
    max-width: 1240px;
    margin: 0 auto;
    padding: 40px 56px 80px;
  }

  /* TopBar */
  .topbar {
    display: flex;
    align-items: center;
    gap: 16px;
    margin-bottom: 28px;
  }
  .breadcrumb {
    font: 400 13px/1 var(--font-ui);
    color: var(--ivory-3);
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
    gap: 6px;
    height: 28px;
    padding: 0 12px;
    background: var(--lime-soft);
    border: 1px solid rgba(197, 240, 74, 0.25);
    border-radius: 999px;
    font: 500 11px/1 var(--font-ui);
    color: var(--lime);
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
    text-decoration: none;
    transition: all var(--dur-base) var(--ease);
  }
  .icon-btn:hover {
    background: var(--surface-hi);
    border-color: var(--border-hi);
    color: var(--ivory);
  }

  /* Hero */
  .hero {
    padding-bottom: 24px;
    margin-bottom: 24px;
    border-bottom: 1px solid var(--border);
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
    font: 400 42px/1.1 var(--font-display);
    font-style: italic;
    color: var(--ivory);
    letter-spacing: -0.02em;
    max-width: 720px;
    margin: 0 0 10px;
  }
  .hero-h1 em {
    font-style: normal;
    color: var(--lime);
  }
  .hero-sub {
    font: 400 15px/1.55 var(--font-ui);
    color: var(--ivory-2);
    max-width: 680px;
    margin: 0;
  }
  .hero-sub a {
    color: var(--lime);
    text-decoration: none;
    border-bottom: 1px dashed rgba(197, 240, 74, 0.4);
  }

  /* Banner */
  .banner {
    display: flex;
    gap: 14px;
    align-items: flex-start;
    padding: 14px 18px;
    border-radius: var(--radius-md);
    border: 1px solid var(--border-hi);
    margin-bottom: 20px;
  }
  .banner[data-tone='warn'] {
    background: rgba(245, 183, 105, 0.08);
    border-color: rgba(245, 183, 105, 0.25);
  }
  .banner[data-tone='error'] {
    background: rgba(240, 108, 90, 0.08);
    border-color: rgba(240, 108, 90, 0.3);
  }
  .banner-ico {
    display: inline-flex;
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

  /* Sections */
  .section {
    background: rgba(255, 255, 255, 0.015);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    padding: 24px 28px;
    margin-bottom: 16px;
  }
  .section-head {
    display: flex;
    align-items: baseline;
    gap: 12px;
    margin-bottom: 18px;
    padding-bottom: 12px;
    border-bottom: 1px solid var(--border);
  }
  .section-head :global(svg) {
    color: var(--lime);
    flex-shrink: 0;
    align-self: center;
  }
  .section-head h2 {
    font: 400 22px/1 var(--font-display);
    font-style: italic;
    color: var(--ivory);
    margin: 0;
    flex: 1;
  }
  .section-hint {
    font-size: 11px;
    color: var(--ivory-4);
  }
  .section-foot {
    display: flex;
    align-items: flex-start;
    gap: 8px;
    margin: 18px 0 0;
    padding-top: 14px;
    border-top: 1px dashed var(--border);
    font: 400 12px/1.5 var(--font-ui);
    color: var(--ivory-3);
  }
  .section-foot :global(svg) {
    color: var(--ivory-4);
    margin-top: 2px;
  }
  .section-foot code {
    font: 500 11px/1.4 var(--font-mono);
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 1px 6px;
    color: var(--ivory);
  }

  /* Runtime grid */
  .runtime-skel {
    padding: 24px;
    color: var(--ivory-3);
    font: 400 13px/1 var(--font-mono);
    text-align: center;
  }
  .runtime-grid {
    display: flex;
    flex-direction: column;
    gap: 4px;
    margin: 0;
  }
  .runtime-row {
    display: grid;
    grid-template-columns: 240px 1fr;
    gap: 16px;
    padding: 10px 12px;
    border-radius: var(--radius-sm);
    border: 1px solid transparent;
    align-items: center;
  }
  .runtime-row:hover {
    background: rgba(255, 255, 255, 0.015);
    border-color: var(--border);
  }
  .runtime-row dt {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    font: 500 11px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.12em;
    color: var(--ivory-3);
    margin: 0;
  }
  .runtime-row dt :global(svg) {
    color: var(--ivory-4);
    flex-shrink: 0;
  }
  .runtime-row dd {
    display: flex;
    align-items: center;
    gap: 8px;
    font: 500 13px/1.4 var(--font-mono);
    color: var(--ivory);
    margin: 0;
    min-width: 0;
  }
  .runtime-row dd.break {
    overflow-wrap: anywhere;
    word-break: break-all;
  }
  .copy-btn {
    display: inline-grid;
    place-items: center;
    width: 22px;
    height: 22px;
    background: var(--surface);
    border: 1px solid var(--border);
    color: var(--ivory-3);
    border-radius: 4px;
    cursor: pointer;
    flex-shrink: 0;
    transition: all var(--dur-fast) var(--ease);
  }
  .copy-btn:hover {
    border-color: var(--border-hi);
    color: var(--lime);
  }

  /* Préférences à venir */
  .prefs-grid {
    list-style: none;
    padding: 0;
    margin: 0;
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
    gap: 10px;
  }
  .pref-card {
    position: relative;
    padding: 14px 16px;
    background: rgba(255, 255, 255, 0.02);
    border: 1px dashed var(--border);
    border-radius: var(--radius-md);
  }
  .pref-label {
    font: 500 13px/1.3 var(--font-ui);
    color: var(--ivory);
    margin-bottom: 4px;
  }
  .pref-hint {
    font: 400 12px/1.5 var(--font-ui);
    color: var(--ivory-3);
  }
  .pref-chantier {
    position: absolute;
    top: 10px;
    right: 12px;
    font: 500 9px/1 var(--font-mono);
    color: var(--amber);
    background: rgba(245, 183, 105, 0.08);
    border: 1px solid rgba(245, 183, 105, 0.25);
    border-radius: var(--radius-pill);
    padding: 3px 8px;
    letter-spacing: 0.06em;
  }

  @media (max-width: 720px) {
    .canvas-inner {
      padding: 24px 16px 60px;
    }
    .hero-h1 {
      font-size: 32px;
    }
    .runtime-row {
      grid-template-columns: 1fr;
      gap: 4px;
    }
  }
</style>
