<script lang="ts">
  import {
    CheckCircleIcon,
    FileSpreadsheetIcon,
    LoaderCircleIcon,
    ArrowRightLeftIcon,
    CalendarIcon,
    CoinsIcon,
    SparklesIcon,
    AlertTriangleIcon,
  } from "@lucide/svelte";
  import * as Card from "$lib/components/ui/card";
  import { Button } from "$lib/components/ui/button";
  import { cn, displayAmount } from "$lib/utils";
  import { trades } from "$lib/stores/trades.svelte";
  import type { TradesSummary } from "$lib/types/bindings";

  interface Props {
    path: string;
    summary: TradesSummary;
    confirming: boolean;
    onconfirm: () => void;
    onreplace: () => void;
  }

  let { path, summary, confirming, onconfirm, onreplace }: Props = $props();

  let filename = $derived(path.split("/").pop() ?? path);
  let existingCount = $derived(trades.rows?.length ?? 0);

  function formatDate(iso: string): string {
    return new Date(iso).toLocaleDateString("en-US", {
      month: "short",
      day: "numeric",
      year: "numeric",
    });
  }

  const cardRoot = "gap-1 py-4 shadow-none";
  const cardHeader = "px-4 gap-1";
  const cardContent = "px-4";
  const cardLabel = "text-xs font-medium uppercase tracking-wider";
  const cardValue = "text-xl font-mono";
  const cardSub = "text-xs text-muted-foreground tabular-nums";
</script>

<div class="flex flex-1 flex-col gap-6 overflow-y-auto px-8 py-6">
  <!-- File banner -->
  <div
    class="flex items-center gap-3 rounded-lg border border-success/20 bg-success/5 px-4 py-3"
  >
    <CheckCircleIcon class="size-5 shrink-0 text-success" />
    <FileSpreadsheetIcon class="size-5 shrink-0 text-muted-foreground" />
    <span class="min-w-0 flex-1 truncate text-sm font-medium">{filename}</span>
    <Button variant="ghost" size="sm" onclick={onreplace}>Replace file</Button>
  </div>

  <!-- Stat cards -->
  <div class="grid grid-cols-4 gap-4">
    <!-- Total Trades -->
    <Card.Root class={[cardRoot, "glass-panel"]}>
      <Card.Header class={cardHeader}>
        <div class="flex items-center gap-2">
          <ArrowRightLeftIcon class="size-3.5 text-muted-foreground" />
          <Card.Description class={cardLabel}>Total Trades</Card.Description>
        </div>
        <Card.Title class={cardValue}>{summary.total_trades}</Card.Title>
      </Card.Header>
      <Card.Content class={cardContent}>
        <span class={cardSub}>{summary.buys} buys · {summary.sells} sells</span>
      </Card.Content>
    </Card.Root>

    <!-- Volume -->
    <Card.Root class={[cardRoot, "glass-panel"]}>
      <Card.Header class={cardHeader}>
        <div class="flex items-center gap-2">
          <CoinsIcon class="size-3.5 text-muted-foreground" />
          <Card.Description class={cardLabel}>Volume</Card.Description>
        </div>
        <Card.Title class={cardValue}>
          {displayAmount(summary.spent).toLocaleString("en-US", {
            style: "currency",
            currency: "EUR",
            maximumFractionDigits: 0,
          })}
        </Card.Title>
      </Card.Header>
      <Card.Content class={cardContent}>
        <span class={cardSub}>
          {displayAmount(summary.fees).toLocaleString("en-US", {
            style: "currency",
            currency: "EUR",
            maximumFractionDigits: 2,
          })} fees
        </span>
      </Card.Content>
    </Card.Root>

    <!-- Date Range -->
    <Card.Root class={[cardRoot, "glass-panel"]}>
      <Card.Header class={cardHeader}>
        <div class="flex items-center gap-2">
          <CalendarIcon class="size-3.5 text-muted-foreground" />
          <Card.Description class={cardLabel}>Date Range</Card.Description>
        </div>
        <Card.Title class={cn(cardValue, "text-base")}>
          {#if summary.date_range}
            {formatDate(summary.date_range[0])}
          {:else}
            —
          {/if}
        </Card.Title>
      </Card.Header>
      <Card.Content class={cardContent}>
        <span class={cardSub}>
          {#if summary.date_range}
            to {formatDate(summary.date_range[1])}
          {/if}
        </span>
      </Card.Content>
    </Card.Root>

    <!-- New Entries (highlighted) -->
    <Card.Root
      class={[
        cardRoot,
        "glow-top-right border-primary/20 shadow-[0_0_15px] shadow-primary/5",
      ]}
    >
      <Card.Header class={cardHeader}>
        <div class="flex items-center gap-2">
          <SparklesIcon class="size-3.5 text-primary" />
          <Card.Description class={[cardLabel, "text-primary"]}>
            New Entries
          </Card.Description>
        </div>
        <Card.Title class={cardValue}>{summary.total_trades}</Card.Title>
      </Card.Header>
      <Card.Content class={cardContent}>
        <span class={cardSub}>
          {#if summary.unknown > 0}
            {summary.unknown} unrelated
          {:else}
            all matched
          {/if}
        </span>
      </Card.Content>
    </Card.Root>
  </div>

  <!-- Duplicate warning -->
  {#if existingCount > 0}
    <div
      class="flex items-start gap-3 rounded-lg border border-yellow-500/20 bg-yellow-500/5 px-4 py-3"
    >
      <AlertTriangleIcon class="mt-0.5 size-5 shrink-0 text-yellow-500" />
      <div class="text-sm">
        <p class="font-medium text-yellow-500">Heads up</p>
        <p class="mt-1 text-muted-foreground">
          You already have {existingCount} trades. Importing will add {summary.total_trades}
          more entries. Duplicate detection is not yet supported.
        </p>
      </div>
    </div>
  {/if}

  <!-- Confirm button -->
  <div class="flex justify-center pt-2">
    <Button
      size="lg"
      onclick={onconfirm}
      disabled={confirming}
      class="min-w-48"
    >
      {#if confirming}
        <LoaderCircleIcon class="size-4 animate-spin" />
        Importing...
      {:else}
        Import {summary.total_trades} Trades
      {/if}
    </Button>
  </div>
</div>
