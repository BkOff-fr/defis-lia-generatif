<script lang="ts">
  import {
    BookOpen,
    Scale,
    Database,
    Users,
    ShieldCheck,
    Cpu,
    ArrowUpRight,
    Copy,
    CheckCircle2,
    Lock,
    HelpCircle
  } from '@lucide/svelte';
  import BrandMark from '$lib/components/BrandMark.svelte';
  import {
    isBackendAvailable,
    metaInfo,
    SobriaIpcError,
    type IpcErrorCode,
    type MetaInfo
  } from '$lib/api';

  // ─── État ────────────────────────────────────────────────────────────
  let meta = $state<MetaInfo | null>(null);
  let bootstrapping = $state(true);
  let loadError = $state<{ code: IpcErrorCode | string; message: string } | null>(null);
  let copiedField = $state<string | null>(null);

  const backendAvailable = $derived(isBackendAvailable());

  $effect(() => {
    void (async () => {
      if (!backendAvailable) {
        bootstrapping = false;
        loadError = {
          code: 'tauri_unavailable',
          message:
            "L'état technique runtime (versions, chemins locaux) est visible dans l'application de bureau Sobr.ia."
        };
        return;
      }
      try {
        meta = await metaInfo();
      } catch (err) {
        if (err instanceof SobriaIpcError) {
          loadError = { code: err.code, message: err.message };
        } else {
          loadError = { code: 'internal', message: "Échec du chargement de l'état technique." };
        }
      } finally {
        bootstrapping = false;
      }
    })();
  });

  async function copyToClipboard(value: string, field: string) {
    try {
      await navigator.clipboard.writeText(value);
      copiedField = field;
      setTimeout(() => {
        if (copiedField === field) copiedField = null;
      }, 1800);
    } catch {
      // Si le clipboard refuse (focus non accordé), l'utilisateur peut
      // sélectionner manuellement (user-select: all sur les champs path).
    }
  }

  // ─── Données statiques ───────────────────────────────────────────────
  const GITHUB_REPO = 'https://github.com/BkOff-fr/defis-lia-generatif';
  const GITHUB_ISSUES = `${GITHUB_REPO}/issues`;

  type Licence = { label: string; kind: string; url?: string };
  const licences: Licence[] = [
    {
      label: 'Sobr.ia (code)',
      kind: 'MIT',
      url: `${GITHUB_REPO}/blob/main/LICENSE`
    },
    {
      label: 'Données ODRÉ (RTE / NaTran / Teréga)',
      kind: 'Etalab 2.0',
      url: 'https://www.etalab.gouv.fr/wp-content/uploads/2017/04/ETALAB-Licence-Ouverte-v2.0.pdf'
    },
    {
      label: 'Référentiel AFNOR SPEC 2314',
      kind: 'publique'
    },
    {
      label: 'Polices (Geist, Instrument Serif, JetBrains Mono)',
      kind: 'SIL OFL 1.1',
      url: 'https://openfontlicense.org/'
    },
    {
      label: 'Documentation',
      kind: 'CC-BY 4.0',
      url: 'https://creativecommons.org/licenses/by/4.0/deed.fr'
    }
  ];

  type Source = { label: string; hint: string; url?: string };
  const sources: Source[] = [
    {
      label: 'HF AI Energy Score',
      hint: 'calibration ε prefill / decode',
      url: 'https://huggingface.co/spaces/AIEnergyScore/Leaderboard'
    },
    {
      label: 'RTE eco2mix',
      hint: 'mix électrique FR annuel',
      url: 'https://www.rte-france.com/eco2mix'
    },
    {
      label: 'Electricity Maps + AIB',
      hint: 'mix électrique EU annuel par pays',
      url: 'https://www.electricitymaps.com/'
    },
    {
      label: 'ADEME Base Empreinte',
      hint: 'équivalents parlants',
      url: 'https://base-empreinte.ademe.fr/'
    },
    {
      label: 'Mytton 2021',
      hint: 'water usage effectiveness (WUE)',
      url: 'https://doi.org/10.1038/s41545-021-00101-w'
    },
    {
      label: 'Luccioni et al. 2023',
      hint: 'validation modèles, arXiv:2211.02001',
      url: 'https://arxiv.org/abs/2211.02001'
    },
    {
      label: 'Gebru et al. 2018',
      hint: 'standard datasheet, arXiv:1803.09010',
      url: 'https://arxiv.org/abs/1803.09010'
    }
  ];

  type Contributor = { name: string; role: string };
  const contributors: Contributor[] = [
    { name: 'Thibault', role: 'auteur / mainteneur' },
    { name: 'Cowork', role: 'assistance architecture' },
    { name: 'Claude Code', role: 'assistance code' }
  ];
