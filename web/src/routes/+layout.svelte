<script lang="ts">
  import '../app.css';
  import { onMount } from 'svelte';
  import {
    Zap,
    Scale,
    TrendingUp,
    Globe,
    ShieldCheck,
    BookOpen,
    Settings2,
    Plus,
    Server,
    BarChart3,
    Library,
    Target,
    FileText
  } from '@lucide/svelte';
  import { get } from 'svelte/store';
  import { isTauriContext, SobriaIpcError } from '$lib/api';
  import { loadPreferences, preferences } from '$lib/preferences';
  import type { ModuleId } from '$lib/preferences';
  import BrandMark from '$lib/components/BrandMark.svelte';

  type Props = { children?: import('svelte').Snippet };
  let { children }: Props = $props();

  type RailItem = { label: string; icon: typeof Zap; href: string; moduleId: ModuleId };

  // Pathname réactif, autonome (évite `$app/stores` qui ne résout pas dans
  // le tsconfig SvelteKit généré — voir .svelte-kit/tsconfig.json).
  // `popstate` ne se déclenche que sur Back/Forward du navigateur ; la
  // navigation client-side de SvelteKit passe par `history.pushState`, qui
  // ne déclenche aucun évènement natif. On instrumente donc `pushState` et
  // `replaceState` (pattern standard des SDK d'analytics SPA) pour capter
  // toute mutation d'URL.
  let pathname = $state(typeof window !== 'undefined' ? window.location.pathname : '/');
  $effect(() => {
    if (typeof window === 'undefined') return;
    const update = () => {
      pathname = window.location.pathname;
    };
    const origPush = history.pushState;
    const origReplace = history.replaceState;
    history.pushState = function (...args) {
      const r = origPush.apply(this, args);
      update();
      return r;
    };
    history.replaceState = function (...args) {
      const r = origReplace.apply(this, args);
      update();
      return r;
    };
    window.addEventListener('popstate', update);
    return () => {
      window.removeEventListener('popstate', update);
      history.pushState = origPush;
      history.replaceState = origReplace;
    };
  });

  // ─── Garde d'onboarding (C10 — ADR-0010) ───────────────────────────────
  // - hors Tauri : on n'a pas de préférences et l'IPC échoue → on laisse
  //   l'app shell s'afficher tel quel, sans redirection. Les pages métier
  //   affichent leur propre bannière `tauri_unavailable` (cf. estimate.spec.ts).
  // - en Tauri : on charge les préférences. Si l'utilisateur n'a jamais fait
  //   l'onboarding (`onboarded === false`) et n'est pas déjà sur la route
  //   `/onboarding`, on redirige.
  onMount(() => {
    void (async () => {
      if (!isTauriContext()) {
        // Hors Tauri : pas d'IPC, on laisse le rail en mode "tous visibles"
        // (cf. flag `loaded` du store). L'utilisateur peut naviguer la coque.
        return;
      }
      try {
        await loadPreferences();
      } catch (e) {
        // Toute autre erreur IPC : on log mais on ne crashe pas la coque.
        // Les pages métier captureront leurs erreurs spécifiques.
        if (e instanceof SobriaIpcError) {
          console.error('[layout] preferences load failed:', e.code, e.message);
        }
        return;
      }
      // Redirige une seule fois après chargement réussi des préférences.
      const prefs = get(preferences);
      if (!prefs.onboarded && pathname !== '/onboarding') {
        // `window.location.replace` plutôt que `goto`/`$app/navigation`
        // car le tsconfig SvelteKit généré n'expose pas `$app/navigation`
        // (cf. .svelte-kit/tsconfig.json — même contrainte que le pathname
        // réactif autonome ci-dessus).
        window.location.replace('/onboarding');
      }
    })();
  });

  // Le rail référence les modules visibles selon `enabled_modules`. Tant que
  // le store n'est pas chargé (premier paint, hors Tauri), on montre tout.
  //
  // **Périmètre v1.0** (cf. ADR-0011) : 13 modules retenus, dont 11 ont une
  // route frontend livrée. M14 (À propos) et M17 (Empreinte projet)
  // attendent leur route — ils sont commentés ici, à activer dès que la
  // route `/m14`/`/m17` existe.
  //
  // Modules différés v1.1+ (M2/M5/M6/M10/M11/M16/M18/M19/M21/M23/M24) :
  // routes placeholder retirées du rail. Les backend Rust restent compilés
  // et activables manuellement via Paramètres.
  const itemsCore: RailItem[] = [
    { label: 'Estimer', icon: Zap, href: '/', moduleId: 'm1' },
    { label: 'Comparer modèles', icon: Scale, href: '/comparer', moduleId: 'm3' },
    { label: 'Simuler « Et si...? »', icon: TrendingUp, href: '/simuler', moduleId: 'm13' },
    { label: 'Tableau de bord', icon: BarChart3, href: '/m15', moduleId: 'm15' },
    { label: 'Eco-budget', icon: Target, href: '/m25', moduleId: 'm25' }
  ];
  const itemsIO: RailItem[] = [
    { label: 'Datacenters Europe', icon: Server, href: '/datacenters', moduleId: 'm12' },
    { label: 'Territoire FR', icon: Globe, href: '/territoire', moduleId: 'm20' },
    { label: 'Rapport CSRD/AGEC', icon: FileText, href: '/rapport-csrd', moduleId: 'm22' }
    // M17 Empreinte projet : ajouter ici quand la route /m17 existe
  ];
  const itemsAudit: RailItem[] = [
    { label: "Journal d'audit", icon: ShieldCheck, href: '/journal', moduleId: 'm7' },
    { label: 'Référentiel modèles', icon: Library, href: '/m9', moduleId: 'm9' },
    { label: 'Méthodologie', icon: BookOpen, href: '/methodo', moduleId: 'm8' }
    // M14 À propos : ajouter ici quand la route /m14 (ou /a-propos) existe
  ];

  function visible(item: RailItem, prefs: typeof $preferences): boolean {
    // Avant le premier load : on montre tout (mode coque / hors Tauri).
    if (!prefs.loaded) return true;
    return prefs.enabled_modules.includes(item.moduleId);
  }

  function isActive(href: string, current: string): boolean {
    return href === '/' ? current === '/' : current.startsWith(href);
  }

  // Onboarding : full-screen wizard, on cache le rail et la décoration.
  const isOnboarding = $derived(pathname === '/onboarding');
