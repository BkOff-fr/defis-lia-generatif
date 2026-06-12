<script lang="ts">
  // Module M22 — Rapport CSRD / AGEC (C14).
  // Consomme la commande IPC `export_csrd_report` exposée par sobria-app.
  // Contrat no-mock : hors Tauri, formulaire désactivé + bannière.
  //
  // Voir :
  //   - briefs/chantiers/C14-rapport-csrd-agec.md
  //   - crates/sobria-app/src/dto.rs (bloc "CSRD report")
  //   - docs/CAHIER-DES-CHARGES-v1.0.md §4 M22

  import {
    AlertTriangle,
    PlugZap,
    HelpCircle,
    Lock,
    FileText,
    Calendar,
    Building,
    Languages,
    FolderOpen,
    Sparkles,
    CheckCircle2,
    Copy,
    Loader2,
    FileCode,
    Activity,
    Leaf,
    Zap,
    Droplet,
    ArrowUpRight
  } from '@lucide/svelte';
  import {
    isBackendAvailable,
    isTauriContext,
    exportCsrdReport,
    SobriaIpcError,
    type CsrdReportResultDto,
    type IpcErrorCode
  } from '$lib/api';
  import { preferences, type ModuleId } from '$lib/preferences';

  const MODULE_ID: ModuleId = 'm22';

  // Module gating
  $effect(() => {
    if ($preferences.loaded && !$preferences.enabled_modules.includes(MODULE_ID)) {
      window.location.replace('/?disabled=' + MODULE_ID);
    }
  });

  // ─── Helpers de date ─────────────────────────────────────────────────────
  //
  // Format <input type="date"> = `YYYY-MM-DD`. RFC 3339 attendu côté Rust :
  // `YYYY-MM-DDTHH:MM:SSZ`. On normalise : period_start → 00:00:00Z,
  // period_end → 23:59:59Z (capte toute la journée incluse).

  function todayIso(): string {
    const d = new Date();
    const y = d.getFullYear();
    const m = String(d.getMonth() + 1).padStart(2, '0');
    const day = String(d.getDate()).padStart(2, '0');
    return `${y}-${m}-${day}`;
  }

  function previousQuarterStart(): string {
    const now = new Date();
    const month = now.getMonth(); // 0-11
    const quarter = Math.floor(month / 3); // 0..3
    // Trimestre PRÉCÉDENT : si on est dans Q (quarter), on prend Q-1.
    // Si Q=0 (Q1), on prend Q4 de l'année précédente.
    let prevQ = quarter - 1;
    let year = now.getFullYear();
    if (prevQ < 0) {
      prevQ = 3;
      year -= 1;
    }
    const startMonth = prevQ * 3 + 1; // 1, 4, 7, 10
    return `${year}-${String(startMonth).padStart(2, '0')}-01`;
  }

  function toIsoUtc(dateYmd: string, endOfDay: boolean): string {
    if (!/^\d{4}-\d{2}-\d{2}$/.test(dateYmd)) return dateYmd;
    return `${dateYmd}T${endOfDay ? '23:59:59' : '00:00:00'}Z`;
  }

  // ─── State formulaire ───────────────────────────────────────────────────
  let orgName = $state('');
  let periodStart = $state(previousQuarterStart());
  let periodEnd = $state(todayIso());
  let locale = $state<'fr' | 'en'>('fr');

  // ─── State génération ───────────────────────────────────────────────────
  let generating = $state(false);
  let report = $state<CsrdReportResultDto | null>(null);
  let genError = $state<{ code: IpcErrorCode; message: string } | null>(null);
  let copiedField = $state<string | null>(null);

  const backendAvailable = $derived(isBackendAvailable());
  const desktopAvailable = $derived(isTauriContext());

  const formValid = $derived(
    orgName.trim().length > 0 &&
      periodStart.length === 10 &&
      periodEnd.length === 10 &&
      periodStart <= periodEnd
  );

  const datesInvalid = $derived(
    periodStart.length === 10 && periodEnd.length === 10 && periodStart > periodEnd
  );

  // ─── Génération ─────────────────────────────────────────────────────────

  async function pickDirAndGenerate() {
    if (!backendAvailable || !formValid) return;

    // Dialog : choisir un dossier où écrire `report.pdf` + `provo.jsonld`.
    let outputDir: string;
    try {
      const dialog = await import('@tauri-apps/plugin-dialog');
      const picked = await dialog.open({
        directory: true,
        multiple: false,
        title: 'Choisir le dossier de sortie pour le rapport CSRD'
      });
      if (typeof picked !== 'string') return; // utilisateur annule
      outputDir = picked;
    } catch (err) {
      genError = {
        code: 'internal',
        message: err instanceof Error ? err.message : 'Échec du dialogue de sélection'
      };
      return;
    }

    generating = true;
    genError = null;
    report = null;
    try {
      const result = await exportCsrdReport(
        {
          period_start: toIsoUtc(periodStart, false),
          period_end: toIsoUtc(periodEnd, true),
          organization_name: orgName.trim(),
          locale
        },
        outputDir
      );
      report = result;
    } catch (err) {
      if (err instanceof SobriaIpcError) {
        genError = { code: err.code, message: err.message };
      } else {
        genError = { code: 'internal', message: 'Échec de la génération du rapport' };
      }
    } finally {
      generating = false;
    }
  }

  async function copyToClipboard(value: string, field: string) {
    try {
      await navigator.clipboard.writeText(value);
      copiedField = field;
      setTimeout(() => {
        if (copiedField === field) copiedField = null;
      }, 1800);
    } catch {
      // Si clipboard refuse (focus non accordé), on ne fait rien — l'user
      // peut toujours sélectionner manuellement (user-select: all sur les
      // champs path/SHA).
    }
  }

  function resetForm() {
    report = null;
    genError = null;
  }

  function fmt(value: number, digits = 2): string {
    if (!Number.isFinite(value)) return '—';
    return new Intl.NumberFormat('fr-FR', { maximumFractionDigits: digits }).format(value);
  }

  type AutoScale = { v: string; u: string };
  function fmtAuto(value: number, base: 'g' | 'Wh' | 'L'): AutoScale {
    if (!Number.isFinite(value)) return { v: '—', u: base };
    if (base === 'g') {
      if (value >= 1e6) return { v: fmt(value / 1e6, 2), u: 't CO₂eq' };
      if (value >= 1e3) return { v: fmt(value / 1e3, 2), u: 'kg CO₂eq' };
      return { v: fmt(value, 2), u: 'g CO₂eq' };
    }
    if (base === 'Wh') {
      if (value >= 1e6) return { v: fmt(value / 1e6, 2), u: 'MWh' };
      if (value >= 1e3) return { v: fmt(value / 1e3, 2), u: 'kWh' };
      return { v: fmt(value, 2), u: 'Wh' };
    }
    if (value >= 1000) return { v: fmt(value / 1000, 2), u: 'm³' };
    if (value >= 1) return { v: fmt(value, 2), u: 'L' };
    return { v: fmt(value * 1000, 1), u: 'mL' };
  }

  // ─── Erreurs ─────────────────────────────────────────────────────────────
  const ERROR_LABELS: Record<string, string> = {
    tauri_unavailable: 'Application de bureau requise',
    invalid_request: 'Paramètres invalides',
    empty_period: 'Aucune entrée pour cette période',
    export_error: 'Échec de la génération PDF',
    internal: 'Erreur interne'
  };
  function errorLabel(code: string): string {
    return ERROR_LABELS[code] ?? 'Erreur';
  }
  function errorHelp(code: string): string {
    switch (code) {
      case 'empty_period':
        return "Aucune estimation enregistrée dans cette plage de dates. Essaie une période plus large, ou commence par effectuer quelques estimations depuis l'Atelier.";
      case 'invalid_request':
        return "Vérifie que la date de début est antérieure à la date de fin, et que le nom de l'organisation est rempli.";
      case 'export_error':
        return 'La génération du PDF a échoué — vérifie que le dossier de sortie est inscriptible.';
      default:
        return '';
    }
  }
