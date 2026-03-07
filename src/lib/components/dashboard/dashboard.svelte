<script lang="ts">
  import { dashboard, refreshDashboard } from "$lib/stores/dashboard.svelte";
  import { trades } from "$lib/stores/trades.svelte";
  import StatCards from "./stat-cards.svelte";
  import BepChart from "./bep-chart.svelte";
</script>

<div class="flex flex-1 flex-col gap-6 overflow-y-auto p-6">
  {#if dashboard.error}
    <div class="flex flex-1 items-center justify-center">
      <span class="text-sm text-destructive">{dashboard.error}</span>
    </div>
  {:else if dashboard.loading}
    <div class="flex flex-1 items-center justify-center">
      <span class="text-sm text-muted-foreground">Loading…</span>
    </div>
  {:else if dashboard.stats && trades.rows}
    <StatCards stats={dashboard.stats} syncing={dashboard.syncing} />
    <BepChart
      trades={trades.rows}
      candles={dashboard.stats.candles}
      syncing={dashboard.syncing}
      onrefresh={refreshDashboard}
    />
  {/if}
</div>
