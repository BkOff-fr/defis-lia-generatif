<script lang="ts">
  // C10 — Onboarding wizard (ADR-0010 + brief C10 §3.3).
  // 4 étapes : Splash → Persona → Bundle → Premier prompt.
  // Persistance : `savePreferences` (optimistic + rollback IPC).
  import { onMount, onDestroy, tick } from 'svelte';
  import {
    ArrowRight,
    ChevronDown,
    ChevronUp,
    Lock,
    Sparkles,
    Zap,
    Check,
    AlertTriangle
  } from '@lucide/svelte';
  import { SobriaIpcError } from '$lib/api';
  import {
    ALL_MODULES,
    ALL_PERSONAS,
    defaultModulesFor,
    moduleDescription,
    moduleLabel,
    personaEmoji,
    personaLabel,
    personaTagline,
    savePreferences,
    type ModuleId,
    type Persona
  } from '$lib/preferences';

  // ─── State (Svelte 5 runes) ──────────────────────────────────────────
  let step = $state<1 | 2 | 3 | 4>(1);
  let persona = $state<Persona | null>(null);
  let chosen = $state<Set<ModuleId>>(new Set());
  let showMore = $state(false);
  let saving = $state(false);
  let error = $state<{ code: string; message: string } | null>(null);

  // Auto-advance splash après 3 s (réversible : on annule si l'utilisateur
  // clique « Continuer » avant la fin).
  let splashTimer: ReturnType<typeof setTimeout> | null = null;
  onMount(() => {
    splashTimer = setTimeout(() => {
      if (step === 1) step = 2;
    }, 3000);
  });
  onDestroy(() => {
    if (splashTimer) clearTimeout(splashTimer);
  });

  function cancelSplashTimer() {
    if (splashTimer) {
      clearTimeout(splashTimer);
      splashTimer = null;
    }
  }

  // Bundle dérivé du persona : mis à jour quand on (re)passe à l'étape 3.
  function pickPersona(p: Persona) {
    persona = p;
    chosen = new Set(defaultModulesFor(p));
    cancelSplashTimer();
    step = 3;
    void focusFirst('#step3');
  }

  function skipPersona() {
    // « Je préfère choisir à la carte » : pas de persona, bundle vide,
    // l'utilisateur compose lui-même à l'étape 3.
    persona = null;
    chosen = new Set();
    cancelSplashTimer();
    step = 3;
    void focusFirst('#step3');
  }

  function toggleModule(m: ModuleId) {
    const next = new Set(chosen);
    if (next.has(m)) next.delete(m);
    else next.add(m);
    chosen = next;
  }

  // Liste segmentée pour l'étape 3 : modules du bundle d'abord, puis le reste.
  const inBundle = $derived(
    persona ? defaultModulesFor(persona) : (ALL_MODULES.slice() as ModuleId[])
  );
  const moreModules = $derived(
    persona ? (ALL_MODULES.filter((m) => !inBundle.includes(m)) as ModuleId[]) : ([] as ModuleId[])
  );

  // Pour mémoire / tests : nombre de modules cochés.
  const chosenCount = $derived(chosen.size);

  // ─── Finalisation ───────────────────────────────────────────────────
  async function finishOnboarding() {
    error = null;
    saving = true;
    try {
      await savePreferences({
        persona,
        enabled_modules: Array.from(chosen).sort(
          (a, b) => ALL_MODULES.indexOf(a) - ALL_MODULES.indexOf(b)
        ),
        onboarded: true,
        lang: 'fr'
      });
      // `window.location.replace` plutôt que `goto`/`$app/navigation`
      // (cf. note dans +layout.svelte).
      window.location.replace('/');
    } catch (e) {
      if (e instanceof SobriaIpcError) {
        error = { code: e.code, message: e.message };
      } else {
        error = { code: 'internal', message: "Échec de l'enregistrement." };
      }
    } finally {
      saving = false;
    }
  }

  // ─── Focus management : a11y ────────────────────────────────────────
  async function focusFirst(selector: string) {
    await tick();
    const el = document.querySelector<HTMLElement>(`${selector} [data-autofocus]`);
    el?.focus();
  }

  // Focus trap simple : Tab tourne dans les éléments focusables du wizard.
  function handleKey(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      // ESC ne ferme pas (onboarding non-bloquant via le lien explicite),
      // mais permet de revenir d'une étape.
      if (step === 3 || step === 4) {
        e.preventDefault();
        step = (step - 1) as 1 | 2 | 3 | 4;
      }
    }
  }

  // Étape 2 → 1 navigation arrière.
  function back() {
    if (step > 1) step = (step - 1) as 1 | 2 | 3 | 4;
  }
