<script lang="ts">
  // C10 — Paramètres (ADR-0010 + brief C10 §3.4).
  // Quatre sections : persona courant, modules activés (par catégorie),
  // modules disponibles non activés, refaire l'onboarding + langue.
  import { onMount } from 'svelte';
  import {
    Settings2,
    HelpCircle,
    Lock,
    Info,
    AlertTriangle,
    PlugZap,
    Check,
    Hammer,
    RefreshCw,
    UserRound,
    Layers,
    PlusCircle,
    Cpu,
    Dice5,
    Folder
  } from '@lucide/svelte';
  import {
    isTauriContext,
    metaInfo,
    SobriaIpcError,
    type IpcErrorCode,
    type MetaInfo
  } from '$lib/api';
  import {
    ALL_MODULES,
    ALL_PERSONAS,
    CATEGORY_LABELS,
    defaultModulesFor,
    moduleCategory,
    moduleDescription,
    moduleLabel,
    personaEmoji,
    personaLabel,
    personaTagline,
    preferences,
    savePreferences,
    type ModuleCategory,
    type ModuleId,
    type Persona
  } from '$lib/preferences';

  // ─── State ───────────────────────────────────────────────────────────
  let meta = $state<MetaInfo | null>(null);
  let bootstrapping = $state(true);
  let loadError = $state<{ code: IpcErrorCode | string; message: string } | null>(null);
  let saveError = $state<{ code: IpcErrorCode | string; message: string } | null>(null);
  let confirmPersona = $state<Persona | null>(null); // dialog de confirmation

  const tauriAvailable = $derived(isTauriContext());

  $effect(() => {
    void (async () => {
      if (!tauriAvailable) {
        bootstrapping = false;
        loadError = {
          code: 'tauri_unavailable',
          message:
            "L'application doit être lancée via `cargo run -p sobria-app`. Les préférences ne peuvent pas être modifiées dans un navigateur seul."
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

  onMount(() => {
    // Si l'utilisateur arrive ici sans avoir fait l'onboarding (cas
    // hors-Tauri où la garde du layout est désactivée), on ne le force pas.
  });

  // ─── Helpers ─────────────────────────────────────────────────────────
  const currentPersona = $derived($preferences.persona);
  const enabled = $derived(new Set($preferences.enabled_modules));

  async function applyPersona(p: Persona) {
    confirmPersona = null;
    saveError = null;
    try {
      await savePreferences({
        persona: p,
        enabled_modules: defaultModulesFor(p),
        onboarded: $preferences.onboarded,
        lang: $preferences.lang
      });
    } catch (e) {
      saveError = errorOf(e);
    }
  }

  async function toggleModule(m: ModuleId) {
    saveError = null;
    const cur = new Set($preferences.enabled_modules);
    if (cur.has(m)) cur.delete(m);
    else cur.add(m);
    const sorted = ALL_MODULES.filter((id) => cur.has(id));
    try {
      await savePreferences({
        persona: $preferences.persona,
        enabled_modules: sorted,
        onboarded: $preferences.onboarded,
        lang: $preferences.lang
      });
    } catch (e) {
      saveError = errorOf(e);
    }
  }

  async function setLang(l: 'fr' | 'en') {
    saveError = null;
    try {
      await savePreferences({
        persona: $preferences.persona,
        enabled_modules: $preferences.enabled_modules,
        onboarded: $preferences.onboarded,
        lang: l
      });
    } catch (e) {
      saveError = errorOf(e);
    }
  }

  async function redoOnboarding() {
    saveError = null;
    try {
      await savePreferences({
        persona: $preferences.persona,
        enabled_modules: $preferences.enabled_modules,
        onboarded: false,
        lang: $preferences.lang
      });
      // `window.location.replace` plutôt que `goto`/`$app/navigation`
      // (cf. note dans +layout.svelte).
      window.location.replace('/onboarding');
    } catch (e) {
      saveError = errorOf(e);
    }
  }

  function errorOf(e: unknown): { code: string; message: string } {
    if (e instanceof SobriaIpcError) return { code: e.code, message: e.message };
    return { code: 'internal', message: 'Échec de la mise à jour.' };
  }

  const errorTone = $derived.by<'info' | 'warn' | 'error'>(() => {
    if (!loadError) return 'info';
    if (loadError.code === 'tauri_unavailable') return 'warn';
    return 'error';
  });

  // Regroupement modules activés par catégorie.
  const enabledByCategory = $derived.by(() => {
    const map = new Map<ModuleCategory, ModuleId[]>();
    for (const id of ALL_MODULES) {
      if (!enabled.has(id)) continue;
      const cat = moduleCategory(id);
      const bucket = map.get(cat) ?? [];
      bucket.push(id);
      map.set(cat, bucket);
    }
    return map;
  });

  const disabledModules = $derived(ALL_MODULES.filter((id) => !enabled.has(id)));

  const CATEGORY_ORDER: ModuleCategory[] = [
    'estimation',
    'visualisation',
    'reporting',
    'pedagogie'
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
      Paramètres · persona &amp; modules
    </div>
    <h1 class="hero-h1">
      Vos <em>paramètres</em> et l'état du moteur Sobr.ia.
    </h1>
    <p class="hero-sub">
      Tout est local. Composez votre atelier à la carte ou repartez d'un bundle persona — les
      changements sont persistés instantanément dans <code>referentiel.sqlite</code>.
    </p>
  </section>

  <!-- ─── Bannière chargement ─────────────────────────────────── -->
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
        <strong>
          {loadError.code === 'tauri_unavailable' ? 'Application non lancée via Tauri' : 'Erreur'}
        </strong>
        <span>{loadError.message}</span>
      </div>
    </div>
  {/if}

  <!-- ─── Bannière sauvegarde ─────────────────────────────────── -->
  {#if saveError}
    <div class="banner" data-tone="error" role="alert">
      <span class="banner-ico"><PlugZap size={18} strokeWidth={1.8} /></span>
      <div class="banner-body">
        <strong>Modification non enregistrée</strong>
        <span>{saveError.message}</span>
      </div>
    </div>
  {/if}

  <!-- ╭─── Section 1 : Persona courant ─────────────────────────╮ -->
  <section class="section">
    <header class="section-head">
      <UserRound size={16} strokeWidth={1.8} />
      <h2>Persona courant</h2>
      <span class="section-hint mono">précoche un bundle de modules</span>
    </header>

    {#if currentPersona}
      <div class="persona-current">
        <span class="emoji" aria-hidden="true">{personaEmoji(currentPersona)}</span>
        <div class="persona-info">
          <div class="persona-name">{personaLabel(currentPersona)}</div>
          <div class="persona-line">{personaTagline(currentPersona)}</div>
        </div>
        <button
          type="button"
          class="btn-ghost"
          onclick={() => {
            confirmPersona = null;
          }}
          disabled={!tauriAvailable}
          aria-label="Conserver le persona actuel"
        >
          Conserver
        </button>
      </div>
    {:else}
      <p class="persona-empty">Aucun persona sélectionné. Vous composez vos modules à la carte.</p>
    {/if}

    <p class="section-foot-hint">Choisir un autre persona remplacera votre sélection actuelle :</p>

    <ul class="persona-options">
      {#each ALL_PERSONAS as p (p)}
        {@const isCurrent = currentPersona === p}
        <li>
          <button
            type="button"
            class="persona-option"
            class:current={isCurrent}
            disabled={!tauriAvailable || isCurrent}
            onclick={() => (confirmPersona = p)}
            data-persona={p}
          >
            <span class="emoji" aria-hidden="true">{personaEmoji(p)}</span>
            <span class="persona-option-body">
              <span class="persona-option-name">{personaLabel(p)}</span>
              <span class="persona-option-line">{personaTagline(p)}</span>
            </span>
            {#if isCurrent}
              <span class="persona-tag mono">actuel</span>
            {:else}
              <span class="persona-tag mono ghost">{defaultModulesFor(p).length} mod.</span>
            {/if}
          </button>
        </li>
      {/each}
    </ul>
  </section>

  <!-- ╭─── Section 2 : Modules activés ────────────────────────╮ -->
  <section class="section">
    <header class="section-head">
      <Layers size={16} strokeWidth={1.8} />
      <h2>Modules activés</h2>
      <span class="section-hint mono">{enabled.size} sur {ALL_MODULES.length}</span>
    </header>

    {#if enabled.size === 0}
      <p class="empty">Aucun module activé — votre rail est vide.</p>
    {:else}
      {#each CATEGORY_ORDER as cat (cat)}
        {@const ids = enabledByCategory.get(cat) ?? []}
        {#if ids.length > 0}
          <div class="category">
            <h3 class="category-title">{CATEGORY_LABELS[cat]}</h3>
            <ul class="modules-list">
              {#each ids as m (m)}
                <li>
                  <label class="module-line" data-checked="true">
                    <input
                      type="checkbox"
                      checked
                      onchange={() => toggleModule(m)}
                      disabled={!tauriAvailable}
                      data-module={m}
                    />
                    <span class="check-box" aria-hidden="true">
                      <Check size={11} strokeWidth={2.5} />
                    </span>
                    <span class="module-body">
                      <span class="module-id mono">{m.toUpperCase()}</span>
                      <span class="module-label">{moduleLabel(m)}</span>
                      <span class="module-desc">{moduleDescription(m)}</span>
                    </span>
                  </label>
                </li>
              {/each}
            </ul>
          </div>
        {/if}
      {/each}
    {/if}
  </section>

  <!-- ╭─── Section 3 : Modules disponibles non activés ────────╮ -->
  <section class="section">
    <header class="section-head">
      <PlusCircle size={16} strokeWidth={1.8} />
      <h2>Modules disponibles</h2>
      <span class="section-hint mono">non activés · {disabledModules.length} dispo.</span>
    </header>

    {#if disabledModules.length === 0}
      <p class="empty">Tous les modules sont déjà dans votre atelier.</p>
    {:else}
      <ul class="modules-list compact">
        {#each disabledModules as m (m)}
          <li>
            <label class="module-line" data-checked="false">
              <input
                type="checkbox"
                checked={false}
                onchange={() => toggleModule(m)}
                disabled={!tauriAvailable}
                data-module={m}
              />
              <span class="check-box" aria-hidden="true"></span>
              <span class="module-body">
                <span class="module-id mono">{m.toUpperCase()}</span>
                <span class="module-label">{moduleLabel(m)}</span>
                <span class="module-desc">{moduleDescription(m)}</span>
              </span>
            </label>
          </li>
        {/each}
      </ul>
    {/if}
  </section>

  <!-- ╭─── Section 4 : Onboarding + langue ─────────────────────╮ -->
  <section class="section">
    <header class="section-head">
      <RefreshCw size={16} strokeWidth={1.8} />
      <h2>Réinitialiser &amp; langue</h2>
      <span class="section-hint mono">FR · EN à venir</span>
    </header>

    <div class="dual">
      <div class="dual-col">
        <h3 class="dual-title">Refaire l'onboarding</h3>
        <p class="dual-sub">
          Revoir les 4 étapes de bienvenue (splash, persona, bundle, premier prompt). Vos modules
          actuels restent en place jusqu'à la fin du nouveau parcours.
        </p>
        <button
          type="button"
          class="btn-ghost"
          onclick={redoOnboarding}
          disabled={!tauriAvailable}
          data-action="redo-onboarding"
        >
          <RefreshCw size={14} strokeWidth={1.8} />
          Refaire l'onboarding
        </button>
      </div>
      <div class="dual-col">
        <h3 class="dual-title">Langue de l'interface</h3>
        <p class="dual-sub">
          La traduction anglaise est en cours (chantier C12). Vous pouvez déjà tester le sélecteur.
        </p>
        <div class="lang-toggle" role="radiogroup" aria-label="Langue de l'interface">
          <button
            type="button"
            class="lang-btn"
            class:active={$preferences.lang === 'fr'}
            onclick={() => setLang('fr')}
            disabled={!tauriAvailable}
            role="radio"
            aria-checked={$preferences.lang === 'fr'}
          >
            FR
          </button>
          <button
            type="button"
            class="lang-btn"
            class:active={$preferences.lang === 'en'}
            onclick={() => setLang('en')}
            disabled={!tauriAvailable}
            role="radio"
            aria-checked={$preferences.lang === 'en'}
          >
            EN
          </button>
        </div>
      </div>
    </div>
  </section>

  <!-- ╭─── Runtime (lecture seule via meta_info) ──────────────╮ -->
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
          <dt><Cpu size={12} strokeWidth={1.8} /> Version d'application</dt>
          <dd class="mono">{meta.app_version}</dd>
        </div>
        <div class="runtime-row">
          <dt><Dice5 size={12} strokeWidth={1.8} /> Seed Monte-Carlo</dt>
          <dd class="mono">{meta.estimator_seed}</dd>
        </div>
        <div class="runtime-row">
          <dt><Dice5 size={12} strokeWidth={1.8} /> Tirages Monte-Carlo (N)</dt>
          <dd class="mono">{new Intl.NumberFormat('fr-FR').format(meta.estimator_n)}</dd>
        </div>
        <div class="runtime-row">
          <dt><Folder size={12} strokeWidth={1.8} /> Ledger d'audit</dt>
          <dd class="mono break">{meta.audit_path}</dd>
        </div>
        <div class="runtime-row">
          <dt><Folder size={12} strokeWidth={1.8} /> Racine des données</dt>
          <dd class="mono break">{meta.data_root}</dd>
        </div>
      </dl>
    {/if}

    <p class="section-foot">
      <Hammer size={11} strokeWidth={1.8} />
      Préférences persistées dans <code>referentiel.sqlite</code>, table
      <code>app_preferences</code>. Pour repartir de zéro : supprimez le fichier puis relancez
      l'application.
    </p>
  </section>
</div>

<!-- ─── Dialog confirmation changement persona ───────────────────── -->
{#if confirmPersona}
  <!-- L'overlay sert de backdrop cliquable pour fermer ; le dialog
       lui-même (rôle="dialog", aria-modal) est l'élément interne. -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="modal-overlay"
    onclick={() => (confirmPersona = null)}
    onkeydown={(e) => {
      if (e.key === 'Escape') confirmPersona = null;
    }}
  >
    <div
      class="modal"
      role="dialog"
      aria-modal="true"
      aria-labelledby="confirm-title"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
      tabindex="-1"
    >
      <h3 id="confirm-title">Remplacer votre sélection ?</h3>
      <p>
        Vous allez passer au bundle <strong>{personaLabel(confirmPersona)}</strong>. Cela remplacera
        votre liste actuelle de modules par celle de ce persona ({defaultModulesFor(confirmPersona)
          .length}
        modules).
      </p>
      <div class="modal-actions">
        <button type="button" class="btn-ghost" onclick={() => (confirmPersona = null)}>
          Annuler
        </button>
        <button
          type="button"
          class="btn-primary"
          onclick={() => confirmPersona && applyPersona(confirmPersona)}
          data-action="confirm-persona"
        >
          Remplacer
        </button>
      </div>
    </div>
  </div>
{/if}

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
  .hero-sub code {
    font: 500 12px/1.4 var(--font-mono);
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 1px 6px;
    color: var(--ivory);
  }

  /* Banner */
  .banner {
    display: flex;
    gap: 14px;
    align-items: flex-start;
    padding: 14px 18px;
    border-radius: var(--radius-md);
    border: 1px solid var(--border-hi);
    margin-bottom: 16px;
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

  /* ─── Section 1 — Persona ──────────────────────────────────────────── */
  .persona-current {
    display: grid;
    grid-template-columns: auto 1fr auto;
    gap: 18px;
    align-items: center;
    padding: 16px 18px;
    background: rgba(197, 240, 74, 0.05);
    border: 1px solid rgba(197, 240, 74, 0.25);
    border-radius: var(--radius-md);
    margin-bottom: 14px;
  }
  .persona-current .emoji {
    font-size: 28px;
  }
  .persona-info {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .persona-name {
    font: 500 16px/1.2 var(--font-display);
    font-style: italic;
    color: var(--ivory);
  }
  .persona-line {
    font: 400 13px/1.4 var(--font-ui);
    color: var(--ivory-3);
  }
  .persona-empty {
    font: 400 13px/1.5 var(--font-ui);
    color: var(--ivory-3);
    background: rgba(255, 255, 255, 0.02);
    border: 1px dashed var(--border);
    border-radius: var(--radius-md);
    padding: 14px 18px;
    margin: 0 0 14px;
  }
  .section-foot-hint {
    font: 400 12px/1.4 var(--font-ui);
    color: var(--ivory-3);
    margin: 4px 0 12px;
  }
  .persona-options {
    list-style: none;
    padding: 0;
    margin: 0;
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
    gap: 8px;
  }
  .persona-option {
    display: grid;
    grid-template-columns: auto 1fr auto;
    align-items: center;
    gap: 12px;
    width: 100%;
    padding: 12px 14px;
    background: rgba(255, 255, 255, 0.025);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    cursor: pointer;
    text-align: left;
    font: inherit;
    color: inherit;
    transition: all var(--dur-base) var(--ease);
  }
  .persona-option:hover:not(:disabled) {
    border-color: rgba(197, 240, 74, 0.4);
    background: rgba(197, 240, 74, 0.04);
  }
  .persona-option.current {
    border-color: rgba(197, 240, 74, 0.5);
    background: rgba(197, 240, 74, 0.06);
    cursor: default;
  }
  .persona-option:disabled {
    opacity: 0.6;
  }
  .persona-option .emoji {
    font-size: 22px;
  }
  .persona-option-body {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }
  .persona-option-name {
    font: 500 14px/1.2 var(--font-ui);
    color: var(--ivory);
  }
  .persona-option-line {
    font: 400 12px/1.4 var(--font-ui);
    color: var(--ivory-3);
  }
  .persona-tag {
    font-size: 10px;
    color: var(--lime);
    background: var(--lime-soft);
    border: 1px solid rgba(197, 240, 74, 0.25);
    border-radius: var(--radius-pill);
    padding: 3px 8px;
    letter-spacing: 0.06em;
    white-space: nowrap;
  }
  .persona-tag.ghost {
    color: var(--ivory-4);
    background: transparent;
    border-color: var(--border);
  }

  /* ─── Section 2 — Modules par catégorie ──────────────────────────── */
  .category {
    margin-bottom: 18px;
  }
  .category + .category {
    padding-top: 16px;
    border-top: 1px dashed var(--border);
  }
  .category-title {
    font: 500 11px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.14em;
    color: var(--ivory-3);
    margin: 0 0 10px;
  }
  .modules-list {
    list-style: none;
    padding: 0;
    margin: 0;
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
    gap: 6px;
  }
  .modules-list.compact {
    grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
  }
  .module-line {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 12px;
    padding: 10px 14px;
    background: rgba(255, 255, 255, 0.02);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: all var(--dur-base) var(--ease);
  }
  .module-line:hover {
    border-color: var(--border-hi);
    background: var(--surface);
  }
  .module-line[data-checked='true'] {
    border-color: rgba(197, 240, 74, 0.28);
    background: rgba(197, 240, 74, 0.04);
  }
  .module-line input[type='checkbox'] {
    position: absolute;
    opacity: 0;
    width: 1px;
    height: 1px;
    pointer-events: none;
  }
  .check-box {
    width: 16px;
    height: 16px;
    align-self: start;
    margin-top: 3px;
    background: var(--ink-2);
    border: 1.5px solid var(--border-hi);
    border-radius: 4px;
    display: grid;
    place-items: center;
    color: var(--ink);
    transition: all var(--dur-base) var(--ease);
  }
  .module-line[data-checked='true'] .check-box {
    background: var(--lime);
    border-color: var(--lime);
  }
  .module-body {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 6px 10px;
    align-items: baseline;
  }
  .module-id {
    font-size: 10px;
    color: var(--ivory-4);
    letter-spacing: 0.08em;
  }
  .module-label {
    font: 500 13px/1.2 var(--font-ui);
    color: var(--ivory);
    grid-column: 2;
  }
  .module-line[data-checked='true'] .module-label {
    color: var(--lime);
  }
  .module-desc {
    grid-row: 2;
    grid-column: 1 / -1;
    margin-top: 2px;
    font: 400 11px/1.4 var(--font-ui);
    color: var(--ivory-3);
  }
  .empty {
    font: 400 13px/1.5 var(--font-ui);
    color: var(--ivory-3);
    text-align: center;
    padding: 22px 12px;
    margin: 0;
  }

  /* ─── Section 4 — Dual ────────────────────────────────────────────── */
  .dual {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 24px;
  }
  .dual-col {
    padding: 16px 18px;
    background: rgba(255, 255, 255, 0.02);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
  }
  .dual-title {
    font: 500 13px/1.2 var(--font-ui);
    color: var(--ivory);
    margin: 0 0 6px;
  }
  .dual-sub {
    font: 400 12px/1.5 var(--font-ui);
    color: var(--ivory-3);
    margin: 0 0 12px;
  }
  .lang-toggle {
    display: inline-flex;
    background: var(--ink-2);
    border: 1px solid var(--border);
    border-radius: var(--radius-pill);
    padding: 3px;
    gap: 2px;
  }
  .lang-btn {
    border: none;
    background: transparent;
    color: var(--ivory-3);
    font: 600 12px/1 var(--font-mono);
    padding: 7px 16px;
    border-radius: var(--radius-pill);
    cursor: pointer;
    transition: all var(--dur-base) var(--ease);
    letter-spacing: 0.06em;
  }
  .lang-btn:hover:not(:disabled) {
    color: var(--ivory);
  }
  .lang-btn.active {
    background: var(--lime);
    color: var(--ink);
  }
  .lang-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  /* ─── Buttons partagés ────────────────────────────────────────────── */
  .btn-primary {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 11px 22px;
    background: var(--lime);
    color: var(--ink);
    border: none;
    border-radius: var(--radius-pill);
    font: 600 13px/1 var(--font-ui);
    cursor: pointer;
    transition: all var(--dur-base) var(--ease);
  }
  .btn-primary:hover {
    transform: translateY(-1px);
    box-shadow: var(--glow-lime-sm);
  }
  .btn-ghost {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 9px 18px;
    background: transparent;
    color: var(--ivory-2);
    border: 1px solid var(--border-hi);
    border-radius: var(--radius-pill);
    font: 500 12px/1 var(--font-ui);
    cursor: pointer;
    transition: all var(--dur-base) var(--ease);
  }
  .btn-ghost:hover:not(:disabled) {
    border-color: var(--lime);
    color: var(--lime);
  }
  .btn-ghost:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  /* ─── Runtime grid (legacy /meta_info) ────────────────────────────── */
  .runtime-skel {
    padding: 18px;
    color: var(--ivory-3);
    font: 400 12px/1 var(--font-mono);
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
    padding: 9px 12px;
    border-radius: var(--radius-sm);
    border: 1px solid transparent;
  }
  .runtime-row:hover {
    background: rgba(255, 255, 255, 0.015);
    border-color: var(--border);
  }
  .runtime-row dt {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    font: 500 10px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.12em;
    color: var(--ivory-3);
  }
  .runtime-row dt :global(svg) {
    color: var(--ivory-4);
  }
  .runtime-row dd {
    font: 500 12px/1.4 var(--font-mono);
    color: var(--ivory);
    margin: 0;
  }
  .runtime-row dd.break {
    overflow-wrap: anywhere;
    word-break: break-all;
  }

  /* ─── Modal confirmation ──────────────────────────────────────────── */
  .modal-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.65);
    backdrop-filter: blur(4px);
    -webkit-backdrop-filter: blur(4px);
    display: grid;
    place-items: center;
    z-index: 50;
    padding: 24px;
    border: none;
  }
  .modal {
    background: var(--ink-2);
    border: 1px solid var(--border-hi);
    border-radius: var(--radius-lg);
    padding: 28px 30px;
    max-width: 460px;
    width: 100%;
    box-shadow: var(--shadow-modal);
    text-align: left;
  }
  .modal h3 {
    font: 400 26px/1.2 var(--font-display);
    font-style: italic;
    color: var(--ivory);
    margin: 0 0 12px;
  }
  .modal p {
    font: 400 14px/1.55 var(--font-ui);
    color: var(--ivory-2);
    margin: 0 0 22px;
  }
  .modal p strong {
    color: var(--lime);
    font-weight: 500;
  }
  .modal-actions {
    display: flex;
    gap: 12px;
    justify-content: flex-end;
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
    .dual {
      grid-template-columns: 1fr;
    }
    .persona-current {
      grid-template-columns: auto 1fr;
    }
    .persona-current .btn-ghost {
      grid-column: 1 / -1;
    }
  }
</style>
