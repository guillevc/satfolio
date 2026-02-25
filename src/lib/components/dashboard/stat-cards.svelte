<script lang="ts">
  import { TrendingUp, TrendingDown } from "@lucide/svelte";
  import * as Card from "$lib/components/ui/card";
  import { cn, displayAmount } from "$lib/utils";
  import type { Candle, PositionSummary } from "$lib/types/bindings";

  let { summary, candles }: { summary: PositionSummary; candles: Candle[] } =
    $props();

  let lastCandle = $derived(candles.at(-1));
  let prevCandle = $derived(candles.at(-2));

  let btcPrice = $derived(lastCandle ? parseFloat(lastCandle.close) : 0);
  let change24h = $derived.by(() => {
    if (!lastCandle || !prevCandle) return 0;
    const curr = parseFloat(lastCandle.close);
    const prev = parseFloat(prevCandle.close);
    return prev !== 0 ? ((curr - prev) / prev) * 100 : 0;
  });

  let held = $derived(displayAmount(summary.held));
  let invested = $derived(displayAmount(summary.invested));
  let bep = $derived(summary.bep ? parseFloat(summary.bep) : 0);
  let tradeCount = $derived(summary.buys + summary.sells);

  let pnl = $derived(btcPrice * held - invested);
  let pnlPct = $derived(invested !== 0 ? (pnl / invested) * 100 : 0);
  let fiatValue = $derived(btcPrice * held);

  function formatUsd(value: number): string {
    return new Intl.NumberFormat("en-US", {
      style: "currency",
      currency: "USD",
      maximumFractionDigits: 0,
    }).format(value);
  }

  function formatUsdFull(value: number): string {
    return new Intl.NumberFormat("en-US", {
      style: "currency",
      currency: "USD",
      minimumFractionDigits: 2,
      maximumFractionDigits: 2,
    }).format(value);
  }

  function formatBtc(value: number): string {
    return `${parseFloat(value.toFixed(4))} BTC`;
  }

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
      <Card.Title class={cardValue}>{formatUsd(btcPrice)}</Card.Title>
    </Card.Header>
    <Card.Content class={cardContent}>
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
        {change24h > 0 ? "+" : ""}{change24h.toFixed(1)}% (24h)
      </span>
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
      <Card.Title class={cardValue}>{formatUsd(bep)}</Card.Title>
    </Card.Header>
    <Card.Content class={cardContent}>
      <span class={cardSub}>{tradeCount} trades</span>
    </Card.Content>
  </Card.Root>

  <!-- Unrealized P&L -->
  <Card.Root class={[cardRoot, "glass-panel"]}>
    <Card.Header class={cardHeader}>
      <Card.Description class={cardLabel}>Unrealized P&L</Card.Description>
      <Card.Title
        class={[cardValue, pnl >= 0 ? "text-success" : "text-destructive"]}
      >
        {pnl >= 0 ? "+" : ""}{formatUsd(pnl)}
      </Card.Title>
    </Card.Header>
    <Card.Content class={cardContent}>
      <span class={cardSub}>
        {pnl >= 0 ? "+" : ""}{pnlPct.toFixed(2)}%
      </span>
    </Card.Content>
  </Card.Root>

  <!-- Total Held -->
  <Card.Root class={[cardRoot, "glass-panel"]}>
    <Card.Header class={cardHeader}>
      <Card.Description class={cardLabel}>Total Held</Card.Description>
      <Card.Title class={cardValue}>{formatBtc(held)}</Card.Title>
    </Card.Header>
    <Card.Content class={cardContent}>
      <span class={cardSub}>{formatUsdFull(fiatValue)}</span>
    </Card.Content>
  </Card.Root>
</div>