</script>

<svelte:head>
  <title>Sobr.ia · À propos</title>
</svelte:head>

<div class="canvas-inner">
  <!-- TopBar -->
  <div class="topbar">
    <nav class="breadcrumb" aria-label="Fil d'Ariane">
      Audit <span class="sep">/</span>
      <span class="current">À propos</span>
    </nav>
    <div class="spacer"></div>
    <span class="local-pill" title="Page servie depuis le binaire local">
      <Lock size={12} strokeWidth={1.8} />
      100 % local
    </span>
    <a class="icon-btn" href="/" aria-label="Retour à l'atelier">
      <HelpCircle size={16} strokeWidth={1.6} />
    </a>
  </div>

  <!-- Header / Hero -->
  <header class="hero">
    <div class="hero-brand">
      <BrandMark size={56} uid="apropos" />
      <div>
        <h1 class="hero-h1">Sobr.ia</h1>
        <div class="hero-version mono">
          {#if bootstrapping}
            chargement…
          {:else if meta}
            v{meta.app_version}
          {:else}
            version indisponible
          {/if}
        </div>
      </div>
    </div>
    <p class="hero-mission">
      Sobr.ia mesure l'impact environnemental de l'usage des LLMs avec rigueur scientifique. Open
      source, frugal, transparent. Candidat au défi data.gouv.fr <em
        >« L'impact environnemental de l'IA générative »</em
      >.
    </p>
  </header>

  <!-- ─── Méthodologie ───────────────────────────────────── -->
  <section class="card" aria-labelledby="h-methodo">
    <header class="card-head">
      <BookOpen size={18} strokeWidth={1.6} />
      <h2 id="h-methodo">Méthodologie</h2>
    </header>
    <ul class="bullet-list">
      <li>Notre estimateur applique l'<strong>AFNOR SPEC 2314</strong>.</li>
      <li>
        Monte-Carlo <span class="mono">N&nbsp;=&nbsp;10⁴</span> tirages, seed déterministe (42).
      </li>
      <li>
        Reproduction usage-only à ±20-25&nbsp;% contre la méthodologie <a
          class="link"
          href="https://doi.org/10.21105/joss.07471"
          rel="noopener noreferrer"
          target="_blank">EcoLogits 2026-01</a
        > sur 3 cas (Llama 70B, Mistral Large 2).
      </li>
    </ul>
    <p class="card-foot">
      <a class="link" href="/methodo">
        Voir la méthodologie complète <ArrowUpRight size={12} strokeWidth={2} />
      </a>
    </p>
  </section>

  <!-- ─── Licences ───────────────────────────────────────── -->
  <section class="card" aria-labelledby="h-licences">
    <header class="card-head">
      <Scale size={18} strokeWidth={1.6} />
      <h2 id="h-licences">Licences</h2>
    </header>
    <ul class="kv-list">
      {#each licences as l (l.label)}
        <li>
          <span class="kv-key">{l.label}</span>
          <span class="kv-sep" aria-hidden="true">·</span>
          {#if l.url}
            <a class="kv-link" href={l.url} target="_blank" rel="noopener noreferrer">
              <span class="badge-pill">{l.kind}</span>
              <ArrowUpRight size={12} strokeWidth={2} />
            </a>
          {:else}
            <span class="badge-pill">{l.kind}</span>
          {/if}
        </li>
      {/each}
    </ul>
  </section>

  <!-- ─── Sources des données ───────────────────────────── -->
  <section class="card" aria-labelledby="h-sources">
    <header class="card-head">
      <Database size={18} strokeWidth={1.6} />
      <h2 id="h-sources">Sources des données</h2>
    </header>
    <ul class="src-list">
      {#each sources as s (s.label)}
        <li>
          {#if s.url}
            <a href={s.url} target="_blank" rel="noopener noreferrer">
              <span class="src-label">{s.label}</span>
              <span class="src-hint">{s.hint}</span>
              <ArrowUpRight size={12} strokeWidth={2} />
            </a>
          {:else}
            <span class="src-static">
              <span class="src-label">{s.label}</span>
              <span class="src-hint">{s.hint}</span>
            </span>
          {/if}
        </li>
      {/each}
    </ul>
  </section>

  <!-- ─── Contributeurs ─────────────────────────────────── -->
  <section class="card" aria-labelledby="h-contrib">
    <header class="card-head">
      <Users size={18} strokeWidth={1.6} />
      <h2 id="h-contrib">Contributeurs</h2>
    </header>
    <ul class="contrib-list">
      {#each contributors as c (c.name)}
        <li>
          <span class="contrib-name">{c.name}</span>
          <span class="contrib-role">{c.role}</span>
        </li>
      {/each}
    </ul>
    <p class="card-foot">
      Ouvert aux contributions externes —
      <a class="link" href={GITHUB_ISSUES} target="_blank" rel="noopener noreferrer">
        GitHub Issues <ArrowUpRight size={12} strokeWidth={2} />
      </a>
    </p>
  </section>

  <!-- ─── Mentions légales ──────────────────────────────── -->
  <section class="card" aria-labelledby="h-legal">
    <header class="card-head">
      <ShieldCheck size={18} strokeWidth={1.6} />
      <h2 id="h-legal">Mentions légales</h2>
    </header>
    <ul class="bullet-list">
      <li>Aucune donnée envoyée à un serveur externe.</li>
      <li>Audit ledger local en SQLite avec journalisation WAL chiffrée.</li>
      <li>
        RGPD : droit à l'oubli implémenté via la commande IPC
        <code class="mono inline">purge_audit_before</code>.
      </li>
      <li>Aucune télémétrie, aucun tracking.</li>
    </ul>
  </section>

  <!-- ─── État technique ────────────────────────────────── -->
  <section class="card" aria-labelledby="h-tech">
    <header class="card-head">
      <Cpu size={18} strokeWidth={1.6} />
      <h2 id="h-tech">État technique</h2>
    </header>

    {#if bootstrapping}
      <p class="tech-empty">Chargement de l'état runtime…</p>
    {:else if loadError}
      <p class="tech-error">
        <code class="mono inline">{loadError.code}</code> — {loadError.message}
      </p>
    {:else if meta}
      {@const auditPath = meta.audit_path}
      {@const dataRoot = meta.data_root}
      <dl class="tech-grid">
        <dt>Version</dt>
        <dd class="mono">v{meta.app_version}</dd>

        <dt>Seed Monte-Carlo</dt>
        <dd class="mono">{meta.estimator_seed}</dd>

        <dt>N tirages</dt>
        <dd class="mono">{new Intl.NumberFormat('fr-FR').format(meta.estimator_n)}</dd>

        <dt>Chemin ledger</dt>
        <dd class="path-row">
          <code class="mono path">{auditPath}</code>
          <button
            class="copy-btn"
            type="button"
            onclick={() => copyToClipboard(auditPath, 'audit')}
            aria-label="Copier le chemin du ledger d'audit"
          >
            {#if copiedField === 'audit'}
              <CheckCircle2 size={12} strokeWidth={2} /> copié
            {:else}
              <Copy size={12} strokeWidth={1.8} /> copier
            {/if}
          </button>
        </dd>

        <dt>Racine données</dt>
        <dd class="path-row">
          <code class="mono path">{dataRoot}</code>
          <button
            class="copy-btn"
            type="button"
            onclick={() => copyToClipboard(dataRoot, 'data')}
            aria-label="Copier le chemin de la racine des données"
          >
            {#if copiedField === 'data'}
              <CheckCircle2 size={12} strokeWidth={2} /> copié
            {:else}
              <Copy size={12} strokeWidth={1.8} /> copier
            {/if}
          </button>
        </dd>
      </dl>
    {/if}
  </section>

  <!-- ─── Footer ─────────────────────────────────────────── -->
  <footer class="page-foot">
    <span>© 2026 Sobr.ia</span>
    <span class="sep" aria-hidden="true">·</span>
    <span>MIT</span>
    <span class="sep" aria-hidden="true">·</span>
    <span>Made in France 🇫🇷</span>
  </footer>
</div>

<style>
  .canvas-inner {
    max-width: 800px;
    margin: 0 auto;
    padding: 40px 32px 80px;
    display: flex;
    flex-direction: column;
    gap: 24px;
  }

  /* TopBar */
  .topbar {
    display: flex;
    align-items: center;
    gap: 16px;
    margin-bottom: 4px;
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
    border-radius: var(--radius-pill);
    font: 500 12px/1 var(--font-ui);
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
    cursor: pointer;
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
    border-bottom: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    gap: 16px;
  }
  .hero-brand {
    display: flex;
    align-items: center;
    gap: 16px;
  }
  .hero-h1 {
    font: 400 36px/1.05 var(--font-display);
    font-style: italic;
    color: var(--ivory);
    letter-spacing: -0.02em;
    margin: 0;
  }
  .hero-version {
    font: 500 12px/1 var(--font-mono);
    color: var(--ivory-3);
    letter-spacing: 0.08em;
    margin-top: 4px;
  }
  .hero-mission {
    font: 400 15px/1.6 var(--font-ui);
    color: var(--ivory-2);
    margin: 0;
  }
  .hero-mission em {
    font-style: italic;
    color: var(--lime);
  }

  /* Cards */
  .card {
    background: rgba(255, 255, 255, 0.015);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    padding: 24px 28px;
  }
  .card-head {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-bottom: 14px;
    padding-bottom: 12px;
    border-bottom: 1px solid var(--border);
  }
  .card-head :global(svg) {
    color: var(--lime);
    flex-shrink: 0;
  }
  .card-head h2 {
    font: 400 22px/1 var(--font-display);
    font-style: italic;
    color: var(--ivory);
    margin: 0;
    flex: 1;
  }
  .card-foot {
    margin: 14px 0 0;
    font-size: 12px;
    color: var(--ivory-3);
  }
  .link {
    color: var(--lime);
    text-decoration: none;
    border-bottom: 1px dashed rgba(197, 240, 74, 0.4);
    display: inline-flex;
    align-items: center;
    gap: 4px;
  }
  .link:hover {
    border-bottom-color: var(--lime);
  }

  /* Bullet list */
  .bullet-list {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .bullet-list li {
    position: relative;
    padding-left: 16px;
    font: 400 14px/1.6 var(--font-ui);
    color: var(--ivory-2);
  }
  .bullet-list li::before {
    content: '';
    position: absolute;
    left: 0;
    top: 9px;
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--lime);
    box-shadow: 0 0 6px var(--lime-glow);
  }
  .bullet-list strong {
    color: var(--ivory);
    font-weight: 500;
  }

  /* Licences (kv) */
  .kv-list {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .kv-list li {
    display: flex;
    align-items: center;
    gap: 10px;
    flex-wrap: wrap;
    padding: 8px 0;
    border-bottom: 1px dashed var(--border);
  }
  .kv-list li:last-child {
    border-bottom: none;
  }
  .kv-key {
    font: 400 13px/1.4 var(--font-ui);
    color: var(--ivory);
    flex: 1;
    min-width: 0;
  }
  .kv-sep {
    color: var(--ivory-4);
  }
  .kv-link {
    color: var(--ivory-3);
    text-decoration: none;
    display: inline-flex;
    align-items: center;
    gap: 4px;
    border-bottom: none;
  }
  .kv-link:hover {
    color: var(--lime);
  }
  .badge-pill {
    display: inline-flex;
    padding: 3px 10px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-pill);
    font: 500 12px/1.4 var(--font-mono);
    color: var(--ivory-2);
    letter-spacing: 0.04em;
  }
  .kv-link:hover .badge-pill {
    border-color: rgba(197, 240, 74, 0.4);
    color: var(--lime);
  }

  /* Sources */
  .src-list {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .src-list li a,
  .src-list li .src-static {
    display: flex;
    align-items: baseline;
    gap: 10px;
    padding: 10px 12px;
    background: rgba(255, 255, 255, 0.02);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    transition: all var(--dur-base) var(--ease);
    text-decoration: none;
  }
  .src-list li a:hover {
    border-color: rgba(197, 240, 74, 0.3);
    background: rgba(197, 240, 74, 0.04);
  }
  .src-label {
    font: 500 13px/1.3 var(--font-ui);
    color: var(--ivory);
    flex: 1;
  }
  .src-hint {
    font: 400 12px/1.3 var(--font-mono);
    color: var(--ivory-3);
    flex-shrink: 0;
  }
  .src-list :global(svg) {
    color: var(--ivory-3);
    flex-shrink: 0;
  }
  .src-list a:hover :global(svg) {
    color: var(--lime);
  }

  /* Contributeurs */
  .contrib-list {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .contrib-list li {
    display: flex;
    align-items: baseline;
    gap: 12px;
    padding: 6px 0;
  }
  .contrib-name {
    font: 500 14px/1.4 var(--font-ui);
    color: var(--ivory);
    min-width: 120px;
  }
  .contrib-role {
    font: 400 12px/1.4 var(--font-ui);
    color: var(--ivory-3);
    font-style: italic;
  }

  /* État technique */
  .tech-empty,
  .tech-error {
    font: 400 13px/1.5 var(--font-ui);
    color: var(--ivory-3);
    margin: 0;
  }
  .tech-error {
    color: var(--ivory-2);
  }
  .tech-grid {
    display: grid;
    grid-template-columns: 140px 1fr;
    gap: 10px 16px;
    margin: 0;
  }
  .tech-grid dt {
    font: 500 12px/1.4 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.12em;
    color: var(--ivory-3);
    padding-top: 6px;
  }
  .tech-grid dd {
    font: 400 13px/1.5 var(--font-mono);
    color: var(--ivory);
    margin: 0;
  }
  .path-row {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }
  .path {
    flex: 1;
    min-width: 0;
    word-break: break-all;
    user-select: all;
    background: var(--ink-2);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    padding: 6px 10px;
    font-size: 12px;
    color: var(--ivory);
  }
  .copy-btn {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    height: 26px;
    padding: 0 10px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    color: var(--ivory-2);
    font: 500 12px/1 var(--font-ui);
    cursor: pointer;
    transition: all var(--dur-base) var(--ease);
    flex-shrink: 0;
  }
  .copy-btn:hover {
    background: var(--surface-hi);
    border-color: var(--border-hi);
    color: var(--ivory);
  }
  .copy-btn :global(svg) {
    color: var(--lime);
  }

  /* Inline code */
  .inline {
    background: var(--ink-2);
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 1px 6px;
    font-size: 12px;
    color: var(--ivory);
  }

  /* Footer */
  .page-foot {
    display: flex;
    align-items: center;
    gap: 8px;
    justify-content: center;
    padding: 16px 0 0;
    font: 400 12px/1 var(--font-mono);
    color: var(--ivory-4);
  }
  .page-foot .sep {
    color: var(--ivory-4);
  }

  @media (max-width: 720px) {
    .canvas-inner {
      padding: 24px 16px 60px;
    }
    .card {
      padding: 18px 18px;
    }
    .hero-h1 {
      font-size: 28px;
    }
    .tech-grid {
      grid-template-columns: 1fr;
    }
    .tech-grid dt {
      padding-top: 4px;
    }
  }
</style>
