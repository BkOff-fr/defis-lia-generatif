<script lang="ts">
  import { Server, Search, ChevronDown, X } from '@lucide/svelte';
  import type { DatacenterSummaryDto } from '$lib/api';

  interface Props {
    datacenters: DatacenterSummaryDto[];
    selected: DatacenterSummaryDto | null;
  }

  let { datacenters, selected = $bindable() }: Props = $props();

  let root: HTMLDivElement | undefined = $state();
  let searchEl: HTMLInputElement | undefined = $state();
  let open = $state(false);
  let query = $state('');
  let activeIndex = $state(0);

  const flagFor = (iso: string): string => {
    // ISO-2 letters → regional indicator emoji.
    if (iso.length !== 2) return '🏳️';
    const base = 0x1f1e6;
    const a = iso.toUpperCase().charCodeAt(0) - 65;
    const b = iso.toUpperCase().charCodeAt(1) - 65;
    return String.fromCodePoint(base + a, base + b);
  };

  const countryName = (iso: string): string => {
    try {
      return new Intl.DisplayNames(['fr'], { type: 'region' }).of(iso) ?? iso;
    } catch {
      return iso;
    }
  };

  const filtered = $derived(
    query.trim() === ''
      ? datacenters
      : datacenters.filter((dc) => {
          const q = query.toLowerCase();
          return (
            dc.name.toLowerCase().includes(q) ||
            dc.city.toLowerCase().includes(q) ||
            dc.operator.toLowerCase().includes(q) ||
            dc.country_iso.toLowerCase().includes(q) ||
            countryName(dc.country_iso).toLowerCase().includes(q)
          );
        })
  );

  const grouped = $derived.by(() => {
    const map = new Map<string, DatacenterSummaryDto[]>();
    for (const dc of filtered) {
      const arr = map.get(dc.country_iso) ?? [];
      arr.push(dc);
      map.set(dc.country_iso, arr);
    }
    return Array.from(map.entries()).sort(([a], [b]) =>
      countryName(a).localeCompare(countryName(b))
    );
  });

  function toggle() {
    open = !open;
    if (open) {
      query = '';
      activeIndex = 0;
    }
  }

  function pick(dc: DatacenterSummaryDto | null) {
    selected = dc;
    open = false;
  }

  function onKey(e: KeyboardEvent) {
    if (!open) return;
    if (e.key === 'Escape') {
      e.preventDefault();
      open = false;
    } else if (e.key === 'ArrowDown') {
      e.preventDefault();
      activeIndex = Math.min(activeIndex + 1, filtered.length); // +1 to allow "Aucun" sentinel at index 0
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      activeIndex = Math.max(activeIndex - 1, 0);
    } else if (e.key === 'Enter') {
      e.preventDefault();
      if (activeIndex === 0) pick(null);
      else pick(filtered[activeIndex - 1] ?? null);
    }
  }

  function onWindowClick(e: MouseEvent) {
    if (!open) return;
    const target = e.target as Node | null;
    if (target && root && !root.contains(target)) {
      open = false;
    }
  }

  const activeOptionId = $derived.by(() => {
    if (!open) return undefined;
    if (activeIndex === 0) return 'dc-opt-none';
    const dc = filtered[activeIndex - 1];
    return dc ? `dc-opt-${dc.id}` : undefined;
  });

  $effect(() => {
    if (open) {
      // Tick so the DOM is mounted, then focus.
      queueMicrotask(() => searchEl?.focus());
    }
  });
</script>

<svelte:window onkeydown={onKey} onclick={onWindowClick} />

<div
  class="picker"
  bind:this={root}
  role="combobox"
  tabindex="-1"
  aria-expanded={open}
  aria-haspopup="listbox"
  aria-controls="dc-picker-listbox"
  aria-activedescendant={activeOptionId}
