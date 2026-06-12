<script lang="ts">
  import { HelpCircle, Lock, ArrowUpRight, Hammer, Zap } from '@lucide/svelte';

  type IpcEntry = { name: string; description: string };
  /** Type partagé pour une icône Lucide-svelte (Svelte 5 Component). */
  type LucideIcon = typeof Zap;

  type Props = {
    /** Identifiant du module CDC (« M4 », « M10 », …) — affiché dans
     *  l'eyebrow. */
    moduleId: string;
    /** Titre éditorial italic de la page. Mettre l'élément à mettre en
     *  exergue dans `<em>…</em>`. */
    title: string;
    /** Sous-titre explicatif (1 paragraphe). */
    subtitle: string;
    /** Crumb « Atelier / … » — texte de la dernière étape. */
    crumb: string;
    /** Icône Lucide pour la card principale. */
    icon: LucideIcon;
    /** Liste des IPC Rust attendus pour activer l'écran. */
    pendingIpcs: IpcEntry[];
    /** Chantier prévu (référence interne, ex. « C10 — outillage »). */
    chantier: string;
    /** Référence(s) d'EF du CDC, séparées par virgule (ex. « EF-M4-01..06 »). */
    efs?: string;
  };

  const {
    moduleId,
    title,
    subtitle,
    crumb,
    icon: Icon,
    pendingIpcs,
    chantier,
    efs
  }: Props = $props();
</script>

<svelte:head>
  <title>Sobr.ia · {crumb}</title>
</svelte:head>

