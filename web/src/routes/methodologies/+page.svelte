<script lang="ts">
  import {
    Layers,
    CheckCircle2,
    ExternalLink,
    BookOpen,
    Award,
    HelpCircle,
    Lock,
    AlertTriangle
  } from '@lucide/svelte';
  import {
    isTauriContext,
    listMethodologies,
    getAppPreferences,
    setAppPreferences,
    SobriaIpcError,
    type AppPreferencesDto,
    type EmpreinteMethod,
    type IpcErrorCode,
    type MethodologyInfoDto
  } from '$lib/api';

  // ─── État ────────────────────────────────────────────────────────────
  let methodologies = $state<MethodologyInfoDto[]>([]);
  let prefs = $state<AppPreferencesDto | null>(null);
  let bootstrapping = $state(true);
  let saving = $state(false);
  let loadError = $state<{ code: IpcErrorCode | string; message: string } | null>(null);
  let toast = $state<string | null>(null);

  const tauriAvailable = $derived(isTauriContext());

  $effect(() => {
    void (async () => {
      if (!tauriAvailable) {
        bootstrapping = false;
        loadError = {
          code: 'tauri_unavailable',
          message:
            "Le catalogue de méthodologies est servi par le binaire local. Lance l'app via `cargo tauri dev`."
        };
        return;
      }
      try {
        const [list, p] = await Promise.all([listMethodologies(), getAppPreferences()]);
        methodologies = list;
        prefs = p;
      } catch (err) {
        if (err instanceof SobriaIpcError) {
          loadError = { code: err.code, message: err.message };
        } else {
          loadError = { code: 'internal', message: 'Échec du chargement du catalogue.' };
        }
      } finally {
        bootstrapping = false;
      }
    })();
  });

  function showToast(msg: string) {
    toast = msg;
    setTimeout(() => {
      if (toast === msg) toast = null;
    }, 2200);
  }

  async function setDefaultMethod(method: EmpreinteMethod) {
    if (!prefs || saving || method === prefs.default_method) return;
    saving = true;
    const next: AppPreferencesDto = { ...prefs, default_method: method };
    // Si la nouvelle default est dans also_show, on l'en retire (on ne se
    // compare pas à soi-même).
    next.also_show_methods = next.also_show_methods.filter((m) => m !== method);
    try {
      await setAppPreferences(next);
      prefs = next;
      const info = methodologies.find((m) => m.method === method);
      showToast(`${info?.display_name ?? method} définie par défaut.`);
    } catch (err) {
      const msg = err instanceof SobriaIpcError ? err.message : 'Échec de la sauvegarde.';
      showToast(`Erreur : ${msg}`);
    } finally {
      saving = false;
    }
  }

  async function toggleAlsoShow(method: EmpreinteMethod) {
    if (!prefs || saving || method === prefs.default_method) return;
    saving = true;
    const isShown = prefs.also_show_methods.includes(method);
    const nextList = isShown
      ? prefs.also_show_methods.filter((m) => m !== method)
      : [...prefs.also_show_methods, method];
    const next: AppPreferencesDto = { ...prefs, also_show_methods: nextList };
    try {
      await setAppPreferences(next);
      prefs = next;
      const info = methodologies.find((m) => m.method === method);
      showToast(
        isShown
          ? `${info?.display_name ?? method} retirée de "Voir aussi".`
          : `${info?.display_name ?? method} ajoutée en référence.`
      );
    } catch (err) {
      const msg = err instanceof SobriaIpcError ? err.message : 'Échec de la sauvegarde.';
      showToast(`Erreur : ${msg}`);
    } finally {
      saving = false;
    }
  }

  function isDefault(m: EmpreinteMethod): boolean {
    return prefs?.default_method === m;
  }

  function isAlsoShown(m: EmpreinteMethod): boolean {
    return prefs?.also_show_methods.includes(m) ?? false;
  }

  function calibrationLabel(c: MethodologyInfoDto['calibration']): string {
    switch (c) {
      case 'peer_reviewed_reproduced':
        return 'Peer-reviewed · reproduit';
      case 'public_method_calibration_pending':
        return 'Méthode publique · calibration en cours';
      case 'indicative':
        return 'Indicative';
    }
  }

  function calibrationTone(c: MethodologyInfoDto['calibration']): string {
    switch (c) {
      case 'peer_reviewed_reproduced':
        return 'tone-validated';
      case 'public_method_calibration_pending':
        return 'tone-indicative';
      case 'indicative':
        return 'tone-extrapolated';
    }
  }
