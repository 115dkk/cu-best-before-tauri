<script lang="ts">
  import Editor from "./components/Editor.svelte";
  import Home from "./components/Home.svelte";
  import Toast from "./components/Toast.svelte";
  import { errorMessage } from "./lib/api";
  import { catalog, loadCatalog } from "./lib/catalog.svelte";
  import { nav } from "./lib/nav.svelte";
  import { showToast } from "./lib/toast.svelte";

  $effect(() => {
    loadCatalog().catch((e: unknown) => showToast(errorMessage(e), "error"));
  });
</script>

{#if !catalog.ready}
  <div class="boot" aria-busy="true"></div>
{:else if nav.screen.name === "editor"}
  {#key nav.screen.sheetId}
    <Editor sheetId={nav.screen.sheetId} />
  {/key}
{:else}
  <Home />
{/if}

<Toast />

<style>
  .boot {
    min-height: 100dvh;
    background: var(--bg);
  }
</style>
