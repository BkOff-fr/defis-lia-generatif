<script lang="ts">
  import {
    ShieldCheck,
    ShieldAlert,
    Download,
    ChevronLeft,
    ChevronRight,
    X,
    Hash,
    HelpCircle,
    Info,
    AlertTriangle,
    PlugZap,
    Trash2,
    Lock,
    RotateCcw
  } from '@lucide/svelte';
  import {
    isTauriContext,
    listAuditEntries,
    verifyAudit,
    exportAuditNdjson,
    SobriaIpcError,
    type AuditEntrySummaryDto,
    type IntegrityReportDto,
    type IpcErrorCode
  } from '$lib/api';
  import { tick } from 'svelte';

  // ─── State ───────────────────────────────────────────────────────────
  const PAGE_SIZE = 50;

  let entries = $state<AuditEntrySummaryDto[]>([]);
  let offset = $state(0);
  let loading = $state(false);
  let bootstrapping = $state(true);
  let error = $state<{ code: IpcErrorCode; message: string } | null>(null);

  let verdict = $state<IntegrityReportDto | null>(null);
  let verdictLoading = $state(false);

  let selected = $state<AuditEntrySummaryDto | null>(null);
  let focusId = $state<number | null>(null);

  let toast = $state<{ msg: string; tone: 'success' | 'error' } | null>(null);
  let toastTimer: ReturnType<typeof setTimeout> | undefined;

  // Indices factices pour les rangées de skeleton (8 lignes vides pendant le
  // chargement initial). Doit être une constante stable pour que la `key` du
  // `{#each}` ne change pas entre rendus.
  const SKEL_ROWS = [0, 1, 2, 3, 4, 5, 6, 7] as const;

  const tauriAvailable = $derived(isTauriContext());
  const canPrev = $derived(offset > 0);
  const canNext = $derived(entries.length === PAGE_SIZE);
  const currentPage = $derived(Math.floor(offset / PAGE_SIZE) + 1);

  // ─── Bootstrap + URL focus ─────────────────────────────────────────────
  $effect(() => {
    void (async () => {
      if (!tauriAvailable) {
        bootstrapping = false;
        error = {
          code: 'tauri_unavailable',
          message:
            "L'application doit être lancée via `cargo run -p sobria-app`. Le ledger d'audit local n'est pas accessible dans un navigateur seul."
        };
        return;
      }

      // ?focus=N — on lit l'URL une seule fois au mount.
      const focus = readFocusFromUrl();
      if (focus !== null) {
        focusId = focus;
      }

      await loadPage(0);
      bootstrapping = false;

      // Si le focus est dans la page courante, on ouvre le drawer + scroll.
      if (focusId !== null) {
        await openFocusedEntry(focusId);
      }
    })();
  });

  function readFocusFromUrl(): number | null {
    if (typeof window === 'undefined') return null;
    const params = new URLSearchParams(window.location.search);
    const raw = params.get('focus');
    if (raw === null) return null;
    const n = Number.parseInt(raw, 10);
    return Number.isFinite(n) && n > 0 ? n : null;
  }

  async function loadPage(newOffset: number) {
    loading = true;
    error = null;
    try {
      const list = await listAuditEntries(PAGE_SIZE, newOffset);
      entries = list;
      offset = newOffset;
    } catch (err) {
      if (err instanceof SobriaIpcError) {
        error = { code: err.code, message: err.message };
      } else {
        error = { code: 'internal', message: 'Échec du chargement du ledger' };
      }
    } finally {
      loading = false;
    }
  }

  async function openFocusedEntry(id: number) {
    const found = entries.find((e) => e.id === id);
    if (!found) return;
    selected = found;
    await tick();
    const row = document.querySelector<HTMLElement>(`[data-row-id="${id}"]`);
    row?.scrollIntoView({ behavior: 'smooth', block: 'center' });
  }

  function prev() {
    if (!canPrev) return;
    void loadPage(Math.max(0, offset - PAGE_SIZE));
  }
  function next() {
    if (!canNext) return;
    void loadPage(offset + PAGE_SIZE);
  }

  // ─── Vérifier intégrité ────────────────────────────────────────────────
  async function checkIntegrity() {
    verdictLoading = true;
    try {
      verdict = await verifyAudit();
    } catch (err) {
      verdict = null;
      if (err instanceof SobriaIpcError) {
        showToast(err.message, 'error');
      } else {
        showToast("Échec de la vérification d'intégrité", 'error');
      }
    } finally {
      verdictLoading = false;
    }
  }

  // ─── Exporter NDJSON via save dialog ───────────────────────────────────
  async function exportNdjson() {
    try {
      // Import dynamique du plugin-dialog : ce code ne tourne que côté
      // Tauri, donc on évite le surcoût de bundle quand l'app est servie
      // hors runtime (où ce bouton n'apparaît de toute façon pas).
      const { save } = await import('@tauri-apps/plugin-dialog');
      const path = await save({
        defaultPath: `sobria-audit-${new Date().toISOString().slice(0, 10)}.ndjson`,
        filters: [{ name: 'NDJSON', extensions: ['ndjson', 'jsonl'] }],
        title: "Exporter le journal d'audit"
      });
      if (!path) return;
      const lines = await exportAuditNdjson(path);
      showToast(
        `${lines} entrée${lines > 1 ? 's' : ''} écrite${lines > 1 ? 's' : ''} dans le fichier`,
        'success'
      );
    } catch (err) {
      if (err instanceof SobriaIpcError) {
        showToast(err.message, 'error');
      } else if (err instanceof Error) {
        showToast(err.message, 'error');
      } else {
        showToast("Échec de l'export NDJSON", 'error');
      }
    }
  }

  function showToast(msg: string, tone: 'success' | 'error') {
    toast = { msg, tone };
    if (toastTimer) clearTimeout(toastTimer);
    toastTimer = setTimeout(() => {
      toast = null;
    }, 4000);
  }

  // ─── Drawer ──────────────────────────────────────────────────────────
  function openEntry(e: AuditEntrySummaryDto) {
    selected = e;
  }
  function closeDrawer() {
    selected = null;
  }

  function handleEscape(e: KeyboardEvent) {
    if (e.key === 'Escape' && selected) {
      closeDrawer();
    }
  }

  // ─── Helpers de formatage ────────────────────────────────────────────
  function fmtDate(iso: string): string {
    const d = new Date(iso);
    if (Number.isNaN(d.getTime())) return iso;
    return new Intl.DateTimeFormat('fr-FR', {
      dateStyle: 'short',
      timeStyle: 'medium'
    }).format(d);
  }

  function fmtCo2(value: number): string {
    if (!Number.isFinite(value)) return '—';
    if (value === 0) return '0';
    // Mêmes échelles auto que ResultBlock — synthétique ici (un seul ratio).
    if (Math.abs(value) >= 1) return `${fmtNum(value)} g`;
    if (Math.abs(value) >= 1e-3) return `${fmtNum(value * 1e3)} mg`;
    if (Math.abs(value) >= 1e-6) return `${fmtNum(value * 1e6)} µg`;
    return `${fmtNum(value * 1e9)} ng`;
  }

  function fmtNum(value: number): string {
    return new Intl.NumberFormat('fr-FR', {
      maximumSignificantDigits: 3,
      minimumSignificantDigits: 1
    }).format(value);
  }

  const errorTone = $derived.by<'info' | 'warn' | 'error'>(() => {
    if (!error) return 'info';
    if (error.code === 'tauri_unavailable') return 'warn';
    return 'error';
  });

  const ERROR_LABELS: Record<string, string> = {
    tauri_unavailable: 'Application non lancée via Tauri',
    audit_error: "Erreur du ledger d'audit",
    io_error: 'Erreur disque',
    internal: 'Erreur interne'
  };
  function errorLabel(code: string): string {
    return ERROR_LABELS[code] ?? 'Erreur';
  }
