<script lang="ts">
  import { api, errorMessage, type EntryView, type Location, type Product } from "../lib/api";
  import { catalog, productLabel } from "../lib/catalog.svelte";
  import { closePicker, goHome, nav, openPicker } from "../lib/nav.svelte";
  import { SheetSession, addEntry, removeEntry, replaceEntry } from "../lib/session.svelte";
  import { showToast } from "../lib/toast.svelte";
  import ProductCard from "./ProductCard.svelte";
  import SlotPicker from "./SlotPicker.svelte";

  interface Props {
    sheetId: string;
  }

  let { sheetId }: Props = $props();

  const session = new SheetSession();
  let location = $state<Location>("store");
  let exporting = $state(false);

  $effect(() => {
    let cancelled = false;
    session.load(sheetId).catch((e: unknown) => {
      if (cancelled) return;
      showToast(errorMessage(e), "error");
      goHome();
    });
    return () => {
      cancelled = true;
    };
  });

  function countOf(loc: Location): number {
    const sheet = session.sheet;
    if (!sheet) return 0;
    return catalog.products.reduce((n, p) => n + sheet.sections[loc][p.key].length, 0);
  }

  function onPicked(entry: EntryView) {
    const target = nav.picker;
    if (!target) return;
    session.apply(
      target.at === undefined
        ? addEntry(location, target.product, entry)
        : replaceEntry(location, target.product, target.at, entry),
    );
    closePicker();
  }

  function remove(product: Product, at: string) {
    session.apply(removeEntry(location, product, at));
  }

  async function exportPng() {
    const sheet = session.sheet;
    if (!sheet || exporting) return;
    exporting = true;
    try {
      const result = await api.exportSheet(sheet.id);
      showToast(`저장됨 · ${result.path}`);
    } catch (e) {
      showToast(errorMessage(e), "error");
    } finally {
      exporting = false;
    }
  }

  const pickerInitial = $derived.by((): EntryView | undefined => {
    const target = nav.picker;
    const sheet = session.sheet;
    if (!sheet || !target || target.at === undefined) return undefined;
    return sheet.sections[location][target.product].find((e) => e.at === target.at);
  });
</script>

<div class="screen">
  <header class="topbar">
    <button type="button" class="icon-btn press" aria-label="뒤로" onclick={goHome}>
      <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M15 5l-7 7 7 7" />
      </svg>
    </button>
    <div class="title">
      <span class="sub">조사표{session.saving ? " · 저장 중" : ""}</span>
      {session.sheet ? session.sheet.created_label : "불러오는 중…"}
    </div>
    <button type="button" class="btn press export" onclick={exportPng} disabled={!session.sheet || exporting}>
      <svg width="18" height="18" viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M10 3v9M6.5 8.5L10 12l3.5-3.5M4 14v2h12v-2" />
      </svg>
      PNG 저장
    </button>
  </header>

  <div class="content">
    <div class="tabs" role="tablist" aria-label="구역">
      {#each catalog.locations as loc (loc.key)}
        <button
          type="button"
          class="tab press"
          class:active={location === loc.key}
          role="tab"
          aria-selected={location === loc.key}
          onclick={() => (location = loc.key)}
        >
          {loc.label}
          <span class="count num" class:zero={countOf(loc.key) === 0}>{countOf(loc.key)}</span>
        </button>
      {/each}
    </div>

    {#if session.sheet}
      {#key location}
        <div class="cards">
          {#each catalog.products as p, i (p.key)}
            <ProductCard
              label={p.label}
              index={i}
              entries={session.sheet.sections[location][p.key]}
              onadd={() => openPicker({ product: p.key })}
              onedit={(at) => openPicker({ product: p.key, at })}
              ondelete={(at) => remove(p.key, at)}
            />
          {/each}
        </div>
      {/key}
    {/if}
  </div>
</div>

{#if session.sheet && nav.picker}
  <SlotPicker
    product={nav.picker.product}
    label={productLabel(nav.picker.product)}
    initial={pickerInitial}
    onconfirm={onPicked}
    onclose={closePicker}
  />
{/if}

<style>
  .export {
    min-height: 40px;
    padding: 0 14px;
    border-radius: 12px;
    font-size: 14px;
    background: var(--accent-soft);
    color: var(--accent-strong);
  }

  .tabs {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 4px;
    padding: 4px;
    border-radius: 18px;
    background: var(--surface);
  }
  .tab {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    min-height: 44px;
    border-radius: 14px;
    font-size: 16px;
    font-weight: 700;
    color: var(--muted);
  }
  .tab.active {
    background: var(--elev);
    color: var(--text);
  }
  .count {
    min-width: 22px;
    padding: 1px 7px;
    border-radius: 999px;
    font-size: 12px;
    font-weight: 700;
    background: var(--accent-soft);
    color: var(--accent-strong);
  }
  .count.zero {
    background: transparent;
    color: var(--faint);
  }

  .cards {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
</style>
