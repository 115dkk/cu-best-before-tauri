<script lang="ts">
  import { api, errorMessage, type SheetSummary } from "../lib/api";
  import { openEditor } from "../lib/nav.svelte";
  import { showToast } from "../lib/toast.svelte";

  let sheets = $state<SheetSummary[] | null>(null);
  let confirming = $state<string | null>(null);
  let creating = $state(false);

  async function refresh() {
    try {
      sheets = await api.listSheets();
    } catch (e) {
      showToast(errorMessage(e), "error");
      sheets = [];
    }
  }

  $effect(() => {
    void refresh();
  });

  async function createSheet() {
    if (creating) return;
    creating = true;
    try {
      const sheet = await api.createSheet();
      openEditor(sheet.id);
    } catch (e) {
      showToast(errorMessage(e), "error");
    } finally {
      creating = false;
    }
  }

  async function remove(id: string) {
    try {
      await api.deleteSheet(id);
      confirming = null;
      sheets = (sheets ?? []).filter((s) => s.id !== id);
    } catch (e) {
      showToast(errorMessage(e), "error");
    }
  }
</script>

<div class="screen">
  <header class="topbar home">
    <div class="title big">소비기한 조사표</div>
  </header>

  <div class="content">
    <button type="button" class="btn primary press new" onclick={createSheet} disabled={creating}>
      <svg width="20" height="20" viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round"><path d="M10 4v12M4 10h12" /></svg>
      새 조사표
    </button>

    {#if sheets === null}
      <p class="empty">불러오는 중…</p>
    {:else if sheets.length === 0}
      <div class="empty rise">
        <p>저장된 조사표가 없습니다.</p>
        <p class="dim">작성한 조사표는 30일 동안 보관됩니다.</p>
      </div>
    {:else}
      <ul class="list">
        {#each sheets as s, i (s.id)}
          <li class="card item rise" style:animation-delay="{Math.min(i, 8) * 50}ms">
            {#if confirming === s.id}
              <div class="confirm">
                <span>이 조사표를 삭제할까요?</span>
                <div class="actions">
                  <button type="button" class="btn ghost press" onclick={() => (confirming = null)}>취소</button>
                  <button type="button" class="btn danger press" onclick={() => remove(s.id)}>삭제</button>
                </div>
              </div>
            {:else}
              <button type="button" class="open press" onclick={() => openEditor(s.id)}>
                <span class="when">{s.created_label}</span>
                <span class="meta num">항목 {s.entry_count} · 수량 {s.total_quantity}</span>
              </button>
              <button type="button" class="icon-btn press trash" aria-label="삭제" onclick={() => (confirming = s.id)}>
                <svg width="18" height="18" viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M4 6h12M8 6V4h4v2M6 6l.8 10h6.4L14 6" />
                </svg>
              </button>
            {/if}
          </li>
        {/each}
      </ul>
    {/if}
  </div>
</div>

<style>
  .topbar.home {
    grid-template-columns: 1fr;
    padding-top: calc(var(--safe-top) + 20px);
    padding-bottom: 6px;
  }
  .title.big {
    font-size: 26px;
    font-weight: 800;
    letter-spacing: -0.02em;
    padding: 0 4px;
  }

  .new {
    min-height: 56px;
    font-size: 17px;
    border-radius: 18px;
    margin-bottom: 4px;
  }

  .empty {
    padding: 40px 12px;
    text-align: center;
    color: var(--muted);
    font-size: 15px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .dim {
    color: var(--faint);
    font-size: 13px;
  }

  .list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .item {
    display: grid;
    grid-template-columns: 1fr 44px;
    align-items: center;
    gap: 4px;
    padding: 6px;
  }
  .open {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 2px;
    min-height: 56px;
    padding: 8px 12px;
    border-radius: 16px;
    text-align: left;
  }
  .open:active {
    background: var(--elev);
  }
  .when {
    font-size: 17px;
    font-weight: 700;
  }
  .meta {
    font-size: 13px;
    color: var(--muted);
  }
  .trash {
    color: var(--faint);
  }

  .confirm {
    grid-column: 1 / -1;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding: 6px 6px 6px 12px;
    font-size: 15px;
  }
  .actions {
    display: flex;
    gap: 4px;
  }
  .actions .btn {
    min-height: 40px;
    padding: 0 14px;
    border-radius: 12px;
  }
</style>
