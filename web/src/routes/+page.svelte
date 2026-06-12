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
    Scale,
    Settings2,
    Layers,
    BarChart3,
    Target,
    Sparkles,
    X,
    ArrowRight
  } from '@lucide/svelte';
  import {
    estimateForComparison,
    estimatePrompt,
    isBackendAvailable,
    listDatacenters,
    listMethodologies,
    listModels,
    SobriaIpcError,
    type DatacenterSummaryDto,
    type EmpreinteMethod,
    type EquivalentDto,
    type ContextOverhead,
    type EstimationRequestDto,
    type EstimationResultDto,
    type InputModality,
    type IpcErrorCode,
    type MethodologyInfoDto,
    type ModelPresetDto
  } from '$lib/api';
  import { preferences, moduleLabel, type ModuleId } from '$lib/preferences';
  import Composer from '$lib/components/Composer.svelte';
  import ModalitiesPanel from '$lib/components/ModalitiesPanel.svelte';
  import ReduceSuggestions from '$lib/components/ReduceSuggestions.svelte';
  import ResultBlock from '$lib/components/ResultBlock.svelte';
  import HypothesisBlock from '$lib/components/HypothesisBlock.svelte';
  import { tick } from 'svelte';

  // ─── Garde de route M1 (C10 — ADR-0010) ──────────────────────────
  // M1 fait partie de tous les bundles persona par défaut. Mais
  // l'utilisateur peut le désactiver manuellement dans /parametres.
  const MODULE_ID: ModuleId = 'm1';
  $effect(() => {
    if (
      $preferences.loaded &&
      !$preferences.enabled_modules.includes(MODULE_ID) &&
      typeof window !== 'undefined' &&
      !window.location.search.includes('disabled=')
    ) {
      // Cas extrême : l'utilisateur a manuellement décoché M1 dans
      // /parametres. On affiche le bandeau via l'URL `?disabled=m1`.
      window.location.replace('/?disabled=' + MODULE_ID);
    }
  });

  // ─── Bandeau « module désactivé » : ?disabled=mXX ───────────────
  // L'URL n'est posée que par la garde de route d'un module désactivé
  // (cf. simuler/+page.svelte). On affiche le bandeau dès que le param
  // est présent et qu'il référence un ID de module connu.
  const KNOWN_MODULE_IDS: ReadonlySet<ModuleId> = new Set([
    'm1',
    'm2',
    'm3',
    'm5',
    'm6',
    'm7',
    'm8',
    'm9',
    'm10',
    'm11',
    'm12',
    'm13',
    'm14',
    'm15',
    'm16',
    'm17',
    'm18',
    'm19',
    'm20',
    'm21',
    'm22',
    'm23',
    'm24',
    'm25'
  ]);
  let disabledModuleId = $state<ModuleId | null>(null);
  $effect(() => {
    if (typeof window === 'undefined') return;
    const raw = new URLSearchParams(window.location.search).get('disabled');
    if (raw && KNOWN_MODULE_IDS.has(raw as ModuleId)) {
      disabledModuleId = raw as ModuleId;
    } else {
      disabledModuleId = null;
    }
  });

  // ─── State ───────────────────────────────────────────────────────────
  let models = $state<ModelPresetDto[]>([]);
  let selectedModelId = $state('');
  let prompt = $state(
    'Écris-moi un résumé de 500 mots sur la photosynthèse, accessible à un lycéen, en distinguant la phase claire et la phase sombre.'
  );
  let tokensOut = $state(720);

  // ─── C34.4 — Modalités + overhead ──────────────────────────────────
  // Vide par défaut → équivalent à Text + zéro overhead (compat v0.8.x).
  // Le ModalitiesPanel pré-remplit overhead.system_prompt_tokens depuis
  // preset.default_context_overhead_tokens à chaque changement de modèle.
  let modalities = $state<InputModality[]>([]);
  let overhead = $state<ContextOverhead>({
    system_prompt_tokens: 0,
    tools_definition_tokens: 0,
    memory_tokens: 0,
    thinking_tokens_p50: 0
  });
  const selectedModel = $derived(models.find((m) => m.id === selectedModelId) ?? null);

  // ─── C25 — Datacenters (M12) ─────────────────────────────────────────
  // Le catalogue est chargé une fois au bootstrap via IPC. Le picker
  // (cf. Composer.svelte → DatacenterPicker) bind ce state à `selected`.
  // Pré-remplissage : si l'utilisateur a un `default_datacenter_id` dans
  // ses préférences, on l'applique automatiquement au démarrage.
  let datacenters = $state<DatacenterSummaryDto[]>([]);
  let selectedDatacenter = $state<DatacenterSummaryDto | null>(null);

  let result = $state<EstimationResultDto | null>(null);
  let loading = $state(false);
  let bootstrapping = $state(true);
  let error = $state<{ code: IpcErrorCode; message: string } | null>(null);

  // ─── C24 — Catalogue de méthodologies + résultats "Voir aussi" ─────
  // Le catalogue est chargé une fois au bootstrap (lecture statique du
  // registry compile-time). Les résultats "voir aussi" sont rafraîchis
  // chaque fois que `result` change : un appel IPC supplémentaire est
  // déclenché pour chaque méthodologie cochée dans
  // `$preferences.also_show_methods`, avec `method` en surcharge.
  let methodologies = $state<MethodologyInfoDto[]>([]);
  type AlsoShowEntry = {
    method: EmpreinteMethod;
    state: 'loading' | 'ok' | 'error';
    result?: EstimationResultDto;
    errorMessage?: string;
  };
  let alsoShowResults = $state<AlsoShowEntry[]>([]);

  // Ancre pour le scroll smooth post-estimation (cf. submitEstimation).
  let resultAnchor: HTMLDivElement | undefined = $state();

  const backendAvailable = $derived(isBackendAvailable());

  // ─── C32.2 — Bannière « Et après ? » post-premier-prompt ─────────────
  // Affichée après le 1er résultat valide, ferme-able définitivement via
  // localStorage. Répond au finding #8 de l'audit produit C32.0 :
  // « Pas de fil narratif post-premier-prompt ».
  const NARRATIVE_BANNER_KEY = 'sobria_narrative_banner_dismissed';
  let narrativeBannerDismissed = $state(false);
  $effect(() => {
    if (typeof window === 'undefined') return;
    try {
      narrativeBannerDismissed = window.localStorage.getItem(NARRATIVE_BANNER_KEY) === 'true';
    } catch {
      // localStorage indisponible : on garde la bannière visible (mode dégradé).
      narrativeBannerDismissed = false;
    }
  });
  function dismissNarrativeBanner() {
    narrativeBannerDismissed = true;
    try {
      window.localStorage.setItem(NARRATIVE_BANNER_KEY, 'true');
    } catch {
      // ignore
    }
  }

  // ─── Bootstrap : on charge les modèles via IPC réel ──────────────────
  $effect(() => {
    void (async () => {
      if (!backendAvailable) {
        bootstrapping = false;
        error = {
          code: 'tauri_unavailable',
          message:
            "Cette fonctionnalité nécessite l'application de bureau Sobr.ia. La démo web présente des données d'exemple sur : Estimer, Comparer, Bibliothèque de modèles, Datacenters et Tableau de bord."
        };
        return;
      }
      try {
        const [list, methods] = await Promise.all([listModels(), listMethodologies()]);
        methodologies = methods;
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
        // C41 — défaut : un modèle courant et sobre (GPT-4o mini est
        // deprecated et la boucle « Réduire » mérite une vraie première
        // impression). Fallback : premier du catalogue.
        selectedModelId =
          fromUrl ?? list.find((m) => m.id === 'claude-haiku-4-5')?.id ?? list[0]?.id ?? '';

        // C25 — Catalogue datacenters (M12) + pré-remplissage depuis les
        // préférences utilisateur. Non-bloquant : si l'IPC échoue, le
        // picker reste actif avec une liste vide (l'utilisateur peut
        // toujours soumettre sans datacenter — l'estimateur retombe sur
        // les PUE/IF par défaut côté Rust).
        try {
          datacenters = await listDatacenters();
          const def = $preferences.default_datacenter_id;
          if (def && !selectedDatacenter) {
            selectedDatacenter = datacenters.find((d) => d.id === def) ?? null;
          }
        } catch (dcErr) {
          console.warn('listDatacenters failed', dcErr);
        }
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
    // Reset des résultats "voir aussi" pour ne pas afficher d'orphelins
    // de l'estimation précédente.
    alsoShowResults = [];
    try {
      // Même heuristique que Composer (3,3 chars/token FR). Cf. note dans
      // Composer.svelte — tokenizer réel en v0.3 (chantier outillage).
      const tokensIn = Math.max(1, Math.ceil(prompt.length / 3.3));
      // C25 — On n'ajoute `datacenter_id` que s'il y en a un choisi.
      // `exactOptionalPropertyTypes` interdit de passer `undefined`
      // explicitement sur un champ optionnel.
      // C34.3 — modalities et overhead inclus seulement s'ils ont du sens
      // (modalities non-vide ou overhead non-zéro). serde côté Rust gère le
      // default si absent — compat audit ledger v0.8.x.
      const hasOverhead =
        overhead.system_prompt_tokens > 0 ||
        overhead.tools_definition_tokens > 0 ||
        overhead.memory_tokens > 0 ||
        overhead.thinking_tokens_p50 > 0;
      const req: EstimationRequestDto = {
        model_id: selectedModelId,
        tokens_in: tokensIn,
        tokens_out_estimated: Math.max(1, tokensOut),
        ...(selectedDatacenter ? { datacenter_id: selectedDatacenter.id } : {}),
        ...(modalities.length > 0 ? { modalities } : {}),
        ...(hasOverhead ? { overhead } : {})
      };
      const r = await estimatePrompt(req);
      result = r;
      // Scroll smooth vers le bloc résultat, après que le DOM ait été
      // recalculé. Respecte `prefers-reduced-motion` via le param `behavior`.
      await tick();
      const reduced = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
      resultAnchor?.scrollIntoView({
        behavior: reduced ? 'auto' : 'smooth',
        block: 'start'
      });
      // Lance en parallèle les estimations "voir aussi" pour les
      // méthodologies cochées par l'utilisateur (cf. /methodologies).
      void fetchAlsoShowResults(tokensIn, Math.max(1, tokensOut));
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

  // C24 — Pour chaque méthodologie additionnelle activée par l'utilisateur,
  // lance une estimation **éphémère** parallèle (sans écriture dans
  // l'audit ledger via `estimate_for_comparison`). Ces calculs sont
  // exploratoires : le ledger ne doit contenir que la décision
  // principale (cf. polish A — hygiène du ledger).
  // On affiche le statut individuel (loading / ok / error) pour ne pas
  // bloquer l'UX si un moteur secondaire échoue.
  async function fetchAlsoShowResults(tokensIn: number, tokensOutFinal: number) {
    const wanted: EmpreinteMethod[] = ($preferences.also_show_methods ?? []).filter(
      (m) => m !== $preferences.default_method
    );
    if (wanted.length === 0) {
      alsoShowResults = [];
      return;
    }
    alsoShowResults = wanted.map((method) => ({ method, state: 'loading' as const }));
    await Promise.all(
      wanted.map(async (method, idx) => {
        try {
          const r = await estimateForComparison(
            {
              model_id: selectedModelId,
              tokens_in: tokensIn,
              tokens_out_estimated: tokensOutFinal
            },
            method
          );
          alsoShowResults[idx] = { method, state: 'ok', result: r };
        } catch (err) {
          const msg = err instanceof SobriaIpcError ? err.message : 'Erreur inconnue';
          alsoShowResults[idx] = { method, state: 'error', errorMessage: msg };
        }
      })
    );
    // Re-assignation pour forcer la réactivité du tableau muté en place.
    alsoShowResults = [...alsoShowResults];
  }

  function methodInfo(m: EmpreinteMethod): MethodologyInfoDto | undefined {
    return methodologies.find((x) => x.method === m);
  }

  // Indicateur CO₂eq (P50) pour comparaison rapide entre méthodos.
  function co2P50(r: EstimationResultDto): number {
    return r.indicators.find((i) => i.indicator === 'co2eq')?.p50 ?? NaN;
  }

  // Écart relatif % vs résultat principal (positif si la méthodo
  // additionnelle estime plus haut, négatif si plus bas).
  function deltaVsPrimaryPct(secondary: EstimationResultDto): number {
    if (!result) return 0;
    const a = co2P50(result);
    const b = co2P50(secondary);
    if (!Number.isFinite(a) || !Number.isFinite(b) || a === 0) return 0;
    return ((b - a) / a) * 100;
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
    tauri_unavailable: 'Application de bureau requise',
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
      Atelier d'estimation
    </div>
    <h1 class="hero-h1">
      Quel est le poids carbone, <em>réel</em>, d'une seule requête à votre LLM ?
    </h1>
    <p class="hero-sub">
      Saisissez votre prompt, choisissez un modèle. Sobr.ia simule 10 000 trajectoires Monte-Carlo
      pour estimer l'énergie, le CO₂, l'eau et les métaux — avec un intervalle d'incertitude P5–P95.
    </p>
  </section>

  <!-- ─── Bannière module désactivé (C10 — redirection /?disabled=mXX) ── -->
  {#if disabledModuleId}
    <div class="disabled-banner" role="status" data-disabled-module={disabledModuleId}>
      <span class="disabled-ico" aria-hidden="true">
        <Settings2 size={15} strokeWidth={1.8} />
      </span>
      <span class="disabled-body">
        Le module <strong>{disabledModuleId.toUpperCase()} · {moduleLabel(disabledModuleId)}</strong
        >
        n'est pas activé.
        <a href="/parametres">→ Activer dans Paramètres</a>
      </span>
    </div>
  {/if}

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
  {:else if backendAvailable && models.length > 0}
    <Composer
      {models}
      bind:selectedModelId
      bind:prompt
      bind:tokensOut
      estimating={loading}
      onsubmit={submitEstimation}
      {datacenters}
      bind:selectedDatacenter
    />
    <!-- C34.4 — modalités d'input + overhead système -->
    <ModalitiesPanel model={selectedModel} bind:modalities bind:overhead {tokensOut} />
  {/if}

  <!-- ─── Résultat ─────────────────────────────────────────────── -->
  {#if result}
    <div bind:this={resultAnchor} style="scroll-margin-top: 24px"></div>
    <ResultBlock {result} />
    <ReduceSuggestions {result} {models} />

    <!-- Équivalents (tuiles visuelles « Pour mettre cela en perspective »).
         C34 — la ligne texte EquivalenceCarbon a été retirée ici : elle
         doublonnait ces tuiles. Le composant reste utilisé en M15/M25. -->
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

    <!-- C32.2 — Bannière "Et après ?" : suggestions contextuelles après
         le 1er prompt. Dismissible définitivement via localStorage. -->
    {#if !narrativeBannerDismissed}
      <section
        class="narrative-banner"
        aria-label="Suggestions de modules à explorer ensuite"
        data-testid="narrative-banner"
      >
        <header class="nb-head">
          <span class="nb-ico" aria-hidden="true">
            <Sparkles size={16} strokeWidth={1.7} />
          </span>
          <div>
            <h2 class="nb-title">Et après ?</h2>
            <p class="nb-sub">
              Maintenant que vous avez votre première mesure, voici 3 pistes pour creuser votre
              usage.
            </p>
          </div>
          <button
            type="button"
            class="nb-dismiss"
            onclick={dismissNarrativeBanner}
            aria-label="Fermer cette bannière"
            data-action="dismiss-narrative"
          >
            <X size={14} strokeWidth={2} />
          </button>
        </header>
        <div class="nb-grid">
          <a class="nb-card" href="/comparer">
            <span class="nb-card-ico" aria-hidden="true">
              <Scale size={18} strokeWidth={1.6} />
            </span>
            <span class="nb-card-title">Comparer ce modèle à d'autres</span>
            <span class="nb-card-sub">
              Benchmark côte-à-côte sur le même prompt — CO₂, énergie, eau.
            </span>
            <span class="nb-card-arrow" aria-hidden="true">
              <ArrowRight size={12} strokeWidth={2} />
            </span>
          </a>
          <a class="nb-card" href="/suivi">
            <span class="nb-card-ico" aria-hidden="true">
              <BarChart3 size={18} strokeWidth={1.6} />
            </span>
            <span class="nb-card-title">Voir votre usage cumulé</span>
            <span class="nb-card-sub">
              Tableau de bord jour/semaine/mois + équivalences humaines.
            </span>
            <span class="nb-card-arrow" aria-hidden="true">
              <ArrowRight size={12} strokeWidth={2} />
            </span>
          </a>
          <a class="nb-card" href="/eco-budget">
            <span class="nb-card-ico" aria-hidden="true">
              <Target size={18} strokeWidth={1.6} />
            </span>
            <span class="nb-card-title">Fixer un budget mensuel</span>
            <span class="nb-card-sub">
              Eco-budget personnel avec alerte quand vous le dépassez.
            </span>
            <span class="nb-card-arrow" aria-hidden="true">
              <ArrowRight size={12} strokeWidth={2} />
            </span>
          </a>
        </div>
      </section>
    {/if}

    <!-- C24 — Panneau "Voir aussi" (méthodologies additionnelles activées
         dans /methodologies). Visible uniquement si l'utilisateur a coché
         au moins une méthodo en référence. -->
    {#if alsoShowResults.length > 0}
      <section class="also-show" aria-label="Comparaison avec d'autres méthodologies">
        <header class="also-head">
          <span class="also-ico" aria-hidden="true">
            <Layers size={16} strokeWidth={1.7} />
          </span>
          <div>
            <h2 class="also-title">Voir aussi (autres méthodologies)</h2>
            <p class="also-sub">
              Le même prompt, calculé en parallèle avec
              <a class="link" href="/methodologies">d'autres méthodologies</a>
              que vous avez activées. Tous les calculs sont audités.
            </p>
          </div>
        </header>
        <div class="also-grid">
          {#each alsoShowResults as entry (entry.method)}
            {@const info = methodInfo(entry.method)}
            <article class="also-card" data-state={entry.state}>
              <div class="also-card-head">
                <span class="also-card-name">{info?.display_name ?? entry.method}</span>
                {#if entry.state === 'loading'}
                  <span class="also-card-pill pill-loading">…en cours</span>
                {:else if entry.state === 'error'}
                  <span class="also-card-pill pill-error">erreur</span>
                {:else if entry.result}
                  {@const dpct = deltaVsPrimaryPct(entry.result)}
                  <span
                    class="also-card-pill"
                    class:pill-up={dpct > 0.5}
                    class:pill-down={dpct < -0.5}
                    class:pill-flat={Math.abs(dpct) <= 0.5}
                    title="Écart relatif vs méthodologie par défaut"
                  >
                    {dpct >= 0 ? '+' : ''}{fmt(dpct, 2)} %
                  </span>
                {/if}
              </div>
              {#if entry.state === 'ok' && entry.result}
                <dl class="also-card-stats">
                  <div class="stat">
                    <dt>CO₂eq P50</dt>
                    <dd>
                      <strong>{fmt(co2P50(entry.result))}</strong>
                      <span class="unit">
                        {entry.result.indicators.find((i) => i.indicator === 'co2eq')?.unit ?? 'g'}
                      </span>
                    </dd>
                  </div>
                  <div class="stat">
                    <dt>Énergie P50</dt>
                    <dd>
                      <strong>
                        {fmt(
                          entry.result.indicators.find((i) => i.indicator === 'energy')?.p50 ?? NaN
                        )}
                      </strong>
                      <span class="unit">
                        {entry.result.indicators.find((i) => i.indicator === 'energy')?.unit ??
                          'Wh'}
                      </span>
                    </dd>
                  </div>
                </dl>
                <span
                  class="also-card-note"
                  title="Estimation éphémère — non journalisée dans le ledger d'audit. L'estimation principale (méthodo par défaut) est seule journalisée pour préserver l'hygiène du ledger."
                >
                  estimation éphémère (non journalisée)
                </span>
              {:else if entry.state === 'error'}
                <p class="also-card-error">{entry.errorMessage ?? 'Erreur inconnue.'}</p>
              {:else}
                <p class="also-card-loading">Calcul en cours…</p>
              {/if}
              {#if info}
                <p class="also-card-ref">
                  <a
                    class="link"
                    href={info.reference_url}
                    rel="noopener noreferrer"
                    target="_blank"
                  >
                    {info.doi ? `doi:${info.doi}` : 'doc officielle'}
                  </a>
                </p>
              {/if}
            </article>
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

  /* ─── C32.2 — Bannière « Et après ? » post-prompt ─────────────────── */
  .narrative-banner {
    margin-top: 8px;
    padding: 18px 22px;
    background: linear-gradient(155deg, rgba(197, 240, 74, 0.05), rgba(197, 240, 74, 0.01));
    border: 1px solid rgba(197, 240, 74, 0.25);
    border-radius: var(--radius-md);
    display: flex;
    flex-direction: column;
    gap: 14px;
  }
  .nb-head {
    display: flex;
    align-items: flex-start;
    gap: 12px;
  }
  .nb-ico {
    display: grid;
    place-items: center;
    width: 32px;
    height: 32px;
    border-radius: var(--radius-sm);
    background: var(--lime-soft);
    border: 1px solid rgba(197, 240, 74, 0.3);
    color: var(--lime);
    flex-shrink: 0;
  }
  .nb-head > div {
    flex: 1;
    min-width: 0;
  }
  .nb-title {
    font: 400 20px/1.15 var(--font-display);
    font-style: italic;
    color: var(--ivory);
    margin: 0 0 4px;
  }
  .nb-sub {
    font: 400 13px/1.5 var(--font-ui);
    color: var(--ivory-3);
    margin: 0;
  }
  .nb-dismiss {
    background: transparent;
    border: 1px solid var(--border);
    color: var(--ivory-3);
    width: 28px;
    height: 28px;
    border-radius: 50%;
    cursor: pointer;
    display: grid;
    place-items: center;
    flex-shrink: 0;
    transition: all var(--dur-base) var(--ease);
  }
  .nb-dismiss:hover {
    border-color: var(--border-hi);
    color: var(--ivory);
    background: var(--surface);
  }
  .nb-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
    gap: 10px;
  }
  .nb-card {
    position: relative;
    display: grid;
    grid-template-columns: auto 1fr auto;
    grid-template-rows: auto auto;
    gap: 4px 12px;
    padding: 14px 16px;
    background: rgba(0, 0, 0, 0.25);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    text-decoration: none;
    color: inherit;
    transition: all var(--dur-base) var(--ease);
  }
  .nb-card:hover {
    border-color: rgba(197, 240, 74, 0.4);
    transform: translateY(-1px);
  }
  .nb-card-ico {
    grid-row: 1 / 3;
    grid-column: 1;
    align-self: center;
    color: var(--lime);
  }
  .nb-card-title {
    grid-row: 1;
    grid-column: 2;
    font: 500 13px/1.25 var(--font-ui);
    color: var(--ivory);
  }
  .nb-card-sub {
    grid-row: 2;
    grid-column: 2;
    font: 400 12px/1.45 var(--font-ui);
    color: var(--ivory-3);
  }
  .nb-card-arrow {
    grid-row: 1 / 3;
    grid-column: 3;
    align-self: center;
    color: var(--ivory-4);
    transition: all var(--dur-base) var(--ease);
  }
  .nb-card:hover .nb-card-arrow {
    color: var(--lime);
    transform: translateX(3px);
  }

  /* ─── C24 — Panneau "Voir aussi" (méthodologies additionnelles) ─── */
  .also-show {
    margin-top: 4px;
    padding: 18px 22px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    display: flex;
    flex-direction: column;
    gap: 14px;
  }
  .also-head {
    display: flex;
    align-items: flex-start;
    gap: 12px;
  }
  .also-ico {
    display: grid;
    place-items: center;
    width: 32px;
    height: 32px;
    border-radius: var(--radius-sm);
    background: var(--surface-hi);
    border: 1px solid var(--border);
    color: var(--ivory-2);
    flex-shrink: 0;
  }
  .also-title {
    font: 500 14px/1.2 var(--font-ui);
    color: var(--ivory);
    margin: 0;
  }
  .also-sub {
    font: 400 12.5px/1.55 var(--font-ui);
    color: var(--ivory-3);
    margin: 4px 0 0;
  }
  .also-sub .link {
    color: var(--lime);
    text-decoration: none;
  }
  .also-sub .link:hover {
    text-decoration: underline;
  }
  .also-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
    gap: 12px;
  }
  .also-card {
    padding: 12px 14px;
    background: var(--surface-hi);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    display: flex;
    flex-direction: column;
    gap: 8px;
    transition: border-color var(--dur-base) var(--ease);
  }
  .also-card[data-state='loading'] {
    opacity: 0.7;
  }
  .also-card-head {
    display: flex;
    align-items: center;
    gap: 8px;
    justify-content: space-between;
  }
  .also-card-name {
    font: 500 13px/1.2 var(--font-ui);
    color: var(--ivory);
  }
  .also-card-pill {
    font: 500 12px/1 var(--font-mono);
    padding: 3px 8px;
    border-radius: var(--radius-pill);
    background: var(--surface);
    border: 1px solid var(--border-hi);
    color: var(--ivory-3);
    white-space: nowrap;
  }
  .pill-up {
    background: rgba(234, 179, 8, 0.1);
    border-color: rgba(234, 179, 8, 0.25);
    color: rgb(250, 204, 21);
  }
  .pill-down {
    background: rgba(34, 197, 94, 0.1);
    border-color: rgba(34, 197, 94, 0.25);
    color: rgb(74, 222, 128);
  }
  .pill-flat {
    color: var(--ivory-3);
  }
  .pill-loading {
    background: var(--surface);
    border-color: var(--border);
  }
  .pill-error {
    background: rgba(239, 68, 68, 0.1);
    border-color: rgba(239, 68, 68, 0.25);
    color: rgb(248, 113, 113);
  }
  .also-card-stats {
    margin: 0;
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 6px 12px;
  }
  .stat dt {
    font: 500 12px/1 var(--font-ui);
    color: var(--ivory-4);
    text-transform: uppercase;
    letter-spacing: 0.06em;
    margin: 0 0 2px;
  }
  .stat dd {
    margin: 0;
    font: 400 13px/1.2 var(--font-ui);
    color: var(--ivory);
  }
  .stat strong {
    font-weight: 500;
  }
  .stat .unit {
    font: 400 12px/1 var(--font-mono);
    color: var(--ivory-3);
    margin-left: 2px;
  }
  .also-card-note {
    font: 400 10.5px/1.3 var(--font-mono);
    color: var(--ivory-4);
    font-style: italic;
    cursor: help;
  }
  .also-card-error {
    font: 400 12px/1.4 var(--font-ui);
    color: rgb(248, 113, 113);
    margin: 0;
  }
  .also-card-loading {
    font: 400 12px/1.4 var(--font-ui);
    color: var(--ivory-3);
    margin: 0;
  }
  .also-card-ref {
    margin: 4px 0 0;
    font: 400 12px/1 var(--font-mono);
  }
  .also-card-ref .link {
    color: var(--ivory-4);
    text-decoration: none;
  }
  .also-card-ref .link:hover {
    color: var(--lime);
    text-decoration: underline;
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

  /* ─── Bannière module désactivé (coral discret) ──────────────── */
  .disabled-banner {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 16px;
    background: rgba(240, 108, 90, 0.06);
    border: 1px dashed rgba(240, 108, 90, 0.35);
    border-radius: var(--radius-md);
    color: var(--coral);
    margin-bottom: 16px;
    font: 400 13px/1.4 var(--font-ui);
  }
  .disabled-ico {
    display: grid;
    place-items: center;
    width: 28px;
    height: 28px;
    background: rgba(240, 108, 90, 0.08);
    border-radius: var(--radius-sm);
    flex-shrink: 0;
  }
  .disabled-body {
    color: var(--ivory-2);
    flex: 1;
  }
  .disabled-body strong {
    color: var(--coral);
    font-weight: 500;
  }
  .disabled-body a {
    color: var(--lime);
    text-decoration: none;
    border-bottom: 1px dashed rgba(197, 240, 74, 0.4);
    margin-left: 6px;
  }
  .disabled-body a:hover {
    border-bottom-color: var(--lime);
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
    font: 500 12px/1 var(--font-ui);
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
