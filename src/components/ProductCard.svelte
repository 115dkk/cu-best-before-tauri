<script lang="ts">
  import type { EntryView } from "../lib/api";

  interface Props {
    label: string;
    entries: EntryView[];
    index: number;
    onadd: () => void;
    onedit: (at: string) => void;
    ondelete: (at: string) => void;
  }

  let { label, entries, index, onadd, onedit, ondelete }: Props = $props();

  const total = $derived(entries.reduce((sum, e) => sum + e.quantity, 0));
</script>

<section class="card rise" style:animation-delay="{index * 60}ms" aria-label={label}>
  <header class="head">
    <h3 class="name">{label}</h3>
    {#if entries.length > 0}
      <span class="badge num">{total}개 · {entries.length}건</span>
    {:else}
      <span class="badge empty">없음</span>
    {/if}
  </header>

  {#if entries.length > 0}
    <ul class="list">
      {#each entries as entry (entry.at)}
        <li class="row">
          <button type="button" class="edit press" class:past={entry.past} onclick={() => onedit(entry.at)}>
            <span class="lead">
              <span class="when num">{entry.label}</span>
              {#if entry.past}<span class="flag">지남</span>{/if}
            </span>
            <span class="qty num">{entry.quantity}<span class="unit">개</span></span>
          </button>
          <button type="button" class="icon-btn press remove" aria-label="삭제" onclick={() => ondelete(entry.at)}>
            <svg width="18" height="18" viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
              <path d="M5 5l10 10M15 5L5 15" />
            </svg>
          </button>
        </li>
      {/each}
    </ul>
  {/if}

  <button type="button" class="add press" onclick={onadd}>
    <svg width="18" height="18" viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round"><path d="M10 4v12M4 10h12" /></svg>
    추가
  </button>
</section>

<style>
  .card {
    padding: 14px 14px 12px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 0 4px;
  }
  .name {
    font-size: 18px;
    font-weight: 800;
    letter-spacing: -0.02em;
  }
  .badge {
    font-size: 12px;
    font-weight: 600;
    padding: 4px 10px;
    border-radius: 999px;
    background: var(--accent-soft);
    color: var(--accent-strong);
  }
  .badge.empty {
    background: transparent;
    color: var(--faint);
  }

  .list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .row {
    display: grid;
    grid-template-columns: 1fr 44px;
    align-items: center;
    gap: 4px;
  }
  .edit {
    display: flex;
    align-items: center;
    justify-content: space-between;
    min-height: 48px;
    padding: 0 14px;
    border-radius: 12px;
    background: var(--elev);
    text-align: left;
  }
  .lead {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
  }
  .when {
    font-size: 17px;
    font-weight: 600;
  }
  .edit.past .when {
    color: var(--muted);
  }
  .flag {
    padding: 2px 7px;
    border-radius: 999px;
    font-size: 11px;
    font-weight: 700;
    line-height: 1.4;
    color: var(--muted);
    background: var(--surface);
  }
  .qty {
    font-size: 19px;
    font-weight: 800;
    letter-spacing: -0.01em;
  }
  .unit {
    margin-left: 2px;
    font-size: 13px;
    font-weight: 600;
    color: var(--muted);
  }
  .remove {
    color: var(--faint);
  }

  .add {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    min-height: 44px;
    border-radius: 12px;
    border: 1.5px dashed var(--line-strong);
    color: var(--accent-strong);
    font-weight: 700;
    font-size: 15px;
  }
  .add:active {
    background: var(--accent-soft);
  }
</style>
