<script lang="ts">
  import Editor from "./components/Editor.svelte";
  import Home from "./components/Home.svelte";
  import Toast from "./components/Toast.svelte";
  import { nav } from "./lib/nav.svelte";
</script>

{#if nav.screen.name === "editor"}
  {#key nav.screen.sheetId}
    <Editor sheetId={nav.screen.sheetId} />
  {/key}
{:else}
  <Home />
{/if}

<Toast />
