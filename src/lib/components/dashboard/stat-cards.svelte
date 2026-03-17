<script lang="ts">
  import { TrendingUp, TrendingDown } from "@lucide/svelte";
  import * as Card from "$lib/components/ui/card";
  import { Skeleton } from "$lib/components/ui/skeleton";
  import { cn } from "$lib/utils";
  import {
    displayAmount,
    formatBtc,
    formatCurrency,
    formatDecimal,
    formatPercent,
  } from "$lib/utils/format";
  import { getQuote } from "$lib/stores/config.svelte";
  import type { DashboardStats } from "$lib/types/bindings";

  let { stats, syncing }: { stats: DashboardStats; syncing: boolean } =
    $props();

  let quote = $derived(getQuote());
  let btcPrice = $derived(displayAmount(stats.btc_price));
  let change24h = $derived(parseFloat(stats.change_24h_pct));
  let bep = $derived(stats.bep ? displayAmount(stats.bep) : 0);
  let held = $derived(displayAmount(stats.held));
  let positionValue = $derived(displayAmount(stats.position_value));
  let pnl = $derived(displayAmount(stats.unrealized_pnl));
  let pnlPct = $derived(parseFloat(stats.unrealized_pnl_pct));

  const cardRoot = "gap-1 py-4 shadow-none";
  const cardHeader = "px-4 gap-1";
  const cardContent = "px-4";
  const cardLabel = "text-xs font-medium uppercase tracking-wider";
  const cardValue = "text-xl font-mono";
  const cardSub = "text-xs text-muted-foreground tabular-nums";
</script>

<div class="grid grid-cols-4 gap-4">
  <!-- BTC Price -->
  <Card.Root class={[cardRoot, "glass-panel"]}>
    <Card.Header class={cardHeader}>
      <Card.Description class={cardLabel}>BTC Price</Card.Description>
      {#if !syncing}
        <Card.Title class={cardValue}
          >{formatCurrency(btcPrice, quote)}</Card.Title
        >
      {:else}
        <Skeleton class="h-7 w-28" />
      {/if}
    </Card.Header>
    <Card.Content class={cardContent}>
      {#if !syncing}
        <span
          class={cn(
            cardSub,
            "inline-flex items-center gap-1",
            change24h >= 0 ? "text-success" : "text-destructive",
          )}
        >
          {#if change24h >= 0}
            <TrendingUp class="size-3.5" />
          {:else}
            <TrendingDown class="size-3.5" />
          {/if}
          {change24h > 0 ? "+" : ""}{formatPercent(change24h, 1)} (24h)
        </span>
      {:else}
        <Skeleton class="h-4 w-20" />
      {/if}
    </Card.Content>
  </Card.Root>

  <!-- Break-Even Price -->
  <Card.Root
    class={[
      cardRoot,
      "glow-top-right border-primary/20 shadow-[0_0_15px] shadow-primary/5",
    ]}
  >
    <Card.Header class={cardHeader}>
      <Card.Description class={[cardLabel, "text-primary"]}
        >Break-Even Price</Card.Description
      >
      <Card.Title class={cardValue}>{formatCurrency(bep, quote)}</Card.Title>
    </Card.Header>
    <Card.Content class={cardContent}>
      <span class={cardSub}>{formatDecimal(stats.trade_count, 0)} trades</span>
    </Card.Content>
  </Card.Root>

  <!-- Unrealized P&L -->
  <Card.Root class={[cardRoot, "glass-panel"]}>
    <Card.Header class={cardHeader}>
      <Card.Description class={cardLabel}>Unrealized P&L</Card.Description>
      {#if !syncing}
        <Card.Title
          class={[cardValue, pnl >= 0 ? "text-success" : "text-destructive"]}
        >
          {pnl >= 0 ? "+" : ""}{formatCurrency(pnl, quote)}
        </Card.Title>
      {:else}
        <Skeleton class="h-7 w-28" />
      {/if}
    </Card.Header>
    <Card.Content class={cardContent}>
      {#if !syncing}
        <span class={cardSub}>
          {pnl >= 0 ? "+" : ""}{formatPercent(pnlPct, 2)}
        </span>
      {:else}
        <Skeleton class="h-4 w-20" />
      {/if}
    </Card.Content>
  </Card.Root>

  <!-- Total Held -->
  <Card.Root class={[cardRoot, "glass-panel"]}>
    <Card.Header class={cardHeader}>
      <Card.Description class={cardLabel}>Total Held</Card.Description>
      <Card.Title class={cardValue}>{formatBtc(held)}</Card.Title>
    </Card.Header>
    <Card.Content class={cardContent}>
      <span class={cardSub}>{formatCurrency(positionValue, quote, 2)}</span>
    </Card.Content>
  </Card.Root>
</div>
