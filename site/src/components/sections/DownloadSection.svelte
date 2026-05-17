<script lang="ts">
  import { onMount } from 'svelte';

  type OsKey = 'windows' | 'macos-arm' | 'macos-intel' | 'linux' | 'android' | 'ios' | 'unknown';

  // URLs latest/download — version-agnostic, GitHub redirige sur le dernier release tagué.
  const REPO_URL = 'https://github.com/BkOff-fr/defis-lia-generatif';
  const LATEST = `${REPO_URL}/releases/latest/download`;

  interface DownloadCard {
    key: OsKey | 'chrome' | 'firefox';
    label: string;
    sublabel: string;
    href: string | null;
    badge?: string;
    icon: string; // emoji or SVG glyph
  }

  const cards: DownloadCard[] = [
    {
      key: 'windows',
      label: 'Windows',
      sublabel: '.exe installer (NSIS)',
      icon: '🪟',
      href: `${LATEST}/Sobr.ia_0.8.0_x64-setup.exe`,
    },
    {
      key: 'macos-arm',
      label: 'macOS (Apple Silicon)',
      sublabel: '.dmg · M1/M2/M3/M4',
      icon: '',
      href: `${LATEST}/Sobr.ia_0.8.0_aarch64.dmg`,
    },
    {
      key: 'macos-intel',
      label: 'macOS (Intel)',
      sublabel: '.dmg · x86_64',
      icon: '',
      href: `${LATEST}/Sobr.ia_0.8.0_x64.dmg`,
    },
    {
      key: 'linux',
      label: 'Linux',
      sublabel: '.deb / .AppImage',
      icon: '🐧',
      href: `${LATEST}/Sobr.ia_0.8.0_amd64.AppImage`,
    },
    {
      key: 'chrome',
      label: 'Chrome / Edge',
      sublabel: 'extension navigateur',
      icon: '🌐',
      href: `${LATEST}/sobria-extension-chrome-v0.8.0.zip`,
    },
    {
      key: 'firefox',
      label: 'Firefox',
      sublabel: 'extension .xpi',
      icon: '🦊',
      href: `${LATEST}/sobria-extension-firefox-v0.8.0.xpi`,
    },
    {
      key: 'android',
      label: 'Android / iOS',
      sublabel: 'mobile',
      icon: '📱',
      href: null,
      badge: 'Bientôt',
    },
  ];

  let detected: OsKey = $state('unknown');

  function detectOs(): OsKey {
    if (typeof navigator === 'undefined') return 'unknown';
    const ua = navigator.userAgent.toLowerCase();
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const platform = ((navigator as any).userAgentData?.platform ?? '').toLowerCase();

    if (platform.includes('android') || ua.includes('android')) return 'android';
    if (/iphone|ipad|ipod/.test(ua)) return 'ios';
    if (platform === 'windows' || /win(dows)?/.test(ua)) return 'windows';
    if (platform === 'macos' || /mac os|macintosh/.test(ua)) {
      // Apple Silicon detection — best-effort, navigator doesn't expose architecture
      // reliably. Default to ARM since most modern Macs are M-series.
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const arch = ((navigator as any).userAgentData?.architecture ?? '').toLowerCase();
      if (arch === 'x86' || /intel/.test(ua)) return 'macos-intel';
      return 'macos-arm';
    }
    if (platform === 'linux' || /linux|x11/.test(ua)) return 'linux';
    return 'unknown';
  }

  function isRecommended(card: DownloadCard, os: OsKey): boolean {
    if (os === card.key) return true;
    if (os === 'android' && card.key === 'android') return true;
    if (os === 'ios' && card.key === 'android') return true; // Android card couvre les 2 "Bientôt"
    return false;
  }

  onMount(() => {
    detected = detectOs();
  });
</script>

<section id="download" class="border-b border-ivory-4/20">
  <div class="max-w-6xl mx-auto px-6 py-24">
    <p class="text-xs uppercase tracking-widest text-ivory-3">4 — Télécharger Sobr.ia</p>
    <h2 class="mt-2 text-4xl md:text-5xl font-display text-ivory">
      Installez Sobr.ia en <span class="italic text-lime-signature">une minute</span>.
    </h2>
    <p class="mt-6 text-ivory-2 max-w-2xl">
      Téléchargement direct depuis GitHub Releases. Aucune inscription, aucun compte cloud. <a
        href="/telecharger/"
        class="text-lime-signature underline underline-offset-4 hover:text-lime-deep"
        >Vérifier les empreintes SHA-256 →</a
      >
    </p>

    <div class="mt-12 grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
      {#each cards as card (card.key)}
        {@const recommended = isRecommended(card, detected)}
        {@const isComingSoon = card.href === null}
        <a
          href={card.href ?? '#'}
          class:list={[
            'group rounded-lg border bg-ink-2 p-5 transition-colors flex items-start gap-4',
            recommended
              ? 'border-lime-signature/60 ring-1 ring-lime-signature/30'
              : 'border-ivory-4/40 hover:border-lime-signature/40',
            isComingSoon && 'pointer-events-none opacity-70',
          ]}
          rel={card.href?.startsWith('http') ? 'noopener external' : undefined}
          aria-disabled={isComingSoon}
        >
          <div class="text-2xl shrink-0" aria-hidden="true">{card.icon}</div>
          <div class="min-w-0 flex-1">
            <div class="flex items-center gap-2 flex-wrap">
              <h3 class="font-sans text-base text-ivory">{card.label}</h3>
              {#if recommended && !isComingSoon}
                <span
                  class="inline-flex items-center rounded-full border border-lime-signature/40 bg-lime-signature/15 text-lime-signature px-2 py-0.5 text-[10px]"
                >
                  Recommandé pour vous
                </span>
              {/if}
              {#if card.badge}
                <span
                  class="inline-flex items-center rounded-full border border-amber/40 bg-amber/10 text-amber px-2 py-0.5 text-[10px]"
                >
                  {card.badge}
                </span>
              {/if}
            </div>
            <p class="mt-1 text-xs text-ivory-3">{card.sublabel}</p>
            {#if !isComingSoon}
              <p
                class="mt-3 text-xs font-sans uppercase tracking-wider text-ivory-3 group-hover:text-lime-signature transition-colors"
              >
                Télécharger →
              </p>
            {/if}
          </div>
        </a>
      {/each}
    </div>

    <p class="mt-8 text-xs text-ivory-3 max-w-2xl">
      Première fois sur macOS ou Windows ? Voir
      <a class="underline underline-offset-4" href="/telecharger/#avertissements-os"
        >comment contourner Gatekeeper / SmartScreen</a
      >
      (binaires non encore codesignés en v0.1.0).
    </p>
  </div>
</section>