</script>

<svelte:window onkeydown={handleEscape} />

<svelte:head>
  <title>Sobr.ia · Journal d'audit</title>
</svelte:head>

<div class="canvas-inner">
  <!-- ─── TopBar ─────────────────────────────────────────────── -->
  <div class="topbar">
    <nav class="breadcrumb" aria-label="Fil d'Ariane">
      Atelier <span class="sep">/</span>
      <span class="current">Journal d'audit</span>
    </nav>
    <div class="spacer"></div>
    <span class="local-pill" title="Le ledger d'audit n'est jamais envoyé vers un service externe">
      <Lock size={12} strokeWidth={1.8} />
      Ledger 100 % local
    </span>
    <a class="icon-btn" href="/methodo" aria-label="Aide & méthodologie">
      <HelpCircle size={16} strokeWidth={1.6} />
    </a>
  </div>

  <!-- ─── Hero compact ──────────────────────────────────────── -->
  <section class="hero">
    <div class="hero-eyebrow">
      <span class="pulse" aria-hidden="true"></span>
      Module M7 · ledger ACID chaîné SHA-256
    </div>
    <h1 class="hero-h1">
      Toutes vos estimations, <em>vérifiables</em>, hors-ligne.
    </h1>
    <p class="hero-sub">
      Chaque estimation est journalisée avec sa signature et chaînée à la précédente. Le ledger
      reste local — exportable en NDJSON signé pour audit externe, jamais envoyé vers un service
      tiers.
    </p>
  </section>

  <!-- ─── Bannière RGPD discrète ─────────────────────────────── -->
  <div class="rgpd-note">
    <Info size={14} strokeWidth={1.8} />
    Le ledger n'est jamais envoyé sur Internet. La purge RGPD entrée par entrée arrive en chantier C11.
  </div>

  <!-- ─── Bannière erreur ───────────────────────────────────── -->
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

  <!-- ─── Toolbar ─────────────────────────────────────────────── -->
  {#if tauriAvailable}
    <div class="toolbar">
      <button
        class="btn-action lime"
        type="button"
        onclick={checkIntegrity}
        disabled={verdictLoading || bootstrapping}
      >
        {#if verdict?.valid}
          <ShieldCheck size={16} strokeWidth={1.8} />
        {:else if verdict && !verdict.valid}
          <ShieldAlert size={16} strokeWidth={1.8} />
        {:else}
          <ShieldCheck size={16} strokeWidth={1.8} />
        {/if}
        {verdictLoading ? 'Vérification…' : 'Vérifier la chaîne'}
      </button>

      <button class="btn-action" type="button" onclick={exportNdjson} disabled={bootstrapping}>
        <Download size={16} strokeWidth={1.8} />
        Exporter NDJSON
      </button>

      <div class="spacer-flex"></div>

      <button
        class="btn-action ghost"
        type="button"
        onclick={() => loadPage(offset)}
        disabled={loading}
        title="Recharger la page courante"
        aria-label="Recharger"
      >
        <RotateCcw size={14} strokeWidth={1.8} />
      </button>
    </div>

    <!-- ─── Verdict ──────────────────────────────────────────── -->
    {#if verdict}
      <div class="verdict" data-tone={verdict.valid ? 'ok' : 'ko'}>
        <div class="verdict-ico">
          {#if verdict.valid}
            <ShieldCheck size={22} strokeWidth={1.8} />
          {:else}
            <ShieldAlert size={22} strokeWidth={1.8} />
          {/if}
        </div>
        <div class="verdict-body">
          <div class="verdict-title">
            {verdict.valid ? 'Chaîne intègre' : 'Chaîne compromise'}
          </div>
          <div class="verdict-sub">
            {verdict.total_entries} entrée{verdict.total_entries > 1 ? 's' : ''} vérifiée{verdict.total_entries >
            1
              ? 's'
              : ''}
            {#if !verdict.valid && verdict.first_invalid_id !== undefined}
              · première rupture à l'entrée
              <a href={`#row-${verdict.first_invalid_id}`} class="link">
                #{verdict.first_invalid_id}
              </a>
            {/if}
          </div>
          <div class="verdict-msg">{verdict.message}</div>
        </div>
      </div>
    {/if}

    <!-- ─── Table ──────────────────────────────────────────────── -->
    <div class="table-wrap scrollable">
      <table class="ledger-table" aria-label="Entrées du ledger d'audit">
        <thead>
          <tr>
            <th class="th-id">#</th>
            <th class="th-date">Horodatage</th>
            <th class="th-model">Modèle</th>
            <th class="th-num">CO₂eq P50</th>
            <th class="th-sig">Signature</th>
            <th class="th-state">État</th>
          </tr>
        </thead>
        <tbody>
          {#if bootstrapping || (loading && entries.length === 0)}
            {#each SKEL_ROWS as i (i)}
              <tr class="row-skel">
                <td colspan="6"><span class="skel-bar"></span></td>
              </tr>
            {/each}
          {:else if entries.length === 0}
            <tr class="row-empty">
              <td colspan="6">
                Aucune entrée pour cette page.
                {#if offset > 0}<button class="link-btn" onclick={() => loadPage(0)}
                    >Revenir au début</button
                  >{/if}
              </td>
            </tr>
          {:else}
            {#each entries as e (e.id)}
              {@const isInvalid =
                verdict &&
                !verdict.valid &&
                verdict.first_invalid_id !== undefined &&
                e.id >= verdict.first_invalid_id}
              <tr
                class="row"
                class:active={selected?.id === e.id}
                class:focused={focusId === e.id}
                class:invalid={isInvalid}
                data-row-id={e.id}
                id={`row-${e.id}`}
                onclick={() => openEntry(e)}
                tabindex="0"
                onkeydown={(ev) => {
                  if (ev.key === 'Enter' || ev.key === ' ') {
                    ev.preventDefault();
                    openEntry(e);
                  }
                }}
              >
                <td class="td-id">#{e.id}</td>
                <td class="td-date">{fmtDate(e.timestamp)}</td>
                <td class="td-model">{e.model_id}</td>
                <td class="td-num">{fmtCo2(e.co2eq_p50)}</td>
                <td class="td-sig">
                  <span class="sig-wrap">
                    <Hash size={10} strokeWidth={1.8} />
                    <span class="sig-text">{e.sig_short}…</span>
                  </span>
                </td>
                <td class="td-state">
                  {#if e.purged}
                    <span class="badge-purged">
                      <Trash2 size={10} strokeWidth={2} /> purgé
                    </span>
                  {:else}
                    <span class="badge-ok">OK</span>
                  {/if}
                </td>
              </tr>
            {/each}
          {/if}
        </tbody>
      </table>
    </div>

    <!-- ─── Pagination ───────────────────────────────────────── -->
    <div class="pagination">
      <button class="btn-page" type="button" onclick={prev} disabled={!canPrev || loading}>
        <ChevronLeft size={14} strokeWidth={1.8} /> Précédent
      </button>
      <span class="page-info mono">
        Page {currentPage} · entrées {offset + 1}–{offset + entries.length}
      </span>
      <button class="btn-page" type="button" onclick={next} disabled={!canNext || loading}>
        Suivant <ChevronRight size={14} strokeWidth={1.8} />
      </button>
    </div>
  {/if}
</div>

<!-- ─── Drawer ──────────────────────────────────────────────── -->
{#if selected}
  <button class="drawer-backdrop" type="button" aria-label="Fermer le détail" onclick={closeDrawer}
  ></button>
  <div class="drawer" role="dialog" aria-modal="true" aria-labelledby="drawer-title">
    <header class="drawer-head">
      <div>
        <div class="drawer-eye">Entrée du ledger</div>
        <div id="drawer-title" class="drawer-title">#{selected.id}</div>
      </div>
      <button class="icon-btn" type="button" onclick={closeDrawer} aria-label="Fermer">
        <X size={16} strokeWidth={1.8} />
      </button>
    </header>

    <div class="drawer-body scrollable">
      <dl class="drawer-grid">
        <dt>Horodatage</dt>
        <dd>{fmtDate(selected.timestamp)}</dd>

        <dt>Modèle</dt>
        <dd>{selected.model_id}</dd>

        <dt>CO₂eq · P50</dt>
        <dd>{fmtCo2(selected.co2eq_p50)}</dd>

        <dt>Signature SHA-256</dt>
        <dd class="mono break">{selected.sig_short}…</dd>

        <dt>État</dt>
        <dd>
          {#if selected.purged}
            <span class="badge-purged">
              <Trash2 size={10} strokeWidth={2} /> purgé (RGPD)
            </span>
          {:else}
            <span class="badge-ok">intègre</span>
          {/if}
        </dd>
      </dl>

      <p class="drawer-note">
        Le détail complet (10 000 tirages Monte-Carlo + hypothèses) est accessible via le bouton <strong
          >Exporter NDJSON</strong
        > — il sera proposé en consultation directe dans une future itération de l'IPC.
      </p>
    </div>
  </div>
{/if}

<!-- ─── Toast ────────────────────────────────────────────────── -->
{#if toast}
  <div class="toast" data-tone={toast.tone} role="status" aria-live="polite">
    {#if toast.tone === 'success'}
      <ShieldCheck size={16} strokeWidth={1.8} />
    {:else}
      <AlertTriangle size={16} strokeWidth={1.8} />
    {/if}
    <span>{toast.msg}</span>
  </div>
{/if}

<style>
  .canvas-inner {
    max-width: 1240px;
    margin: 0 auto;
    padding: 40px 56px 80px;
  }

  /* ─── TopBar (clone allégé de l'écran Estimer) ───────────── */
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
    cursor: pointer;
    transition: all var(--dur-base) var(--ease);
    text-decoration: none;
  }
  .icon-btn:hover {
    background: var(--surface-hi);
    border-color: var(--border-hi);
    color: var(--ivory);
  }

  /* ─── Hero ─────────────────────────────────────────────────── */
  .hero {
    margin-bottom: 24px;
    padding-bottom: 24px;
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
    margin: 0 0 8px;
  }
  .hero-h1 em {
    font-style: normal;
    color: var(--lime);
  }
  .hero-sub {
    font: 400 15px/1.55 var(--font-ui);
    color: var(--ivory-2);
    max-width: 620px;
    margin: 0;
  }

  /* ─── Bannière RGPD ───────────────────────────────────────── */
  .rgpd-note {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    font: 400 12px/1.4 var(--font-ui);
    color: var(--ivory-3);
    padding: 8px 14px;
    background: rgba(255, 255, 255, 0.015);
    border: 1px dashed var(--border);
    border-radius: var(--radius-md);
    margin-bottom: 20px;
  }

  /* ─── Bannière erreur ─────────────────────────────────────── */
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

  /* ─── Toolbar ────────────────────────────────────────────── */
  .toolbar {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-bottom: 16px;
    flex-wrap: wrap;
  }
  .btn-action {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    height: 38px;
    padding: 0 16px;
    background: var(--surface);
    color: var(--ivory);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    font: 500 13px/1 var(--font-ui);
    cursor: pointer;
    transition: all var(--dur-base) var(--ease);
  }
  .btn-action:hover:not(:disabled) {
    border-color: var(--border-hi);
    background: var(--surface-hi);
  }
  .btn-action:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
  .btn-action.lime {
    background: var(--lime);
    color: var(--ink);
    border-color: var(--lime);
    font-weight: 600;
    box-shadow:
      0 0 0 0 var(--lime-glow),
      0 4px 16px -6px rgba(197, 240, 74, 0.5);
  }
  .btn-action.lime:hover:not(:disabled) {
    transform: translateY(-1px);
    box-shadow:
      0 0 0 4px rgba(197, 240, 74, 0.15),
      0 8px 24px -6px rgba(197, 240, 74, 0.6);
  }
  .btn-action.ghost {
    background: transparent;
    width: 38px;
    padding: 0;
    justify-content: center;
  }
  .spacer-flex {
    flex: 1;
  }

  /* ─── Verdict ────────────────────────────────────────────── */
  .verdict {
    display: flex;
    gap: 16px;
    align-items: flex-start;
    padding: 16px 20px;
    border-radius: var(--radius-md);
    border: 1px solid;
    margin-bottom: 20px;
    animation: rise 400ms var(--ease);
  }
  .verdict[data-tone='ok'] {
    background: rgba(197, 240, 74, 0.06);
    border-color: rgba(197, 240, 74, 0.3);
  }
  .verdict[data-tone='ko'] {
    background: rgba(240, 108, 90, 0.08);
    border-color: rgba(240, 108, 90, 0.35);
  }
  .verdict-ico {
    display: inline-flex;
    padding-top: 2px;
    flex-shrink: 0;
  }
  .verdict[data-tone='ok'] .verdict-ico {
    color: var(--lime);
  }
  .verdict[data-tone='ko'] .verdict-ico {
    color: var(--coral);
  }
  .verdict-body {
    flex: 1;
    min-width: 0;
  }
  .verdict-title {
    font: 500 16px/1.2 var(--font-ui);
    color: var(--ivory);
    margin-bottom: 4px;
  }
  .verdict-sub {
    font: 400 12px/1.4 var(--font-mono);
    color: var(--ivory-2);
    margin-bottom: 6px;
  }
  .verdict-sub .link {
    color: var(--lime);
    border-bottom: 1px dashed rgba(197, 240, 74, 0.4);
    text-decoration: none;
  }
  .verdict-msg {
    font: 400 12px/1.4 var(--font-ui);
    color: var(--ivory-3);
    font-style: italic;
  }

  /* ─── Table ──────────────────────────────────────────────── */
  .table-wrap {
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    background: rgba(255, 255, 255, 0.015);
    overflow: auto;
    max-height: 60vh;
  }
  .ledger-table {
    width: 100%;
    border-collapse: collapse;
    /* `table-layout: fixed` empêche un contenu long (signature mono large,
       nom de modèle inattendu) de pousser les colonnes voisines. Les
       largeurs déclarées sur les `th` sont strictement respectées. */
    table-layout: fixed;
    font: 400 13px/1 var(--font-ui);
  }
  .ledger-table thead th {
    text-align: left;
    font: 500 10px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.12em;
    color: var(--ivory-3);
    padding: 12px 16px;
    background: var(--ink-2);
    border-bottom: 1px solid var(--border);
    position: sticky;
    top: 0;
    z-index: 1;
  }
  .ledger-table .th-id {
    width: 70px;
  }
  .ledger-table .th-date {
    width: 200px;
  }
  .ledger-table .th-model {
    width: auto;
  }
  .ledger-table .th-num {
    text-align: right;
    width: 140px;
  }
  .ledger-table .th-sig {
    width: 220px;
  }
  .ledger-table .th-state {
    width: 100px;
    text-align: center;
  }
  .row {
    cursor: pointer;
    transition: background var(--dur-fast) var(--ease);
  }
  .row td {
    padding: 12px 16px;
    border-bottom: 1px solid var(--border);
    color: var(--ivory-2);
    vertical-align: middle;
  }
  .row:hover td {
    background: rgba(255, 255, 255, 0.03);
    color: var(--ivory);
  }
  .row.active td {
    background: rgba(197, 240, 74, 0.05);
  }
  .row.focused td {
    box-shadow: inset 3px 0 0 var(--lime);
  }
  .row.invalid td {
    background: rgba(240, 108, 90, 0.04);
  }
  .row.invalid .td-sig {
    color: var(--coral);
  }
  .td-id {
    font: 500 12px/1 var(--font-mono);
    color: var(--ivory-3);
  }
  .td-date {
    font: 400 12px/1 var(--font-mono);
    white-space: nowrap;
  }
  .td-model {
    font: 500 13px/1 var(--font-ui);
    color: var(--ivory);
  }
  .td-num {
    font: 500 13px/1 var(--font-mono);
    color: var(--lime);
    text-align: right;
  }
  .td-sig {
    /* `<td>` doit rester en `display: table-cell` pour respecter la grille
       du tableau ; on déporte le flex sur un span enfant. */
    font: 400 11px/1 var(--font-mono);
    color: var(--ivory-3);
    vertical-align: middle;
  }
  .sig-wrap {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    max-width: 180px;
  }
  .sig-text {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    min-width: 0;
  }
  .td-state {
    text-align: center;
  }
  .badge-ok,
  .badge-purged {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 3px 8px;
    border-radius: var(--radius-pill);
    font: 500 10px/1 var(--font-mono);
    letter-spacing: 0.06em;
  }
  .badge-ok {
    background: var(--lime-soft);
    border: 1px solid rgba(197, 240, 74, 0.3);
    color: var(--lime);
  }
  .badge-purged {
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid var(--border-hi);
    color: var(--ivory-3);
  }

  .row-skel td {
    padding: 12px 16px;
    border-bottom: 1px solid var(--border);
  }
  .skel-bar {
    display: block;
    height: 14px;
    background: linear-gradient(
      90deg,
      rgba(255, 255, 255, 0.02),
      rgba(255, 255, 255, 0.06),
      rgba(255, 255, 255, 0.02)
    );
    background-size: 200% 100%;
    border-radius: 4px;
    animation: shimmer 1.4s linear infinite;
  }
  @keyframes shimmer {
    from {
      background-position: 200% 0;
    }
    to {
      background-position: -200% 0;
    }
  }
  .row-empty td {
    padding: 30px 16px;
    text-align: center;
    color: var(--ivory-3);
    font: 400 13px/1.5 var(--font-ui);
  }
  .link-btn {
    background: none;
    border: none;
    color: var(--lime);
    cursor: pointer;
    border-bottom: 1px dashed rgba(197, 240, 74, 0.4);
    font: inherit;
    padding: 0;
    margin-left: 8px;
  }

  /* ─── Pagination ────────────────────────────────────────── */
  .pagination {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-top: 18px;
    justify-content: flex-end;
  }
  .btn-page {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 32px;
    padding: 0 12px;
    background: var(--surface);
    color: var(--ivory-2);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    font: 500 12px/1 var(--font-ui);
    cursor: pointer;
    transition: all var(--dur-base) var(--ease);
  }
  .btn-page:hover:not(:disabled) {
    border-color: var(--border-hi);
    color: var(--ivory);
  }
  .btn-page:disabled {
    opacity: 0.35;
    cursor: not-allowed;
  }
  .page-info {
    font-size: 11px;
    color: var(--ivory-3);
    letter-spacing: 0.04em;
  }

  /* ─── Drawer ────────────────────────────────────────────── */
  .drawer-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(10, 13, 11, 0.6);
    backdrop-filter: blur(4px);
    z-index: 40;
    border: none;
    padding: 0;
    cursor: default;
    animation: fade 200ms var(--ease);
  }
  .drawer {
    position: fixed;
    top: 0;
    right: 0;
    bottom: 0;
    width: 420px;
    max-width: 92vw;
    background: var(--ink-3);
    border-left: 1px solid var(--border-hi);
    z-index: 50;
    box-shadow: var(--shadow-modal);
    display: flex;
    flex-direction: column;
    animation: slide-in 280ms var(--ease);
  }
  @keyframes slide-in {
    from {
      transform: translateX(100%);
    }
    to {
      transform: translateX(0);
    }
  }
  @keyframes fade {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }
  .drawer-head {
    padding: 20px 24px 16px;
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    border-bottom: 1px solid var(--border);
  }
  .drawer-eye {
    font: 500 10px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.14em;
    color: var(--ivory-3);
    margin-bottom: 6px;
  }
  .drawer-title {
    font: 400 28px/1 var(--font-display);
    font-style: italic;
    color: var(--ivory);
  }
  .drawer-body {
    padding: 20px 24px;
    overflow-y: auto;
    flex: 1;
  }
  .drawer-grid {
    display: grid;
    grid-template-columns: 140px 1fr;
    gap: 12px 16px;
    margin: 0 0 20px;
  }
  .drawer-grid dt {
    font: 500 10px/1.2 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.12em;
    color: var(--ivory-3);
    padding-top: 4px;
  }
  .drawer-grid dd {
    font: 400 13px/1.4 var(--font-ui);
    color: var(--ivory);
    margin: 0;
    min-width: 0;
  }
  .drawer-grid dd.mono {
    font-family: var(--font-mono);
    color: var(--ivory-2);
  }
  .drawer-grid dd.break {
    overflow-wrap: anywhere;
    word-break: break-all;
  }
  .drawer-note {
    margin: 16px 0 0;
    padding: 12px 14px;
    background: rgba(255, 255, 255, 0.02);
    border: 1px dashed var(--border);
    border-radius: var(--radius-md);
    font: 400 12px/1.5 var(--font-ui);
    color: var(--ivory-3);
  }
  .drawer-note strong {
    color: var(--ivory-2);
    font-weight: 600;
  }

  /* ─── Toast ─────────────────────────────────────────────── */
  .toast {
    position: fixed;
    bottom: 24px;
    right: 24px;
    z-index: 60;
    display: inline-flex;
    align-items: center;
    gap: 10px;
    padding: 12px 18px;
    background: var(--ink-3);
    border: 1px solid var(--border-hi);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-modal);
    font: 500 13px/1 var(--font-ui);
    color: var(--ivory);
    animation: toast-in 280ms var(--ease);
    max-width: 420px;
  }
  .toast[data-tone='success'] {
    border-color: rgba(197, 240, 74, 0.4);
  }
  .toast[data-tone='success'] :global(svg) {
    color: var(--lime);
  }
  .toast[data-tone='error'] {
    border-color: rgba(240, 108, 90, 0.4);
  }
  .toast[data-tone='error'] :global(svg) {
    color: var(--coral);
  }
  @keyframes toast-in {
    from {
      opacity: 0;
      transform: translateY(8px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
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
    .drawer {
      width: 100%;
    }
  }
</style>
