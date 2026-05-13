<script lang="ts">
  // Module M25 — Eco-budget personnel (C19).
  // Consomme `list_personal_goals`, `set_personal_goal`, `delete_personal_goal`,
  // `get_budget_status` exposés par sobria-app.
  // Contrat no-mock : hors Tauri, formulaire désactivé + bannière.
  //
  // Voir :
  //   - briefs/chantiers/C19-dashboard-eco-budget.md §2 M25
  //   - briefs/chantiers/C19-PROMPTS-CLAUDE-CODE-M15-M25.md
  //   - crates/sobria-app/src/dto.rs (bloc "dashboard + eco-budget")

  import {
    AlertTriangle,
    PlugZap,
    HelpCircle,
    Lock,
    Target,
    Trash2,
    Save,
    Info,
    Leaf,
    Zap,
    Droplet,
    Loader2
  } from '@lucide/svelte';
  import {
    isTauriContext,
    setPersonalGoal,
    deletePersonalGoal,
    getBudgetStatus,
    SobriaIpcError,
    type PersonalGoalDto,
    type BudgetStatusDto,
    type GoalIndicator,
    type GoalPeriod,
    type GoalUnit,
    type BudgetStatusLevel,
    type IpcErrorCode
  } from '$lib/api';
  import { preferences, type ModuleId } from '$lib/preferences';

  const MODULE_ID: ModuleId = 'm25';

  // Module gating (cf. ADR-0010)
  $effect(() => {
    if ($preferences.loaded && !$preferences.enabled_modules.includes(MODULE_ID)) {
      window.location.replace('/?disabled=' + MODULE_ID);
    }
  });

  // ─── Référentiel indicateur ↔ unité ─────────────────────────────────────
  // Source de vérité côté Rust : `goals_store::Goal::expected_unit()`. Toute
  // dérive ici cassera la validation IPC (rejet `invalid_request`).

  const UNIT_OF: Record<GoalIndicator, GoalUnit> = {
    co2eq: 'gCO2eq',
    energy: 'Wh',
    water: 'L'
  };

  const INDICATOR_LABEL: Record<GoalIndicator, string> = {
    co2eq: 'CO₂eq',
    energy: 'Énergie',
    water: 'Eau'
  };

  const PERIOD_LABEL: Record<GoalPeriod, string> = {
    daily: 'Quotidien',
    weekly: 'Hebdomadaire',
    monthly: 'Mensuel'
  };

  // Pour les badges/textes : on affiche "gCO2eq" tel quel (alignement IPC).
  function unitDisplay(u: GoalUnit): string {
    return u === 'gCO2eq' ? 'gCO₂eq' : u;
  }

  // ─── State ──────────────────────────────────────────────────────────────

  let indicator = $state<GoalIndicator>('co2eq');
  let period = $state<GoalPeriod>('monthly');
  let valueMaxStr = $state('');

  let statuses = $state<BudgetStatusDto[]>([]);
  let loading = $state(true);
  let saving = $state(false);
  let deletingKey = $state<string | null>(null);
  let formError = $state<{ code: IpcErrorCode; message: string } | null>(null);
  let loadError = $state<{ code: IpcErrorCode; message: string } | null>(null);

  const tauriAvailable = $derived(isTauriContext());

  const unitForCurrent = $derived(UNIT_OF[indicator]);
  const valueMaxNum = $derived(Number(valueMaxStr.replace(',', '.')));
  const valueMaxValid = $derived(Number.isFinite(valueMaxNum) && valueMaxNum > 0);
  const formValid = $derived(valueMaxStr.trim() !== '' && valueMaxValid);

  // ─── Charge initiale ────────────────────────────────────────────────────

  async function reload(): Promise<void> {
    if (!tauriAvailable) {
      loading = false;
      return;
    }
    loading = true;
    loadError = null;
    try {
      const data = await getBudgetStatus();
      statuses = data;
    } catch (err) {
      if (err instanceof SobriaIpcError) {
        loadError = { code: err.code, message: err.message };
      } else {
        loadError = { code: 'internal', message: 'Échec du chargement des objectifs' };
      }
      statuses = [];
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    // Le store de préférences est nécessaire au gating ; on déclenche la
    // charge IPC dès que le contexte Tauri est disponible.
    if (tauriAvailable) {
      void reload();
    } else {
      loading = false;
    }
  });

  // ─── Soumission ─────────────────────────────────────────────────────────

  async function onSubmit(): Promise<void> {
    if (!tauriAvailable || !formValid || saving) return;
    saving = true;
    formError = null;
    try {
      const goal: PersonalGoalDto = {
        indicator,
        period,
        value_max: valueMaxNum,
        unit: unitForCurrent
      };
      await setPersonalGoal(goal);
      valueMaxStr = '';
      await reload();
    } catch (err) {
      if (err instanceof SobriaIpcError) {
        formError = { code: err.code, message: err.message };
      } else {
        formError = { code: 'internal', message: "Échec de l'enregistrement" };
      }
    } finally {
      saving = false;
    }
  }

  // ─── Suppression ────────────────────────────────────────────────────────

  function keyOf(g: PersonalGoalDto): string {
    return `${g.indicator}|${g.period}`;
  }

  async function onDelete(g: PersonalGoalDto): Promise<void> {
    if (!tauriAvailable) return;
    const label = `${INDICATOR_LABEL[g.indicator]} / ${PERIOD_LABEL[g.period]}`;
    const ok = window.confirm(`Supprimer l'objectif « ${label} » ?`);
    if (!ok) return;
    deletingKey = keyOf(g);
    try {
      await deletePersonalGoal(g.indicator, g.period);
      await reload();
    } catch (err) {
      if (err instanceof SobriaIpcError) {
        formError = { code: err.code, message: err.message };
      } else {
        formError = { code: 'internal', message: 'Échec de la suppression' };
      }
    } finally {
      deletingKey = null;
    }
  }

  // ─── Édition par clic sur une barre ─────────────────────────────────────

  function onEdit(g: PersonalGoalDto): void {
    indicator = g.indicator;
    period = g.period;
    valueMaxStr = String(g.value_max);
    formError = null;
    // Scroll doux vers le formulaire si on est sur petit écran.
    if (typeof document !== 'undefined') {
      document.getElementById('m25-form')?.scrollIntoView({ behavior: 'smooth', block: 'start' });
    }
  }

  // ─── Formatage ──────────────────────────────────────────────────────────

  function fmt(n: number, digits = 1): string {
    if (!Number.isFinite(n)) return '—';
    return new Intl.NumberFormat('fr-FR', { maximumFractionDigits: digits }).format(n);
  }

  function fmtPct(n: number): string {
    if (!Number.isFinite(n)) return '—';
    // Saturation visuelle au-delà : on affiche la vraie valeur.
    return `${new Intl.NumberFormat('fr-FR', { maximumFractionDigits: 0 }).format(n)}%`;
  }

  function fmtDate(iso: string): string {
    // RFC 3339 → DD/MM/YYYY
    const m = /^(\d{4})-(\d{2})-(\d{2})/.exec(iso);
    if (!m) return iso;
    return `${m[3]}/${m[2]}/${m[1]}`;
  }

  function progressClamped(pct: number): number {
    if (!Number.isFinite(pct) || pct < 0) return 0;
    return Math.min(pct, 100);
  }

  function indicatorIcon(i: GoalIndicator): typeof Leaf {
    if (i === 'co2eq') return Leaf;
    if (i === 'energy') return Zap;
    return Droplet;
  }

  function statusToneVar(s: BudgetStatusLevel): string {
    if (s === 'ok') return 'var(--lime)';
    if (s === 'warning') return 'var(--amber, #f5b769)';
    return 'var(--coral)';
  }

  function statusLabel(s: BudgetStatusLevel): string {
    if (s === 'ok') return 'Dans le budget';
    if (s === 'warning') return 'Attention';
    return 'Dépassé';
  }

  // ─── Erreurs ────────────────────────────────────────────────────────────

  const ERROR_LABELS: Record<string, string> = {
    tauri_unavailable: 'Application non lancée via Tauri',
    invalid_request: 'Paramètres invalides',
    internal: 'Erreur interne',
    io_error: 'Erreur disque',
    not_found: 'Introuvable'
  };

  function errorLabel(code: string): string {
    return ERROR_LABELS[code] ?? 'Erreur';
  }

  function errorHelp(code: string): string {
    switch (code) {
      case 'invalid_request':
        return "Vérifie que la valeur cible est strictement positive et que l'indicateur correspond bien à l'unité (CO₂eq → gCO2eq, Énergie → Wh, Eau → L).";
      case 'tauri_unavailable':
        return "L'écran s'ouvre uniquement via `cargo run -p sobria-app`. En navigateur seul, l'IPC est indisponible.";
      default:
        return '';
    }
  }
