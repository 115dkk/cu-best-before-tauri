<script lang="ts">
  import { fade, fly } from "svelte/transition";
  import { api, errorMessage, type EntryView, type Product, type SlotOptions } from "../lib/api";
  import { showToast } from "../lib/toast.svelte";
  import Wheel, { type WheelItem } from "./Wheel.svelte";

  interface Props {
    product: Product;
    label: string;
    /** Present when editing an existing entry. */
    initial?: EntryView;
    onconfirm: (entry: EntryView) => void;
    onclose: () => void;
  }

  let { product, label, initial, onconfirm, onclose }: Props = $props();

  /** Wheel range for new entries; the backend caps merged quantities at 999. */
  const WHEEL_MAX = 99;

  let options = $state<SlotOptions | null>(null);
  let dateKey = $state("");
  let timeKey = $state("");
  let qtyKey = $state("1");

  const dates = $derived(options?.dates ?? []);
  const dateItems = $derived<WheelItem[]>(
    dates.map((d) => ({ key: d.date, label: d.past ? `${d.label} · 지남` : d.label })),
  );
  const times = $derived(dates.find((d) => d.date === dateKey)?.times ?? []);
  const timeItems = $derived<WheelItem[]>(times.map((t) => ({ key: t.at, label: t.label })));
  const qtyItems = $derived.by((): WheelItem[] => {
    const values = Array.from({ length: WHEEL_MAX }, (_, i) => i + 1);
    // An entry merged past the wheel range keeps its exact quantity selectable.
    if (initial && initial.quantity > WHEEL_MAX) values.push(initial.quantity);
    return values.map((n) => ({ key: String(n), label: `${n}개` }));
  });
  const quantity = $derived(Number(qtyKey));
  const canConfirm = $derived(timeKey !== "" && quantity >= 1);

  $effect(() => {
    let cancelled = false;
    api
      .slotOptions(product, initial?.at)
      .then((opts) => {
        if (cancelled) return;
        options = opts;
        const date = opts.dates.find((d) => d.date === initial?.at.slice(0, 10)) ?? opts.dates[0];
        dateKey = date?.date ?? "";
        const time = date?.times.find((t) => t.at === initial?.at) ?? date?.times[0];
        timeKey = time?.at ?? "";
        qtyKey = String(initial?.quantity ?? 1);
      })
      .catch((e: unknown) => showToast(errorMessage(e), "error"));
    return () => {
      cancelled = true;
    };
  });

  function selectDate(key: string) {
    dateKey = key;
    const next = dates.find((d) => d.date === key)?.times ?? [];
    const prevHour = times.find((t) => t.at === timeKey)?.hour;
    const same = next.find((t) => t.hour === prevHour);
    timeKey = (same ?? next[0])?.at ?? "";
  }

  function confirm() {
    if (!canConfirm) return;
    const time = times.find((t) => t.at === timeKey);
    if (!time) return;
    // The label and `past` are recomputed by the backend on save; placeholders keep the type honest.
    const past = dates.find((d) => d.date === dateKey)?.past ?? false;
    onconfirm({ at: time.at, quantity, label: "", past });
  }
</script>

<div class="backdrop" transition:fade={{ duration: 180 }} onclick={onclose} aria-hidden="true"></div>

<div
  class="sheet"
  role="dialog"
  aria-modal="true"
  aria-label="{label} 소비기한 선택"
  in:fly={{ y: 40, duration: 280 }}
  out:fly={{ y: 14, duration: 160 }}
>
  <div class="grip" aria-hidden="true"></div>
  <header class="head">
    <div>
      <h2 class="name">{label}</h2>
      <p class="hint">{initial ? "항목 수정" : "소비기한과 수량"}</p>
    </div>
    <button type="button" class="icon-btn press" aria-label="닫기" onclick={onclose}>
      <svg width="20" height="20" viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
        <path d="M5 5l10 10M15 5L5 15" />
      </svg>
    </button>
  </header>

  <div class="columns" aria-hidden="true">
    <span class="col-label date">날짜</span>
    <span class="col-label time">시각</span>
    <span class="col-label qty">수량</span>
  </div>

  <div class="wheels" class:loading={options === null}>
    <div class="band" aria-hidden="true"></div>
    {#if options}
      <div class="col date">
        <Wheel items={dateItems} value={dateKey} onchange={selectDate} ariaLabel="날짜" />
      </div>
      <div class="col time">
        <Wheel items={timeItems} value={timeKey} onchange={(k) => (timeKey = k)} ariaLabel="시각" />
      </div>
      <div class="col qty">
        <Wheel items={qtyItems} value={qtyKey} onchange={(k) => (qtyKey = k)} ariaLabel="수량" />
      </div>
    {:else}
      <p class="placeholder">불러오는 중…</p>
    {/if}
  </div>

  <button type="button" class="btn primary press confirm" onclick={confirm} disabled={!canConfirm}>
    {initial ? "수정" : "추가"}
  </button>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 20;
    background: rgba(6, 7, 12, 0.6);
  }
  .sheet {
    position: fixed;
    left: 0;
    right: 0;
    bottom: 0;
    z-index: 21;
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 8px 20px calc(var(--safe-bottom) + 20px);
    border-radius: 28px 28px 0 0;
    background: var(--surface);
    box-shadow: var(--shadow-sheet);
  }
  .grip {
    width: 40px;
    height: 4px;
    margin: 0 auto 4px;
    border-radius: 999px;
    background: var(--line-strong);
  }
  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }
  .name {
    font-size: 22px;
    font-weight: 800;
    letter-spacing: -0.02em;
  }
  .hint {
    margin-top: 2px;
    font-size: 13px;
    color: var(--muted);
  }

  .columns,
  .wheels {
    display: flex;
    gap: 4px;
    padding: 0 8px;
  }
  .columns {
    margin-bottom: -6px;
  }
  .col-label {
    font-size: 12px;
    font-weight: 600;
    color: var(--faint);
    text-align: center;
  }
  .col,
  .col-label {
    position: relative;
    min-width: 0;
  }
  .col.date,
  .col-label.date {
    flex: 1.5;
  }
  .col.time,
  .col-label.time {
    flex: 1.1;
  }
  .col.qty,
  .col-label.qty {
    flex: 0.8;
  }

  .wheels {
    position: relative;
    border-radius: var(--radius-md);
    background: var(--card);
    padding: 4px 8px;
  }
  .wheels.loading {
    min-height: 228px;
    align-items: center;
    justify-content: center;
  }
  .band {
    position: absolute;
    left: 8px;
    right: 8px;
    top: 50%;
    height: 44px;
    transform: translateY(-50%);
    border-radius: 12px;
    background: var(--elev);
    pointer-events: none;
  }
  .placeholder {
    color: var(--muted);
    font-size: 14px;
  }

  .confirm {
    min-height: 54px;
    font-size: 17px;
    border-radius: 18px;
  }
</style>
