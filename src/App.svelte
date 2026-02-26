<script lang="ts">
  import { onMount } from "svelte";
  import "./app.css";
  import TitleBar from "$lib/components/title-bar.svelte";
  import AppSidebar from "$lib/components/app-sidebar.svelte";
  import { Dashboard } from "$lib/components/dashboard";
  import { Trades } from "$lib/components/trades";
  import { type View, viewTitles } from "$lib/types";
  import { loadSample } from "$lib/api";
  import { loadDashboard } from "$lib/stores/dashboard.svelte";
  import { loadTrades } from "$lib/stores/trades.svelte";

  let view: View = $state("dashboard");

  onMount(async () => {
    await loadSample();
    loadDashboard();
    loadTrades();
  });
</script>

<svelte:head>
  <title>betc — {viewTitles[view]}</title>
</svelte:head>

<div
  class="dark flex h-screen flex-col overflow-hidden bg-background text-foreground"
>
  <TitleBar title={viewTitles[view]} />

  <div class="flex flex-1 overflow-hidden">
    <AppSidebar active={view} onnavigate={(v) => (view = v)} />

    <main
      class="flex flex-1 flex-col overflow-hidden"
      style="background-image: radial-gradient(circle, oklch(1 0 0 / 0.03) 1px, transparent 1px); background-size: 24px 24px;"
    >
      {#if view === "dashboard"}
        <Dashboard />
      {:else if view === "trades"}
        <Trades />
      {:else}
        <div class="flex h-full items-center justify-center">
          <p class="text-muted-foreground">Coming soon</p>
        </div>
      {/if}
    </main>
  </div>
</div>