</script>

<svelte:head>
  <title>Sobr.ia · Objectifs & habitudes</title>
</svelte:head>

<div class="canvas-inner">
  <!-- TopBar -->
  <div class="topbar">
    <nav class="breadcrumb" aria-label="Fil d'Ariane">
      Atelier <span class="sep">/</span>
      <span class="current">Objectifs & habitudes</span>
    </nav>
    <div class="spacer"></div>
    <span class="local-pill">
      <Lock size={12} strokeWidth={1.8} />
      Suivi 100 % local
    </span>
    <a class="icon-btn" href="/methodo" aria-label="Méthodologie">
      <HelpCircle size={16} strokeWidth={1.6} />
    </a>
  </div>

  <!-- Hero -->
  <section class="hero">
    <div class="hero-eyebrow">
      <span class="pulse" aria-hidden="true"></span>
      Module M25 · Eco-budget personnel
    </div>
    <h1 class="hero-h1">
      Pose un <em>budget</em>, suis l'impact.
    </h1>
    <p class="hero-sub">
      Définis tes propres seuils mensuels, hebdomadaires ou journaliers — sur le CO₂eq, l'énergie ou
      l'eau. Sobr.ia mesure ta consommation réelle (P50 Monte-Carlo) et t'alerte avant que tu n'aies
      dépassé.
    </p>
  </section>

  <!-- Bannière hors-Tauri -->
  {#if !tauriAvailable}
    <div class="banner" data-tone="warn" role="alert">
      <span class="banner-ico" aria-hidden="true"
        ><AlertTriangle size={18} strokeWidth={1.8} /></span
      >
      <div class="banner-body">
        <strong>Application non lancée via Tauri</strong>
        <span>
          L'application doit être lancée via <span class="mono">cargo run -p sobria-app</span> (ou
          <span class="mono">cargo tauri dev</span>). Les objectifs sont stockés dans la base SQLite
          locale — pas de serveur, pas d'envoi externe.
        </span>
      </div>
    </div>
  {/if}

  <!-- Layout 2 colonnes : formulaire | liste objectifs -->
  <div class="grid">
    <!-- ── Colonne gauche : formulaire ───────────────────────────────────── -->
    <section id="m25-form" class="form-card" aria-label="Formulaire objectif">
      <header class="fh">
        <div class="eyebrow">
          <Target size={11} strokeWidth={1.8} /> Nouveau / éditer
        </div>
        <h2>Définir un objectif</h2>
      </header>

      <form
        class="form"
        onsubmit={(e) => {
          e.preventDefault();
          void onSubmit();
        }}
        novalidate
      >
        <label class="field">
          <span id="lbl-indicator">Indicateur</span>
          <select
            bind:value={indicator}
            disabled={!tauriAvailable || saving}
            class="select-input"
            aria-labelledby="lbl-indicator"
          >
            <option value="co2eq">CO₂eq</option>
            <option value="energy">Énergie</option>
            <option value="water">Eau</option>
          </select>
        </label>

        <label class="field">
          <span id="lbl-period">Période</span>
          <select
            bind:value={period}
            disabled={!tauriAvailable || saving}
            class="select-input"
            aria-labelledby="lbl-period"
          >
            <option value="daily">Quotidien</option>
            <option value="weekly">Hebdomadaire</option>
            <option value="monthly">Mensuel</option>
          </select>
        </label>

        <label class="field">
          <span id="lbl-value">Valeur cible</span>
          <div class="value-row">
            <input
              type="number"
              inputmode="decimal"
              min="0"
              step="any"
              bind:value={valueMaxStr}
              disabled={!tauriAvailable || saving}
              required
              placeholder="Ex. 500"
              class="text-input"
              aria-labelledby="lbl-value"
              aria-invalid={valueMaxStr.trim() !== '' && !valueMaxValid}
            />
            <span class="unit-badge mono" aria-label="Unité">{unitDisplay(unitForCurrent)}</span>
          </div>
          {#if valueMaxStr.trim() !== '' && !valueMaxValid}
            <p class="field-err">La valeur cible doit être strictement positive.</p>
          {/if}
        </label>

        {#if formError}
          <div class="form-err" role="alert">
            <span class="err-ico"><PlugZap size={14} strokeWidth={1.8} /></span>
            <div>
              <strong>{errorLabel(formError.code)}</strong>
              <span>{formError.message}</span>
              {#if errorHelp(formError.code)}
                <span class="help">{errorHelp(formError.code)}</span>
              {/if}
            </div>
          </div>
        {/if}

        <button
          type="submit"
          class="btn-primary"
          disabled={!tauriAvailable || saving || !formValid}
          aria-busy={saving}
        >
          {#if saving}
            <Loader2 size={14} strokeWidth={2} class="spin" /> Enregistrement…
          {:else}
            <Save size={14} strokeWidth={2} /> Enregistrer l'objectif
          {/if}
        </button>

        <p class="meth-note">
          <Info size={11} strokeWidth={1.8} />
          <span>
            Les périodes <strong>hebdomadaires</strong> suivent ISO 8601 (lundi 00:00 → dimanche
            23:59). Les valeurs sont calculées en <strong>P50</strong> (médiane Monte-Carlo) — voir
            <a href="/methodo">Méthodologie</a> pour le détail.
          </span>
        </p>
      </form>
    </section>

    <!-- ── Colonne droite : liste des budgets ───────────────────────────── -->
    <section class="list-card" aria-label="Objectifs actifs et statut budget">
      <header class="lh">
        <div class="eyebrow">
          <Target size={11} strokeWidth={1.8} /> Suivi en temps réel
        </div>
        <h2>Objectifs actifs</h2>
      </header>

      {#if loading}
        <div class="state-loading" aria-live="polite">
          <Loader2 size={16} strokeWidth={2} class="spin" />
          <span>Chargement des objectifs…</span>
        </div>
      {:else if loadError}
        <div class="form-err" role="alert">
          <span class="err-ico"><PlugZap size={14} strokeWidth={1.8} /></span>
          <div>
            <strong>{errorLabel(loadError.code)}</strong>
            <span>{loadError.message}</span>
          </div>
        </div>
      {:else if statuses.length === 0}
        <!-- État vide -->
        <div class="empty">
          <div class="empty-ico" aria-hidden="true">
            <Target size={28} strokeWidth={1.4} />
          </div>
          <p class="empty-text">
            <strong>Aucun objectif défini.</strong>
            Commencez par définir un budget mensuel CO₂eq pour suivre votre empreinte IA.
          </p>
        </div>
      {:else}
        <ul class="budget-list">
          {#each statuses as st (keyOf(st.goal))}
            {@const ind = st.goal.indicator}
            {@const Icon = indicatorIcon(ind)}
            {@const tone = statusToneVar(st.status)}
            {@const pct = progressClamped(st.consumed_pct)}
            {@const unit = unitDisplay(st.goal.unit)}
            <li class="budget-item" data-status={st.status}>
              <div class="bi-head">
                <span class="bi-ico" style="color: {tone}">
                  <Icon size={14} strokeWidth={1.8} />
                </span>
                <div class="bi-title">
                  <span class="bi-name">
                    {INDICATOR_LABEL[ind]} / {PERIOD_LABEL[st.goal.period]}
                  </span>
                  <span class="bi-period mono">
                    {fmtDate(st.period_start)} → {fmtDate(st.period_end)}
                  </span>
                </div>
                <button
                  type="button"
                  class="trash-btn"
                  onclick={() => onDelete(st.goal)}
                  disabled={!tauriAvailable || deletingKey === keyOf(st.goal)}
                  aria-label="Supprimer objectif {INDICATOR_LABEL[ind]} {PERIOD_LABEL[
                    st.goal.period
                  ]}"
                >
                  {#if deletingKey === keyOf(st.goal)}
                    <Loader2 size={13} strokeWidth={2} class="spin" />
                  {:else}
                    <Trash2 size={13} strokeWidth={1.8} />
                  {/if}
                </button>
              </div>

              <div class="bi-progress-line">
                <span class="bi-progress-text">
                  <strong>{fmt(st.current_value, 1)}</strong>
                  <span class="muted">/ {fmt(st.goal.value_max, 1)} {unit}</span>
                </span>
                <span class="bi-pct mono" style="color: {tone}">{fmtPct(st.consumed_pct)}</span>
              </div>

              <button
                type="button"
                class="bi-bar"
                onclick={() => onEdit(st.goal)}
                disabled={!tauriAvailable}
                aria-label="Modifier objectif {INDICATOR_LABEL[ind]} {PERIOD_LABEL[st.goal.period]}"
                title="Cliquer pour modifier"
              >
                <span
                  class="bi-bar-fill"
                  role="progressbar"
                  aria-valuenow={Math.round(st.consumed_pct)}
                  aria-valuemin="0"
                  aria-valuemax="100"
                  aria-label="{INDICATOR_LABEL[ind]} {PERIOD_LABEL[st.goal.period]} : {fmtPct(
                    st.consumed_pct
                  )} consommé"
                  style="width: {pct}%; background: {tone};"
                ></span>
              </button>

              {#if st.status === 'exceeded'}
                <p class="bi-badge bi-badge-exceeded" aria-live="polite">
                  Dépassé de {fmt(Math.abs(st.remaining), 1)}
                  {unit}
                </p>
              {:else if st.status === 'warning'}
                <p class="bi-badge bi-badge-warning" aria-live="polite">
                  Plus que {fmt(Math.max(st.remaining, 0), 1)}
                  {unit} restant
                  <span class="muted">· {statusLabel(st.status)}</span>
                </p>
              {/if}
            </li>
          {/each}
        </ul>
      {/if}
    </section>
  </div>
</div>

<style>
  .canvas-inner {
    max-width: 1180px;
    margin: 0 auto;
    padding: 40px 56px 80px;
    display: flex;
    flex-direction: column;
    gap: 24px;
  }

  /* ── TopBar ─────────────────────────────────────────────────────────── */
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
    cursor: pointer;
    text-decoration: none;
    transition: all var(--dur-base) var(--ease);
  }
  .icon-btn:hover {
    background: var(--surface-hi);
    border-color: var(--border-hi);
    color: var(--ivory);
  }

  /* ── Hero ───────────────────────────────────────────────────────────── */
  .hero {
    padding-bottom: 14px;
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

  /* ── Bannière warn ──────────────────────────────────────────────────── */
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

  /* ── Grid 2 colonnes (1/3 form, 2/3 list) ───────────────────────────── */
  .grid {
    display: grid;
    grid-template-columns: minmax(280px, 1fr) minmax(0, 2fr);
    gap: 22px;
    align-items: start;
  }

  /* ── Form card ──────────────────────────────────────────────────────── */
  .form-card,
  .list-card {
    padding: 24px 26px;
    background: linear-gradient(180deg, rgba(255, 255, 255, 0.025), rgba(255, 255, 255, 0.005));
    border: 1px solid var(--border);
    border-radius: var(--radius-xl);
  }
  .fh,
  .lh {
    margin-bottom: 16px;
    padding-bottom: 12px;
    border-bottom: 1px solid var(--border);
  }
  .fh .eyebrow,
  .lh .eyebrow {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    font: 500 10px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.14em;
    color: var(--ivory-3);
    margin-bottom: 6px;
  }
  .fh h2,
  .lh h2 {
    font: 400 22px/1.15 var(--font-display);
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
    font: 500 10px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.12em;
    color: var(--ivory-3);
  }
  .text-input,
  .select-input {
    padding: 10px 14px;
    background: rgba(0, 0, 0, 0.3);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    color: var(--ivory);
    font: 400 14px/1.2 var(--font-ui);
    transition: border-color var(--dur-base) var(--ease);
    width: 100%;
  }
  .text-input:focus,
  .select-input:focus {
    outline: 2px solid var(--lime);
    outline-offset: 2px;
    border-color: rgba(197, 240, 74, 0.4);
  }
  .text-input:disabled,
  .select-input:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .text-input::placeholder {
    color: var(--ivory-4);
  }
  .select-input {
    cursor: pointer;
  }

  .value-row {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 8px;
    align-items: stretch;
  }
  .unit-badge {
    display: inline-grid;
    place-items: center;
    padding: 0 12px;
    background: rgba(126, 182, 255, 0.08);
    border: 1px solid rgba(126, 182, 255, 0.25);
    border-radius: var(--radius-md);
    color: var(--ivory);
    font: 500 12px/1 var(--font-mono);
    min-width: 60px;
    text-align: center;
    user-select: none;
  }

  .field-err {
    margin: 0;
    padding: 6px 10px;
    background: rgba(240, 108, 90, 0.08);
    border: 1px solid rgba(240, 108, 90, 0.3);
    border-radius: var(--radius-sm);
    font: 400 11px/1.4 var(--font-ui);
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
    font-size: 11px;
  }

  .btn-primary {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    height: 42px;
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
  .btn-primary :global(svg.spin),
  .trash-btn :global(svg.spin),
  .state-loading :global(svg.spin) {
    animation: spin 1s linear infinite;
  }
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .meth-note {
    display: flex;
    gap: 8px;
    align-items: flex-start;
    margin: 4px 0 0;
    padding: 10px 12px;
    background: rgba(126, 182, 255, 0.06);
    border: 1px dashed rgba(126, 182, 255, 0.22);
    border-radius: var(--radius-sm);
    font: 400 11px/1.5 var(--font-ui);
    color: var(--ivory-2);
  }
  .meth-note :global(svg) {
    color: rgba(126, 182, 255, 0.9);
    flex-shrink: 0;
    margin-top: 2px;
  }
  .meth-note a {
    color: var(--lime);
    text-decoration: none;
    border-bottom: 1px dashed rgba(197, 240, 74, 0.3);
  }
  .meth-note a:hover {
    color: var(--ivory);
  }
  .meth-note strong {
    color: var(--ivory);
    font-weight: 600;
  }

  /* ── List card ──────────────────────────────────────────────────────── */
  .state-loading {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 18px 16px;
    color: var(--ivory-3);
    font: 400 13px/1 var(--font-ui);
  }

  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    gap: 14px;
    padding: 36px 20px;
    border: 1px dashed var(--border-hi);
    border-radius: var(--radius-lg);
    background: rgba(0, 0, 0, 0.18);
  }
  .empty-ico {
    width: 56px;
    height: 56px;
    display: grid;
    place-items: center;
    border-radius: 50%;
    background: var(--lime-soft);
    border: 1px solid rgba(197, 240, 74, 0.25);
    color: var(--lime);
  }
  .empty-text {
    margin: 0;
    max-width: 360px;
    font: 400 13px/1.55 var(--font-ui);
    color: var(--ivory-2);
  }
  .empty-text strong {
    display: block;
    color: var(--ivory);
    font-weight: 600;
    margin-bottom: 4px;
  }

  .budget-list {
    display: flex;
    flex-direction: column;
    gap: 14px;
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .budget-item {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 14px 16px;
    background: rgba(0, 0, 0, 0.22);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    transition: border-color var(--dur-base) var(--ease);
  }
  .budget-item:hover {
    border-color: var(--border-hi);
  }
  .budget-item[data-status='exceeded'] {
    background: rgba(240, 108, 90, 0.06);
    border-color: rgba(240, 108, 90, 0.25);
  }
  .budget-item[data-status='warning'] {
    background: rgba(245, 183, 105, 0.05);
    border-color: rgba(245, 183, 105, 0.22);
  }

  .bi-head {
    display: grid;
    grid-template-columns: auto 1fr auto;
    gap: 10px;
    align-items: center;
  }
  .bi-ico {
    display: inline-grid;
    place-items: center;
    width: 28px;
    height: 28px;
    border-radius: 8px;
    background: rgba(255, 255, 255, 0.04);
    border: 1px solid var(--border);
  }
  .bi-title {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .bi-name {
    font: 600 13px/1.2 var(--font-ui);
    color: var(--ivory);
  }
  .bi-period {
    font: 400 11px/1.2 var(--font-mono);
    color: var(--ivory-3);
  }
  .trash-btn {
    display: inline-grid;
    place-items: center;
    width: 30px;
    height: 30px;
    background: transparent;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    color: var(--ivory-3);
    cursor: pointer;
    transition: all var(--dur-base) var(--ease);
  }
  .trash-btn:hover:not(:disabled) {
    color: var(--coral);
    border-color: rgba(240, 108, 90, 0.4);
    background: rgba(240, 108, 90, 0.08);
  }
  .trash-btn:disabled {
    opacity: 0.35;
    cursor: not-allowed;
  }

  .bi-progress-line {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 8px;
    font: 400 12px/1.3 var(--font-ui);
    color: var(--ivory-2);
  }
  .bi-progress-text strong {
    color: var(--ivory);
    font-weight: 600;
  }
  .bi-progress-text .muted,
  .bi-badge .muted {
    color: var(--ivory-3);
  }
  .bi-pct {
    font: 600 13px/1 var(--font-mono);
  }

  .bi-bar {
    position: relative;
    width: 100%;
    height: 10px;
    padding: 0;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid var(--border);
    border-radius: 999px;
    overflow: hidden;
    cursor: pointer;
    transition: box-shadow var(--dur-base) var(--ease);
  }
  .bi-bar:hover:not(:disabled) {
    box-shadow: 0 0 0 3px rgba(197, 240, 74, 0.18);
  }
  .bi-bar:focus-visible {
    outline: 2px solid var(--lime);
    outline-offset: 2px;
  }
  .bi-bar:disabled {
    cursor: default;
  }
  .bi-bar-fill {
    display: block;
    height: 100%;
    border-radius: 999px;
    transition: width var(--dur-base) var(--ease);
  }

  .bi-badge {
    margin: 4px 0 0;
    padding: 6px 10px;
    border-radius: var(--radius-sm);
    font: 500 11px/1.3 var(--font-ui);
    align-self: flex-start;
  }
  .bi-badge-exceeded {
    background: rgba(240, 108, 90, 0.12);
    border: 1px solid rgba(240, 108, 90, 0.35);
    color: var(--coral);
  }
  .bi-badge-warning {
    background: rgba(245, 183, 105, 0.12);
    border: 1px solid rgba(245, 183, 105, 0.35);
    color: var(--amber, #f5b769);
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
    .grid {
      grid-template-columns: 1fr;
    }
  }
</style>