<div class="canvas-inner">
  <!-- TopBar -->
  <div class="topbar">
    <nav class="breadcrumb" aria-label="Fil d'Ariane">
      Atelier <span class="sep">/</span>
      <span class="current">{crumb}</span>
    </nav>
    <div class="spacer"></div>
    <span class="local-pill" title="Tout traitement reste local">
      <Lock size={12} strokeWidth={1.8} />
      100 % local
    </span>
    <a class="icon-btn" href="/methodo" aria-label="Méthodologie">
      <HelpCircle size={16} strokeWidth={1.6} />
    </a>
  </div>

  <!-- Hero -->
  <section class="hero">
    <div class="hero-eyebrow">
      <span class="pulse" aria-hidden="true"></span>
      Module {moduleId} · en chantier
    </div>
    <!-- Le HTML autorisé ici (`<em>…</em>`) vient toujours d'un littéral
         de code (page stub), jamais d'une entrée utilisateur — pas de
         vecteur XSS. La règle ESLint est désactivée pour cette ligne. -->
    <!-- eslint-disable-next-line svelte/no-at-html-tags -->
    <h1 class="hero-h1">{@html title}</h1>
    <p class="hero-sub">{subtitle}</p>
  </section>

  <!-- Carte status -->
  <section class="status-card" aria-label="Statut d'implémentation">
    <header class="status-head">
      <span class="status-ico" aria-hidden="true">
        <Icon size={20} strokeWidth={1.6} />
      </span>
      <div class="status-body">
        <div class="status-eye">
          <Hammer size={11} strokeWidth={1.8} />
          Bientôt disponible
        </div>
        <h2>Écran prévu pour le chantier {chantier}.</h2>
        <p>
          Cet écran fait partie du périmètre v1.0 du cahier des charges (cf. <a href="/methodo"
            >méthodologie</a
          >) mais nécessite encore du côté <code>sobria-app</code> les commandes IPC listées ci-dessous.
          Une fois implémentées par le cœur Rust, l'UI sera plombée en ~1-2 jours d'effort.
        </p>
        {#if efs}
          <p class="ef-line mono">
            EF couvertes : <b>{efs}</b>
          </p>
        {/if}
      </div>
    </header>

    <div class="ipc-block">
      <div class="ipc-title">IPC attendus</div>
      <ul class="ipc-list">
        {#each pendingIpcs as ipc (ipc.name)}
          <li>
            <code class="ipc-name">{ipc.name}</code>
            <span class="ipc-desc">{ipc.description}</span>
          </li>
        {/each}
      </ul>
    </div>

    <footer class="status-foot">
      <a class="back-link" href="/"> ← Retour à l'atelier d'estimation </a>
      <a class="methodo-link" href="/methodo">
        Voir la méthodologie complète
        <ArrowUpRight size={11} strokeWidth={2} />
      </a>
    </footer>
  </section>
</div>

<style>
  .canvas-inner {
    max-width: 1240px;
    margin: 0 auto;
    padding: 40px 56px 80px;
  }

  /* TopBar (clone allégé) */
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
    text-decoration: none;
    transition: all var(--dur-base) var(--ease);
  }
  .icon-btn:hover {
    background: var(--surface-hi);
    border-color: var(--border-hi);
    color: var(--ivory);
  }

  /* Hero compact (sans bordure pour distinguer des écrans actifs) */
  .hero {
    margin-bottom: 24px;
  }
  .hero-eyebrow {
    font: 500 12px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.16em;
    color: var(--amber);
    margin-bottom: 14px;
    display: inline-flex;
    align-items: center;
    gap: 8px;
  }
  .hero-eyebrow .pulse {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--amber);
    box-shadow: 0 0 10px rgba(245, 183, 105, 0.6);
  }
  .hero-h1 {
    font: 400 42px/1.1 var(--font-display);
    font-style: italic;
    color: var(--ivory);
    letter-spacing: -0.02em;
    max-width: 760px;
    margin: 0 0 10px;
  }
  .hero-h1 :global(em) {
    font-style: normal;
    color: var(--lime);
  }
  .hero-sub {
    font: 400 15px/1.55 var(--font-ui);
    color: var(--ivory-2);
    max-width: 680px;
    margin: 0;
  }

  /* Status card */
  .status-card {
    background: linear-gradient(160deg, rgba(245, 183, 105, 0.04), rgba(255, 255, 255, 0.005));
    border: 1px dashed rgba(245, 183, 105, 0.3);
    border-radius: var(--radius-lg);
    padding: 26px 30px;
    animation: rise 400ms var(--ease);
  }
  .status-head {
    display: flex;
    gap: 16px;
    align-items: flex-start;
    padding-bottom: 18px;
    margin-bottom: 18px;
    border-bottom: 1px dashed var(--border);
  }
  .status-ico {
    display: grid;
    place-items: center;
    width: 44px;
    height: 44px;
    background: rgba(245, 183, 105, 0.12);
    color: var(--amber);
    border: 1px solid rgba(245, 183, 105, 0.3);
    border-radius: var(--radius-md);
    flex-shrink: 0;
  }
  .status-body {
    flex: 1;
    min-width: 0;
  }
  .status-eye {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font: 500 12px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.14em;
    color: var(--amber);
    margin-bottom: 6px;
  }
  .status-body h2 {
    font: 400 22px/1.2 var(--font-display);
    font-style: italic;
    color: var(--ivory);
    margin: 0 0 8px;
    letter-spacing: -0.01em;
  }
  .status-body p {
    font: 400 14px/1.55 var(--font-ui);
    color: var(--ivory-2);
    margin: 0;
  }
  .status-body p + p {
    margin-top: 6px;
  }
  .status-body a {
    color: var(--lime);
    text-decoration: none;
    border-bottom: 1px dashed rgba(197, 240, 74, 0.4);
  }
  .status-body code {
    font-family: var(--font-mono);
    font-size: 12px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 1px 6px;
    color: var(--ivory);
  }
  .ef-line {
    color: var(--ivory-3);
    font-size: 12px;
  }
  .ef-line b {
    color: var(--lime);
    font-weight: 500;
  }

  /* IPC block */
  .ipc-block {
    background: rgba(0, 0, 0, 0.2);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    padding: 14px 18px;
  }
  .ipc-title {
    font: 500 12px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.14em;
    color: var(--ivory-3);
    margin-bottom: 10px;
  }
  .ipc-list {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .ipc-list li {
    display: grid;
    grid-template-columns: minmax(180px, max-content) 1fr;
    gap: 12px;
    align-items: baseline;
    padding: 6px 0;
  }
  .ipc-list li + li {
    border-top: 1px dashed var(--border);
  }
  .ipc-name {
    font: 500 12px/1.4 var(--font-mono);
    color: var(--lime);
    background: var(--lime-soft);
    border: 1px solid rgba(197, 240, 74, 0.25);
    border-radius: 4px;
    padding: 2px 8px;
  }
  .ipc-desc {
    font: 400 12px/1.5 var(--font-ui);
    color: var(--ivory-2);
  }

  .status-foot {
    display: flex;
    gap: 12px;
    margin-top: 18px;
    padding-top: 18px;
    border-top: 1px dashed var(--border);
    flex-wrap: wrap;
  }
  .back-link {
    font: 500 13px/1 var(--font-ui);
    color: var(--ivory-2);
    text-decoration: none;
    transition: color var(--dur-base) var(--ease);
  }
  .back-link:hover {
    color: var(--ivory);
  }
  .methodo-link {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    margin-left: auto;
    font: 500 13px/1 var(--font-ui);
    color: var(--lime);
    text-decoration: none;
    border-bottom: 1px dashed rgba(197, 240, 74, 0.4);
  }
  .methodo-link:hover {
    border-bottom-color: var(--lime);
  }

  @keyframes rise {
    from {
      opacity: 0;
      transform: translateY(8px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  @media (max-width: 720px) {
    .canvas-inner {
      padding: 24px 16px 60px;
    }
    .hero-h1 {
      font-size: 32px;
    }
  }
</style>