</script>

<svelte:head>
  <title>Sobr.ia · Rapport réglementaire (CSRD/AGEC)</title>
</svelte:head>

<div class="canvas-inner">
  <!-- TopBar -->
  <div class="topbar">
    <nav class="breadcrumb" aria-label="Fil d'Ariane">
      Atelier <span class="sep">/</span>
      <span class="current">Rapport réglementaire (CSRD/AGEC)</span>
    </nav>
    <div class="spacer"></div>
    <span class="local-pill">
      <Lock size={12} strokeWidth={1.8} />
      Génération 100 % locale
    </span>
    <a class="icon-btn" href="/methodo" aria-label="Méthodologie">
      <HelpCircle size={16} strokeWidth={1.6} />
    </a>
  </div>

  <!-- Hero -->
  <section class="hero">
    <div class="hero-eyebrow">
      <span class="pulse" aria-hidden="true"></span>
      Rapport réglementaire · AFNOR SPEC 2314 · PROV-O W3C
    </div>
    <h1 class="hero-h1">
      Un rapport <em>conforme</em>, prêt à signer.
    </h1>
    <p class="hero-sub">
      Générez un PDF officiel pour votre scope 3 IA (CSRD) ou votre bilan numérique (AGEC) sur la
      période de votre choix. Le bundle JSON-LD PROV-O accompagne le PDF pour la reproductibilité
      audit.
    </p>
  </section>

  <!-- C42 — bannière au chargement hors Tauri : le formulaire reste lisible
       mais la génération (écriture disque) est clairement annoncée comme
       desktop-only, au lieu d'échouer tard avec un code interne. -->
  {#if !desktopAvailable}
    <div class="banner" data-tone="warn" role="alert">
      <span class="banner-ico" aria-hidden="true"
        ><AlertTriangle size={18} strokeWidth={1.8} /></span
      >
      <div class="banner-body">
        <strong>Application de bureau requise</strong>
        <span>
          La génération du rapport (PDF + bundle PROV-O) écrit sur votre disque et lit votre ledger
          d'audit : elle nécessite l'application de bureau Sobr.ia. La génération tourne en local —
          pas de serveur, pas d'envoi externe.
        </span>
      </div>
    </div>
  {/if}

  <!-- Formulaire OU Card de succès -->
  {#if report}
    {@const r = report}
    <section class="success" aria-live="polite">
      <header class="sh">
        <span class="ico"><CheckCircle2 size={16} strokeWidth={1.8} /></span>
        <div>
          <div class="eyebrow">Rapport généré</div>
          <h3>{orgName} — {periodStart} → {periodEnd}</h3>
        </div>
        <button class="reset-btn" type="button" onclick={resetForm}>Nouveau rapport</button>
      </header>

      <div class="stats">
        <div class="stat">
          <div class="stat-l"><Activity size={11} strokeWidth={1.8} /> Requêtes journalisées</div>
          <div class="stat-v">{fmt(r.total_requests, 0)}</div>
          <div class="stat-r mono">{r.audit_entries_count} entrées d'audit</div>
        </div>
        {#snippet metric(
          label: string,
          value: number,
          base: 'g' | 'Wh' | 'L',
          icon: typeof Leaf,
          tone: string
        )}
          {@const a = fmtAuto(value, base)}
          <div class="stat">
            <div class="stat-l">
              {#if icon}
                {@const Icon = icon}
                <Icon size={11} strokeWidth={1.8} />
              {/if}
              {label}
            </div>
            <div class="stat-v" style="color: {tone}">{a.v}<span class="u">{a.u}</span></div>
          </div>
        {/snippet}
        {@render metric('CO₂eq P50 cumulé', r.total_co2eq_g_p50, 'g', Leaf, '#c5f04a')}
        {@render metric('Énergie P50', r.total_energy_wh_p50, 'Wh', Zap, '#7eb6ff')}
        {@render metric('Eau P50', r.total_water_l_p50, 'L', Droplet, '#b794f4')}
      </div>

      <div class="files">
        <div class="file-row">
          <FileText size={14} strokeWidth={1.8} />
          <div class="fr-main">
            <div class="fr-label">PDF officiel</div>
            <div class="fr-path mono">{r.pdf_path}</div>
          </div>
          <button
            class="copy-btn"
            type="button"
            onclick={() => copyToClipboard(r.pdf_path, 'pdf')}
            aria-label="Copier le chemin du PDF"
          >
            {#if copiedField === 'pdf'}
              <CheckCircle2 size={12} strokeWidth={2} /> copié
            {:else}
              <Copy size={12} strokeWidth={1.8} /> copier
            {/if}
          </button>
        </div>
        <div class="file-row">
          <FileCode size={14} strokeWidth={1.8} />
          <div class="fr-main">
            <div class="fr-label">JSON-LD PROV-O</div>
            <div class="fr-path mono">{r.provo_path}</div>
          </div>
          <button
            class="copy-btn"
            type="button"
            onclick={() => copyToClipboard(r.provo_path, 'provo')}
            aria-label="Copier le chemin du PROV-O"
          >
            {#if copiedField === 'provo'}
              <CheckCircle2 size={12} strokeWidth={2} /> copié
            {:else}
              <Copy size={12} strokeWidth={1.8} /> copier
            {/if}
          </button>
        </div>
        <div class="sha-row">
          <span class="sha-label">SHA-256 PDF</span>
          <code class="sha-val mono">{r.pdf_sha256}</code>
          <button
            class="copy-btn"
            type="button"
            onclick={() => copyToClipboard(r.pdf_sha256, 'sha')}
            aria-label="Copier le SHA-256"
          >
            {#if copiedField === 'sha'}
              <CheckCircle2 size={12} strokeWidth={2} /> copié
            {:else}
              <Copy size={12} strokeWidth={1.8} /> copier
            {/if}
          </button>
        </div>
      </div>

      <footer class="sfoot">
        <a class="meth-link" href="/methodo#methode">
          Méthodologie AFNOR SPEC 2314 <ArrowUpRight size={11} strokeWidth={2} />
        </a>
        <span class="hint">
          Astuce : copie le SHA-256 dans ton dépôt Git pour figer la provenance du rapport.
        </span>
      </footer>
    </section>
  {:else}
    <section class="form-card" aria-label="Formulaire rapport CSRD">
      <header class="fh">
        <div class="eyebrow">
          <FileText size={11} strokeWidth={1.8} /> Génération du rapport
        </div>
        <h2>4 champs, un dossier de sortie, un PDF conforme.</h2>
      </header>

      <form
        class="form"
        onsubmit={(e) => {
          e.preventDefault();
          void pickDirAndGenerate();
        }}
        novalidate
      >
        <label class="field">
          <span><Building size={11} strokeWidth={1.8} /> Nom de l'organisation</span>
          <input
            type="text"
            bind:value={orgName}
            placeholder="Ex. Mairie de Lille"
            maxlength="120"
            required
            disabled={!backendAvailable || generating}
            class="text-input"
            aria-required="true"
          />
        </label>

        <div class="field-row">
          <label class="field">
            <span><Calendar size={11} strokeWidth={1.8} /> Date début</span>
            <input
              type="date"
              bind:value={periodStart}
              required
              disabled={!backendAvailable || generating}
              class="date-input mono"
              max={periodEnd}
            />
          </label>
          <label class="field">
            <span><Calendar size={11} strokeWidth={1.8} /> Date fin</span>
            <input
              type="date"
              bind:value={periodEnd}
              required
              disabled={!backendAvailable || generating}
              class="date-input mono"
              min={periodStart}
              max={todayIso()}
            />
          </label>
        </div>

        {#if datesInvalid}
          <p class="field-err">La date de début doit être antérieure à la date de fin.</p>
        {/if}

        <label class="field">
          <span><Languages size={11} strokeWidth={1.8} /> Langue du rapport</span>
          <select
            bind:value={locale}
            disabled={!backendAvailable || generating}
            class="select-input"
          >
            <option value="fr">Français</option>
            <option value="en" disabled>English (v1.1)</option>
          </select>
        </label>

        {#if genError}
          <div class="form-err" role="alert">
            <span class="err-ico"><PlugZap size={14} strokeWidth={1.8} /></span>
            <div>
              <strong>{errorLabel(genError.code)}</strong>
              <span>{genError.message}</span>
              {#if errorHelp(genError.code)}
                <span class="help">{errorHelp(genError.code)}</span>
              {/if}
            </div>
          </div>
        {/if}

        <div class="actions">
          <button
            type="submit"
            class="btn-primary"
            disabled={!desktopAvailable || generating || !formValid}
            title={desktopAvailable
              ? undefined
              : 'La génération PDF/PROV-O écrit sur votre disque — application de bureau requise'}
            aria-busy={generating}
          >
            {#if generating}
              <Loader2 size={14} strokeWidth={2} class="spin" /> Génération en cours…
            {:else}
              <FolderOpen size={14} strokeWidth={2} /> Choisir le dossier puis générer
            {/if}
          </button>
          <button
            type="button"
            class="btn-ghost"
            disabled={generating}
            onclick={() => {
              orgName = '';
              periodStart = previousQuarterStart();
              periodEnd = todayIso();
              locale = 'fr';
              genError = null;
            }}
          >
            Réinitialiser
          </button>
          <span class="kbd-hint mono">
            <Sparkles size={11} strokeWidth={1.8} /> Génération &lt; 1 s pour 1000 entrées
          </span>
        </div>
      </form>
    </section>
  {/if}
</div>

<style>
  .canvas-inner {
    max-width: 960px;
    margin: 0 auto;
    padding: 40px 56px 80px;
    display: flex;
    flex-direction: column;
    gap: 24px;
  }

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
    cursor: pointer;
    text-decoration: none;
    transition: all var(--dur-base) var(--ease);
  }
  .icon-btn:hover {
    background: var(--surface-hi);
    border-color: var(--border-hi);
    color: var(--ivory);
  }

  .hero {
    padding-bottom: 14px;
    border-bottom: 1px solid var(--border);
  }
  .hero-eyebrow {
    font: 500 12px/1 var(--font-ui);
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
    margin: 0 0 8px;
  }
  .hero-h1 em {
    font-style: normal;
    color: var(--lime);
  }
  .hero-sub {
    font: 400 15px/1.55 var(--font-ui);
    color: var(--ivory-2);
    max-width: 720px;
    margin: 0;
  }

  /* Bannière warn */
  .banner {
    display: flex;
    gap: 14px;
    align-items: flex-start;
    padding: 14px 18px;
    border-radius: var(--radius-md);
    border: 1px solid var(--border-hi);
  }
  .banner[data-tone='warn'] {
    background: rgba(245, 183, 105, 0.08);
    border-color: rgba(245, 183, 105, 0.25);
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

  /* Form card */
  .form-card {
    padding: 28px 32px;
    background: linear-gradient(180deg, rgba(255, 255, 255, 0.025), rgba(255, 255, 255, 0.005));
    border: 1px solid var(--border);
    border-radius: var(--radius-xl);
  }
  .fh {
    margin-bottom: 18px;
    padding-bottom: 14px;
    border-bottom: 1px solid var(--border);
  }
  .fh .eyebrow {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    font: 500 12px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.14em;
    color: var(--ivory-3);
    margin-bottom: 6px;
  }
  .fh h2 {
    font: 400 24px/1.15 var(--font-display);
    font-style: italic;
    color: var(--ivory);
    letter-spacing: -0.01em;
    margin: 0;
  }

  .form {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }
  .field {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .field > span:first-child {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    font: 500 12px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.12em;
    color: var(--ivory-3);
  }
  .field-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
  }
  .text-input,
  .date-input,
  .select-input {
    padding: 10px 14px;
    background: rgba(0, 0, 0, 0.3);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    color: var(--ivory);
    font: 400 14px/1.2 var(--font-ui);
    transition: border-color var(--dur-base) var(--ease);
  }
  .text-input:focus,
  .date-input:focus,
  .select-input:focus {
    outline: 2px solid var(--lime);
    outline-offset: 2px;
    border-color: rgba(197, 240, 74, 0.4);
  }
  .text-input:disabled,
  .date-input:disabled,
  .select-input:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .text-input::placeholder {
    color: var(--ivory-4);
  }
  .date-input {
    font-family: var(--font-mono);
    color-scheme: dark;
  }
  .select-input {
    cursor: pointer;
  }

  .field-err {
    margin: -6px 0 0;
    padding: 6px 10px;
    background: rgba(240, 108, 90, 0.08);
    border: 1px solid rgba(240, 108, 90, 0.3);
    border-radius: var(--radius-sm);
    font: 400 12px/1.4 var(--font-ui);
    color: var(--coral);
  }

  .form-err {
    display: flex;
    gap: 8px;
    align-items: flex-start;
    padding: 10px 14px;
    background: rgba(240, 108, 90, 0.08);
    border: 1px solid rgba(240, 108, 90, 0.3);
    border-radius: var(--radius-md);
  }
  .err-ico {
    color: var(--coral);
    flex-shrink: 0;
    padding-top: 2px;
  }
  .form-err > div {
    display: flex;
    flex-direction: column;
    gap: 4px;
    font: 400 12px/1.4 var(--font-ui);
    color: var(--ivory);
  }
  .form-err strong {
    color: var(--coral);
    font-weight: 600;
  }
  .form-err .help {
    color: var(--ivory-3);
    font-style: italic;
    font-size: 12px;
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 12px;
    margin-top: 6px;
  }
  .btn-primary {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    height: 44px;
    padding: 0 18px;
    background: var(--lime);
    color: var(--ink);
    border: none;
    border-radius: var(--radius-md);
    font: 600 13px/1 var(--font-ui);
    cursor: pointer;
    transition: all var(--dur-base) var(--ease);
    box-shadow:
      0 0 0 0 var(--lime-glow),
      0 6px 24px -8px rgba(197, 240, 74, 0.5);
  }
  .btn-primary:hover:not(:disabled) {
    transform: translateY(-1px);
    box-shadow:
      0 0 0 4px rgba(197, 240, 74, 0.15),
      0 8px 32px -8px rgba(197, 240, 74, 0.7);
  }
  .btn-primary:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }
  .btn-primary :global(svg.spin) {
    animation: spin 1s linear infinite;
  }
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .btn-ghost {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 44px;
    padding: 0 14px;
    background: transparent;
    color: var(--ivory-2);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    font: 500 13px/1 var(--font-ui);
    cursor: pointer;
    transition: all var(--dur-base) var(--ease);
  }
  .btn-ghost:hover:not(:disabled) {
    border-color: var(--border-hi);
    color: var(--ivory);
  }
  .btn-ghost:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .kbd-hint {
    margin-left: auto;
    display: inline-flex;
    align-items: center;
    gap: 5px;
    font: 400 12px/1 var(--font-mono);
    color: var(--ivory-4);
  }

  /* Success card */
  .success {
    padding: 28px 32px;
    background: linear-gradient(160deg, rgba(197, 240, 74, 0.05), rgba(255, 255, 255, 0.005));
    border: 1px solid rgba(197, 240, 74, 0.25);
    border-radius: var(--radius-xl);
    animation: rise 350ms var(--ease) backwards;
  }
  @keyframes rise {
    from {
      opacity: 0;
      transform: translateY(10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
  .sh {
    display: flex;
    align-items: flex-start;
    gap: 12px;
    margin-bottom: 18px;
    padding-bottom: 14px;
    border-bottom: 1px solid var(--border);
  }
  .sh .ico {
    display: inline-grid;
    place-items: center;
    width: 32px;
    height: 32px;
    background: var(--lime-soft);
    border: 1px solid rgba(197, 240, 74, 0.35);
    border-radius: 8px;
    color: var(--lime);
    flex-shrink: 0;
  }
  .sh h3 {
    font: 400 22px/1.15 var(--font-display);
    font-style: italic;
    color: var(--ivory);
    letter-spacing: -0.01em;
    margin: 0;
  }
  .sh .eyebrow {
    font: 500 12px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.14em;
    color: var(--lime);
    margin-bottom: 4px;
  }
  .reset-btn {
    margin-left: auto;
    padding: 6px 12px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 999px;
    font: 500 12px/1 var(--font-ui);
    color: var(--ivory-2);
    cursor: pointer;
  }
  .reset-btn:hover {
    border-color: var(--border-hi);
    color: var(--ivory);
  }

  .stats {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
    gap: 10px;
    margin-bottom: 18px;
  }
  .stat {
    padding: 12px 14px;
    background: rgba(0, 0, 0, 0.22);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
  }
  .stat-l {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    font: 500 12px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.12em;
    color: var(--ivory-3);
    margin-bottom: 6px;
  }
  .stat-v {
    font: 400 26px/1 var(--font-display);
    font-style: italic;
    color: var(--ivory);
    letter-spacing: -0.01em;
  }
  .stat-v .u {
    font: 400 12px/1 var(--font-ui);
    font-style: normal;
    color: var(--ivory-3);
    margin-left: 5px;
  }
  .stat-r {
    margin-top: 4px;
    font: 400 12px/1 var(--font-mono);
    color: var(--ivory-3);
  }

  .files {
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin-bottom: 16px;
  }
  .file-row {
    display: grid;
    grid-template-columns: 14px 1fr auto;
    gap: 10px;
    align-items: center;
    padding: 10px 14px;
    background: rgba(0, 0, 0, 0.25);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
  }
  .file-row :global(svg:first-child) {
    color: var(--lime);
  }
  .fr-main {
    min-width: 0;
  }
  .fr-label {
    font: 500 12px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.12em;
    color: var(--ivory-3);
    margin-bottom: 4px;
  }
  .fr-path {
    font: 400 12px/1.3 var(--font-mono);
    color: var(--ivory);
    word-break: break-all;
    user-select: all;
  }

  .sha-row {
    display: grid;
    grid-template-columns: auto 1fr auto;
    gap: 10px;
    align-items: center;
    padding: 10px 14px;
    background: rgba(0, 0, 0, 0.25);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
  }
  .sha-label {
    font: 500 12px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.12em;
    color: var(--ivory-3);
  }
  .sha-val {
    font: 400 12px/1.3 var(--font-mono);
    color: var(--lime);
    word-break: break-all;
    user-select: all;
    background: transparent;
    border: none;
    padding: 0;
  }

  .copy-btn {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 5px 9px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 999px;
    font: 500 12px/1 var(--font-mono);
    color: var(--ivory-2);
    cursor: pointer;
    transition: all var(--dur-base) var(--ease);
  }
  .copy-btn:hover {
    border-color: var(--border-hi);
    color: var(--ivory);
  }

  .sfoot {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 14px;
    padding-top: 12px;
    border-top: 1px solid var(--border);
  }
  .meth-link {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font: 500 12px/1 var(--font-ui);
    color: var(--lime);
    text-decoration: none;
    border-bottom: 1px dashed rgba(197, 240, 74, 0.3);
    padding-bottom: 1px;
  }
  .meth-link:hover {
    color: var(--ivory);
  }
  .hint {
    font: 400 12px/1.4 var(--font-ui);
    color: var(--ivory-3);
    font-style: italic;
  }

  .mono {
    font-family: var(--font-mono);
  }

  @media (max-width: 960px) {
    .canvas-inner {
      padding: 24px 16px 60px;
    }
    .hero-h1 {
      font-size: 32px;
    }
    .field-row {
      grid-template-columns: 1fr;
    }
  }
</style>
