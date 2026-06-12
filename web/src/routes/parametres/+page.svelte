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
    Folder,
    GraduationCap,
    Code2,
    Building2,
    Landmark,
    Microscope,
    Puzzle,
    Server,
    Users,
    LogOut,
    ExternalLink,
    KeyRound,
    X,
    Sparkles,
    Download,
    Terminal
  } from '@lucide/svelte';
  import {
    getPairingCodeStatus,
    getReferentielStatus,
    isBackendAvailable,
    listMethodologies,
    listPairings,
    metaInfo,
    regeneratePairingCode,
    reloadReferentiel,
    revokePairing,
    SobriaIpcError,
    teamEnroll,
    teamLogout,
    teamPing,
    type EmpreinteMethod,
    type IpcErrorCode,
    type MetaInfo,
    type MethodologyInfoDto,
    type PairingCodeDto,
    type PairingDto,
    type ReferentielStatusDto,
    type TeamHealthResponseDto
  } from '$lib/api';
  import { loadTeam, saveTeamField, teamStore, type TeamMode } from '$lib/team-store';
  import {
    ALL_MODULES,
    ALL_PERSONAS,
    CATEGORY_LABELS,
    defaultModulesFor,
    moduleCategory,
    moduleDescription,
    moduleHref,
    moduleLabel,
    moduleReason,
    personaLabel,
    personaTagline,
    preferences,
    savePreferences,
    type ModuleCategory,
    type ModuleId,
    type Persona
  } from '$lib/preferences';

  // Icônes Lucide par persona (mirror de /onboarding/+page.svelte).
  const PERSONA_ICONS: Record<Persona, typeof GraduationCap> = {
    student: GraduationCap,
    pro_tech: Code2,
    enterprise: Building2,
    public_sector: Landmark,
    researcher: Microscope
  };

  // ─── State ───────────────────────────────────────────────────────────
  let meta = $state<MetaInfo | null>(null);
  let bootstrapping = $state(true);
  let loadError = $state<{ code: IpcErrorCode | string; message: string } | null>(null);
  let saveError = $state<{ code: IpcErrorCode | string; message: string } | null>(null);
  let confirmPersona = $state<Persona | null>(null); // dialog de confirmation
  // Polish H2 — catalogue de méthodologies pour la section dédiée
  let methodologies = $state<MethodologyInfoDto[]>([]);
  // C26.5 — Référentiel Gold (état + reload)
  let referentiel = $state<ReferentielStatusDto | null>(null);
  let reloading = $state(false);
  let reloadMessage = $state<string | null>(null);

  // C27.5 — Extension navigateur (pairing perso)
  let pairingCode = $state<PairingCodeDto | null>(null);
  let pairingCodeNow = $state(Date.now());
  let pairings = $state<PairingDto[]>([]);
  let pairingBusy = $state(false);
  let pairingError = $state<string | null>(null);
  let pairingTicker: ReturnType<typeof setInterval> | null = null;

  // C29.1 — Mode Équipe self-hosted (UI câblée aux 8 IPC team_*)
  // C32.3 — Dialog « Activer Mode Équipe » (3 étapes guidées pour DSI).
  let teamActivateDialogOpen = $state(false);
  let teamUrlDraft = $state('');
  let teamCode = $state('');
  let teamPassword = $state('');
  let teamPasswordConfirm = $state('');
  let teamDisplayName = $state('');
  let teamBusy = $state(false);
  let teamError = $state<string | null>(null);
  let teamPingResult = $state<TeamHealthResponseDto | null>(null);
  let teamPingError = $state<string | null>(null);
  let teamEnrollOk = $state<string | null>(null);
  // Fingerprint éphémère par session : généré à chaque mount, persisté côté
  // Rust après /enroll. Si l'utilisateur logout puis ré-enrôle, il aura un
  // nouveau fingerprint — c'est intentionnel.
  let teamFingerprint = $state('');

  const backendAvailable = $derived(isBackendAvailable());

  $effect(() => {
    void (async () => {
      if (!backendAvailable) {
        bootstrapping = false;
        loadError = {
          code: 'tauri_unavailable',
          message: "Les préférences complètes nécessitent l'application de bureau Sobr.ia."
        };
        return;
      }
      // C41 — bootstrap scindé (cf. C37 §bugs) : les sections couvertes par
      // le mode démo (runtime, méthodologies, référentiel) ne doivent pas
      // être sacrifiées si le pairing/équipe (desktop-only) rejette.
      try {
        const [m, list, ref] = await Promise.all([
          metaInfo(),
          listMethodologies(),
          getReferentielStatus()
        ]);
        meta = m;
        methodologies = list;
        referentiel = ref;
      } catch (err) {
        if (err instanceof SobriaIpcError) {
          loadError = { code: err.code, message: err.message };
        } else {
          loadError = { code: 'internal', message: 'Échec du chargement des paramètres.' };
        }
      }
      // Pairing extension + mode équipe : best-effort, desktop-only.
      try {
        const [code, pairs] = await Promise.all([getPairingCodeStatus(), listPairings()]);
        pairingCode = code;
        pairings = pairs;
      } catch (err) {
        if (err instanceof SobriaIpcError) {
          pairingError = err.message;
        }
      }
      await loadTeam().catch(() => {
        /* échec silencieux : mode équipe optionnel */
      });
      teamUrlDraft = $teamStore.url ?? '';
      bootstrapping = false;
    })();
  });

  onMount(() => {
    // Tick 1s pour faire défiler le compte-à-rebours du code de pairing.
    pairingTicker = setInterval(() => {
      pairingCodeNow = Date.now();
    }, 1000);
    // C29.1 — fingerprint éphémère pour l'enrollment équipe.
    if (typeof crypto !== 'undefined' && typeof crypto.randomUUID === 'function') {
      teamFingerprint = `app-desktop-${crypto.randomUUID()}`;
    } else {
      teamFingerprint = `app-desktop-${Date.now()}-${Math.random().toString(36).slice(2)}`;
    }
    return () => {
      if (pairingTicker) clearInterval(pairingTicker);
    };
  });

  // ─── C27.5 — extension navigateur ────────────────────────────────────
  const pairingSecondsLeft = $derived.by(() => {
    if (!pairingCode) return 0;
    const left = Math.max(
      0,
      Math.floor((new Date(pairingCode.expires_at).getTime() - pairingCodeNow) / 1000)
    );
    return left;
  });
  const pairingActiveCount = $derived(pairings.filter((p) => !p.revoked_at).length);

  async function handleGeneratePairingCode() {
    pairingError = null;
    pairingBusy = true;
    try {
      pairingCode = await regeneratePairingCode();
    } catch (e) {
      pairingError = e instanceof SobriaIpcError ? e.message : 'Échec de la génération du code.';
    } finally {
      pairingBusy = false;
    }
  }

  async function handleRevokePairing(id: string) {
    pairingError = null;
    pairingBusy = true;
    try {
      await revokePairing(id);
      pairings = await listPairings();
    } catch (e) {
      pairingError = e instanceof SobriaIpcError ? e.message : 'Échec de la révocation.';
    } finally {
      pairingBusy = false;
    }
  }

  function formatPairingTimer(seconds: number): string {
    const m = Math.floor(seconds / 60);
    const s = seconds % 60;
    return `${m}:${s.toString().padStart(2, '0')}`;
  }

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
        lang: $preferences.lang,
        default_method: $preferences.default_method,
        also_show_methods: $preferences.also_show_methods,
        default_datacenter_id: $preferences.default_datacenter_id
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
        lang: $preferences.lang,
        default_method: $preferences.default_method,
        also_show_methods: $preferences.also_show_methods,
        default_datacenter_id: $preferences.default_datacenter_id
      });
    } catch (e) {
      saveError = errorOf(e);
    }
  }

  // Parqué v1.1 — la toggle FR/EN est désactivée dans le markup en attendant
  // les traductions ; cette fonction reste prête à être recâblée (cf. ancre
  // « parked-helpers » en fin de script pour la référence qui satisfait
  // `noUnusedLocals`).
  async function setLang(l: 'fr' | 'en') {
    saveError = null;
    try {
      await savePreferences({
        persona: $preferences.persona,
        enabled_modules: $preferences.enabled_modules,
        onboarded: $preferences.onboarded,
        lang: l,
        default_method: $preferences.default_method,
        also_show_methods: $preferences.also_show_methods,
        default_datacenter_id: $preferences.default_datacenter_id
      });
    } catch (e) {
      saveError = errorOf(e);
    }
  }

  // Polish H2 — Méthodologie par défaut (C24).
  // Quand l'user switche, on s'assure que la nouvelle méthodo n'est plus
  // dans also_show_methods (impossible de se comparer à soi-même).
  async function setDefaultMethod(m: EmpreinteMethod) {
    if (m === $preferences.default_method) return;
    saveError = null;
    const filtered = $preferences.also_show_methods.filter((x) => x !== m);
    try {
      await savePreferences({
        persona: $preferences.persona,
        enabled_modules: $preferences.enabled_modules,
        onboarded: $preferences.onboarded,
        lang: $preferences.lang,
        default_method: m,
        also_show_methods: filtered,
        default_datacenter_id: $preferences.default_datacenter_id
      });
    } catch (e) {
      saveError = errorOf(e);
    }
  }

  // Polish H2 — Toggle d'une méthodologie en référence (« Voir aussi »).
  async function toggleAlsoShowMethod(m: EmpreinteMethod) {
    if (m === $preferences.default_method) return;
    saveError = null;
    const isShown = $preferences.also_show_methods.includes(m);
    const next = isShown
      ? $preferences.also_show_methods.filter((x) => x !== m)
      : [...$preferences.also_show_methods, m];
    try {
      await savePreferences({
        persona: $preferences.persona,
        enabled_modules: $preferences.enabled_modules,
        onboarded: $preferences.onboarded,
        lang: $preferences.lang,
        default_method: $preferences.default_method,
        also_show_methods: next,
        default_datacenter_id: $preferences.default_datacenter_id
      });
    } catch (e) {
      saveError = errorOf(e);
    }
  }

  // Parqué — helper de lookup pour la future card de détail méthodologie
  // (tooltip, drawer). Sera consommé par le markup en cours d'itération.
  function methodInfo(m: EmpreinteMethod): MethodologyInfoDto | undefined {
    return methodologies.find((x) => x.method === m);
  }

  async function redoOnboarding() {
    saveError = null;
    try {
      await savePreferences({
        persona: $preferences.persona,
        enabled_modules: $preferences.enabled_modules,
        onboarded: false,
        lang: $preferences.lang,
        default_method: $preferences.default_method,
        also_show_methods: $preferences.also_show_methods,
        default_datacenter_id: $preferences.default_datacenter_id
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
  // Feature I5 — Split modules non-activés en deux groupes pour la
  // discoverability : ceux qu'on peut activer maintenant vs ceux différés
  // v1.1+ (cf. ADR-0011). L'user voit ainsi clairement ce qui existe et
  // ce qui arrive plus tard, plutôt que de chercher en vain dans le rail.
  const disabledAvailable = $derived(disabledModules.filter((id) => moduleHref(id) !== null));
  const disabledDeferred = $derived(disabledModules.filter((id) => moduleHref(id) === null));

  const CATEGORY_ORDER: ModuleCategory[] = [
    'estimation',
    'visualisation',
    'reporting',
    'pedagogie'
  ];

  // ─── C26.5 — Recharger le référentiel Gold ───────────────────────────
  async function handleReloadReferentiel() {
    if (reloading) return;
    reloading = true;
    reloadMessage = null;
    try {
      const result = await reloadReferentiel();
      if (result.status) {
        referentiel = result.status;
      }
      reloadMessage = result.message;
    } catch (e) {
      const err = errorOf(e);
      reloadMessage = err.message;
    } finally {
      reloading = false;
    }
  }

  function shortHash(h: string): string {
    if (!h || h.length < 12) return h;
    return `${h.slice(0, 8)}…${h.slice(-4)}`;
  }

  function formatDate(iso: string): string {
    if (!iso) return '—';
    try {
      const d = new Date(iso);
      return new Intl.DateTimeFormat('fr-FR', {
        dateStyle: 'medium',
        timeStyle: 'short'
      }).format(d);
    } catch {
      return iso;
    }
  }

  // ─── C29.1 — Mode Équipe self-hosted ─────────────────────────────────
  // Tous les handlers passent par le store optimistic + l'IPC sous-jacent.
  // Pas de fallback : si l'IPC échoue, l'erreur est affichée à l'utilisateur.

  function teamErrorMessage(e: unknown): string {
    if (e instanceof SobriaIpcError) return e.message;
    return "Échec de l'opération Mode Équipe.";
  }

  const teamUrlIsValid = $derived.by(() => /^https?:\/\/.+/i.test(teamUrlDraft));
  const teamUrlIsHttps = $derived(teamUrlDraft.startsWith('https://'));
  const teamCodeIsValid = $derived(/^\d{12}$/.test(teamCode));
  const teamPasswordStrong = $derived(teamPassword.length >= 8);
  const teamPasswordsMatch = $derived(teamPassword === teamPasswordConfirm);
  const teamEnrollReady = $derived(
    teamCodeIsValid && teamPasswordStrong && teamPasswordsMatch && $teamStore.url !== null
  );

  async function handleSaveTeamUrl() {
    teamError = null;
    if (!teamUrlIsValid) {
      teamError = 'URL invalide. Format attendu : https://<host>:<port>';
      return;
    }
    teamBusy = true;
    try {
      await saveTeamField('url', teamUrlDraft.trim());
      // Reset des résultats de ping/enroll précédents.
      teamPingResult = null;
      teamPingError = null;
    } catch (e) {
      teamError = teamErrorMessage(e);
    } finally {
      teamBusy = false;
    }
  }

  async function handleTeamPing() {
    teamPingResult = null;
    teamPingError = null;
    teamBusy = true;
    try {
      teamPingResult = await teamPing();
      // Recharger le snapshot : last_seen_at vient d'être mis à jour.
      await loadTeam();
    } catch (e) {
      teamPingError = teamErrorMessage(e);
    } finally {
      teamBusy = false;
    }
  }

  async function handleTeamEnroll() {
    teamError = null;
    teamEnrollOk = null;
    if (!teamEnrollReady) {
      teamError =
        "Vérifiez l'URL, le code (12 chiffres), et le mot de passe (≥ 8 caractères, confirmation identique).";
      return;
    }
    teamBusy = true;
    try {
      const displayName = teamDisplayName.trim() || null;
      const resp = await teamEnroll(teamCode, teamPassword, teamFingerprint, displayName);
      teamEnrollOk = `Enrôlement réussi (user_id ${resp.user_id.slice(0, 12)}…).`;
      // Purge des champs sensibles.
      teamCode = '';
      teamPassword = '';
      teamPasswordConfirm = '';
      teamDisplayName = '';
      await loadTeam();
    } catch (e) {
      teamError = teamErrorMessage(e);
    } finally {
      teamBusy = false;
    }
  }

  async function handleSetTeamMode(mode: TeamMode) {
    teamError = null;
    teamBusy = true;
    try {
      await saveTeamField('mode', mode);
    } catch (e) {
      teamError = teamErrorMessage(e);
    } finally {
      teamBusy = false;
    }
  }

  async function handleToggleAcceptCert(e: Event) {
    teamError = null;
    const target = e.currentTarget as HTMLInputElement;
    teamBusy = true;
    try {
      await saveTeamField('accept_invalid_certs', target.checked);
    } catch (err) {
      teamError = teamErrorMessage(err);
    } finally {
      teamBusy = false;
    }
  }

  async function handleTeamLogout() {
    // Confirm UX explicite — bouton destructif.
    if (!window.confirm('Se déconnecter du serveur équipe ? Vos tokens locaux seront supprimés.')) {
      return;
    }
    teamError = null;
    teamBusy = true;
    try {
      await teamLogout();
      teamPingResult = null;
      teamEnrollOk = null;
      await loadTeam();
    } catch (e) {
      teamError = teamErrorMessage(e);
    } finally {
      teamBusy = false;
    }
  }

  // ─── parked-helpers ──────────────────────────────────────────────────
  // Référence-ancre : `setLang` (toggle FR/EN désactivée jusqu'à v1.1) et
  // `methodInfo` (à câbler dans la card méthodologie) sont déclarés mais
  // pas encore utilisés par le markup. Cette ligne les marque comme « lus »
  // pour `noUnusedLocals` / `@typescript-eslint/no-unused-vars`, sans
  // impact runtime (`void` discard l'expression).
  void [setLang, methodInfo];
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
          {loadError.code === 'tauri_unavailable' ? 'Application de bureau requise' : 'Erreur'}
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
      {@const CurrentIcon = PERSONA_ICONS[currentPersona]}
      <div class="persona-current">
        <span class="persona-icon" aria-hidden="true">
          <CurrentIcon size={22} strokeWidth={1.5} />
        </span>
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
          disabled={!backendAvailable}
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
        {@const OptionIcon = PERSONA_ICONS[p]}
        <li>
          <button
            type="button"
            class="persona-option"
            class:current={isCurrent}
            disabled={!backendAvailable || isCurrent}
            onclick={() => (confirmPersona = p)}
            data-persona={p}
          >
            <span class="persona-icon" aria-hidden="true">
              <OptionIcon size={20} strokeWidth={1.5} />
            </span>
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
                  <label
                    class="module-line"
                    data-checked="true"
                    title={moduleReason(currentPersona, m) ?? ''}
                  >
                    <input
                      type="checkbox"
                      checked
                      onchange={() => toggleModule(m)}
                      disabled={!backendAvailable}
                      data-module={m}
                    />
                    <span class="check-box" aria-hidden="true">
                      <Check size={11} strokeWidth={2.5} />
                    </span>
                    <span class="module-body">
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

  <!-- ╭─── Section 3a : Modules disponibles à activer ────────╮ -->
  <section class="section">
    <header class="section-head">
      <PlusCircle size={16} strokeWidth={1.8} />
      <h2>Modules disponibles</h2>
      <span class="section-hint mono">activables · {disabledAvailable.length} dispo.</span>
    </header>

    {#if disabledAvailable.length === 0}
      <p class="empty">Tous les modules disponibles sont déjà dans votre atelier.</p>
    {:else}
      <ul class="modules-list compact">
        {#each disabledAvailable as m (m)}
          <li>
            <label class="module-line" data-checked="false">
              <input
                type="checkbox"
                checked={false}
                onchange={() => toggleModule(m)}
                disabled={!backendAvailable}
                data-module={m}
              />
              <span class="check-box" aria-hidden="true"></span>
              <span class="module-body">
                <span class="module-label">{moduleLabel(m)}</span>
                <span class="module-desc">{moduleDescription(m)}</span>
              </span>
            </label>
          </li>
        {/each}
      </ul>
    {/if}
  </section>

  <!-- ╭─── Section 3b (Feature I5) : Modules à venir en v1.1+ ────╮ -->
  {#if disabledDeferred.length > 0}
    <section class="section">
      <header class="section-head">
        <PlusCircle size={16} strokeWidth={1.8} />
        <h2>À venir en v1.1+</h2>
        <span class="section-hint mono">{disabledDeferred.length} modules différés · ADR-0011</span>
      </header>
      <p class="section-intro">
        Ces modules ont été <strong>différés à v1.1+</strong> pour focaliser v1.0 (cible candidature data.gouv)
        sur les 13 modules essentiels. Ils restent dans le référentiel et peuvent être activés ; l'écran
        dédié n'est pas encore livré.
      </p>
      <ul class="modules-list compact">
        {#each disabledDeferred as m (m)}
          <li>
            <div class="module-line is-deferred" data-module={m}>
              <span class="check-box deferred" aria-hidden="true">v1.1</span>
              <span class="module-body">
                <span class="module-label">{moduleLabel(m)}</span>
                <span class="module-desc">{moduleDescription(m)}</span>
              </span>
            </div>
          </li>
        {/each}
      </ul>
    </section>
  {/if}

  <!-- ╭─── Polish H2 : Méthodologie scientifique (C24) ──────────╮ -->
  <section class="section">
    <header class="section-head">
      <Layers size={16} strokeWidth={1.8} />
      <h2>Méthodologie scientifique</h2>
      <a class="section-hint mono section-hint-link" href="/methodologies">→ catalogue complet</a>
    </header>
    <p class="section-intro">
      Sobr.ia embarque <strong>plusieurs méthodologies d'estimation d'empreinte LLM</strong>
      au choix. La méthodologie par défaut est utilisée par tous les calculs (M1 Atelier, M3 Comparer,
      M13 Simulateur, M22 Rapport CSRD…). Les méthodologies en référence apparaissent dans le panneau
      « Voir aussi » à côté du résultat principal.
    </p>

    {#if methodologies.length === 0}
      <p class="empty mono">Catalogue indisponible (hors Tauri).</p>
    {:else}
      <div class="method-list" role="list">
        {#each methodologies as m (m.method)}
          {@const isDefault = $preferences.default_method === m.method}
          {@const isRef = $preferences.also_show_methods.includes(m.method)}
          <article
            class="method-row"
            role="listitem"
            data-method={m.method}
            class:is-default={isDefault}
          >
            <div class="method-head">
              <span class="method-name">{m.display_name}</span>
              {#if isDefault}
                <span class="badge-method badge-default">
                  <Check size={11} strokeWidth={2.2} /> par défaut
                </span>
              {/if}
              {#if m.doi}
                <a
                  class="doi mono"
                  href={m.reference_url}
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  doi:{m.doi}
                </a>
              {/if}
            </div>
            <p class="method-desc">{m.short_description}</p>
            <div class="method-actions">
              <button
                type="button"
                class="btn-tiny"
                class:active={isDefault}
                onclick={() => setDefaultMethod(m.method)}
                disabled={!backendAvailable || isDefault}
              >
                {isDefault ? '✓ méthodo par défaut' : 'Définir comme défaut'}
              </button>
              <label class="ref-check">
                <input
                  type="checkbox"
                  checked={isRef}
                  disabled={!backendAvailable || isDefault}
                  onchange={() => toggleAlsoShowMethod(m.method)}
                />
                <span>Afficher en référence (« Voir aussi »)</span>
              </label>
            </div>
          </article>
        {/each}
      </div>
    {/if}
  </section>

  <!-- ╭─── Section 4 : Onboarding + langue ─────────────────────╮ -->
  <section class="section">
    <header class="section-head">
      <RefreshCw size={16} strokeWidth={1.8} />
      <h2>Réinitialiser &amp; langue</h2>
      <span class="section-hint mono">i18n v1.1</span>
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
          disabled={!backendAvailable}
          data-action="redo-onboarding"
        >
          <RefreshCw size={14} strokeWidth={1.8} />
          Refaire l'onboarding
        </button>
      </div>
      <div class="dual-col">
        <h3 class="dual-title">Langue de l'interface</h3>
        <p class="dual-sub">
          La traduction anglaise est prévue en v1.1 (chantier C12 non démarré). Le sélecteur est
          désactivé : l'interface reste en français quel que soit le choix sauvegardé.
        </p>
        <div
          class="lang-toggle is-disabled"
          role="radiogroup"
          aria-label="Langue de l'interface (désactivé, v1.1)"
          title="Le sélecteur sera réactivé en v1.1 quand les chaînes EN seront disponibles."
        >
          <button type="button" class="lang-btn active" disabled role="radio" aria-checked="true">
            FR
          </button>
          <button
            type="button"
            class="lang-btn"
            disabled
            role="radio"
            aria-checked="false"
            aria-disabled="true"
          >
            EN <span class="lang-pending mono">v1.1</span>
          </button>
        </div>
      </div>
    </div>
  </section>

  <!-- ╭─── Référentiel Gold (C26.5 — pipeline médaillon) ──────╮ -->
  <section class="section">
    <header class="section-head">
      <Layers size={16} strokeWidth={1.8} />
      <h2>Référentiel</h2>
      <span class="section-hint mono"
        >data/gold/referentiel.sqlite · `get_referentiel_status` IPC</span
      >
    </header>

    {#if bootstrapping}
      <div class="runtime-skel">Chargement…</div>
    {:else if referentiel}
      {#if referentiel.available}
        <dl class="runtime-grid">
          <div class="runtime-row">
            <dt><Info size={12} strokeWidth={1.8} /> Version</dt>
            <dd class="mono">{referentiel.version}</dd>
          </div>
          <div class="runtime-row">
            <dt><Cpu size={12} strokeWidth={1.8} /> Snapshot</dt>
            <dd class="mono">{formatDate(referentiel.snapshot_date)}</dd>
          </div>
          <div class="runtime-row">
            <dt><Lock size={12} strokeWidth={1.8} /> SHA-256</dt>
            <dd class="mono" title={referentiel.sha256}>{shortHash(referentiel.sha256)}</dd>
          </div>
          <div class="runtime-row">
            <dt><PlugZap size={12} strokeWidth={1.8} /> Sources</dt>
            <dd class="mono">{referentiel.source_count}</dd>
          </div>
          <div class="runtime-row">
            <dt><Layers size={12} strokeWidth={1.8} /> Modèles</dt>
            <dd class="mono">{referentiel.model_count}</dd>
          </div>
          <div class="runtime-row">
            <dt><Folder size={12} strokeWidth={1.8} /> Chemin</dt>
            <dd class="mono break">{referentiel.path}</dd>
          </div>
        </dl>
      {:else}
        <div class="callout callout-warn">
          <AlertTriangle size={14} strokeWidth={1.8} />
          <div>
            <strong>Référentiel Gold indisponible.</strong>
            <p class="callout-msg">{referentiel.message}</p>
            <p class="callout-msg mono">{referentiel.path}</p>
          </div>
        </div>
      {/if}
      <div class="reload-row">
        <button
          type="button"
          class="btn-secondary"
          onclick={handleReloadReferentiel}
          disabled={reloading}
          aria-busy={reloading}
        >
          <RefreshCw size={14} strokeWidth={1.8} />
          {reloading ? 'Rechargement (dvc pull)…' : 'Recharger le référentiel'}
        </button>
        {#if reloadMessage}
          <p class="reload-msg">{reloadMessage}</p>
        {/if}
      </div>
    {/if}

    <p class="section-foot">
      <Hammer size={11} strokeWidth={1.8} />
      Le référentiel Gold est généré par <code>cargo run -p sobria-ingest -- pipeline run</code>
      et versionné via DVC. Voir
      <a href="/methodo">Méthodologie</a> et <code>docs/operations/dvc.md</code>.
    </p>
  </section>

  <!-- ╭─── Extension navigateur (C27.5 — pairing perso) ───────╮ -->
  <section class="section">
    <header class="section-head">
      <Puzzle size={16} strokeWidth={1.8} />
      <h2>Extension navigateur</h2>
      <span class="section-hint mono">pairing perso · <code>com.sobria.bridge</code></span>
    </header>

    {#if bootstrapping}
      <div class="runtime-skel">Chargement…</div>
    {:else}
      <p class="ext-intro">
        Appairez l'extension navigateur Sobr.ia à votre instance locale. L'extension observe vos
        prompts dans Chat&nbsp;LLM (ChatGPT, Claude, Mistral, etc.), calcule l'estimation, et la
        transmet à cette app via le <em>native messaging bridge</em>. Aucune donnée n'est envoyée à
        un serveur distant — tout reste sur votre machine.
      </p>

      <div class="dual ext-grid">
        <!-- Bloc gauche : code de pairing ─────────────────────── -->
        <div class="ext-pane">
          <h3 class="ext-pane-title">Code de pairing</h3>
          {#if pairingCode && pairingSecondsLeft > 0}
            <div class="pairing-code" aria-live="polite">
              {#each pairingCode.code.split('') as digit}
                <span class="pairing-digit">{digit}</span>
              {/each}
            </div>
            <p class="pairing-timer">
              Expire dans <strong>{formatPairingTimer(pairingSecondsLeft)}</strong>
            </p>
            <p class="ext-hint">
              Ouvrez la page <code>Options</code> de l'extension Sobr.ia dans votre navigateur et saisissez
              ce code dans la section « Pairing avec l'app desktop ».
            </p>
          {:else}
            <p class="ext-hint">
              Aucun code en attente. Cliquez sur « Générer un code » pour démarrer l'appairage.
            </p>
          {/if}
          <div class="reload-row">
            <button
              type="button"
              class="btn-secondary"
              onclick={handleGeneratePairingCode}
              disabled={pairingBusy}
              aria-busy={pairingBusy}
            >
              <RefreshCw size={14} strokeWidth={1.8} />
              {pairingCode && pairingSecondsLeft > 0
                ? 'Régénérer un nouveau code'
                : 'Générer un code'}
            </button>
            {#if pairingError}
              <p class="reload-msg" style="color: var(--danger, #ff6b6b)">{pairingError}</p>
            {/if}
          </div>
        </div>

        <!-- Bloc droit : appariements actifs ──────────────────── -->
        <div class="ext-pane">
          <h3 class="ext-pane-title">
            Appariements <span class="ext-count"
              >({pairingActiveCount} actif{pairingActiveCount > 1 ? 's' : ''})</span
            >
          </h3>
          {#if pairings.length === 0}
            <p class="ext-hint">Aucune extension appariée pour le moment.</p>
          {:else}
            <ul class="pairing-list">
              {#each pairings as p (p.id)}
                <li class="pairing-row" class:revoked={!!p.revoked_at}>
                  <div class="pairing-info">
                    <div class="pairing-fp mono">{p.fingerprint}</div>
                    <div class="pairing-meta mono">
                      Créé {formatDate(p.created_at)}
                      {#if p.last_seen_at}
                        · vu {formatDate(p.last_seen_at)}
                      {/if}
                      {#if p.revoked_at}
                        · <span class="pairing-revoked-label">révoqué</span>
                      {/if}
                    </div>
                  </div>
                  {#if !p.revoked_at}
                    <button
                      type="button"
                      class="btn-ghost btn-revoke"
                      onclick={() => handleRevokePairing(p.id)}
                      disabled={pairingBusy}
                      aria-label="Révoquer ce pairing"
                      title="Révoquer ce pairing"
                    >
                      <X size={14} strokeWidth={1.8} />
                    </button>
                  {/if}
                </li>
              {/each}
            </ul>
          {/if}
        </div>
      </div>
    {/if}

    <p class="section-foot">
      <Hammer size={11} strokeWidth={1.8} />
      Binaire local <code>sobria-bridge</code> + extension WebExtension MV3. Voir
      <code>crates/sobria-bridge/README.md</code> et
      <a href="/methodo">ADR-0013</a>.
    </p>
  </section>

  <!-- ╭─── Mode Équipe self-hosted (C28.6 + C29.1) ────────────╮ -->
  <section class="section" data-testid="team-section">
    <header class="section-head">
      <Server size={16} strokeWidth={1.8} />
      <h2>Mode Équipe self-hosted</h2>
      <span class="section-hint mono">8 IPC <code>team_*</code> · ADR-0013 Phase 2</span>
    </header>

    {#if bootstrapping}
      <div class="runtime-skel">Chargement…</div>
    {:else}
      <p class="ext-intro">
        Si votre organisation déploie le binaire <code>sobria-team-aggregator</code>
        (souverain, auto-hébergé, aucun cloud Sobr.ia), vous pouvez y rattacher cette app pour que vos
        estimations remontent dans le dashboard de l'équipe pour le reporting CSRD ou FinOps. L'opt-in
        est explicite : si vous restez en mode <em>Local</em>, rien ne sort de votre machine.
      </p>

      <!-- Bloc Statut ─────────────────────────────────────────────── -->
      <div class="team-status" aria-live="polite">
        <div class="team-status-row">
          <span class="team-status-label">Statut</span>
          {#if !$teamStore.url}
            <span class="team-pill team-pill-mute" data-testid="team-status-pill"
              >Non configuré</span
            >
          {:else if !$teamStore.enrolled}
            <span class="team-pill team-pill-warn" data-testid="team-status-pill"
              >URL OK · non enrôlé</span
            >
          {:else}
            <span class="team-pill team-pill-ok" data-testid="team-status-pill">Connecté</span>
          {/if}
        </div>
        {#if $teamStore.url}
          <div class="team-status-row">
            <span class="team-status-label">Serveur</span>
            <code class="team-mono">{$teamStore.url}</code>
          </div>
        {/if}
        {#if $teamStore.enrolled}
          <div class="team-status-row">
            <span class="team-status-label">User ID</span>
            <code class="team-mono">{$teamStore.user_id}</code>
          </div>
          <div class="team-status-row">
            <span class="team-status-label">Fingerprint</span>
            <code class="team-mono team-truncate" title={$teamStore.fingerprint ?? ''}
              >{$teamStore.fingerprint}</code
            >
          </div>
          <div class="team-status-row">
            <span class="team-status-label">Estimations envoyées</span>
            <code class="team-mono">{$teamStore.estimations_sent}</code>
          </div>
          <div class="team-status-row">
            <span class="team-status-label">Dernière synchro</span>
            <code class="team-mono"
              >{$teamStore.last_seen_at ? formatDate($teamStore.last_seen_at) : 'jamais'}</code
            >
          </div>
        {/if}
      </div>

      <!-- C32.3 — Panneau « Activer Mode Équipe » : guide 5-min pour
           les DSI qui n'ont pas encore déployé le binaire serveur. Visible
           uniquement quand non configuré (URL vide). -->
      {#if !$teamStore.url}
        <div class="team-activate" data-testid="team-activate-panel">
          <div class="team-activate-head">
            <span class="team-activate-ico" aria-hidden="true">
              <Sparkles size={16} strokeWidth={1.7} />
            </span>
            <div>
              <h3 class="team-activate-title">Pas encore de serveur d'équipe ?</h3>
              <p class="team-activate-sub">
                Sobr.ia fournit un binaire Rust standalone que vous déployez vous-même (poste admin,
                NAS, VPS). Aucun cloud Sobr.ia impliqué. Comptez 5 minutes pour démarrer.
              </p>
            </div>
          </div>
          <button
            type="button"
            class="team-activate-btn"
            onclick={() => (teamActivateDialogOpen = true)}
            data-testid="team-activate-btn"
          >
            <Server size={14} strokeWidth={1.8} />
            Activer Mode Équipe (mon entreprise)
          </button>
          <a
            href="https://github.com/BkOff-fr/defis-lia-generatif/blob/main/docs/operations/team-aggregator.md"
            class="team-activate-link"
            target="_blank"
            rel="noopener noreferrer"
          >
            Voir le guide complet (5 minutes) →
          </a>
        </div>
      {/if}

      <!-- Bloc Configuration ─────────────────────────────────────── -->
      <div class="team-block">
        <h3 class="ext-pane-title">Configuration</h3>
        <div class="team-form-row">
          <label for="team-url-input">URL serveur</label>
          <input
            id="team-url-input"
            type="text"
            class="team-input"
            placeholder="https://votre-serveur:8443"
            bind:value={teamUrlDraft}
            disabled={!backendAvailable || teamBusy}
            data-testid="team-url-input"
          />
        </div>
        {#if teamUrlDraft && !teamUrlIsValid}
          <p class="team-hint warn">
            L'URL doit commencer par <code>https://</code> ou <code>http://</code>.
          </p>
        {:else if teamUrlDraft && !teamUrlIsHttps}
          <p class="team-hint warn">
            <AlertTriangle size={12} strokeWidth={1.8} />
            <span>URL non chiffrée (<code>http://</code>) — à éviter sauf usage local.</span>
          </p>
        {/if}
        <label class="team-toggle">
          <input
            type="checkbox"
            checked={$teamStore.accept_invalid_certs}
            disabled={!backendAvailable || teamBusy}
            onchange={handleToggleAcceptCert}
            data-testid="team-accept-cert"
          />
          <span>Accepter les certificats auto-signés</span>
        </label>
        {#if $teamStore.accept_invalid_certs}
          <p class="team-hint warn">
            <AlertTriangle size={12} strokeWidth={1.8} />
            <span
              >Cette option désactive la validation TLS. À n'utiliser que pour un serveur de
              confiance sur LAN.</span
            >
          </p>
        {/if}
        <div class="team-actions">
          <button
            type="button"
            class="btn-secondary"
            onclick={handleSaveTeamUrl}
            disabled={!backendAvailable || teamBusy || !teamUrlIsValid}
            data-testid="team-save-url"
          >
            <Check size={14} strokeWidth={1.8} />
            Enregistrer l'URL
          </button>
          <button
            type="button"
            class="btn-secondary"
            onclick={handleTeamPing}
            disabled={!backendAvailable || teamBusy || !$teamStore.url}
            data-testid="team-ping"
          >
            <PlugZap size={14} strokeWidth={1.8} />
            Vérifier la connexion
          </button>
        </div>
        {#if teamPingResult}
          <p class="team-hint ok" data-testid="team-ping-ok">
            <Check size={12} strokeWidth={1.8} />
            <span>Serveur joignable — version {teamPingResult.version}.</span>
          </p>
        {/if}
        {#if teamPingError}
          <p class="team-hint err" data-testid="team-ping-err">
            <AlertTriangle size={12} strokeWidth={1.8} />
            <span>{teamPingError}</span>
          </p>
        {/if}
      </div>

      <!-- Bloc Enrôlement (visible si pas encore enrôlé) ──────────── -->
      {#if !$teamStore.enrolled}
        <div class="team-block" data-testid="team-enroll-block">
          <h3 class="ext-pane-title">Enrôlement</h3>
          <p class="ext-hint">
            Saisissez le code à 12 chiffres reçu de votre administrateur (valide 7 jours) et
            choisissez votre mot de passe (≥ 8 caractères).
          </p>
          <div class="team-form-row">
            <label for="team-code-input">Enrollment code</label>
            <input
              id="team-code-input"
              type="text"
              class="team-input mono"
              inputmode="numeric"
              maxlength="12"
              placeholder="123456789012"
              bind:value={teamCode}
              disabled={!backendAvailable || teamBusy}
              data-testid="team-code-input"
            />
          </div>
          <div class="team-form-row">
            <label for="team-password-input">Mot de passe</label>
            <input
              id="team-password-input"
              type="password"
              class="team-input"
              bind:value={teamPassword}
              disabled={!backendAvailable || teamBusy}
              data-testid="team-password-input"
            />
          </div>
          <div class="team-form-row">
            <label for="team-password-confirm-input">Confirmer le mot de passe</label>
            <input
              id="team-password-confirm-input"
              type="password"
              class="team-input"
              bind:value={teamPasswordConfirm}
              disabled={!backendAvailable || teamBusy}
              data-testid="team-password-confirm-input"
            />
          </div>
          {#if teamPassword && !teamPasswordStrong}
            <p class="team-hint warn">Au moins 8 caractères.</p>
          {/if}
          {#if teamPasswordConfirm && !teamPasswordsMatch}
            <p class="team-hint warn">Les deux mots de passe ne correspondent pas.</p>
          {/if}
          <div class="team-form-row">
            <label for="team-display-name-input">Nom affiché (optionnel)</label>
            <input
              id="team-display-name-input"
              type="text"
              class="team-input"
              placeholder="ex. Marie Dupont"
              bind:value={teamDisplayName}
              disabled={!backendAvailable || teamBusy}
              data-testid="team-display-name-input"
            />
          </div>
          <div class="team-actions">
            <button
              type="button"
              class="btn-secondary"
              onclick={handleTeamEnroll}
              disabled={!backendAvailable || teamBusy || !teamEnrollReady}
              data-testid="team-enroll-btn"
            >
              <KeyRound size={14} strokeWidth={1.8} />
              M'enrôler sur ce serveur
            </button>
          </div>
        </div>
      {/if}

      <!-- Bloc Enrôlé ─────────────────────────────────────────────── -->
      {#if $teamStore.enrolled && $teamStore.url}
        <div class="team-block" data-testid="team-enrolled-block">
          <div class="team-actions">
            <a
              class="btn-secondary"
              href={`${$teamStore.url}/user/dashboard`}
              target="_blank"
              rel="noopener noreferrer"
              data-testid="team-open-dashboard"
            >
              <ExternalLink size={14} strokeWidth={1.8} />
              Ouvrir mon dashboard équipe
            </a>
            <button
              type="button"
              class="btn-secondary btn-destructive"
              onclick={handleTeamLogout}
              disabled={!backendAvailable || teamBusy}
              data-testid="team-logout-btn"
            >
              <LogOut size={14} strokeWidth={1.8} />
              Se déconnecter
            </button>
          </div>
        </div>
      {/if}

      <!-- Bloc Dispatcher ────────────────────────────────────────── -->
      <div class="team-block">
        <h3 class="ext-pane-title">Dispatcher des estimations</h3>
        <p class="ext-hint">Où envoyer chaque estimation calculée par l'app ?</p>
        <div class="team-radio-grid" role="radiogroup" aria-label="Mode de dispatch">
          {#each [{ id: 'local', label: 'Local seul', desc: "Aucun envoi externe. Tout reste dans le ledger d'audit local." }, { id: 'team', label: 'Mode Équipe', desc: 'Envoi vers votre serveur self-hosted uniquement (pas de ledger local).' }, { id: 'both', label: 'Les deux', desc: 'Journalise localement ET pousse vers le serveur équipe.' }] as opt (opt.id)}
            <label class="team-radio" title={opt.desc}>
              <input
                type="radio"
                name="team-mode"
                value={opt.id}
                checked={$teamStore.mode === opt.id}
                disabled={!backendAvailable || teamBusy}
                onchange={() => handleSetTeamMode(opt.id as TeamMode)}
                data-testid={`team-mode-${opt.id}`}
              />
              <span class="team-radio-content">
                <strong>{opt.label}</strong>
                <span class="team-radio-desc">{opt.desc}</span>
              </span>
            </label>
          {/each}
        </div>
      </div>

      {#if teamEnrollOk}
        <p class="team-hint ok" data-testid="team-enroll-ok">
          <Check size={12} strokeWidth={1.8} />
          <span>{teamEnrollOk}</span>
        </p>
      {/if}
      {#if teamError}
        <p class="team-hint err" data-testid="team-error">
          <AlertTriangle size={12} strokeWidth={1.8} />
          <span>{teamError}</span>
        </p>
      {/if}
    {/if}

    <p class="section-foot">
      <Users size={11} strokeWidth={1.8} />
      Serveur déployé par votre organisation (aucun cloud Sobr.ia). Voir
      <code>docs/operations/team-aggregator.md</code> et
      <a href="/methodo">ADR-0013</a>.
    </p>
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

<!-- C32.3 — Dialog « Activer Mode Équipe » (3 étapes guidées). ─────── -->
{#if teamActivateDialogOpen}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="modal-overlay"
    onclick={() => (teamActivateDialogOpen = false)}
    onkeydown={(e) => {
      if (e.key === 'Escape') teamActivateDialogOpen = false;
    }}
  >
    <div
      class="modal team-activate-modal"
      role="dialog"
      aria-modal="true"
      aria-labelledby="team-activate-title"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
      tabindex="-1"
    >
      <header class="ta-modal-head">
        <h3 id="team-activate-title">Activer Mode Équipe — 3 étapes</h3>
        <button
          type="button"
          class="ta-close"
          onclick={() => (teamActivateDialogOpen = false)}
          aria-label="Fermer la fenêtre"
        >
          <X size={14} strokeWidth={2} />
        </button>
      </header>

      <p class="ta-intro">
        Vous allez déployer un binaire Rust standalone sur votre infrastructure (poste admin, NAS,
        VPS interne). Aucun cloud Sobr.ia n'est impliqué — vos données restent chez vous.
      </p>

      <ol class="ta-steps">
        <li class="ta-step">
          <span class="ta-step-num">1</span>
          <div class="ta-step-body">
            <h4 class="ta-step-title">
              <Download size={14} strokeWidth={1.8} />
              Télécharger le binaire
            </h4>
            <p class="ta-step-text">
              Récupérez <code>sobria-team-aggregator</code> pour votre OS (Linux / macOS / Windows) depuis
              les Releases GitHub. ~15 MB.
            </p>
            <a
              href="https://github.com/BkOff-fr/defis-lia-generatif/releases"
              target="_blank"
              rel="noopener noreferrer"
              class="ta-step-link"
            >
              Releases GitHub →
            </a>
          </div>
        </li>

        <li class="ta-step">
          <span class="ta-step-num">2</span>
          <div class="ta-step-body">
            <h4 class="ta-step-title">
              <Terminal size={14} strokeWidth={1.8} />
              Initialiser le serveur
            </h4>
            <p class="ta-step-text">
              Sur le poste qui hébergera l'agrégateur, lancez ces commandes une seule fois :
            </p>
            <pre class="ta-cmd"><code
                >chmod +x sobria-team-aggregator-linux-x86_64
./sobria-team-aggregator --data-dir ./team-data init \
    --admin-username admin --admin-password 'CHANGE-ME'
./sobria-team-aggregator --data-dir ./team-data serve --port 8443</code
              ></pre>
            <p class="ta-step-hint">
              Le serveur écoute en HTTPS sur le port 8443 avec un certificat auto-signé (rotation
              possible plus tard via
              <code>serve --regen-cert</code>).
            </p>
          </div>
        </li>

        <li class="ta-step">
          <span class="ta-step-num">3</span>
          <div class="ta-step-body">
            <h4 class="ta-step-title">
              <Users size={14} strokeWidth={1.8} />
              Distribuer les codes aux employés
            </h4>
            <p class="ta-step-text">
              Ouvrez <code>https://votre-serveur:8443/admin</code> avec les identifiants admin. Créez
              un code d'enrôlement à 12 chiffres par employé, distribuez-les. Chaque employé colle son
              code dans le bloc « Enrôlement » ci-dessous.
            </p>
          </div>
        </li>
      </ol>

      <div class="ta-modal-foot">
        <a
          href="https://github.com/BkOff-fr/defis-lia-generatif/blob/main/docs/operations/team-aggregator.md"
          class="ta-doc-link"
          target="_blank"
          rel="noopener noreferrer"
        >
          Voir le guide complet (5 minutes) →
        </a>
        <button
          type="button"
          class="btn-primary"
          onclick={() => (teamActivateDialogOpen = false)}
          data-action="team-activate-close"
        >
          J'ai compris
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

  /* Hero */
  .hero {
    padding-bottom: 24px;
    margin-bottom: 24px;
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
    font-size: 12px;
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
    font: 500 12px/1.4 var(--font-mono);
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
  .persona-current .persona-icon {
    width: 44px;
    height: 44px;
    display: inline-grid;
    place-items: center;
    background: var(--lime-soft);
    border: 1px solid rgba(197, 240, 74, 0.32);
    border-radius: var(--radius-md);
    color: var(--lime);
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
  .persona-option .persona-icon {
    width: 36px;
    height: 36px;
    display: inline-grid;
    place-items: center;
    background: var(--lime-soft);
    border: 1px solid rgba(197, 240, 74, 0.25);
    border-radius: var(--radius-sm);
    color: var(--lime);
    transition: all var(--dur-base) var(--ease);
  }
  .persona-option:hover:not(:disabled) .persona-icon {
    background: rgba(197, 240, 74, 0.22);
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
    font-size: 12px;
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
    font: 500 12px/1 var(--font-ui);
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
  /* Feature I5 — modules différés (v1.1+) : non-interactif, badge clair */
  .module-line.is-deferred {
    cursor: default;
    opacity: 0.85;
  }
  .module-line.is-deferred:hover {
    border-color: var(--border);
    background: rgba(255, 255, 255, 0.02);
  }
  .check-box.deferred {
    width: auto;
    height: auto;
    padding: 2px 6px;
    background: var(--surface-hi);
    border: 1px solid var(--border-hi);
    border-radius: var(--radius-pill);
    font: 500 12px/1 var(--font-mono);
    color: var(--ivory-3);
    letter-spacing: 0.04em;
    margin-top: 4px;
  }
  .module-body {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .module-label {
    font: 500 13px/1.2 var(--font-ui);
    color: var(--ivory);
  }
  .module-line[data-checked='true'] .module-label {
    color: var(--lime);
  }
  .module-desc {
    font: 400 12px/1.4 var(--font-ui);
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
  /* Polish H2 — Section Méthodologie (C24) */
  .section-intro {
    font: 400 13px/1.6 var(--font-ui);
    color: var(--ivory-2);
    margin: 0 0 18px;
    max-width: 760px;
  }
  .section-hint-link {
    color: var(--lime);
    text-decoration: none;
  }
  .section-hint-link:hover {
    text-decoration: underline;
  }
  .method-list {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .method-row {
    background: var(--ink-2);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    padding: 14px 16px;
    display: flex;
    flex-direction: column;
    gap: 8px;
    transition: border-color var(--dur-base) var(--ease);
  }
  .method-row.is-default {
    border-color: rgba(197, 240, 74, 0.4);
    background: linear-gradient(135deg, rgba(197, 240, 74, 0.04), transparent);
  }
  .method-row[data-method='ecologits'].is-default {
    border-color: rgba(96, 165, 250, 0.4);
    background: linear-gradient(135deg, rgba(96, 165, 250, 0.04), transparent);
  }
  .method-head {
    display: flex;
    align-items: center;
    gap: 10px;
    flex-wrap: wrap;
  }
  .method-name {
    font: 500 13.5px/1.2 var(--font-ui);
    color: var(--ivory);
    flex: 0 0 auto;
  }
  .badge-method {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 3px 9px;
    border-radius: var(--radius-pill);
    font: 500 10.5px/1 var(--font-ui);
  }
  .badge-default {
    background: rgba(197, 240, 74, 0.12);
    border: 1px solid rgba(197, 240, 74, 0.3);
    color: var(--lime);
  }
  .method-row[data-method='ecologits'] .badge-default {
    background: rgba(96, 165, 250, 0.12);
    border-color: rgba(96, 165, 250, 0.3);
    color: rgb(147, 197, 253);
  }
  .doi {
    font: 400 12px/1 var(--font-mono);
    color: var(--ivory-4);
    text-decoration: none;
    margin-left: auto;
  }
  .doi:hover {
    color: var(--lime);
  }
  .method-desc {
    font: 400 12.5px/1.55 var(--font-ui);
    color: var(--ivory-3);
    margin: 0;
  }
  .method-actions {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 16px;
    margin-top: 4px;
  }
  .btn-tiny {
    background: transparent;
    border: 1px solid var(--border-hi);
    border-radius: var(--radius-sm);
    color: var(--ivory-2);
    font: 500 12px/1 var(--font-ui);
    padding: 7px 14px;
    cursor: pointer;
    transition: all var(--dur-base) var(--ease);
  }
  .btn-tiny:hover:not(:disabled) {
    border-color: var(--lime);
    color: var(--lime);
  }
  .btn-tiny:disabled {
    cursor: not-allowed;
    opacity: 0.7;
  }
  .btn-tiny.active {
    background: rgba(197, 240, 74, 0.1);
    border-color: rgba(197, 240, 74, 0.3);
    color: var(--lime);
  }
  .ref-check {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font: 400 12px/1.3 var(--font-ui);
    color: var(--ivory-2);
    cursor: pointer;
  }
  .ref-check input[type='checkbox'] {
    width: 14px;
    height: 14px;
    accent-color: var(--lime);
    cursor: pointer;
  }
  .ref-check input[type='checkbox']:disabled {
    cursor: not-allowed;
    opacity: 0.5;
  }

  .lang-toggle {
    display: inline-flex;
    background: var(--ink-2);
    border: 1px solid var(--border);
    border-radius: var(--radius-pill);
    padding: 3px;
    gap: 2px;
  }
  /* Polish H4 — toggle FR/EN visuellement désactivé (i18n v1.1) */
  .lang-toggle.is-disabled {
    opacity: 0.55;
    cursor: not-allowed;
  }
  .lang-pending {
    font-size: 12px;
    color: var(--ivory-4);
    margin-left: 4px;
    letter-spacing: 0.04em;
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
    font: 500 12px/1 var(--font-ui);
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

  /* ─── C32.3 — Panneau « Activer Mode Équipe » + dialog 3 étapes ────── */
  .team-activate {
    margin-top: 12px;
    padding: 16px 18px;
    background: linear-gradient(155deg, rgba(197, 240, 74, 0.05), rgba(197, 240, 74, 0.01));
    border: 1px dashed rgba(197, 240, 74, 0.3);
    border-radius: var(--radius-md);
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .team-activate-head {
    display: flex;
    align-items: flex-start;
    gap: 12px;
  }
  .team-activate-ico {
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
  .team-activate-head > div {
    flex: 1;
    min-width: 0;
  }
  .team-activate-title {
    font: 500 14px/1.25 var(--font-ui);
    color: var(--ivory);
    margin: 0 0 4px;
  }
  .team-activate-sub {
    font: 400 12.5px/1.55 var(--font-ui);
    color: var(--ivory-3);
    margin: 0;
  }
  .team-activate-btn {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    align-self: flex-start;
    padding: 10px 18px;
    background: var(--lime);
    color: var(--ink);
    border: none;
    border-radius: var(--radius-pill);
    font: 600 12.5px/1 var(--font-ui);
    cursor: pointer;
    transition: all var(--dur-base) var(--ease);
  }
  .team-activate-btn:hover {
    transform: translateY(-1px);
    box-shadow: var(--glow-lime);
  }
  .team-activate-link {
    font: 400 11.5px/1 var(--font-ui);
    color: var(--ivory-3);
    text-decoration: none;
    border-bottom: 1px dashed var(--border-hi);
    align-self: flex-start;
  }
  .team-activate-link:hover {
    color: var(--lime);
    border-bottom-color: var(--lime);
  }

  /* Dialog modal version élargie pour les 3 étapes. */
  .team-activate-modal {
    max-width: 620px;
  }
  .ta-modal-head {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 12px;
  }
  .ta-modal-head h3 {
    margin: 0;
  }
  .ta-close {
    background: transparent;
    border: 1px solid var(--border);
    color: var(--ivory-3);
    width: 28px;
    height: 28px;
    border-radius: 50%;
    cursor: pointer;
    display: grid;
    place-items: center;
    transition: all var(--dur-base) var(--ease);
  }
  .ta-close:hover {
    color: var(--ivory);
    border-color: var(--border-hi);
  }
  .ta-intro {
    font: 400 13px/1.55 var(--font-ui);
    color: var(--ivory-3);
    margin: 0 0 18px;
  }
  .ta-steps {
    list-style: none;
    padding: 0;
    margin: 0 0 18px;
    display: flex;
    flex-direction: column;
    gap: 14px;
  }
  .ta-step {
    display: grid;
    grid-template-columns: 32px 1fr;
    gap: 12px;
  }
  .ta-step-num {
    display: grid;
    place-items: center;
    width: 28px;
    height: 28px;
    border-radius: 50%;
    background: var(--lime-soft);
    color: var(--lime);
    font: 700 14px/1 var(--font-mono);
    border: 1px solid rgba(197, 240, 74, 0.4);
  }
  .ta-step-title {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font: 500 13px/1.2 var(--font-ui);
    color: var(--ivory);
    margin: 4px 0 6px;
  }
  .ta-step-title :global(svg) {
    color: var(--lime);
  }
  .ta-step-text {
    font: 400 12.5px/1.55 var(--font-ui);
    color: var(--ivory-2);
    margin: 0 0 8px;
  }
  .ta-step-link {
    display: inline-block;
    font: 500 12px/1 var(--font-ui);
    color: var(--lime);
    text-decoration: none;
    border-bottom: 1px dashed rgba(197, 240, 74, 0.5);
  }
  .ta-step-link:hover {
    color: var(--ivory);
  }
  .ta-cmd {
    margin: 0 0 8px;
    padding: 10px 12px;
    background: var(--ink);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    font: 400 12px/1.55 var(--font-mono);
    color: var(--ivory-2);
    overflow-x: auto;
    white-space: pre;
  }
  .ta-step-hint {
    font: 400 12px/1.5 var(--font-ui);
    color: var(--ivory-4);
    margin: 0;
    font-style: italic;
  }
  .ta-modal-foot {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
    margin-top: 12px;
  }
  .ta-doc-link {
    font: 500 12px/1 var(--font-ui);
    color: var(--ivory-3);
    text-decoration: none;
    border-bottom: 1px dashed var(--border-hi);
  }
  .ta-doc-link:hover {
    color: var(--lime);
    border-bottom-color: var(--lime);
  }

  /* C26.5 — Référentiel Gold (callout warn + bouton recharger) ─────── */
  .callout {
    display: flex;
    align-items: flex-start;
    gap: 12px;
    padding: 14px 16px;
    border: 1px solid var(--border-subtle, rgba(255, 255, 255, 0.08));
    border-radius: var(--radius-md, 8px);
    background: rgba(255, 255, 255, 0.02);
    margin: 8px 0 14px;
  }
  .callout-warn {
    border-color: rgba(255, 176, 0, 0.35);
    background: rgba(255, 176, 0, 0.06);
    color: var(--ivory, #f4f0e8);
  }
  .callout strong {
    color: var(--ivory, #f4f0e8);
    font-weight: 500;
  }
  .callout-msg {
    font: 400 13px/1.5 var(--font-ui);
    color: var(--ivory-2, rgba(244, 240, 232, 0.75));
    margin: 4px 0 0;
  }
  .reload-row {
    display: flex;
    align-items: center;
    gap: 12px;
    flex-wrap: wrap;
    margin-top: 10px;
  }
  .reload-msg {
    font: 400 12px/1.5 var(--font-ui);
    color: var(--ivory-2, rgba(244, 240, 232, 0.7));
    margin: 0;
  }
  .btn-secondary {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font: 500 13px/1 var(--font-ui);
    color: var(--ivory, #f4f0e8);
    background: rgba(255, 255, 255, 0.04);
    border: 1px solid var(--border-subtle, rgba(255, 255, 255, 0.12));
    border-radius: var(--radius-sm, 6px);
    padding: 8px 14px;
    cursor: pointer;
    transition: background 120ms ease;
  }
  .btn-secondary:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.08);
  }
  .btn-secondary:disabled {
    opacity: 0.55;
    cursor: not-allowed;
  }

  /* C27.5 — Extension navigateur (pairing perso) ──────────────────── */
  .ext-intro {
    font: 400 13px/1.55 var(--font-ui);
    color: var(--ivory-2, rgba(244, 240, 232, 0.78));
    margin: 0 0 18px;
    max-width: 760px;
  }
  .ext-grid {
    margin-top: 8px;
  }
  .ext-pane {
    padding: 18px 18px 14px;
    border: 1px solid var(--border-subtle, rgba(255, 255, 255, 0.08));
    border-radius: var(--radius-md, 10px);
    background: rgba(255, 255, 255, 0.02);
  }
  .ext-pane-title {
    font: 500 13px/1 var(--font-ui);
    color: var(--ivory, #f4f0e8);
    text-transform: uppercase;
    letter-spacing: 0.06em;
    margin: 0 0 14px;
  }
  .ext-count {
    color: var(--ivory-3, rgba(244, 240, 232, 0.55));
    font-weight: 400;
    letter-spacing: 0;
    text-transform: none;
  }
  .ext-hint {
    font: 400 12.5px/1.55 var(--font-ui);
    color: var(--ivory-2, rgba(244, 240, 232, 0.72));
    margin: 0 0 14px;
  }
  .pairing-code {
    display: flex;
    gap: 8px;
    justify-content: center;
    margin: 8px 0 14px;
  }
  .pairing-digit {
    flex: 0 0 auto;
    width: 44px;
    height: 56px;
    display: grid;
    place-items: center;
    font: 500 30px/1 var(--font-mono);
    color: var(--lime, #c5f04a);
    background: rgba(197, 240, 74, 0.06);
    border: 1px solid rgba(197, 240, 74, 0.28);
    border-radius: var(--radius-sm, 8px);
  }
  .pairing-timer {
    font: 400 12px/1 var(--font-ui);
    color: var(--ivory-3, rgba(244, 240, 232, 0.6));
    text-align: center;
    margin: 0 0 14px;
  }
  .pairing-timer strong {
    color: var(--ivory, #f4f0e8);
    font-weight: 500;
    font-family: var(--font-mono);
  }
  .pairing-list {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .pairing-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    padding: 10px 12px;
    border: 1px solid var(--border-subtle, rgba(255, 255, 255, 0.06));
    border-radius: var(--radius-sm, 6px);
    background: rgba(255, 255, 255, 0.015);
  }
  .pairing-row.revoked {
    opacity: 0.55;
  }
  .pairing-info {
    min-width: 0;
    flex: 1;
  }
  .pairing-fp {
    font: 500 12.5px/1.3 var(--font-mono);
    color: var(--ivory, #f4f0e8);
    overflow-wrap: anywhere;
  }
  .pairing-meta {
    font: 400 12px/1.4 var(--font-mono);
    color: var(--ivory-3, rgba(244, 240, 232, 0.55));
    margin-top: 2px;
  }
  .pairing-revoked-label {
    color: var(--danger, #ff8090);
  }
  .btn-revoke {
    padding: 4px;
    border-radius: var(--radius-sm, 6px);
  }

  /* C29.1 — Mode Équipe self-hosted ────────────────────────────────── */
  .team-status {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin: 0 0 18px;
    padding: 14px 16px;
    border: 1px solid var(--border-subtle, rgba(255, 255, 255, 0.08));
    border-radius: var(--radius-md, 10px);
    background: rgba(255, 255, 255, 0.02);
  }
  .team-status-row {
    display: grid;
    grid-template-columns: 160px 1fr;
    gap: 12px;
    align-items: center;
    font: 400 12.5px/1.4 var(--font-ui);
  }
  .team-status-label {
    color: var(--ivory-3, rgba(244, 240, 232, 0.55));
    text-transform: uppercase;
    letter-spacing: 0.06em;
    font-size: 12px;
  }
  .team-mono {
    font: 500 12.5px/1.3 var(--font-mono);
    color: var(--ivory, #f4f0e8);
    overflow-wrap: anywhere;
  }
  .team-truncate {
    display: inline-block;
    max-width: 100%;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .team-pill {
    display: inline-flex;
    align-items: center;
    padding: 3px 10px;
    border-radius: 999px;
    font: 500 12px/1.4 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.06em;
    border: 1px solid transparent;
  }
  .team-pill-mute {
    background: rgba(255, 255, 255, 0.04);
    color: var(--ivory-3, rgba(244, 240, 232, 0.6));
    border-color: rgba(255, 255, 255, 0.08);
  }
  .team-pill-warn {
    background: rgba(255, 178, 92, 0.08);
    color: var(--amber, #ffb25c);
    border-color: rgba(255, 178, 92, 0.32);
  }
  .team-pill-ok {
    background: rgba(197, 240, 74, 0.08);
    color: var(--lime, #c5f04a);
    border-color: rgba(197, 240, 74, 0.32);
  }
  .team-block {
    margin: 0 0 18px;
    padding: 16px 18px 14px;
    border: 1px solid var(--border-subtle, rgba(255, 255, 255, 0.08));
    border-radius: var(--radius-md, 10px);
    background: rgba(255, 255, 255, 0.015);
  }
  .team-form-row {
    display: grid;
    grid-template-columns: 200px 1fr;
    gap: 12px;
    align-items: center;
    margin: 0 0 10px;
  }
  .team-form-row label {
    font: 400 12.5px/1.4 var(--font-ui);
    color: var(--ivory-2, rgba(244, 240, 232, 0.78));
  }
  .team-input {
    font: 400 13px/1.4 var(--font-ui);
    padding: 8px 10px;
    border-radius: var(--radius-sm, 6px);
    border: 1px solid var(--border-subtle, rgba(255, 255, 255, 0.12));
    background: rgba(255, 255, 255, 0.04);
    color: var(--ivory, #f4f0e8);
  }
  .team-input.mono {
    font-family: var(--font-mono);
    letter-spacing: 0.05em;
  }
  .team-input:focus {
    outline: 1px solid var(--lime, #c5f04a);
    outline-offset: 1px;
  }
  .team-input:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .team-toggle {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    margin: 4px 0 10px;
    font: 400 12.5px/1.4 var(--font-ui);
    color: var(--ivory-2, rgba(244, 240, 232, 0.78));
    cursor: pointer;
  }
  .team-toggle input[type='checkbox']:disabled {
    cursor: not-allowed;
  }
  .team-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 10px;
    margin: 12px 0 6px;
  }
  .team-actions .btn-secondary {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    text-decoration: none;
  }
  .btn-destructive {
    color: var(--coral, #ff8090) !important;
    border-color: rgba(255, 128, 144, 0.32) !important;
  }
  .btn-destructive:hover:not(:disabled) {
    background: rgba(255, 128, 144, 0.08) !important;
  }
  .team-hint {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    margin: 4px 0 0;
    font: 400 12px/1.4 var(--font-ui);
    color: var(--ivory-3, rgba(244, 240, 232, 0.6));
  }
  .team-hint.warn {
    color: var(--amber, #ffb25c);
  }
  .team-hint.err {
    color: var(--coral, #ff8090);
  }
  .team-hint.ok {
    color: var(--lime, #c5f04a);
  }
  .team-radio-grid {
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin-top: 8px;
  }
  .team-radio {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 10px;
    align-items: start;
    padding: 10px 12px;
    border: 1px solid var(--border-subtle, rgba(255, 255, 255, 0.08));
    border-radius: var(--radius-sm, 6px);
    background: rgba(255, 255, 255, 0.015);
    cursor: pointer;
  }
  .team-radio:has(input:checked) {
    border-color: var(--lime, #c5f04a);
    background: rgba(197, 240, 74, 0.04);
  }
  .team-radio:has(input:disabled) {
    opacity: 0.55;
    cursor: not-allowed;
  }
  .team-radio-content {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .team-radio-content strong {
    font: 500 13px/1.3 var(--font-ui);
    color: var(--ivory, #f4f0e8);
  }
  .team-radio-desc {
    font: 400 11.5px/1.45 var(--font-ui);
    color: var(--ivory-3, rgba(244, 240, 232, 0.6));
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
    .pairing-digit {
      width: 36px;
      height: 48px;
      font-size: 24px;
    }
    .team-status-row,
    .team-form-row {
      grid-template-columns: 1fr;
      gap: 4px;
    }
  }
</style>