</script>

{#if isOnboarding}
  <!-- Wizard : layout autonome, pas de rail ni topo. -->
  {@render children?.()}
{:else}
  <!-- Ambient mesh + grain layer -->
  <div class="amb" aria-hidden="true"></div>

  <!-- Topographic contour decoration (top-right) -->
  <svg class="topo" viewBox="0 0 600 600" fill="none" aria-hidden="true">
    <g stroke="rgb(197 240 74)" stroke-width="0.6" fill="none" opacity="0.5">
      <path d="M 300 300 m -200, 0 a 200,200 0 1,0 400,0 a 200,200 0 1,0 -400,0" />
      <path d="M 300 300 m -160, 0 a 160,180 -10 1,0 320,20 a 160,180 -10 1,0 -320,-20" />
      <path d="M 300 300 m -120, -10 a 120,140 -20 1,0 240,40 a 120,140 -20 1,0 -240,-40" />
      <path d="M 300 300 m -80, -10 a 80,100 -30 1,0 160,40 a 80,100 -30 1,0 -160,-40" />
      <path d="M 300 300 m -45, -8 a 45,55 -40 1,0 90,20 a 45,55 -40 1,0 -90,-20" />
      <path d="M 300 300 m -18, -4 a 18,22 -50 1,0 36,8 a 18,22 -50 1,0 -36,-8" />
    </g>
  </svg>

  <div class="app">
    <nav class="rail" aria-label="Navigation principale">
      <a class="brand-mark" href="/" title="Sobr.ia" aria-label="Sobr.ia — accueil">
        <BrandMark size={44} uid="rail" />
      </a>

      {#each itemsCore as item (item.href)}
        {#if visible(item, $preferences)}
          {@const Icon = item.icon}
          {@const active = isActive(item.href, pathname)}
          <a
            class="rail-btn"
            class:active
            href={item.href}
            title={item.label}
            aria-label={item.label}
            aria-current={active ? 'page' : undefined}
            data-module-id={item.moduleId}
          >
            <Icon size={20} strokeWidth={1.6} />
          </a>
        {/if}
      {/each}

      <div class="rail-sep" aria-hidden="true"></div>

      {#each itemsIO as item (item.href)}
        {#if visible(item, $preferences)}
          {@const Icon = item.icon}
          {@const active = isActive(item.href, pathname)}
          <a
            class="rail-btn"
            class:active
            href={item.href}
            title={item.label}
            aria-label={item.label}
            aria-current={active ? 'page' : undefined}
            data-module-id={item.moduleId}
          >
            <Icon size={20} strokeWidth={1.6} />
          </a>
        {/if}
      {/each}

      <div class="rail-sep" aria-hidden="true"></div>

      {#each itemsAudit as item (item.href)}
        {#if visible(item, $preferences)}
          {@const Icon = item.icon}
          {@const active = isActive(item.href, pathname)}
          <a
            class="rail-btn"
            class:active
            href={item.href}
            title={item.label}
            aria-label={item.label}
            aria-current={active ? 'page' : undefined}
            data-module-id={item.moduleId}
          >
            <Icon size={20} strokeWidth={1.6} />
          </a>
        {/if}
      {/each}

      <div class="rail-foot">
        <a
          class="rail-btn rail-add"
          href="/parametres"
          title="Ajouter des modules"
          aria-label="Ajouter des modules — Paramètres"
        >
          <Plus size={18} strokeWidth={2} />
        </a>
        <a class="rail-btn" href="/parametres" title="Paramètres" aria-label="Paramètres">
          <Settings2 size={20} strokeWidth={1.6} />
        </a>
        <div class="rail-version">v0.3.0 · LOCAL</div>
      </div>
    </nav>

    <main class="canvas scrollable">
      {@render children?.()}
    </main>
  </div>
{/if}

<style>
  :global(html),
  :global(body) {
    overflow: hidden;
  }

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
      radial-gradient(ellipse 800px 500px at 80% 12%, rgba(197, 240, 74, 0.1), transparent 60%),
      radial-gradient(ellipse 700px 600px at 18% 88%, rgba(126, 182, 255, 0.06), transparent 65%),
      radial-gradient(ellipse 600px 400px at 50% 50%, rgba(245, 183, 105, 0.04), transparent 70%);
    filter: blur(20px);
  }
  .amb::after {
    content: '';
    position: absolute;
    inset: 0;
    background-image: url("data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='200' height='200'><filter id='n'><feTurbulence type='fractalNoise' baseFrequency='0.9' numOctaves='2' stitchTiles='stitch'/><feColorMatrix values='0 0 0 0 1, 0 0 0 0 1, 0 0 0 0 1, 0 0 0 0.05 0'/></filter><rect width='100%25' height='100%25' filter='url(%23n)'/></svg>");
    opacity: 0.7;
    mix-blend-mode: overlay;
  }

  .topo {
    position: fixed;
    top: -40px;
    right: -100px;
    width: 600px;
    height: 600px;
    z-index: 0;
    opacity: 0.12;
    pointer-events: none;
    transform-origin: center;
    animation: drift 60s linear infinite;
  }
  @keyframes drift {
    from {
      transform: rotate(0);
    }
    to {
      transform: rotate(360deg);
    }
  }

  .app {
    position: relative;
    z-index: 1;
    display: grid;
    grid-template-columns: var(--rail-w) 1fr;
    height: 100vh;
  }

  .rail {
    background: rgba(10, 13, 11, 0.55);
    backdrop-filter: blur(20px) saturate(140%);
    -webkit-backdrop-filter: blur(20px) saturate(140%);
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 22px 0 18px;
    gap: 4px;
    /* Scroll vertical autonome quand trop d'entrées (rail haut > viewport). */
    height: 100vh;
    overflow-y: auto;
    overflow-x: hidden;
    /* Firefox */
    scrollbar-width: thin;
    scrollbar-color: rgba(255, 255, 255, 0.08) transparent;
  }
  .rail::-webkit-scrollbar {
    width: 6px;
  }
  .rail::-webkit-scrollbar-track {
    background: transparent;
  }
  .rail::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.08);
    border-radius: 3px;
  }
  .rail::-webkit-scrollbar-thumb:hover {
    background: rgba(255, 255, 255, 0.16);
  }
  /* Cache la scrollbar quand inactive, montre au hover du rail. */
  @media (hover: hover) {
    .rail {
      scrollbar-color: transparent transparent;
    }
    .rail:hover {
      scrollbar-color: rgba(255, 255, 255, 0.08) transparent;
    }
    .rail::-webkit-scrollbar-thumb {
      background: transparent;
      transition: background 200ms var(--ease, ease);
    }
    .rail:hover::-webkit-scrollbar-thumb {
      background: rgba(255, 255, 255, 0.08);
    }
  }

  .brand-mark {
    width: 44px;
    height: 44px;
    display: grid;
    place-items: center;
    margin-bottom: 18px;
    position: relative;
    border-bottom: none;
    animation: breath 4s ease-in-out infinite;
  }
  .brand-mark::after {
    content: '';
    position: absolute;
    inset: -6px;
    border-radius: 50%;
    background: radial-gradient(circle, var(--lime-glow), transparent 70%);
    filter: blur(8px);
    z-index: -1;
  }
  @keyframes breath {
    0%,
    100% {
      transform: scale(1);
    }
    50% {
      transform: scale(1.04);
    }
  }

  .rail-sep {
    width: 24px;
    height: 1px;
    background: var(--border);
    margin: 10px 0;
  }

  .rail-btn {
    width: 44px;
    height: 44px;
    display: grid;
    place-items: center;
    border: none;
    background: transparent;
    color: var(--ivory-3);
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: all var(--dur-base) var(--ease);
    position: relative;
    text-decoration: none;
  }
  .rail-btn:hover {
    background: var(--surface-hi);
    color: var(--ivory);
  }
  .rail-btn:hover :global(svg) {
    transform: scale(1.12);
    transition: transform 250ms var(--ease-spring);
  }
  .rail-btn.active {
    background: var(--surface-hi);
    color: var(--lime);
  }
  .rail-btn.active::before {
    content: '';
    position: absolute;
    left: -22px;
    top: 50%;
    transform: translateY(-50%);
    width: 3px;
    height: 22px;
    background: var(--lime);
    border-radius: 2px;
    box-shadow: 0 0 12px var(--lime-glow);
  }

  .rail-add {
    color: var(--lime);
    background: var(--lime-soft);
    border: 1px dashed rgba(197, 240, 74, 0.35);
    width: 36px;
    height: 36px;
    margin-bottom: 4px;
  }
  .rail-add:hover {
    background: rgba(197, 240, 74, 0.22);
    border-style: solid;
  }

  .rail-foot {
    margin-top: auto;
    display: flex;
    flex-direction: column;
    gap: 4px;
    align-items: center;
  }
  .rail-version {
    font: 500 9px/1.3 var(--font-mono);
    color: var(--ivory-4);
    writing-mode: vertical-rl;
    text-orientation: mixed;
    letter-spacing: 0.08em;
    padding: 12px 0;
  }

  .canvas {
    overflow-y: auto;
    position: relative;
  }
</style>
