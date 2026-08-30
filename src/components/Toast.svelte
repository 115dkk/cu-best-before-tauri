<script lang="ts">
  import { fly } from "svelte/transition";
  import { toast } from "../lib/toast.svelte";
</script>

{#if toast.visible}
  <div
    class="toast"
    class:error={toast.kind === "error"}
    role="status"
    aria-live="polite"
    in:fly={{ y: 16, duration: 240 }}
    out:fly={{ y: 8, duration: 160 }}
  >
    {toast.message}
  </div>
{/if}

<style>
  .toast {
    position: fixed;
    left: 50%;
    bottom: calc(var(--safe-bottom) + 24px);
    transform: translateX(-50%);
    z-index: 50;
    max-width: min(92vw, 420px);
    padding: 12px 16px;
    border-radius: 14px;
    background: var(--elev);
    color: var(--text);
    font-size: 14px;
    font-weight: 500;
    box-shadow: var(--shadow-card);
    text-wrap: pretty;
  }
  .toast.error {
    background: var(--danger-soft);
    color: var(--danger);
  }
</style>
