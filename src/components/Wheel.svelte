<script lang="ts">
  // Scroll-snap wheel picker (iOS/One UI style). The selected item is the one
  // resting in the vertical centre; parent draws the highlight band.
  export interface WheelItem {
    key: string;
    label: string;
  }

  interface Props {
    items: WheelItem[];
    value: string;
    onchange: (key: string) => void;
    ariaLabel: string;
  }

  let { items, value, onchange, ariaLabel }: Props = $props();

  const ITEM_H = 44;
  const VISIBLE = 5;
  const HEIGHT = ITEM_H * VISIBLE;

  let el = $state<HTMLDivElement | null>(null);
  let mounted = false;
  let settleTimer: ReturnType<typeof setTimeout> | undefined;

  function clampIndex(i: number): number {
    return Math.min(Math.max(i, 0), Math.max(items.length - 1, 0));
  }

  // Keep the scroll position in sync with `value` (initial mount + external changes).
  $effect(() => {
    const idx = clampIndex(items.findIndex((it) => it.key === value));
    const node = el;
    if (!node) return;
    const top = idx * ITEM_H;
    if (Math.abs(node.scrollTop - top) > 1) {
      node.scrollTo({ top, behavior: mounted ? "smooth" : "instant" });
    }
    mounted = true;
  });

  function settle() {
    const node = el;
    if (!node) return;
    const idx = clampIndex(Math.round(node.scrollTop / ITEM_H));
    const item = items[idx];
    if (item && item.key !== value) onchange(item.key);
  }

  function onScroll() {
    // Fallback for engines without `scrollend`; harmless double-call otherwise.
    clearTimeout(settleTimer);
    settleTimer = setTimeout(settle, 140);
  }

  function pick(key: string) {
    if (key !== value) onchange(key);
  }
</script>

<div
  class="wheel"
  style:height="{HEIGHT}px"
  bind:this={el}
  onscroll={onScroll}
  onscrollend={settle}
  role="listbox"
  aria-label={ariaLabel}
  tabindex="-1"
>
  <div class="pad" style:height="{(HEIGHT - ITEM_H) / 2}px"></div>
  {#each items as item (item.key)}
    <button
      type="button"
      class="item num"
      class:active={item.key === value}
      style:height="{ITEM_H}px"
      role="option"
      aria-selected={item.key === value}
      onclick={() => pick(item.key)}
    >
      {item.label}
    </button>
  {/each}
  <div class="pad" style:height="{(HEIGHT - ITEM_H) / 2}px"></div>
</div>

<style>
  .wheel {
    position: relative;
    overflow-y: auto;
    overscroll-behavior: contain;
    scroll-snap-type: y mandatory;
    scrollbar-width: none;
    -webkit-mask-image: linear-gradient(
      to bottom,
      transparent 0%,
      #000 28%,
      #000 72%,
      transparent 100%
    );
    mask-image: linear-gradient(to bottom, transparent 0%, #000 28%, #000 72%, transparent 100%);
  }
  .wheel::-webkit-scrollbar {
    display: none;
  }
  .pad {
    flex: none;
  }
  .item {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    scroll-snap-align: center;
    scroll-snap-stop: always;
    font-size: 21px;
    font-weight: 500;
    color: var(--faint);
    letter-spacing: -0.01em;
    white-space: nowrap;
    transition-property: color, transform, font-weight;
    transition-duration: 160ms;
    transition-timing-function: var(--ease-out);
  }
  .item.active {
    color: var(--text);
    font-weight: 700;
    transform: scale(1.08);
  }
</style>