>
  <button type="button" class="trigger" onclick={toggle} aria-label="Choisir un datacenter">
    <span class="ico"><Server size={18} strokeWidth={1.6} /></span>
    {#if selected}
      <span class="col">
        <span class="ll">Datacenter</span>
        <span class="vv">{flagFor(selected.country_iso)} {selected.name} · {selected.city}</span>
        <span class="vm"
          >{selected.operator} · {selected.if_electrical_g_per_kwh.toFixed(0)} g/kWh · PUE {selected.pue.toFixed(
            2
          )}</span
        >
      </span>
    {:else}
      <span class="col">
        <span class="ll">Datacenter</span>
        <span class="vv">Aucun choisi</span>
        <span class="vm">L'estimation utilise vos PUE/IF par défaut</span>
      </span>
    {/if}
    <span class="chev" aria-hidden="true"><ChevronDown size={14} strokeWidth={1.8} /></span>
  </button>

  {#if open}
    <div id="dc-picker-listbox" class="panel" role="listbox" tabindex="-1">
      <div class="search">
        <Search size={14} strokeWidth={1.8} />
        <input
          type="text"
          bind:value={query}
          bind:this={searchEl}
          placeholder="Rechercher (nom, ville, opérateur, pays)…"
          autocomplete="off"
        />
        {#if query}
          <button
            type="button"
            class="clear"
            aria-label="Effacer la recherche"
            onclick={() => (query = '')}
          >
            <X size={12} strokeWidth={2} />
          </button>
        {/if}
      </div>

      <ul class="options">
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <li
          id="dc-opt-none"
          class="option none"
          class:active={activeIndex === 0}
          role="option"
          aria-selected={selected === null}
          onclick={() => pick(null)}
        >
          <span class="opt-label">Aucun choisi</span>
          <span class="opt-meta">Utilise les paramètres par défaut</span>
        </li>
        {#each grouped as [iso, dcs]}
          <li class="group" role="separator">
            <span>{flagFor(iso)} {countryName(iso)}</span>
          </li>
          {#each dcs as dc}
            {@const flatIndex = filtered.indexOf(dc) + 1}
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <li
              id={`dc-opt-${dc.id}`}
              class="option"
              class:active={activeIndex === flatIndex}
              role="option"
              aria-selected={selected?.id === dc.id}
              onclick={() => pick(dc)}
            >
              <span class="opt-label">{dc.name} · {dc.city}</span>
              <span class="opt-meta"
                >{dc.operator} · {dc.if_electrical_g_per_kwh.toFixed(0)} g/kWh · PUE {dc.pue.toFixed(
                  2
                )}</span
              >
            </li>
          {/each}
        {/each}
        {#if filtered.length === 0}
          <li class="empty">Aucun datacenter ne correspond.</li>
        {/if}
      </ul>
    </div>
  {/if}
</div>

<style>
  .picker {
    position: relative;
    width: 100%;
  }
  .trigger {
    display: grid;
    grid-template-columns: auto 1fr auto;
    align-items: center;
    gap: 12px;
    width: 100%;
    padding: 12px 14px;
    background: var(--surface);
    border: 1px solid color-mix(in oklab, var(--ivory-3) 14%, transparent);
    border-radius: 12px;
    cursor: pointer;
    text-align: left;
  }
  .trigger:hover {
    border-color: color-mix(in oklab, var(--ivory-3) 28%, transparent);
  }
  .ico {
    display: grid;
    place-items: center;
    width: 32px;
    height: 32px;
    background: color-mix(in oklab, var(--lime) 12%, transparent);
    border-radius: 8px;
    color: var(--lime);
  }
  .col {
    display: grid;
    gap: 2px;
    min-width: 0;
  }
  .ll {
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--ivory-3);
  }
  .vv {
    font-size: 14px;
    color: var(--ivory);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .vm {
    font-size: 12px;
    color: var(--ivory-3);
    font-feature-settings: 'tnum';
  }
  .chev {
    color: var(--ivory-3);
  }
  .panel {
    position: absolute;
    top: calc(100% + 6px);
    left: 0;
    right: 0;
    z-index: 30;
    background: var(--ink-2);
    border: 1px solid var(--border-hi);
    border-radius: 12px;
    box-shadow: 0 12px 32px color-mix(in oklab, black 12%, transparent);
    max-height: 360px;
    display: grid;
    grid-template-rows: auto 1fr;
    overflow: hidden;
  }
  .search {
    display: grid;
    grid-template-columns: auto 1fr auto;
    align-items: center;
    gap: 8px;
    padding: 10px 12px;
    border-bottom: 1px solid color-mix(in oklab, var(--ivory-3) 10%, transparent);
    color: var(--ivory-3);
  }
  .search input {
    border: none;
    outline: none;
    background: transparent;
    color: var(--ivory);
    font-size: 13px;
    width: 100%;
  }
  .clear {
    background: transparent;
    border: none;
    color: var(--ivory-3);
    cursor: pointer;
    display: inline-flex;
    border-radius: 4px;
  }
  .clear:hover {
    color: var(--ivory);
  }
  .search:focus-within {
    color: var(--ivory-2);
    box-shadow: inset 0 -2px 0 var(--lime);
  }
  .options {
    list-style: none;
    padding: 6px 0;
    margin: 0;
    overflow-y: auto;
  }
  .group {
    padding: 8px 14px 4px;
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--ivory-3);
  }
  .option {
    display: grid;
    gap: 1px;
    padding: 8px 14px;
    cursor: pointer;
  }
  .option:hover,
  .option.active {
    background: color-mix(in oklab, var(--lime) 8%, transparent);
  }
  .option .opt-label {
    font-size: 13px;
    color: var(--ivory);
  }
  .option .opt-meta {
    font-size: 12px;
    color: var(--ivory-3);
    font-feature-settings: 'tnum';
  }
  .option.none {
    border-bottom: 1px solid color-mix(in oklab, var(--ivory-3) 8%, transparent);
  }
  .empty {
    padding: 14px;
    text-align: center;
    color: var(--ivory-3);
    font-size: 12px;
  }
</style>
