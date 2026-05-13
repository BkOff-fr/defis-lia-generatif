<script lang="ts">
  // Module M17 — Empreinte projet / datasheet Gebru (C20).
  // Consomme les commandes IPC `list_projects`, `get_project`, `create_project`,
  // `update_project`, `delete_project`, `generate_project_datasheet`.
  // Contrat no-mock : hors Tauri, la coque pédagogique reste rendue (header +
  // bannière) mais aucune liste / formulaire actif. Voir :
  //   - briefs/chantiers/C20-empreinte-projet-datasheet.md
  //   - briefs/chantiers/C20-PROMPT-CLAUDE-CODE-M17.md
  //   - crates/sobria-app/src/dto.rs (bloc "projects + datasheet")

  import { onMount } from 'svelte';
  import {
    AlertTriangle,
    PlugZap,
    HelpCircle,
    Lock,
    FileText,
    Plus,
    Trash2,
    Pencil,
    Eye,
    Copy,
    Download,
    RefreshCw,
    ChevronDown,
    CheckCircle2,
    Loader2,
    X,
    Calendar,
    Activity,
    Leaf,
    Zap,
    Droplet,
    ArrowUpRight,
    BookOpen
  } from '@lucide/svelte';
  import {
    isTauriContext,
    listProjects,
    createProject,
    updateProject,
    deleteProject,
    generateProjectDatasheet,
    exportCsrdReport,
    SobriaIpcError,
    type ProjectDto,
    type DatasheetDto,
    type IpcErrorCode
  } from '$lib/api';
  import { preferences, type ModuleId } from '$lib/preferences';

  const MODULE_ID: ModuleId = 'm17';

  // Module gating (cf. ADR-0010)
  $effect(() => {
    if ($preferences.loaded && !$preferences.enabled_modules.includes(MODULE_ID)) {
      window.location.replace('/?disabled=' + MODULE_ID);
    }
  });

  // ─── Constantes UI ──────────────────────────────────────────────────────
  const NAME_MAX = 200;
  const DESCRIPTION_MAX = 5000;
  const TAGS_MAX = 10;
  const TAG_MAX_LEN = 50;
  const TAG_RX = /^[a-z0-9-]+$/;

  // ─── State principal ────────────────────────────────────────────────────
  type Panel =
    | { kind: 'empty' }
    | { kind: 'create' }
    | { kind: 'edit'; projectId: number }
    | { kind: 'datasheet'; projectId: number };

  let projects = $state<ProjectDto[]>([]);
  let loading = $state(true);
  let loadError = $state<{ code: IpcErrorCode; message: string } | null>(null);
  let panel = $state<Panel>({ kind: 'empty' });

  const tauriAvailable = $derived(isTauriContext());

  // ─── Formulaires (create / edit) ────────────────────────────────────────
  let fName = $state('');
  let fDescription = $state('');
  let fPeriodStart = $state('');
  let fPeriodEnd = $state('');
  let fTags = $state<string[]>([]);
  let fTagInput = $state('');
  let fTagError = $state<string | null>(null);
  let submitting = $state(false);
  let formError = $state<{ code: IpcErrorCode; message: string } | null>(null);

  // ─── Datasheet ──────────────────────────────────────────────────────────
  let datasheet = $state<DatasheetDto | null>(null);
  let datasheetLoading = $state(false);
  let datasheetError = $state<{ code: IpcErrorCode; message: string } | null>(null);
  let expandedSections = $state<Record<string, boolean>>({
    motivation: true,
    composition: true,
    collectionProcess: false,
    preprocessing: false,
    uses: false,
    distribution: false,
    maintenance: false
  });
  let copyState = $state<'idle' | 'copied'>('idle');
  let saveState = $state<'idle' | 'saving' | 'saved' | 'error'>('idle');
  let saveMsg = $state<string | null>(null);
  let csrdState = $state<'idle' | 'pending' | 'done' | 'error'>('idle');
  let csrdMsg = $state<string | null>(null);

  // ─── Suppression (modale confirmation) ──────────────────────────────────
  let confirmDeleteId = $state<number | null>(null);

  // ─── Charge initiale ────────────────────────────────────────────────────
  async function loadProjects(): Promise<void> {
    if (!tauriAvailable) {
      loading = false;
      return;
    }
    loading = true;
    loadError = null;
    try {
      projects = await listProjects();
    } catch (err) {
      if (err instanceof SobriaIpcError) {
        loadError = { code: err.code, message: err.message };
      } else {
        loadError = { code: 'internal', message: 'Échec du chargement des projets' };
      }
      projects = [];
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    void loadProjects();
  });

  // ─── Helpers date ───────────────────────────────────────────────────────
  function todayIso(): string {
    const d = new Date();
    return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(
      d.getDate()
    ).padStart(2, '0')}`;
  }

  function toIsoUtc(dateYmd: string, endOfDay: boolean): string {
    if (!/^\d{4}-\d{2}-\d{2}$/.test(dateYmd)) return dateYmd;
    return `${dateYmd}T${endOfDay ? '23:59:59' : '00:00:00'}Z`;
  }

  function fromIsoToYmd(iso: string): string {
    const m = /^(\d{4})-(\d{2})-(\d{2})/.exec(iso);
    return m ? `${m[1]}-${m[2]}-${m[3]}` : '';
  }

  function fmtDate(iso: string): string {
    const m = /^(\d{4})-(\d{2})-(\d{2})/.exec(iso);
    if (!m) return iso;
    return `${m[3]}/${m[2]}/${m[1]}`;
  }

  function fmtNum(n: number, digits = 2): string {
    if (!Number.isFinite(n)) return '—';
    return new Intl.NumberFormat('fr-FR', { maximumFractionDigits: digits }).format(n);
  }

  function fmtInt(n: number): string {
    if (!Number.isFinite(n)) return '—';
    return new Intl.NumberFormat('fr-FR', { maximumFractionDigits: 0 }).format(n);
  }

  type AutoScale = { v: string; u: string };
  function fmtCo2(g: number): AutoScale {
    if (!Number.isFinite(g)) return { v: '—', u: 'g CO₂eq' };
    if (g >= 1e6) return { v: fmtNum(g / 1e6), u: 't CO₂eq' };
    if (g >= 1e3) return { v: fmtNum(g / 1e3), u: 'kg CO₂eq' };
    return { v: fmtNum(g), u: 'g CO₂eq' };
  }
  function fmtEnergy(wh: number): AutoScale {
    if (!Number.isFinite(wh)) return { v: '—', u: 'Wh' };
    if (wh >= 1e6) return { v: fmtNum(wh / 1e6), u: 'MWh' };
    if (wh >= 1e3) return { v: fmtNum(wh / 1e3), u: 'kWh' };
    return { v: fmtNum(wh), u: 'Wh' };
  }
  function fmtWater(l: number): AutoScale {
    if (!Number.isFinite(l)) return { v: '—', u: 'L' };
    if (l >= 1000) return { v: fmtNum(l / 1000), u: 'm³' };
    if (l >= 1) return { v: fmtNum(l), u: 'L' };
    return { v: fmtNum(l * 1000, 1), u: 'mL' };
  }

  // ─── Form helpers ───────────────────────────────────────────────────────
  function resetForm(): void {
    fName = '';
    fDescription = '';
    fPeriodStart = '';
    fPeriodEnd = todayIso();
    fTags = [];
    fTagInput = '';
    fTagError = null;
    formError = null;
  }

  function loadFormFrom(p: ProjectDto): void {
    fName = p.name;
    fDescription = p.description;
    fPeriodStart = fromIsoToYmd(p.period_start);
    fPeriodEnd = fromIsoToYmd(p.period_end);
    fTags = [...p.tags];
    fTagInput = '';
    fTagError = null;
    formError = null;
  }

  function tryAddTag(): void {
    const raw = fTagInput.trim().toLowerCase();
    if (raw.length === 0) return;
    if (fTags.length >= TAGS_MAX) {
      fTagError = `Maximum ${TAGS_MAX} tags.`;
      return;
    }
    if (raw.length > TAG_MAX_LEN) {
      fTagError = `Tag trop long (≤ ${TAG_MAX_LEN} caractères).`;
      return;
    }
    if (!TAG_RX.test(raw)) {
      fTagError = "Format invalide — uniquement a-z, 0-9 et '-'.";
      return;
    }
    if (fTags.includes(raw)) {
      fTagError = 'Tag déjà présent.';
      return;
    }
    fTags = [...fTags, raw];
    fTagInput = '';
    fTagError = null;
  }

  function removeTag(t: string): void {
    fTags = fTags.filter((x) => x !== t);
  }

  function onTagKeydown(e: KeyboardEvent): void {
    if (e.key === 'Enter' || e.key === ',') {
      e.preventDefault();
      tryAddTag();
    } else if (e.key === 'Backspace' && fTagInput.length === 0 && fTags.length > 0) {
      const last = fTags[fTags.length - 1];
      if (typeof last === 'string') {
        fTags = fTags.slice(0, -1);
        fTagInput = last;
      }
    }
  }

  // ─── Validation côté front ──────────────────────────────────────────────
  const createValid = $derived(
    panel.kind === 'create' &&
      fName.trim().length > 0 &&
      fName.length <= NAME_MAX &&
      fDescription.length <= DESCRIPTION_MAX &&
      fPeriodStart.length === 10 &&
      fPeriodEnd.length === 10 &&
      fPeriodStart < fPeriodEnd
  );

  const editValid = $derived(
    panel.kind === 'edit' &&
      fName.trim().length > 0 &&
      fName.length <= NAME_MAX &&
      fDescription.length <= DESCRIPTION_MAX
  );

  // ─── Actions liste ──────────────────────────────────────────────────────
  function openCreate(): void {
    resetForm();
    fPeriodEnd = todayIso();
    panel = { kind: 'create' };
  }

  function openEdit(p: ProjectDto): void {
    loadFormFrom(p);
    panel = { kind: 'edit', projectId: p.id };
  }

  async function openDatasheet(p: ProjectDto): Promise<void> {
    panel = { kind: 'datasheet', projectId: p.id };
    await regenerateDatasheet(p.id);
  }

  async function regenerateDatasheet(id: number): Promise<void> {
    if (!tauriAvailable) return;
    datasheetLoading = true;
    datasheetError = null;
    datasheet = null;
    copyState = 'idle';
    saveState = 'idle';
    saveMsg = null;
    csrdState = 'idle';
    csrdMsg = null;
    try {
      datasheet = await generateProjectDatasheet(id);
    } catch (err) {
      if (err instanceof SobriaIpcError) {
        datasheetError = { code: err.code, message: err.message };
      } else {
        datasheetError = { code: 'internal', message: 'Échec de la génération du datasheet' };
      }
    } finally {
      datasheetLoading = false;
    }
  }

  // ─── CRUD ───────────────────────────────────────────────────────────────
  async function submitCreate(): Promise<void> {
    if (!tauriAvailable || !createValid) return;
    submitting = true;
    formError = null;
    try {
      const created = await createProject({
        name: fName.trim(),
        description: fDescription,
        period_start: toIsoUtc(fPeriodStart, false),
        period_end: toIsoUtc(fPeriodEnd, true),
        tags: fTags
      });
      projects = [created, ...projects];
      panel = { kind: 'datasheet', projectId: created.id };
      await regenerateDatasheet(created.id);
    } catch (err) {
      if (err instanceof SobriaIpcError) {
        formError = { code: err.code, message: err.message };
      } else {
        formError = { code: 'internal', message: 'Échec de la création du projet' };
      }
    } finally {
      submitting = false;
    }
  }

  async function submitEdit(): Promise<void> {
    if (!tauriAvailable || !editValid || panel.kind !== 'edit') return;
    submitting = true;
    formError = null;
    const id = panel.projectId;
    try {
      const updated = await updateProject(id, {
        name: fName.trim(),
        description: fDescription,
        tags: fTags
      });
      projects = projects.map((p) => (p.id === id ? updated : p));
      panel = { kind: 'datasheet', projectId: id };
      await regenerateDatasheet(id);
    } catch (err) {
      if (err instanceof SobriaIpcError) {
        formError = { code: err.code, message: err.message };
      } else {
        formError = { code: 'internal', message: 'Échec de la mise à jour' };
      }
    } finally {
      submitting = false;
    }
  }

  async function confirmDelete(): Promise<void> {
    if (!tauriAvailable || confirmDeleteId === null) return;
    const id = confirmDeleteId;
    try {
      await deleteProject(id);
      projects = projects.filter((p) => p.id !== id);
      if (
        (panel.kind === 'edit' || panel.kind === 'datasheet') &&
        panel.projectId === id
      ) {
        panel = { kind: 'empty' };
        datasheet = null;
      }
    } catch (err) {
      if (err instanceof SobriaIpcError) {
        loadError = { code: err.code, message: err.message };
      } else {
        loadError = { code: 'internal', message: 'Échec de la suppression' };
      }
    } finally {
      confirmDeleteId = null;
    }
  }

  // ─── Datasheet : copy / save / CSRD chain ───────────────────────────────
  async function copyJsonld(): Promise<void> {
    if (!datasheet) return;
    try {
      const text = JSON.stringify(datasheet.jsonld, null, 2);
      await navigator.clipboard.writeText(text);
      copyState = 'copied';
      setTimeout(() => {
        if (copyState === 'copied') copyState = 'idle';
      }, 1800);
    } catch {
      // clipboard refusé (focus, permission) — silencieux, l'utilisateur
      // peut toujours utiliser "Télécharger".
    }
  }

  async function saveJsonld(): Promise<void> {
    if (!datasheet) return;
    // Plugin-fs n'est pas installé (cf. CLAUDE.md §3 — pas de dépendance sans
    // ADR). On utilise le mécanisme standard `<a download>` + Blob URL :
    // la webview Tauri ouvre le dialogue de sauvegarde natif et écrit le
    // fichier via le canal "download" du navigateur Chromium embarqué.
    saveState = 'saving';
    saveMsg = null;
    try {
      const text = JSON.stringify(datasheet.jsonld, null, 2);
      const blob = new Blob([text], { type: 'application/ld+json' });
      const url = URL.createObjectURL(blob);
      const fname = `${slugify(datasheet.project.name)}-datasheet.jsonld`;
      const a = document.createElement('a');
      a.href = url;
      a.download = fname;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      // Libère l'URL après que le navigateur ait initié le téléchargement.
      setTimeout(() => URL.revokeObjectURL(url), 1000);
      saveState = 'saved';
      saveMsg = fname;
      setTimeout(() => {
        if (saveState === 'saved') saveState = 'idle';
      }, 2800);
    } catch (err) {
      saveState = 'error';
      saveMsg = err instanceof Error ? err.message : "Échec de l'enregistrement";
    }
  }

  async function generateCsrd(): Promise<void> {
    if (!tauriAvailable || !datasheet) return;
    csrdState = 'pending';
    csrdMsg = null;
    try {
      const dialog = await import('@tauri-apps/plugin-dialog');
      const picked = await dialog.open({
        directory: true,
        multiple: false,
        title: 'Choisir le dossier de sortie pour le rapport CSRD'
      });
      if (typeof picked !== 'string') {
        csrdState = 'idle';
        return;
      }
      const result = await exportCsrdReport(
        {
          period_start: datasheet.project.period_start,
          period_end: datasheet.project.period_end,
          organization_name: datasheet.project.name,
          locale: 'fr'
        },
        picked
      );
      csrdState = 'done';
      csrdMsg = result.pdf_path;
    } catch (err) {
      csrdState = 'error';
      if (err instanceof SobriaIpcError) {
        csrdMsg = `${csrdErrorLabel(err.code)} — ${err.message}`;
      } else {
        csrdMsg = err instanceof Error ? err.message : 'Échec de la génération CSRD';
      }
    }
  }

  function slugify(s: string): string {
    return s
      .toLowerCase()
      .normalize('NFD')
      .replace(/[̀-ͯ]/g, '')
      .replace(/[^a-z0-9]+/g, '-')
      .replace(/^-+|-+$/g, '')
      .slice(0, 80) || 'projet';
  }

  // ─── Erreurs ────────────────────────────────────────────────────────────
  const ERROR_LABELS: Record<string, string> = {
    tauri_unavailable: 'Application non lancée via Tauri',
    invalid_request: 'Paramètres invalides',
    not_found: 'Projet introuvable',
    audit_error: "Erreur de lecture du journal d'audit",
    internal: 'Erreur interne'
  };
  function errorLabel(code: string): string {
    return ERROR_LABELS[code] ?? 'Erreur';
  }
  function errorHelp(code: string): string {
    switch (code) {
      case 'invalid_request':
        return 'Vérifie les contraintes : nom requis (≤ 200), description ≤ 5000, période_début < période_fin, tags slug-like (a-z, 0-9, tiret) ≤ 10.';
      case 'not_found':
        return 'Le projet a peut-être été supprimé depuis un autre onglet. Recharge la liste.';
      case 'tauri_unavailable':
        return "L'écran s'ouvre uniquement via `cargo run -p sobria-app`. En navigateur seul, l'IPC est indisponible.";
      default:
        return '';
    }
  }

  function csrdErrorLabel(code: string): string {
    if (code === 'empty_period') return 'Aucune entrée du ledger sur la période';
    if (code === 'export_error') return 'Échec de la génération PDF';
    return errorLabel(code);
  }

  // ─── Dérivés ────────────────────────────────────────────────────────────
  const currentProject = $derived.by<ProjectDto | null>(() => {
    const p = panel;
    if (p.kind !== 'edit' && p.kind !== 'datasheet') return null;
    return projects.find((pr) => pr.id === p.projectId) ?? null;
  });

  function toggleSection(key: string): void {
    expandedSections = { ...expandedSections, [key]: !expandedSections[key] };
  }

  // Extracteurs JSON-LD (le datasheet est `@graph[1]`, voir
  // crates/sobria-export/src/datasheet.rs).
  function gebruSection(key: string): string | null {
    if (!datasheet) return null;
    const graph = datasheet.jsonld['@graph'];
    if (!Array.isArray(graph) || graph.length < 2) return null;
    const node = graph[1] as Record<string, unknown>;
    const value = node[`sobria:${key}`];
    if (typeof value === 'string') return value;
    return null;
  }

  type DistributionInfo = {
    encodingFormat?: string;
    license?: string;
    dataLicense?: string;
  };
  function gebruDistribution(): DistributionInfo | null {
    if (!datasheet) return null;
    const graph = datasheet.jsonld['@graph'];
    if (!Array.isArray(graph) || graph.length < 2) return null;
    const node = graph[1] as Record<string, unknown>;
    const dist = node['sobria:distribution'];
    if (!dist || typeof dist !== 'object') return null;
    const d = dist as Record<string, unknown>;
    const out: DistributionInfo = {};
    if (typeof d['schema:encodingFormat'] === 'string')
      out.encodingFormat = d['schema:encodingFormat'];
    if (typeof d['schema:license'] === 'string') out.license = d['schema:license'];
    if (typeof d['sobria:dataLicense'] === 'string') out.dataLicense = d['sobria:dataLicense'];
    return out;
  }

  type MaintenanceInfo = {
    contact?: string;
    softwareVersion?: string;
    dateModified?: string;
  };
  function gebruMaintenance(): MaintenanceInfo | null {
    if (!datasheet) return null;
    const graph = datasheet.jsonld['@graph'];
    if (!Array.isArray(graph) || graph.length < 2) return null;
    const node = graph[1] as Record<string, unknown>;
    const m = node['sobria:maintenance'];
    if (!m || typeof m !== 'object') return null;
    const r = m as Record<string, unknown>;
    const out: MaintenanceInfo = {};
    if (typeof r['sobria:contact'] === 'string') out.contact = r['sobria:contact'];
    if (typeof r['schema:softwareVersion'] === 'string')
      out.softwareVersion = r['schema:softwareVersion'];
    if (typeof r['schema:dateModified'] === 'string') out.dateModified = r['schema:dateModified'];
    return out;
  }
</script>

<svelte:head>
  <title>Sobr.ia · Empreinte projet</title>
</svelte:head>

<div class="canvas-inner">
  <!-- TopBar -->
  <div class="topbar">
    <nav class="breadcrumb" aria-label="Fil d'Ariane">
      Atelier <span class="sep">/</span>
      <span class="current">Empreinte projet</span>
    </nav>
    <div class="spacer"></div>
    <span class="local-pill">
      <Lock size={12} strokeWidth={1.8} />
      Datasheet 100 % local
    </span>
    <a class="icon-btn" href="/methodo" aria-label="Méthodologie">
      <HelpCircle size={16} strokeWidth={1.6} />
    </a>
  </div>

  <!-- Hero -->
  <section class="hero">
    <div class="hero-eyebrow">
      <span class="pulse" aria-hidden="true"></span>
      Module M17 · Datasheets for Datasets · Gebru et al. 2018
    </div>
    <h1 class="hero-h1">
      Documente, <em>publie</em>, reproduis.
    </h1>
    <p class="hero-sub">
      Regroupe tes estimations en projets nommés et génère leur datasheet selon le standard
      académique adopté par NeurIPS, ICML et FAccT. Format JSON-LD reproductible — schema.org,
      W3C PROV-O, DCAT.
    </p>
  </section>

  <!-- Bannière hors-Tauri -->
  {#if !tauriAvailable}
    <div class="banner" data-tone="warn" role="alert">
      <span class="banner-ico" aria-hidden="true">
        <AlertTriangle size={18} strokeWidth={1.8} />
      </span>
      <div class="banner-body">
        <strong>Application non lancée via Tauri</strong>
        <span>
          L'application doit être lancée via <span class="mono">cargo run -p sobria-app</span> (ou
          <span class="mono">cargo tauri dev</span>). Les projets et leurs datasheets restent
          100 % locaux — aucun envoi externe.
        </span>
      </div>
    </div>
  {/if}

  {#if loadError}
    <div class="form-err" role="alert">
      <span class="err-ico"><PlugZap size={14} strokeWidth={1.8} /></span>
      <div>
        <strong>{errorLabel(loadError.code)}</strong>
        <span>{loadError.message}</span>
        {#if errorHelp(loadError.code)}
          <span class="help">{errorHelp(loadError.code)}</span>
        {/if}
      </div>
    </div>
  {/if}

  <!-- Layout 2 colonnes -->
  <div class="grid">
    <!-- ── Colonne gauche : liste projets ─────────────────────────────── -->
    <aside class="col-left" aria-labelledby="m17-list-title">
      <header class="col-head">
        <h2 id="m17-list-title" class="col-title">Projets</h2>
        <button
          type="button"
          class="btn-primary"
          onclick={openCreate}
          disabled={!tauriAvailable}
          aria-label="Nouveau projet"
        >
          <Plus size={14} strokeWidth={2} />
          Nouveau projet
        </button>
      </header>

      {#if loading && tauriAvailable}
        <div class="loading-row" aria-live="polite">
          <Loader2 size={14} strokeWidth={2} class="spin" /> Chargement…
        </div>
      {:else if projects.length === 0}
        <div class="empty-state">
          <FileText size={28} strokeWidth={1.5} />
          <p class="empty-title">Aucun projet</p>
          <p class="empty-body">
            {#if tauriAvailable}
              Créez votre premier projet pour générer un datasheet selon le standard académique
              <strong>Gebru et al. 2018</strong>.
            {:else}
              Lance l'app via <span class="mono">cargo run -p sobria-app</span> pour charger
              tes projets et générer leurs datasheets.
            {/if}
          </p>
        </div>
      {:else}
        <ul class="project-list" aria-label="Liste des projets">
          {#each projects as p (p.id)}
            {@const isActive =
              (panel.kind === 'edit' || panel.kind === 'datasheet') && panel.projectId === p.id}
            <li class="project-card" class:active={isActive}>
              <div class="pc-head">
                <h3 class="pc-name">{p.name}</h3>
              </div>
              {#if p.description}
                <p class="pc-desc">{p.description}</p>
              {/if}
              <div class="pc-meta mono">
                <Calendar size={11} strokeWidth={1.8} />
                {fmtDate(p.period_start)} → {fmtDate(p.period_end)}
              </div>
              {#if p.tags.length > 0}
                <div class="pc-tags" aria-label="Tags">
                  {#each p.tags as t (t)}
                    <span class="tag-pill">{t}</span>
                  {/each}
                </div>
              {/if}
              <div class="pc-actions">
                <button
                  type="button"
                  class="action-btn"
                  onclick={() => void openDatasheet(p)}
                  aria-label={`Voir le datasheet de ${p.name}`}
                >
                  <Eye size={12} strokeWidth={1.8} /> Datasheet
                </button>
                <button
                  type="button"
                  class="action-btn"
                  onclick={() => openEdit(p)}
                  aria-label={`Éditer le projet ${p.name}`}
                >
                  <Pencil size={12} strokeWidth={1.8} /> Éditer
                </button>
                <button
                  type="button"
                  class="action-btn danger"
                  onclick={() => (confirmDeleteId = p.id)}
                  aria-label={`Supprimer le projet ${p.name}`}
                >
                  <Trash2 size={12} strokeWidth={1.8} />
                </button>
              </div>
            </li>
          {/each}
        </ul>
      {/if}
    </aside>

    <!-- ── Colonne droite : panel contextuel ──────────────────────────── -->
    <section class="col-right" aria-live="polite">
      {#if panel.kind === 'empty'}
        <div class="placeholder">
          <FileText size={36} strokeWidth={1.4} />
          <p class="ph-title">Sélectionne ou crée un projet</p>
          <p class="ph-body">
            Le panel droit affichera ici son formulaire de création, d'édition, ou le datasheet
            Gebru généré.
          </p>
        </div>

      <!-- ─── Formulaire CREATE ─────────────────────────────────────── -->
      {:else if panel.kind === 'create'}
        <header class="panel-head">
          <div>
            <div class="ph-eyebrow">Création</div>
            <h2 class="panel-title">Nouveau projet</h2>
          </div>
          <button
            type="button"
            class="ghost-btn"
            onclick={() => (panel = { kind: 'empty' })}
            aria-label="Fermer le formulaire"
          >
            <X size={14} strokeWidth={2} />
          </button>
        </header>

        <form
          class="form"
          onsubmit={(e) => {
            e.preventDefault();
            void submitCreate();
          }}
        >
          <div class="field">
            <label for="f-name">
              Nom du projet
              <span class="req" aria-hidden="true">*</span>
            </label>
            <input
              id="f-name"
              type="text"
              bind:value={fName}
              maxlength={NAME_MAX}
              required
              aria-describedby="f-name-counter"
              placeholder="Ex: Étude Q1 2026 Claude Sonnet"
            />
            <span id="f-name-counter" class="counter mono">{fName.length} / {NAME_MAX}</span>
          </div>

          <div class="field">
            <label for="f-desc">Description</label>
            <textarea
              id="f-desc"
              bind:value={fDescription}
              maxlength={DESCRIPTION_MAX}
              rows={4}
              aria-describedby="f-desc-counter"
              placeholder="Contexte, objectif, source des prompts…"
            ></textarea>
            <span id="f-desc-counter" class="counter mono"
              >{fDescription.length} / {DESCRIPTION_MAX}</span
            >
          </div>

          <div class="row-2">
            <div class="field">
              <label for="f-pstart">Période — début<span class="req" aria-hidden="true">*</span></label>
              <input id="f-pstart" type="date" bind:value={fPeriodStart} required />
            </div>
            <div class="field">
              <label for="f-pend">Période — fin<span class="req" aria-hidden="true">*</span></label>
              <input id="f-pend" type="date" bind:value={fPeriodEnd} required />
            </div>
          </div>
          {#if fPeriodStart && fPeriodEnd && fPeriodStart >= fPeriodEnd}
            <p class="inline-err" role="alert">
              La date de début doit être antérieure à la date de fin.
            </p>
          {/if}

          <div class="field">
            <label for="f-tag-input">
              Tags
              <span class="hint mono">a-z, 0-9, '-' · max {TAGS_MAX}</span>
            </label>
            <div class="tag-input-wrap">
              {#each fTags as t (t)}
                <span class="tag-chip">
                  {t}
                  <button
                    type="button"
                    class="tag-x"
                    onclick={() => removeTag(t)}
                    aria-label={`Retirer le tag ${t}`}
                  >
                    <X size={10} strokeWidth={2} />
                  </button>
                </span>
              {/each}
              <input
                id="f-tag-input"
                type="text"
                bind:value={fTagInput}
                onkeydown={onTagKeydown}
                onblur={tryAddTag}
                placeholder={fTags.length >= TAGS_MAX ? `Limite ${TAGS_MAX} atteinte` : 'Ajouter un tag puis Entrée'}
                disabled={fTags.length >= TAGS_MAX}
                aria-describedby={fTagError ? 'f-tag-err' : undefined}
              />
            </div>
            {#if fTagError}
              <p id="f-tag-err" class="inline-err" role="alert">{fTagError}</p>
            {/if}
          </div>

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

          <div class="form-actions">
            <button
              type="button"
              class="ghost-btn"
              onclick={() => (panel = { kind: 'empty' })}
              disabled={submitting}
            >
              Annuler
            </button>
            <button type="submit" class="btn-primary" disabled={!createValid || submitting}>
              {#if submitting}
                <Loader2 size={14} strokeWidth={2} class="spin" /> Création…
              {:else}
                <Plus size={14} strokeWidth={2} /> Créer le projet
              {/if}
            </button>
          </div>
        </form>

      <!-- ─── Formulaire EDIT ───────────────────────────────────────── -->
      {:else if panel.kind === 'edit'}
        <header class="panel-head">
          <div>
            <div class="ph-eyebrow">Édition</div>
            <h2 class="panel-title">{currentProject?.name ?? 'Projet'}</h2>
          </div>
          <button
            type="button"
            class="ghost-btn"
            onclick={() => (panel = { kind: 'empty' })}
            aria-label="Fermer le formulaire"
          >
            <X size={14} strokeWidth={2} />
          </button>
        </header>

        <form
          class="form"
          onsubmit={(e) => {
            e.preventDefault();
            void submitEdit();
          }}
        >
          <div class="field">
            <label for="f-name-e">
              Nom du projet
              <span class="req" aria-hidden="true">*</span>
            </label>
            <input
              id="f-name-e"
              type="text"
              bind:value={fName}
              maxlength={NAME_MAX}
              required
              aria-describedby="f-name-e-counter"
            />
            <span id="f-name-e-counter" class="counter mono">{fName.length} / {NAME_MAX}</span>
          </div>

          <div class="field">
            <label for="f-desc-e">Description</label>
            <textarea
              id="f-desc-e"
              bind:value={fDescription}
              maxlength={DESCRIPTION_MAX}
              rows={4}
              aria-describedby="f-desc-e-counter"
            ></textarea>
            <span id="f-desc-e-counter" class="counter mono"
              >{fDescription.length} / {DESCRIPTION_MAX}</span
            >
          </div>

          <div class="row-2">
            <div class="field">
              <label for="f-pstart-e">
                Période — début
                <span class="hint">immutable</span>
              </label>
              <input
                id="f-pstart-e"
                type="date"
                value={fPeriodStart}
                readonly
                aria-describedby="f-dates-locked"
              />
            </div>
            <div class="field">
              <label for="f-pend-e">
                Période — fin
                <span class="hint">immutable</span>
              </label>
              <input
                id="f-pend-e"
                type="date"
                value={fPeriodEnd}
                readonly
                aria-describedby="f-dates-locked"
              />
            </div>
          </div>
          <p id="f-dates-locked" class="hint-row">
            Dates immutables pour préserver la reproductibilité du datasheet (cf. brief §1.1).
          </p>

          <div class="field">
            <label for="f-tag-input-e">
              Tags
              <span class="hint mono">a-z, 0-9, '-' · max {TAGS_MAX}</span>
            </label>
            <div class="tag-input-wrap">
              {#each fTags as t (t)}
                <span class="tag-chip">
                  {t}
                  <button
                    type="button"
                    class="tag-x"
                    onclick={() => removeTag(t)}
                    aria-label={`Retirer le tag ${t}`}
                  >
                    <X size={10} strokeWidth={2} />
                  </button>
                </span>
              {/each}
              <input
                id="f-tag-input-e"
                type="text"
                bind:value={fTagInput}
                onkeydown={onTagKeydown}
                onblur={tryAddTag}
                placeholder={fTags.length >= TAGS_MAX ? `Limite ${TAGS_MAX} atteinte` : 'Ajouter un tag puis Entrée'}
                disabled={fTags.length >= TAGS_MAX}
                aria-describedby={fTagError ? 'f-tag-err-e' : undefined}
              />
            </div>
            {#if fTagError}
              <p id="f-tag-err-e" class="inline-err" role="alert">{fTagError}</p>
            {/if}
          </div>

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

          <div class="form-actions">
            <button
              type="button"
              class="ghost-btn"
              onclick={() => (panel = { kind: 'empty' })}
              disabled={submitting}
            >
              Annuler
            </button>
            <button type="submit" class="btn-primary" disabled={!editValid || submitting}>
              {#if submitting}
                <Loader2 size={14} strokeWidth={2} class="spin" /> Enregistrement…
              {:else}
                <CheckCircle2 size={14} strokeWidth={2} /> Enregistrer
              {/if}
            </button>
          </div>
        </form>

      <!-- ─── DATASHEET VIEW ────────────────────────────────────────── -->
      {:else if panel.kind === 'datasheet'}
        {@const pid = panel.projectId}
        <header class="panel-head">
          <div>
            <div class="ph-eyebrow">Datasheet Gebru</div>
            <h2 class="panel-title">{currentProject?.name ?? 'Projet'}</h2>
            {#if datasheet}
              <div class="sha-row mono" title="SHA-256 du JSON-LD (signature de provenance)">
                <span class="sha-l">sha256</span>
                <code class="sha-v">{datasheet.sha256}</code>
              </div>
            {/if}
          </div>
          <div class="head-actions">
            <button
              type="button"
              class="ghost-btn"
              onclick={() => void regenerateDatasheet(pid)}
              disabled={datasheetLoading}
              aria-label="Régénérer le datasheet"
            >
              {#if datasheetLoading}
                <Loader2 size={14} strokeWidth={2} class="spin" />
              {:else}
                <RefreshCw size={14} strokeWidth={2} />
              {/if}
              Régénérer
            </button>
          </div>
        </header>

        {#if datasheetError}
          <div class="form-err" role="alert">
            <span class="err-ico"><PlugZap size={14} strokeWidth={1.8} /></span>
            <div>
              <strong>{errorLabel(datasheetError.code)}</strong>
              <span>{datasheetError.message}</span>
              {#if errorHelp(datasheetError.code)}
                <span class="help">{errorHelp(datasheetError.code)}</span>
              {/if}
            </div>
          </div>
        {:else if datasheetLoading && !datasheet}
          <p class="loading-row" aria-live="polite">
            <Loader2 size={14} strokeWidth={2} class="spin" /> Génération du datasheet…
          </p>
        {:else if datasheet}
          {@const c = datasheet.composition}
          {@const co2 = fmtCo2(c.total_co2eq_g_p50)}
          {@const en = fmtEnergy(c.total_energy_wh_p50)}
          {@const wa = fmtWater(c.total_water_l_p50)}

          <!-- Composition card -->
          <section class="composition" aria-label="Composition agrégée">
            <div class="comp-grid">
              <div class="comp-stat">
                <div class="cs-l"><Activity size={11} strokeWidth={1.8} /> Requêtes</div>
                <div class="cs-v">{fmtInt(c.total_requests)}</div>
              </div>
              <div class="comp-stat">
                <div class="cs-l"><Leaf size={11} strokeWidth={1.8} /> CO₂eq P50</div>
                <div class="cs-v">{co2.v}<span class="u">{co2.u}</span></div>
              </div>
              <div class="comp-stat">
                <div class="cs-l"><Zap size={11} strokeWidth={1.8} /> Énergie P50</div>
                <div class="cs-v">{en.v}<span class="u">{en.u}</span></div>
              </div>
              <div class="comp-stat">
                <div class="cs-l"><Droplet size={11} strokeWidth={1.8} /> Eau P50</div>
                <div class="cs-v">{wa.v}<span class="u">{wa.u}</span></div>
              </div>
            </div>
            {#if c.unique_models.length > 0}
              <div class="models-row">
                <span class="mr-l">Modèles uniques :</span>
                {#each c.unique_models as m (m)}
                  <span class="model-chip mono">{m}</span>
                {/each}
              </div>
            {/if}
            {#if c.date_first_entry && c.date_last_entry}
              <div class="dates-row mono">
                <span>1ʳᵉ entrée : {fmtDate(c.date_first_entry)}</span>
                <span class="sep">·</span>
                <span>Dernière : {fmtDate(c.date_last_entry)}</span>
              </div>
            {:else}
              <p class="empty-mini">
                Aucune entrée d'audit dans la période — le datasheet reste publiable comme cadre
                méthodologique vide.
              </p>
            {/if}
          </section>

          <!-- 7 sections Gebru repliables -->
          <div
            class="gebru-sections"
            aria-label="Sections Gebru"
            title="Format standard pour documenter datasets et modèles ML, publié par Gebru, Morgenstern, Vecchione et al. en 2018. Adopté par NeurIPS, ICML, FAccT. Sobr.ia génère ce format automatiquement depuis ton ledger d'audit, pour faciliter la publication scientifique reproductible."
          >
            {#each [{ k: 'motivation', label: '1. Motivation', text: gebruSection('motivation') }, { k: 'composition', label: '2. Composition (résumé)', text: null }, { k: 'collectionProcess', label: '3. Collection process', text: gebruSection('collectionProcess') }, { k: 'preprocessing', label: '4. Preprocessing / labeling', text: gebruSection('preprocessing') }, { k: 'uses', label: '5. Uses', text: gebruSection('uses') }, { k: 'distribution', label: '6. Distribution', text: null }, { k: 'maintenance', label: '7. Maintenance', text: null }] as section (section.k)}
              {@const isOpen = !!expandedSections[section.k]}
              <article class="gebru-card" class:open={isOpen}>
                <button
                  type="button"
                  class="gebru-head"
                  aria-expanded={isOpen}
                  aria-controls={`gebru-${section.k}`}
                  onclick={() => toggleSection(section.k)}
                >
                  <span class="gebru-label">{section.label}</span>
                  <span class="gebru-vocab mono">sobria:{section.k}</span>
                  <ChevronDown size={14} strokeWidth={2} class="chev" />
                </button>
                {#if isOpen}
                  <div class="gebru-body" id={`gebru-${section.k}`}>
                    {#if section.k === 'composition'}
                      <p class="gebru-text">
                        {fmtInt(c.total_requests)} requêtes,
                        {c.unique_models.length} modèle(s) unique(s),
                        {co2.v} {co2.u} cumulé (P50),
                        {en.v} {en.u},
                        {wa.v} {wa.u}.
                      </p>
                    {:else if section.k === 'distribution'}
                      {@const dist = gebruDistribution()}
                      {#if dist}
                        <dl class="gebru-dl">
                          {#if dist.encodingFormat}
                            <dt>Format</dt>
                            <dd class="mono">{dist.encodingFormat}</dd>
                          {/if}
                          {#if dist.license}
                            <dt>Licence code</dt>
                            <dd>
                              <a href={dist.license} target="_blank" rel="noopener noreferrer">
                                {dist.license} <ArrowUpRight size={10} strokeWidth={2} />
                              </a>
                            </dd>
                          {/if}
                          {#if dist.dataLicense}
                            <dt>Licence données</dt>
                            <dd>{dist.dataLicense}</dd>
                          {/if}
                        </dl>
                      {/if}
                    {:else if section.k === 'maintenance'}
                      {@const m = gebruMaintenance()}
                      {#if m}
                        <dl class="gebru-dl">
                          {#if m.contact}
                            <dt>Contact</dt>
                            <dd>{m.contact}</dd>
                          {/if}
                          {#if m.softwareVersion}
                            <dt>Version Sobr.ia</dt>
                            <dd class="mono">v{m.softwareVersion}</dd>
                          {/if}
                          {#if m.dateModified}
                            <dt>Dernière modification</dt>
                            <dd class="mono">{m.dateModified}</dd>
                          {/if}
                        </dl>
                      {/if}
                    {:else if section.text}
                      <p class="gebru-text">{section.text}</p>
                    {:else}
                      <p class="empty-mini">—</p>
                    {/if}
                  </div>
                {/if}
              </article>
            {/each}
          </div>

          <!-- Actions datasheet -->
          <div class="ds-actions">
            <button type="button" class="ghost-btn" onclick={() => void copyJsonld()}>
              {#if copyState === 'copied'}
                <CheckCircle2 size={14} strokeWidth={2} /> Copié
              {:else}
                <Copy size={14} strokeWidth={2} /> Copier le JSON-LD
              {/if}
            </button>
            <button
              type="button"
              class="ghost-btn"
              onclick={() => void saveJsonld()}
              disabled={!tauriAvailable || saveState === 'saving'}
            >
              {#if saveState === 'saving'}
                <Loader2 size={14} strokeWidth={2} class="spin" /> Enregistrement…
              {:else if saveState === 'saved'}
                <CheckCircle2 size={14} strokeWidth={2} /> Enregistré
              {:else}
                <Download size={14} strokeWidth={2} /> Télécharger en .jsonld
              {/if}
            </button>
            <button
              type="button"
              class="btn-primary"
              onclick={() => void generateCsrd()}
              disabled={!tauriAvailable || csrdState === 'pending'}
            >
              {#if csrdState === 'pending'}
                <Loader2 size={14} strokeWidth={2} class="spin" /> Génération…
              {:else}
                <FileText size={14} strokeWidth={2} /> Générer rapport PDF CSRD
              {/if}
            </button>
          </div>

          {#if saveState === 'saved' && saveMsg}
            <p class="action-feedback ok mono">Fichier écrit : {saveMsg}</p>
          {:else if saveState === 'error' && saveMsg}
            <p class="action-feedback err">{saveMsg}</p>
          {/if}

          {#if csrdState === 'done' && csrdMsg}
            <p class="action-feedback ok mono">PDF CSRD écrit : {csrdMsg}</p>
          {:else if csrdState === 'error' && csrdMsg}
            <p class="action-feedback err">{csrdMsg}</p>
          {/if}

          <!-- Footer méthodo -->
          <footer class="ds-footer">
            <p>
              <BookOpen size={11} strokeWidth={1.8} />
              <strong>Standard utilisé :</strong>
              <a
                href="https://arxiv.org/abs/1803.09010"
                target="_blank"
                rel="noopener noreferrer"
              >
                Gebru et al. 2018 — Datasheets for Datasets (arXiv:1803.09010)
                <ArrowUpRight size={10} strokeWidth={2} />
              </a>
            </p>
            <p class="vocab-row">
              <span class="vocab-l">Vocabulaires :</span>
              <a href="https://schema.org/" target="_blank" rel="noopener noreferrer">
                schema.org <ArrowUpRight size={10} strokeWidth={2} />
              </a>
              <span class="sep">·</span>
              <a href="https://www.w3.org/TR/prov-o/" target="_blank" rel="noopener noreferrer">
                W3C PROV-O <ArrowUpRight size={10} strokeWidth={2} />
              </a>
              <span class="sep">·</span>
              <a href="https://www.w3.org/TR/vocab-dcat-3/" target="_blank" rel="noopener noreferrer">
                DCAT <ArrowUpRight size={10} strokeWidth={2} />
              </a>
            </p>
          </footer>
        {/if}
      {/if}
    </section>
  </div>
</div>

<!-- ─── Modale de confirmation suppression ────────────────────────────── -->
{#if confirmDeleteId !== null}
  {@const target = projects.find((p) => p.id === confirmDeleteId)}
  <div
    class="modal-backdrop"
    role="dialog"
    aria-modal="true"
    aria-labelledby="confirm-title"
    onclick={() => (confirmDeleteId = null)}
    onkeydown={(e) => {
      if (e.key === 'Escape') confirmDeleteId = null;
    }}
    tabindex="-1"
  >
    <div class="modal-card" onclick={(e) => e.stopPropagation()} role="presentation">
      <h3 id="confirm-title" class="modal-title">Supprimer ce projet ?</h3>
      <p class="modal-body">
        Cette action est irréversible. Le datasheet associé ne pourra plus être régénéré.
        {#if target}
          <strong class="modal-target">« {target.name} »</strong>
        {/if}
      </p>
      <div class="modal-actions">
        <button type="button" class="ghost-btn" onclick={() => (confirmDeleteId = null)}>
          Annuler
        </button>
        <button type="button" class="btn-danger" onclick={() => void confirmDelete()}>
          <Trash2 size={14} strokeWidth={2} /> Supprimer
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .canvas-inner {
    max-width: 1280px;
    margin: 0 auto;
    padding: 40px 56px 80px;
    display: flex;
    flex-direction: column;
    gap: 22px;
  }

  /* ── TopBar ──────────────────────────────────────────────────────────── */
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

  /* ── Hero ────────────────────────────────────────────────────────────── */
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
    max-width: 760px;
    margin: 0;
  }

  /* ── Bannière warn ───────────────────────────────────────────────────── */
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

  /* ── Grid 2 colonnes ─────────────────────────────────────────────────── */
  .grid {
    display: grid;
    grid-template-columns: minmax(280px, 1fr) minmax(0, 2fr);
    gap: 22px;
    align-items: start;
  }
  .col-left,
  .col-right {
    background: linear-gradient(180deg, rgba(255, 255, 255, 0.025), rgba(255, 255, 255, 0.005));
    border: 1px solid var(--border);
    border-radius: var(--radius-xl);
    padding: 22px 24px;
  }

  /* ── Colonne gauche ──────────────────────────────────────────────────── */
  .col-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 14px;
    margin-bottom: 16px;
    padding-bottom: 12px;
    border-bottom: 1px solid var(--border);
  }
  .col-title {
    font: 400 22px/1 var(--font-display);
    font-style: italic;
    color: var(--ivory);
    margin: 0;
    letter-spacing: -0.01em;
  }

  .btn-primary {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 8px 14px;
    background: var(--lime);
    color: var(--ink);
    border: none;
    border-radius: var(--radius-md);
    font: 600 12px/1 var(--font-ui);
    cursor: pointer;
    transition: all var(--dur-base) var(--ease);
  }
  .btn-primary:hover:not(:disabled) {
    filter: brightness(1.08);
    box-shadow: 0 6px 18px -8px rgba(197, 240, 74, 0.6);
  }
  .btn-primary:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }

  .btn-danger {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 8px 14px;
    background: var(--coral);
    color: var(--ink);
    border: none;
    border-radius: var(--radius-md);
    font: 600 12px/1 var(--font-ui);
    cursor: pointer;
    transition: filter var(--dur-base) var(--ease);
  }
  .btn-danger:hover {
    filter: brightness(1.08);
  }

  .ghost-btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 7px 12px;
    background: transparent;
    color: var(--ivory-2);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    font: 500 12px/1 var(--font-ui);
    cursor: pointer;
    transition: all var(--dur-base) var(--ease);
  }
  .ghost-btn:hover:not(:disabled) {
    background: var(--surface-hi);
    color: var(--ivory);
    border-color: var(--border-hi);
  }
  .ghost-btn:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }
  .ghost-btn :global(svg.spin),
  .btn-primary :global(svg.spin) {
    animation: spin 1s linear infinite;
  }
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    padding: 40px 12px;
    text-align: center;
    color: var(--ivory-3);
  }
  .empty-state :global(svg) {
    color: var(--lime);
    opacity: 0.55;
  }
  .empty-title {
    font: 500 14px/1.2 var(--font-ui);
    color: var(--ivory);
    margin: 4px 0 0;
  }
  .empty-body {
    font: 400 12px/1.5 var(--font-ui);
    color: var(--ivory-3);
    margin: 0;
    max-width: 280px;
  }
  .empty-body strong {
    color: var(--ivory-2);
    font-weight: 600;
  }

  .loading-row {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 10px 4px;
    font: 500 12px/1 var(--font-mono);
    color: var(--ivory-3);
  }
  .loading-row :global(svg.spin) {
    animation: spin 1s linear infinite;
  }

  .project-list {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .project-card {
    padding: 14px 16px;
    background: rgba(0, 0, 0, 0.22);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    display: flex;
    flex-direction: column;
    gap: 8px;
    transition: all var(--dur-base) var(--ease);
  }
  .project-card:hover {
    background: rgba(0, 0, 0, 0.32);
    border-color: var(--border-hi);
  }
  .project-card.active {
    border-color: rgba(197, 240, 74, 0.35);
    background: rgba(197, 240, 74, 0.06);
    box-shadow: 0 0 0 1px rgba(197, 240, 74, 0.18);
  }
  .pc-name {
    font: 400 18px/1.2 var(--font-display);
    font-style: italic;
    color: var(--ivory);
    margin: 0;
    letter-spacing: -0.01em;
  }
  .pc-desc {
    font: 400 12px/1.45 var(--font-ui);
    color: var(--ivory-2);
    margin: 0;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
  .pc-meta {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font: 500 11px/1 var(--font-mono);
    color: var(--ivory-3);
  }
  .pc-tags {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }
  .tag-pill {
    display: inline-flex;
    align-items: center;
    padding: 2px 8px;
    background: var(--lime-soft);
    border: 1px solid rgba(197, 240, 74, 0.28);
    border-radius: 999px;
    font: 500 10px/1.4 var(--font-mono);
    color: var(--lime);
  }
  .pc-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    padding-top: 6px;
    border-top: 1px dashed var(--border);
  }
  .action-btn {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 5px 9px;
    background: transparent;
    border: 1px solid var(--border);
    color: var(--ivory-2);
    border-radius: var(--radius-sm);
    font: 500 10px/1 var(--font-ui);
    cursor: pointer;
    transition: all var(--dur-base) var(--ease);
  }
  .action-btn:hover {
    background: var(--surface-hi);
    color: var(--ivory);
    border-color: var(--border-hi);
  }
  .action-btn.danger {
    color: var(--coral);
    border-color: rgba(240, 108, 90, 0.32);
  }
  .action-btn.danger:hover {
    background: rgba(240, 108, 90, 0.1);
    color: var(--coral);
  }

  /* ── Colonne droite ──────────────────────────────────────────────────── */
  .col-right {
    min-height: 320px;
    display: flex;
    flex-direction: column;
    gap: 18px;
  }
  .placeholder {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 48px 24px;
    text-align: center;
    color: var(--ivory-3);
  }
  .placeholder :global(svg) {
    color: var(--lime);
    opacity: 0.4;
  }
  .ph-title {
    font: 400 22px/1.2 var(--font-display);
    font-style: italic;
    color: var(--ivory-2);
    margin: 6px 0 0;
  }
  .ph-body {
    font: 400 13px/1.5 var(--font-ui);
    color: var(--ivory-3);
    margin: 0;
    max-width: 360px;
  }
  .panel-head {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 14px;
    padding-bottom: 12px;
    border-bottom: 1px solid var(--border);
  }
  .ph-eyebrow {
    font: 500 10px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.14em;
    color: var(--ivory-3);
    margin-bottom: 6px;
  }
  .panel-title {
    font: 400 26px/1.15 var(--font-display);
    font-style: italic;
    color: var(--ivory);
    margin: 0;
    letter-spacing: -0.01em;
  }
  .head-actions {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .sha-row {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: 8px;
  }
  .sha-l {
    font: 500 10px/1 var(--font-mono);
    color: var(--ivory-4);
    text-transform: uppercase;
    letter-spacing: 0.1em;
  }
  .sha-v {
    font: 500 11px/1.3 var(--font-mono);
    color: var(--ivory-3);
    background: var(--surface);
    padding: 3px 7px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
    word-break: break-all;
    user-select: all;
  }

  /* ── Form ────────────────────────────────────────────────────────────── */
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
  .field label {
    font: 500 11px/1.2 var(--font-ui);
    color: var(--ivory-2);
    display: inline-flex;
    align-items: center;
    gap: 6px;
    text-transform: uppercase;
    letter-spacing: 0.08em;
  }
  .req {
    color: var(--coral);
    margin-left: 2px;
  }
  .hint {
    font: 400 10px/1 var(--font-mono);
    color: var(--ivory-4);
    text-transform: none;
    letter-spacing: 0;
    margin-left: 4px;
  }
  .hint-row {
    font: 400 11px/1.4 var(--font-ui);
    font-style: italic;
    color: var(--ivory-3);
    margin: -6px 0 0;
  }
  .field input,
  .field textarea {
    width: 100%;
    background: var(--surface);
    color: var(--ivory);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    padding: 9px 11px;
    font: 400 13px/1.4 var(--font-ui);
    transition: border-color var(--dur-base) var(--ease);
  }
  .field input:focus,
  .field textarea:focus {
    outline: none;
    border-color: var(--lime);
    box-shadow: 0 0 0 2px rgba(197, 240, 74, 0.15);
  }
  .field input[readonly] {
    background: rgba(255, 255, 255, 0.03);
    color: var(--ivory-3);
    cursor: not-allowed;
  }
  .field textarea {
    resize: vertical;
    font-family: var(--font-ui);
  }
  .counter {
    align-self: flex-end;
    font: 500 10px/1 var(--font-mono);
    color: var(--ivory-4);
  }
  .row-2 {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
  }
  .tag-input-wrap {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    align-items: center;
    padding: 6px 8px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    min-height: 38px;
  }
  .tag-input-wrap:focus-within {
    border-color: var(--lime);
    box-shadow: 0 0 0 2px rgba(197, 240, 74, 0.15);
  }
  .tag-input-wrap input {
    flex: 1;
    min-width: 100px;
    background: transparent;
    border: none;
    padding: 4px 2px;
    font: 400 12px/1.2 var(--font-ui);
    color: var(--ivory);
  }
  .tag-input-wrap input:focus {
    outline: none;
    box-shadow: none;
    border: none;
  }
  .tag-chip {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 3px 4px 3px 9px;
    background: var(--lime-soft);
    border: 1px solid rgba(197, 240, 74, 0.28);
    border-radius: 999px;
    font: 500 11px/1 var(--font-mono);
    color: var(--lime);
  }
  .tag-x {
    width: 16px;
    height: 16px;
    display: inline-grid;
    place-items: center;
    background: transparent;
    border: none;
    border-radius: 50%;
    color: var(--lime);
    cursor: pointer;
  }
  .tag-x:hover {
    background: rgba(197, 240, 74, 0.2);
  }
  .inline-err {
    font: 400 11px/1.4 var(--font-ui);
    color: var(--coral);
    margin: 0;
  }
  .form-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    padding-top: 8px;
    border-top: 1px solid var(--border);
  }

  /* ── Erreur form ─────────────────────────────────────────────────────── */
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

  /* ── Datasheet composition ───────────────────────────────────────────── */
  .composition {
    padding: 18px 20px;
    background: rgba(0, 0, 0, 0.22);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    display: flex;
    flex-direction: column;
    gap: 14px;
  }
  .comp-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
    gap: 12px;
  }
  .comp-stat {
    padding: 10px 12px;
    background: rgba(255, 255, 255, 0.015);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
  }
  .cs-l {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    font: 500 10px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: var(--ivory-3);
    margin-bottom: 8px;
  }
  .cs-v {
    font: 400 24px/1 var(--font-display);
    font-style: italic;
    color: var(--ivory);
    letter-spacing: -0.01em;
  }
  .cs-v .u {
    font: 400 12px/1 var(--font-ui);
    font-style: normal;
    color: var(--ivory-3);
    margin-left: 4px;
  }
  .models-row {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 6px;
    font: 400 11px/1.4 var(--font-ui);
    color: var(--ivory-3);
  }
  .mr-l {
    color: var(--ivory-3);
    margin-right: 4px;
  }
  .model-chip {
    display: inline-flex;
    align-items: center;
    padding: 2px 8px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 999px;
    font: 500 10px/1.4 var(--font-mono);
    color: var(--ivory-2);
  }
  .dates-row {
    display: flex;
    align-items: center;
    gap: 6px;
    font: 500 11px/1 var(--font-mono);
    color: var(--ivory-3);
  }
  .dates-row .sep {
    color: var(--ivory-4);
  }
  .empty-mini {
    font: 400 12px/1.4 var(--font-ui);
    font-style: italic;
    color: var(--ivory-3);
    margin: 0;
  }

  /* ── Gebru sections ──────────────────────────────────────────────────── */
  .gebru-sections {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .gebru-card {
    background: rgba(0, 0, 0, 0.18);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    overflow: hidden;
    transition: border-color var(--dur-base) var(--ease);
  }
  .gebru-card.open {
    border-color: var(--border-hi);
  }
  .gebru-head {
    width: 100%;
    display: grid;
    grid-template-columns: 1fr auto 14px;
    align-items: center;
    gap: 10px;
    padding: 11px 14px;
    background: transparent;
    border: none;
    color: var(--ivory);
    cursor: pointer;
    text-align: left;
    transition: background var(--dur-base) var(--ease);
  }
  .gebru-head:hover {
    background: var(--surface-hi);
  }
  .gebru-head:focus-visible {
    outline: 2px solid var(--lime);
    outline-offset: -2px;
  }
  .gebru-label {
    font: 500 13px/1.2 var(--font-ui);
  }
  .gebru-vocab {
    font: 500 10px/1 var(--font-mono);
    color: var(--ivory-4);
  }
  .gebru-head :global(svg.chev) {
    color: var(--ivory-3);
    transition: transform 200ms var(--ease);
  }
  .gebru-card.open .gebru-head :global(svg.chev) {
    transform: rotate(180deg);
  }
  .gebru-body {
    padding: 0 14px 14px;
    border-top: 1px dashed var(--border);
    animation: slide-down 200ms var(--ease);
    overflow: hidden;
  }
  @keyframes slide-down {
    from {
      opacity: 0;
      transform: translateY(-4px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
  .gebru-text {
    font: 400 13px/1.55 var(--font-ui);
    color: var(--ivory-2);
    margin: 12px 0 0;
    white-space: pre-wrap;
  }
  .gebru-dl {
    margin: 12px 0 0;
    display: grid;
    grid-template-columns: max-content 1fr;
    gap: 6px 16px;
  }
  .gebru-dl dt {
    font: 500 10px/1.4 var(--font-ui);
    color: var(--ivory-4);
    text-transform: uppercase;
    letter-spacing: 0.08em;
  }
  .gebru-dl dd {
    font: 400 12px/1.4 var(--font-ui);
    color: var(--ivory-2);
    margin: 0;
  }
  .gebru-dl dd a {
    color: var(--lime);
    text-decoration: none;
    display: inline-flex;
    align-items: center;
    gap: 4px;
  }
  .gebru-dl dd a:hover {
    text-decoration: underline;
  }

  /* ── Actions datasheet ───────────────────────────────────────────────── */
  .ds-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    padding-top: 8px;
    border-top: 1px solid var(--border);
  }
  .action-feedback {
    margin: 0;
    padding: 8px 12px;
    border-radius: var(--radius-sm);
    font: 400 11px/1.4 var(--font-ui);
    word-break: break-all;
  }
  .action-feedback.ok {
    background: var(--lime-soft);
    color: var(--lime);
    border: 1px solid rgba(197, 240, 74, 0.25);
  }
  .action-feedback.err {
    background: rgba(240, 108, 90, 0.08);
    color: var(--coral);
    border: 1px solid rgba(240, 108, 90, 0.3);
  }

  /* ── Footer méthodo ──────────────────────────────────────────────────── */
  .ds-footer {
    padding-top: 14px;
    border-top: 1px solid var(--border);
    font: 400 12px/1.55 var(--font-ui);
    color: var(--ivory-3);
  }
  .ds-footer p {
    margin: 0 0 6px;
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 6px;
  }
  .ds-footer a {
    color: var(--lime);
    text-decoration: none;
    display: inline-flex;
    align-items: center;
    gap: 4px;
    border-bottom: 1px dashed rgba(197, 240, 74, 0.4);
    padding-bottom: 1px;
  }
  .ds-footer a:hover {
    border-bottom-style: solid;
  }
  .vocab-row .vocab-l {
    color: var(--ivory-4);
    margin-right: 2px;
  }
  .vocab-row .sep {
    color: var(--ivory-4);
  }

  /* ── Modale confirmation ─────────────────────────────────────────────── */
  .modal-backdrop {
    position: fixed;
    inset: 0;
    z-index: 100;
    background: rgba(0, 0, 0, 0.55);
    backdrop-filter: blur(4px);
    display: grid;
    place-items: center;
    padding: 24px;
  }
  .modal-card {
    width: min(420px, 100%);
    background: var(--surface);
    border: 1px solid var(--border-hi);
    border-radius: var(--radius-xl);
    padding: 24px 26px;
    box-shadow: 0 24px 60px -16px rgba(0, 0, 0, 0.6);
  }
  .modal-title {
    font: 400 22px/1.2 var(--font-display);
    font-style: italic;
    color: var(--ivory);
    margin: 0 0 10px;
    letter-spacing: -0.01em;
  }
  .modal-body {
    font: 400 13px/1.5 var(--font-ui);
    color: var(--ivory-2);
    margin: 0 0 18px;
  }
  .modal-target {
    display: block;
    margin-top: 6px;
    color: var(--ivory);
    font-weight: 600;
  }
  .modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }

  .mono {
    font-family: var(--font-mono);
  }

  /* ── Responsive ──────────────────────────────────────────────────────── */
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
    .row-2 {
      grid-template-columns: 1fr;
    }
  }
</style>