</script>

<svelte:head>
  <title>Sobr.ia · Bienvenue</title>
</svelte:head>

<svelte:window onkeydown={handleKey} />

<!-- Décoration ambient + topo (la layout les a retirées pour /onboarding) -->
<div class="amb" aria-hidden="true"></div>
<svg class="topo" viewBox="0 0 600 600" fill="none" aria-hidden="true">
  <g stroke="rgb(197 240 74)" stroke-width="0.6" fill="none" opacity="0.4">
    <path d="M 300 300 m -200, 0 a 200,200 0 1,0 400,0 a 200,200 0 1,0 -400,0" />
    <path d="M 300 300 m -160, 0 a 160,180 -10 1,0 320,20 a 160,180 -10 1,0 -320,-20" />
    <path d="M 300 300 m -120, -10 a 120,140 -20 1,0 240,40 a 120,140 -20 1,0 -240,-40" />
    <path d="M 300 300 m -80, -10 a 80,100 -30 1,0 160,40 a 80,100 -30 1,0 -160,-40" />
  </g>
</svg>

<!-- Progression : 4 puces dans le coin haut-droit -->
<div class="progress" aria-label="Progression de l'onboarding">
  {#each [1, 2, 3, 4] as n}
    <span class="dot" class:done={step > n} class:current={step === n} aria-hidden="true"></span>
  {/each}
  <span class="visually-hidden">Étape {step} sur 4</span>
</div>

<main class="wizard" aria-labelledby="wizard-title">
  <!-- ╭───── ÉTAPE 1 — SPLASH ─────────────────────────────────────╮ -->
  {#if step === 1}
    <section class="splash" id="step1">
      <div class="brand-stage">
        <svg class="brand-logo" viewBox="0 0 88 88" fill="none" aria-hidden="true">
          <path
            d="M 24 28 C 24 18, 64 18, 64 36 C 64 46, 24 46, 24 56 C 24 70, 64 70, 64 60"
            stroke="#c5f04a"
            stroke-width="4.5"
            stroke-linecap="round"
            fill="none"
          />
          <circle cx="64" cy="28" r="4.5" fill="#c5f04a" />
        </svg>
        <h1 id="wizard-title" class="brand-wordmark">
          sobr<em>.</em>ia
        </h1>
      </div>

      <p class="tagline">Mesurez la sobriété de votre IA générative.</p>
      <p class="mission">Une mesure scientifique, accessible à tout le monde.</p>

      <button
        type="button"
        class="btn-primary splash-cta"
        onclick={() => {
          cancelSplashTimer();
          step = 2;
          void focusFirst('#step2');
        }}
        data-autofocus
      >
        Continuer
        <ArrowRight size={16} strokeWidth={2} />
      </button>

      <div class="splash-foot">
        <Lock size={11} strokeWidth={1.8} />
        100 % local · aucune donnée envoyée
      </div>
    </section>
  {/if}

  <!-- ╭───── ÉTAPE 2 — PERSONA PICKER ─────────────────────────────╮ -->
  {#if step === 2}
    <section class="persona-stage" id="step2">
      <header class="step-head">
        <span class="eyebrow"><span class="pulse-dot" aria-hidden="true"></span> Étape 2 sur 4</span
        >
        <h2 class="display">Vous êtes…</h2>
        <p class="step-sub">
          Sobr.ia sert cinq publics. Choisissez celui qui vous ressemble pour démarrer avec un
          ensemble pertinent — vous pourrez tout modifier ensuite.
        </p>
      </header>

      <ul class="persona-grid">
        {#each ALL_PERSONAS as p, i (p)}
          <li>
            <button
              type="button"
              class="persona-card"
              onclick={() => pickPersona(p)}
              style="--i:{i}"
              data-autofocus={i === 0 ? '' : undefined}
              data-persona={p}
            >
              <span class="persona-emoji" aria-hidden="true">{personaEmoji(p)}</span>
              <span class="persona-label">{personaLabel(p)}</span>
              <span class="persona-tagline">{personaTagline(p)}</span>
              <span class="persona-arrow" aria-hidden="true">
                <ArrowRight size={14} strokeWidth={1.8} />
              </span>
            </button>
          </li>
        {/each}
      </ul>

      <button type="button" class="link-discrete" onclick={skipPersona}>
        Je préfère choisir à la carte
        <ArrowRight size={11} strokeWidth={1.8} />
      </button>
    </section>
  {/if}

  <!-- ╭───── ÉTAPE 3 — BUNDLE ─────────────────────────────────────╮ -->
  {#if step === 3}
    <section class="bundle-stage" id="step3">
      <header class="step-head">
        <span class="eyebrow"><span class="pulse-dot" aria-hidden="true"></span> Étape 3 sur 4</span
        >
        <h2 class="display">Voici votre première sélection</h2>
        <p class="step-sub">
          {#if persona}
            Bundle <strong>{personaLabel(persona)}</strong>. Vous pourrez modifier cette liste à
            tout moment dans Paramètres.
          {:else}
            Composez votre atelier — cochez les modules qui vous intéressent.
          {/if}
        </p>
        <p class="counter mono">
          {chosenCount} module{chosenCount > 1 ? 's' : ''} sélectionné{chosenCount > 1 ? 's' : ''}
        </p>
      </header>

      <fieldset class="modules-fieldset">
        <legend class="visually-hidden">Modules pré-cochés</legend>
        <ul class="modules-grid">
          {#each inBundle as m, i (m)}
            <li>
              <label class="module-row" data-checked={chosen.has(m)} style="--i:{i}">
                <input
                  type="checkbox"
                  checked={chosen.has(m)}
                  onchange={() => toggleModule(m)}
                  data-autofocus={i === 0 ? '' : undefined}
                  data-module={m}
                />
                <span class="check-box" aria-hidden="true">
                  {#if chosen.has(m)}
                    <Check size={12} strokeWidth={2.5} />
                  {/if}
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
      </fieldset>

      {#if moreModules.length > 0}
        <button
          type="button"
          class="more-toggle"
          onclick={() => (showMore = !showMore)}
          aria-expanded={showMore}
          aria-controls="more-modules"
        >
          {#if showMore}
            <ChevronUp size={14} strokeWidth={2} />
            Masquer les autres modules
          {:else}
            <ChevronDown size={14} strokeWidth={2} />
            + Plus de modules disponibles
            <span class="more-count">({moreModules.length})</span>
          {/if}
        </button>

        {#if showMore}
          <fieldset id="more-modules" class="modules-fieldset more">
            <legend class="visually-hidden">Modules supplémentaires</legend>
            <ul class="modules-grid">
              {#each moreModules as m, i (m)}
                <li>
                  <label class="module-row" data-checked={chosen.has(m)} style="--i:{i}">
                    <input
                      type="checkbox"
                      checked={chosen.has(m)}
                      onchange={() => toggleModule(m)}
                      data-module={m}
                    />
                    <span class="check-box" aria-hidden="true">
                      {#if chosen.has(m)}
                        <Check size={12} strokeWidth={2.5} />
                      {/if}
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
          </fieldset>
        {/if}
      {/if}

      <div class="actions">
        <button type="button" class="btn-ghost" onclick={back}>Précédent</button>
        <button
          type="button"
          class="btn-primary"
          onclick={() => {
            step = 4;
            void focusFirst('#step4');
          }}
          disabled={chosenCount === 0}
        >
          Continuer
          <ArrowRight size={16} strokeWidth={2} />
        </button>
      </div>
    </section>
  {/if}

  <!-- ╭───── ÉTAPE 4 — READY ───────────────────────────────────────╮ -->
  {#if step === 4}
    <section class="ready-stage" id="step4">
      <header class="step-head">
        <span class="eyebrow"><span class="pulse-dot" aria-hidden="true"></span> Étape 4 sur 4</span
        >
        <h2 class="display">C'est parti.</h2>
        <p class="step-sub">
          Sobr.ia est paré. À l'écran suivant, l'atelier d'estimation vous attend : choisissez un
          modèle, écrivez 50 à 200 tokens, cliquez « Estimer l'impact ». Le moteur Monte-Carlo fera
          10 000 tirages pour vous donner CO₂eq, énergie, eau et métaux avec leur intervalle P5-P95.
        </p>
      </header>

      <!-- Trois mini-cartes résumant ce qui attend l'utilisateur — pas de
           mock UI interactif (le brief demandait un tooltip sur M1 réel,
           ré-implémenté sur la route `/` au prochain ralliement). -->
      <ul class="ready-cards">
        <li class="ready-card">
          <span class="ready-ico" aria-hidden="true"><Sparkles size={20} strokeWidth={1.6} /></span>
          <span class="ready-title">Premier prompt</span>
          <span class="ready-sub">Modèle + texte + « Estimer ». Environ 2 secondes.</span>
        </li>
        <li class="ready-card">
          <span class="ready-ico" aria-hidden="true"><Zap size={20} strokeWidth={1.6} /></span>
          <span class="ready-title">
            {chosenCount} module{chosenCount > 1 ? 's' : ''} actif{chosenCount > 1 ? 's' : ''}
          </span>
          <span class="ready-sub"
            >Visibles dans le rail. Modifiables dans Paramètres à tout moment.</span
          >
        </li>
        <li class="ready-card">
          <span class="ready-ico" aria-hidden="true"><Check size={20} strokeWidth={2} /></span>
          <span class="ready-title">100 % local</span>
          <span class="ready-sub">Aucune donnée envoyée vers un serveur tiers.</span>
        </li>
      </ul>

      {#if error}
        <div class="error-banner" role="alert">
          <AlertTriangle size={16} strokeWidth={1.8} />
          <div>
            <strong>Échec de l'enregistrement</strong>
            <span>{error.message}</span>
          </div>
        </div>
      {/if}

      <div class="actions">
        <button
          type="button"
          class="link-discrete"
          onclick={finishOnboarding}
          disabled={saving}
          data-action="skip-step-4"
        >
          Passer cette étape
        </button>
        <button
          type="button"
          class="btn-primary"
          onclick={finishOnboarding}
          disabled={saving}
          data-autofocus
          data-action="finish"
        >
          {saving ? 'Enregistrement…' : "Ouvrir l'atelier"}
          {#if !saving}
            <ArrowRight size={16} strokeWidth={2} />
          {/if}
        </button>
      </div>
    </section>
  {/if}
</main>

<style>
  /* ─── Décoration ambient (clone allégé du layout) ───────────────────── */
  .amb {
    position: fixed;
    inset: 0;
    z-index: 0;
    pointer-events: none;
    overflow: hidden;
  }
  .amb::before {
    content: '';
    position: absolute;
    inset: -10%;
    background:
      radial-gradient(ellipse 900px 600px at 78% 18%, rgba(197, 240, 74, 0.13), transparent 60%),
      radial-gradient(ellipse 700px 600px at 18% 88%, rgba(126, 182, 255, 0.07), transparent 65%);
    filter: blur(28px);
  }
  .amb::after {
    content: '';
    position: absolute;
    inset: 0;
    background-image: url("data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='200' height='200'><filter id='n'><feTurbulence type='fractalNoise' baseFrequency='0.9' numOctaves='2' stitchTiles='stitch'/><feColorMatrix values='0 0 0 0 1, 0 0 0 0 1, 0 0 0 0 1, 0 0 0 0.05 0'/></filter><rect width='100%25' height='100%25' filter='url(%23n)'/></svg>");
    opacity: 0.6;
    mix-blend-mode: overlay;
  }
  .topo {
    position: fixed;
    top: -60px;
    right: -120px;
    width: 640px;
    height: 640px;
    z-index: 0;
    opacity: 0.1;
    pointer-events: none;
    animation: drift 80s linear infinite;
  }
  @keyframes drift {
    from {
      transform: rotate(0);
    }
    to {
      transform: rotate(360deg);
    }
  }

  /* ─── Progression ───────────────────────────────────────────────────── */
  .progress {
    position: fixed;
    top: 28px;
    right: 32px;
    display: flex;
    align-items: center;
    gap: 8px;
    z-index: 5;
  }
  .dot {
    width: 28px;
    height: 4px;
    background: var(--border);
    border-radius: 2px;
    transition: all var(--dur-slow) var(--ease);
  }
  .dot.current {
    background: var(--lime);
    box-shadow: 0 0 12px var(--lime-glow);
  }
  .dot.done {
    background: var(--lime-deep);
  }

  /* ─── Wizard shell ──────────────────────────────────────────────────── */
  /* Plein écran scrollable. Pas de centrage vertical global (sinon les
     étapes hautes comme l'étape 3 « Bundle » avec ses 24 modules ont leur
     bas hors viewport et l'utilisateur ne peut pas remonter au CTA).
     Le centrage vertical du splash est géré localement dans .splash. */
  .wizard {
    position: relative;
    z-index: 1;
    min-height: 100vh;
    height: 100vh;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 72px 32px 64px;
  }
  /* Chaque section prend la largeur dispo et défile naturellement. */
  .wizard > section {
    width: 100%;
    flex: 0 0 auto;
  }

  /* ─── Typo / atomes partagés ────────────────────────────────────────── */
  .display {
    font: 400 56px/1.05 var(--font-display);
    font-style: italic;
    letter-spacing: -0.02em;
    color: var(--ivory);
    margin: 0;
  }
  .eyebrow {
    font: 500 11px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.16em;
    color: var(--ivory-3);
    display: inline-flex;
    align-items: center;
    gap: 10px;
  }
  .pulse-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--lime);
    box-shadow: 0 0 10px var(--lime-glow);
  }
  .visually-hidden {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    border: 0;
  }

  /* ─── Boutons ───────────────────────────────────────────────────────── */
  .btn-primary {
    display: inline-flex;
    align-items: center;
    gap: 10px;
    padding: 14px 26px;
    background: var(--lime);
    color: var(--ink);
    border: none;
    border-radius: var(--radius-pill);
    font: 600 14px/1 var(--font-ui);
    letter-spacing: 0.01em;
    cursor: pointer;
    transition: all var(--dur-base) var(--ease);
    box-shadow: var(--glow-lime);
  }
  .btn-primary:hover:not(:disabled) {
    transform: translateY(-2px);
    box-shadow:
      0 0 0 5px rgba(197, 240, 74, 0.16),
      0 14px 36px -8px rgba(197, 240, 74, 0.55);
  }
  .btn-primary:disabled {
    opacity: 0.55;
    cursor: not-allowed;
    box-shadow: none;
  }
  .btn-ghost {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 12px 22px;
    background: transparent;
    color: var(--ivory-2);
    border: 1px solid var(--border);
    border-radius: var(--radius-pill);
    font: 500 13px/1 var(--font-ui);
    cursor: pointer;
    transition: all var(--dur-base) var(--ease);
  }
  .btn-ghost:hover {
    border-color: var(--border-hi);
    color: var(--ivory);
    background: var(--surface);
  }
  .link-discrete {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    margin-top: 28px;
    background: transparent;
    border: none;
    color: var(--ivory-3);
    font: 400 13px/1 var(--font-ui);
    cursor: pointer;
    padding: 8px 4px;
    border-bottom: 1px dashed var(--border-hi);
    transition: color var(--dur-base) var(--ease);
  }
  .link-discrete:hover:not(:disabled) {
    color: var(--lime);
    border-bottom-color: var(--lime);
  }
  .link-discrete:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  /* ─── ÉTAPE 1 — Splash ──────────────────────────────────────────────── */
  /* Splash : centré verticalement dans le viewport (contenu court). */
  .splash {
    min-height: calc(100vh - 136px);
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
    gap: 8px;
    animation: rise 600ms var(--ease) backwards;
  }
  .brand-stage {
    display: flex;
    align-items: center;
    gap: 22px;
    margin-bottom: 26px;
    position: relative;
  }
  .brand-stage::before {
    content: '';
    position: absolute;
    inset: -34px -60px;
    background: radial-gradient(ellipse, rgba(197, 240, 74, 0.18), transparent 70%);
    filter: blur(28px);
    z-index: -1;
  }
  .brand-logo {
    width: 88px;
    height: 88px;
    animation: breath 4s ease-in-out infinite;
  }
  @keyframes breath {
    0%,
    100% {
      transform: scale(1);
    }
    50% {
      transform: scale(1.05);
    }
  }
  .brand-wordmark {
    font: 400 64px/1 var(--font-display);
    color: var(--ivory);
    letter-spacing: -0.03em;
    margin: 0;
  }
  .brand-wordmark em {
    font-style: italic;
    color: var(--lime);
  }
  .tagline {
    font: 400 22px/1.3 var(--font-display);
    font-style: italic;
    color: var(--ivory-2);
    margin: 0;
    max-width: 560px;
  }
  .mission {
    font: 400 14px/1.5 var(--font-ui);
    color: var(--ivory-3);
    margin: 4px 0 38px;
    max-width: 480px;
  }
  .splash-cta {
    animation: rise 700ms var(--ease) 200ms backwards;
  }
  .splash-foot {
    margin-top: 36px;
    display: inline-flex;
    align-items: center;
    gap: 7px;
    color: var(--ivory-4);
    font: 500 11px/1 var(--font-mono);
    letter-spacing: 0.06em;
  }

  /* ─── ÉTAPE 2 — Persona ─────────────────────────────────────────────── */
  .persona-stage {
    width: 100%;
    max-width: 1080px;
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    animation: rise 500ms var(--ease) backwards;
  }
  .step-head {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 14px;
    margin-bottom: 40px;
    max-width: 640px;
  }
  .step-sub {
    font: 400 15px/1.55 var(--font-ui);
    color: var(--ivory-3);
    margin: 4px 0 0;
  }
  .step-sub strong {
    color: var(--lime);
    font-weight: 500;
  }
  .persona-grid {
    list-style: none;
    padding: 0;
    margin: 0;
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
    gap: 14px;
    width: 100%;
  }
  .persona-card {
    position: relative;
    display: grid;
    grid-template-rows: auto auto 1fr auto;
    grid-template-columns: 1fr auto;
    gap: 6px 14px;
    padding: 22px 22px 20px;
    background: linear-gradient(155deg, rgba(255, 255, 255, 0.035), rgba(255, 255, 255, 0.005));
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    text-align: left;
    cursor: pointer;
    font: inherit;
    color: inherit;
    transition: all 350ms var(--ease-spring);
    animation: rise 500ms var(--ease) calc(80ms * var(--i, 0)) backwards;
  }
  .persona-card:hover,
  .persona-card:focus-visible {
    transform: translateY(-4px);
    border-color: rgba(197, 240, 74, 0.5);
    background: linear-gradient(155deg, rgba(197, 240, 74, 0.07), rgba(255, 255, 255, 0.01));
    box-shadow: var(--shadow-pop);
    outline: none;
  }
  .persona-card:hover .persona-arrow,
  .persona-card:focus-visible .persona-arrow {
    color: var(--lime);
    transform: translateX(4px);
  }
  .persona-emoji {
    grid-row: 1;
    grid-column: 1 / -1;
    font-size: 26px;
    line-height: 1;
    margin-bottom: 6px;
  }
  .persona-label {
    grid-row: 2;
    grid-column: 1;
    font: 400 22px/1.2 var(--font-display);
    font-style: italic;
    color: var(--ivory);
    letter-spacing: -0.01em;
  }
  .persona-tagline {
    grid-row: 3;
    grid-column: 1 / -1;
    font: 400 13px/1.5 var(--font-ui);
    color: var(--ivory-3);
    margin-top: 4px;
  }
  .persona-arrow {
    grid-row: 2;
    grid-column: 2;
    color: var(--ivory-4);
    transition: all var(--dur-base) var(--ease);
    display: inline-flex;
    align-items: center;
  }

  /* ─── ÉTAPE 3 — Bundle ──────────────────────────────────────────────── */
  .bundle-stage {
    width: 100%;
    max-width: 880px;
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    animation: rise 500ms var(--ease) backwards;
  }
  .counter {
    font-size: 12px;
    color: var(--lime);
    background: var(--lime-soft);
    border: 1px solid rgba(197, 240, 74, 0.28);
    border-radius: var(--radius-pill);
    padding: 5px 12px;
  }
  .modules-fieldset {
    width: 100%;
    border: none;
    padding: 0;
    margin: 0 0 16px;
  }
  .modules-fieldset.more {
    margin-top: 12px;
    padding-top: 16px;
    border-top: 1px dashed var(--border);
  }
  .modules-grid {
    list-style: none;
    padding: 0;
    margin: 0;
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(310px, 1fr));
    gap: 8px;
  }
  .module-row {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 14px;
    padding: 14px 16px;
    background: rgba(255, 255, 255, 0.02);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: all var(--dur-base) var(--ease);
    text-align: left;
    animation: rise 400ms var(--ease) calc(28ms * var(--i, 0)) backwards;
  }
  .module-row:hover {
    border-color: var(--border-hi);
    background: var(--surface);
  }
  .module-row[data-checked='true'] {
    border-color: rgba(197, 240, 74, 0.35);
    background: rgba(197, 240, 74, 0.04);
  }
  .module-row input[type='checkbox'] {
    position: absolute;
    opacity: 0;
    width: 1px;
    height: 1px;
    pointer-events: none;
  }
  .check-box {
    width: 18px;
    height: 18px;
    align-self: start;
    margin-top: 2px;
    background: var(--ink-2);
    border: 1.5px solid var(--border-hi);
    border-radius: 5px;
    display: grid;
    place-items: center;
    color: var(--ink);
    transition: all var(--dur-base) var(--ease);
  }
  .module-row[data-checked='true'] .check-box {
    background: var(--lime);
    border-color: var(--lime);
  }
  .module-row input:focus-visible + .check-box {
    box-shadow: 0 0 0 3px rgba(197, 240, 74, 0.3);
  }
  .module-body {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 8px 10px;
    align-items: baseline;
  }
  .module-id {
    font-size: 11px;
    color: var(--ivory-4);
    letter-spacing: 0.08em;
    grid-row: 1;
    grid-column: 1;
  }
  .module-label {
    grid-row: 1;
    grid-column: 2;
    font: 500 14px/1.25 var(--font-ui);
    color: var(--ivory);
  }
  .module-row[data-checked='true'] .module-label {
    color: var(--lime);
  }
  .module-desc {
    grid-row: 2;
    grid-column: 1 / -1;
    margin-top: 4px;
    font: 400 12px/1.45 var(--font-ui);
    color: var(--ivory-3);
  }
  .more-toggle {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    background: transparent;
    border: 1px dashed var(--border-hi);
    color: var(--ivory-2);
    border-radius: var(--radius-pill);
    padding: 9px 18px;
    font: 500 12px/1 var(--font-ui);
    cursor: pointer;
    transition: all var(--dur-base) var(--ease);
    margin-top: 16px;
  }
  .more-toggle:hover {
    border-color: var(--lime);
    color: var(--lime);
  }
  .more-count {
    color: var(--ivory-4);
    font-family: var(--font-mono);
    font-size: 11px;
  }
  .actions {
    display: flex;
    align-items: center;
    gap: 18px;
    margin-top: 36px;
    flex-wrap: wrap;
    justify-content: center;
  }

  /* ─── ÉTAPE 4 — Ready ──────────────────────────────────────────────── */
  .ready-stage {
    width: 100%;
    max-width: 820px;
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    animation: rise 500ms var(--ease) backwards;
  }
  .ready-cards {
    list-style: none;
    padding: 0;
    margin: 28px 0 0;
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
    gap: 14px;
    width: 100%;
  }
  .ready-card {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 8px;
    padding: 18px 20px;
    background: linear-gradient(155deg, rgba(255, 255, 255, 0.035), rgba(255, 255, 255, 0.005));
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    text-align: left;
  }
  .ready-ico {
    display: inline-grid;
    place-items: center;
    width: 34px;
    height: 34px;
    border-radius: var(--radius-sm);
    color: var(--lime);
    background: var(--lime-soft);
    border: 1px solid rgba(197, 240, 74, 0.28);
  }
  .ready-title {
    font: 500 14px/1.25 var(--font-ui);
    color: var(--ivory);
  }
  .ready-sub {
    font: 400 12px/1.45 var(--font-ui);
    color: var(--ivory-3);
  }

  .error-banner {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    margin-top: 22px;
    padding: 12px 16px;
    background: rgba(240, 108, 90, 0.08);
    border: 1px solid rgba(240, 108, 90, 0.3);
    color: var(--coral);
    border-radius: var(--radius-md);
    font: 400 13px/1.45 var(--font-ui);
    text-align: left;
    width: 100%;
  }
  .error-banner strong {
    display: block;
    color: var(--ivory);
    font-weight: 600;
  }
  .error-banner span {
    color: var(--ivory-2);
  }

  /* ─── Reveal ─────────────────────────────────────────────────────────── */
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

  /* ─── Responsive ─────────────────────────────────────────────────────── */
  @media (max-width: 720px) {
    .wizard {
      padding: 60px 18px 40px;
    }
    .display {
      font-size: 40px;
    }
    .brand-wordmark {
      font-size: 48px;
    }
    .brand-logo {
      width: 64px;
      height: 64px;
    }
    .tagline {
      font-size: 18px;
    }
    .progress {
      top: 18px;
      right: 18px;
    }
    .dot {
      width: 20px;
    }
  }
</style>
