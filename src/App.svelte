<script lang="ts">
  import { onMount } from "svelte";
  import "./app.css";
  import TitleBar from "$lib/components/title-bar.svelte";
  import AppSidebar from "$lib/components/app-sidebar.svelte";
  import { Dashboard } from "$lib/components/dashboard";
  import { Trades } from "$lib/components/trades";
  import { ImportPage } from "$lib/components/import";
  import { Settings } from "$lib/components/settings";
  import { type View, viewTitles } from "$lib/types";
  import { loadSample } from "$lib/api";
  import { loadDashboard } from "$lib/stores/dashboard.svelte";
  import { loadTrades } from "$lib/stores/trades.svelte";
  import {
    importedFiles,
    loadImportedFiles,
  } from "$lib/stores/imported-files.svelte";

  let view: View = $state("import");
  let hasImports = $derived(importedFiles.list.length > 0);

  onMount(async () => {
    await loadImportedFiles();
    if (hasImports) {
      view = "dashboard";
      await loadSample();
      loadDashboard();
      loadTrades();
    }
  });
</script>

<svelte:head>
  <title>betc — {viewTitles[view]}</title>
</svelte:head>

<div
  class="flex h-screen flex-col overflow-hidden bg-background text-foreground"
>
  <TitleBar title={viewTitles[view]} />

  <div class="flex flex-1 overflow-hidden">
    <AppSidebar active={view} onnavigate={(v) => (view = v)} {hasImports} />

    <main
      class="flex flex-1 flex-col overflow-hidden"
      style="background-image: radial-gradient(circle, oklch(1 0 0 / 0.03) 1px, transparent 1px); background-size: 24px 24px;"
    >
      {#if view === "dashboard"}
        <Dashboard />
      {:else if view === "trades"}
        <Trades />
      {:else if view === "import"}
        <ImportPage />
      {:else if view === "settings"}
        <Settings />
      {/if}
    </main>
  </div>
</div>
