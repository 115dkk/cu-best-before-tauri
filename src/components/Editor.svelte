<script lang="ts">
  import {
    api,
    errorMessage,
    LOCATION_LABEL,
    LOCATIONS,
    PRODUCTS,
    type Entry,
    type Location,
    type Product,
    type Sheet,
  } from "../lib/api";
  import { sheetLabel } from "../lib/format";
  import { closeModal, goHome, nav, openModal } from "../lib/nav.svelte";
  import { showToast } from "../lib/toast.svelte";
  import ProductCard from "./ProductCard.svelte";
  import SlotPicker from "./SlotPicker.svelte";

  interface Props {
    sheetId: string;
  }

  let { sheetId }: Props = $props();

  interface PickerTarget {
    product: Product;
    /** Index of the entry being edited; undefined when adding. */
    entryIndex?: number;
  }

  let sheet = $state<Sheet | null>(null);
  let location = $state<Location>("store");
  let picker = $state<PickerTarget | null>(null);
  let exporting = $state(false);

  $effect(() => {
    let cancelled = false;
    api
      .getSheet(sheetId)
      .then((s) => {
        if (!cancelled) sheet = s;
      })
      .catch((e: unknown) => {
        showToast(errorMessage(e), "error");
        goHome();
      });
    return () => {
      cancelled = true;
    };
  });

  // The back gesture pops the modal history entry; drop the picker with it.
  $effect(() => {
    if (!nav.modal) picker = null;
  });

  function countOf(loc: Location): number {
    if (!sheet) return 0;
    return PRODUCTS.reduce((n, p) => n + sheet!.sections[loc][p].length, 0);
  }

  /** Apply a mutation to a copy, persist, and adopt the normalized result. */
  async function commit(mutate: (next: Sheet) => void) {
    if (!sheet) return;
    const next: Sheet = structuredClone($state.snapshot(sheet));
    mutate(next);
    try {
      sheet = await api.saveSheet(next);
    } catch (e) {
      showToast(errorMessage(e), "error");
    }
  }

  function openAdd(product: Product) {
    picker = { product };
    openModal();
  }

  function openEdit(product: Product, entryIndex: number) {
    picker = { product, entryIndex };
    openModal();
  }

  function onPicked(entry: Entry) {
    const target = picker;
    if (!target) return;
    const loc = location;
    void commit((next) => {
      const list = next.sections[loc][target.product];
      if (target.entryIndex === undefined) list.push(entry);
      else list.splice(target.entryIndex, 1, entry);
    });
    closeModal();
  }

  function remove(product: Product, entryIndex: number) {
    const loc = location;
    void commit((next) => {
      next.sections[loc][product].splice(entryIndex, 1);
    });
  }

  async function exportPng() {
    if (!sheet || exporting) return;
    exporting = true;
    try {
      const result = await api.exportSheet(sheet.id);
      showToast(`사진에 저장됨 · ${result.file_name}`);
    } catch (e) {
      showToast(errorMessage(e), "error");
    } finally {
      exporting = false;
    }
  }

  const pickerInitial = $derived.by((): Entry | undefined => {
    if (!sheet || !picker || picker.entryIndex === undefined) return undefined;
    return sheet.sections[location][picker.product][picker.entryIndex];
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
      {#if sheet}
        <span class="sub">조사표</span>{sheetLabel(sheet.created_at)}
      {:else}
        <span class="sub">조사표</span>불러오는 중…
      {/if}
    </div>
    <button type="button" class="btn press export" onclick={exportPng} disabled={!sheet || exporting}>
      <svg width="18" height="18" viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M10 3v9M6.5 8.5L10 12l3.5-3.5M4 14v2h12v-2" />
      </svg>
      PNG 저장
    </button>
  </header>

  <div class="content">
    <div class="tabs" role="tablist" aria-label="구역">
      {#each LOCATIONS as loc (loc)}
        <button
          type="button"
          class="tab press"
          class:active={location === loc}
          role="tab"
          aria-selected={location === loc}
          onclick={() => (location = loc)}
        >
          {LOCATION_LABEL[loc]}
          <span class="count num" class:zero={countOf(loc) === 0}>{countOf(loc)}</span>
        </button>
      {/each}
    </div>

    {#if sheet}
      {#key location}
        <div class="cards">
          {#each PRODUCTS as product, i (product)}
            <ProductCard
              {product}
              index={i}
              entries={sheet.sections[location][product]}
              onadd={() => openAdd(product)}
              onedit={(idx) => openEdit(product, idx)}
              ondelete={(idx) => remove(product, idx)}
            />
          {/each}
        </div>
      {/key}
    {/if}
  </div>
</div>

{#if sheet && picker && nav.modal}
  <SlotPicker product={picker.product} initial={pickerInitial} onconfirm={onPicked} onclose={closeModal} />
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