</script>

<svelte:head>
  <title>Sobr.ia · Méthodologies</title>
</svelte:head>

<div class="canvas-inner">
  <!-- TopBar -->
  <div class="topbar">
    <nav class="breadcrumb" aria-label="Fil d'Ariane">
      Audit <span class="sep">/</span>
      <span class="current">Méthodologies</span>
    </nav>
    <div class="spacer"></div>
    <span class="local-pill" title="Catalogue servi par le binaire local">
      <Lock size={12} strokeWidth={1.8} />
      100 % local
    </span>
    <a class="icon-btn" href="/" aria-label="Retour à l'atelier">
      <HelpCircle size={16} strokeWidth={1.6} />
    </a>
  </div>

  <!-- Hero -->
  <header class="hero">
    <div class="hero-brand">
      <Layers size={28} strokeWidth={1.6} />
      <div>
        <h1 class="hero-h1">Catalogue de méthodologies</h1>
        <p class="hero-sub">
          Plusieurs méthodologies scientifiques d'estimation de l'empreinte LLM sont embarquées dans
          Sobr.ia. Tu choisis ta méthodo par défaut, et tu peux en activer d'autres en référence
          pour comparer les résultats côté Atelier.
        </p>
        <aside class="page-crosslink" aria-label="Documentation méthodologique">
          <strong>Cette page sert à <em>choisir</em> ta méthodologie.</strong>
          Pour comprendre <em>comment</em> Sobr.ia calcule concrètement (formules, paramètres,
          Monte-Carlo, sources scientifiques) :
          <a class="crosslink-cta" href="/methodo"> → Comment ça marche (doc méthodologique) </a>
        </aside>
      </div>
    </div>
  </header>

  {#if loadError}
    <div class="banner" role="alert">
      <AlertTriangle size={18} strokeWidth={1.8} />
      <div>
        <strong>{loadError.code}</strong>
        <p>{loadError.message}</p>
      </div>
    </div>
  {/if}

  {#if bootstrapping}
    <p class="loading-text">Chargement du catalogue…</p>
  {:else}
    <ul class="method-list" role="list">
      {#each methodologies as m (m.method)}
        {@const def = isDefault(m.method)}
        {@const ref = isAlsoShown(m.method)}
        <li class="method-card" class:is-default={def}>
          <header class="m-head">
            <div class="m-title-row">
              <h2 class="m-title">{m.display_name}</h2>
              {#if def}
                <span class="badge badge-default" title="Méthodologie utilisée par défaut">
                  <CheckCircle2 size={12} strokeWidth={2} />
                  Par défaut
                </span>
              {/if}
              <span class="badge {calibrationTone(m.calibration)}" title={m.license}>
                <Award size={12} strokeWidth={2} />
                {calibrationLabel(m.calibration)}
              </span>
            </div>
            <p class="m-desc">{m.short_description}</p>
          </header>

          <div class="m-meta">
            <div class="meta-cell">
              <span class="meta-label">Source</span>
              <a class="link" href={m.reference_url} rel="noopener noreferrer" target="_blank">
                {m.doi ? `doi:${m.doi}` : 'doc officielle'}
                <ExternalLink size={11} strokeWidth={2} />
              </a>
            </div>
            <div class="meta-cell">
              <span class="meta-label">Maintenue par</span>
              <span class="meta-val">{m.maintained_by}</span>
            </div>
            <div class="meta-cell">
              <span class="meta-label">Publiée</span>
              <span class="meta-val mono">{m.year_published}</span>
            </div>
            <div class="meta-cell">
              <span class="meta-label">Licence</span>
              <span class="meta-val">{m.license}</span>
            </div>
          </div>

          <footer class="m-actions">
            <button
              class="action primary"
              type="button"
              disabled={def || saving || !prefs}
              onclick={() => setDefaultMethod(m.method)}
            >
              {def ? 'Méthodo par défaut active' : 'Définir comme par défaut'}
            </button>
            <label class="action-check">
              <input
                type="checkbox"
                checked={ref}
                disabled={def || saving || !prefs}
                onchange={() => toggleAlsoShow(m.method)}
              />
              <span>Afficher en référence dans l'Atelier (« Voir aussi »)</span>
            </label>
          </footer>
        </li>
      {/each}
    </ul>

    <section class="card" aria-labelledby="h-comment">
      <header class="card-head">
        <BookOpen size={18} strokeWidth={1.6} />
        <h2 id="h-comment">Comment ça fonctionne ?</h2>
      </header>
      <p class="card-text">
        Quand tu lances une estimation depuis l'Atelier (M1) ou un batch (M18), Sobr.ia utilise la
        méthodologie marquée <strong>« Par défaut »</strong>. Les méthodologies cochées
        <em>« Afficher en référence »</em>
        tournent <strong>en plus</strong> et leurs résultats apparaissent dans un panneau
        <em>« Voir aussi »</em> à côté du résultat principal — utile pour comparer les écarts entre approches
        scientifiques sans changer son point de vue par défaut.
      </p>
      <p class="card-text mono small">
        Toutes les estimations sont chaînées dans l'audit ledger SHA-256 avec la méthodologie
        utilisée. Tu peux ré-exporter un rapport CSRD à partir de la même méthodologie, même des
        années plus tard.
      </p>
    </section>
  {/if}

  {#if toast}
    <div class="toast" role="status">{toast}</div>
  {/if}

  <!-- Footer -->
  <footer class="legal-footer">
    <span>© 2026 Sobr.ia</span>
    <span class="sep" aria-hidden="true">·</span>
    <span>MIT</span>
    <span class="sep" aria-hidden="true">·</span>
    <span>C24 multi-méthodologie</span>
  </footer>
</div>

<style>
  .canvas-inner {
    max-width: 920px;
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

  /* Hero */
  .hero {
    padding-bottom: 16px;
    border-bottom: 1px solid var(--border);
  }
  .hero-brand {
    display: flex;
    gap: 18px;
    align-items: flex-start;
  }
  .hero-h1 {
    font: 400 28px/1.1 var(--font-display);
    font-style: italic;
    color: var(--ivory);
    letter-spacing: -0.01em;
    margin: 0 0 6px;
  }
  .hero-sub {
    font: 400 14px/1.6 var(--font-ui);
    color: var(--ivory-2);
    margin: 0;
    max-width: 720px;
  }

  /* Polish B — cross-link vers /methodo (doc) */
  .page-crosslink {
    margin-top: 14px;
    padding: 12px 16px;
    background: var(--surface);
    border: 1px dashed var(--border-hi);
    border-radius: var(--radius-md);
    font: 400 13px/1.55 var(--font-ui);
    color: var(--ivory-2);
    max-width: 720px;
  }
  .page-crosslink strong {
    display: block;
    color: var(--ivory);
    font-weight: 500;
    margin-bottom: 4px;
  }
  .page-crosslink em {
    font-style: italic;
  }
  .crosslink-cta {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    margin-top: 6px;
    color: var(--lime);
    text-decoration: none;
    font-weight: 500;
  }
  .crosslink-cta:hover {
    text-decoration: underline;
  }

  /* Banner d'erreur */
  .banner {
    display: flex;
    gap: 12px;
    align-items: flex-start;
    padding: 12px 16px;
    background: rgba(239, 68, 68, 0.08);
    border: 1px solid rgba(239, 68, 68, 0.25);
    border-radius: var(--radius-md);
    color: var(--ivory);
  }
  .banner p {
    font: 400 13px/1.5 var(--font-ui);
    margin: 4px 0 0;
    color: var(--ivory-2);
  }
  .banner strong {
    font: 600 12px/1 var(--font-mono);
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--ivory);
  }

  /* Loading */
  .loading-text {
    font: 400 13px/1.5 var(--font-ui);
    color: var(--ivory-3);
    text-align: center;
    padding: 40px 0;
  }

  /* Liste de méthodes */
  .method-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .method-card {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    padding: 20px 22px;
    display: flex;
    flex-direction: column;
    gap: 16px;
    transition: border-color var(--dur-base) var(--ease);
  }
  .method-card.is-default {
    border-color: var(--lime);
    box-shadow: 0 0 0 1px rgba(197, 240, 74, 0.15);
  }

  .m-head {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .m-title-row {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 10px;
  }
  .m-title {
    font: 500 17px/1.2 var(--font-ui);
    color: var(--ivory);
    margin: 0;
    flex: 1 1 auto;
  }
  .m-desc {
    font: 400 13.5px/1.55 var(--font-ui);
    color: var(--ivory-2);
    margin: 0;
  }

  /* Badges */
  .badge {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 3px 10px;
    border-radius: var(--radius-pill);
    font: 500 11px/1 var(--font-ui);
    letter-spacing: 0.01em;
    white-space: nowrap;
  }
  .badge-default {
    background: var(--lime-soft);
    border: 1px solid rgba(197, 240, 74, 0.3);
    color: var(--lime);
  }
  .tone-validated {
    background: rgba(34, 197, 94, 0.1);
    border: 1px solid rgba(34, 197, 94, 0.25);
    color: rgb(74, 222, 128);
  }
  .tone-indicative {
    background: rgba(234, 179, 8, 0.1);
    border: 1px solid rgba(234, 179, 8, 0.25);
    color: rgb(250, 204, 21);
  }
  .tone-extrapolated {
    background: var(--surface-hi);
    border: 1px solid var(--border-hi);
    color: var(--ivory-3);
  }

  /* Méta-grille */
  .m-meta {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(160px, 1fr));
    gap: 12px;
    padding: 12px 0;
    border-top: 1px dashed var(--border);
    border-bottom: 1px dashed var(--border);
  }
  .meta-cell {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .meta-label {
    font: 500 10px/1 var(--font-ui);
    color: var(--ivory-4);
    text-transform: uppercase;
    letter-spacing: 0.08em;
  }
  .meta-val {
    font: 400 13px/1.4 var(--font-ui);
    color: var(--ivory-2);
  }
  .meta-val.mono,
  .mono {
    font-family: var(--font-mono);
  }
  .small {
    font-size: 11px;
  }
  .link {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    color: var(--lime);
    text-decoration: none;
    font: 400 13px/1.4 var(--font-ui);
  }
  .link:hover {
    text-decoration: underline;
  }

  /* Actions */
  .m-actions {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .action {
    align-self: flex-start;
    padding: 8px 18px;
    background: transparent;
    border: 1px solid var(--border-hi);
    border-radius: var(--radius-sm);
    color: var(--ivory-2);
    font: 500 13px/1 var(--font-ui);
    cursor: pointer;
    transition: all var(--dur-base) var(--ease);
  }
  .action.primary:not(:disabled) {
    background: var(--lime);
    border-color: var(--lime);
    color: var(--ink);
  }
  .action.primary:not(:disabled):hover {
    filter: brightness(1.05);
  }
  .action:disabled {
    cursor: not-allowed;
    opacity: 0.55;
  }

  .action-check {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    font: 400 13px/1.4 var(--font-ui);
    color: var(--ivory-2);
    cursor: pointer;
  }
  .action-check input[type='checkbox'] {
    width: 14px;
    height: 14px;
    accent-color: var(--lime);
    cursor: pointer;
  }
  .action-check input[type='checkbox']:disabled {
    cursor: not-allowed;
    opacity: 0.55;
  }

  /* Card "Comment ça fonctionne" */
  .card {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    padding: 18px 22px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .card-head {
    display: flex;
    align-items: center;
    gap: 10px;
    color: var(--ivory);
  }
  .card-head h2 {
    font: 500 14px/1 var(--font-ui);
    margin: 0;
  }
  .card-text {
    font: 400 13.5px/1.6 var(--font-ui);
    color: var(--ivory-2);
    margin: 0;
  }

  /* Toast */
  .toast {
    position: fixed;
    bottom: 24px;
    left: 50%;
    transform: translateX(-50%);
    background: var(--ink-hi);
    color: var(--ivory);
    border: 1px solid var(--border-hi);
    border-radius: var(--radius-pill);
    padding: 10px 18px;
    font: 500 13px/1 var(--font-ui);
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
    z-index: 50;
  }

  /* Footer */
  .legal-footer {
    margin-top: 24px;
    padding-top: 16px;
    border-top: 1px solid var(--border);
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    font: 400 11px/1.4 var(--font-ui);
    color: var(--ivory-4);
  }
  .legal-footer .sep {
    color: var(--ivory-4);
  }
</style>
