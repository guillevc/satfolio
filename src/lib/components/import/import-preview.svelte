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
    TableIcon,
    ShieldCheckIcon,
    RotateCcwIcon,
  } from "@lucide/svelte";
  import * as Card from "$lib/components/ui/card";
  import { Badge } from "$lib/components/ui/badge";
  import { Button } from "$lib/components/ui/button";
  import { Separator } from "$lib/components/ui/separator";
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

  function formatDateShort(iso: string): string {
    return new Date(iso).toLocaleDateString("en-US", {
      month: "short",
      day: "numeric",
    });
  }

  function formatDateYear(iso: string): string {
    return new Date(iso).toLocaleDateString("en-US", { year: "numeric" });
  }

  const statCard = "gap-1 py-4 shadow-none";
  const statHeader = "px-4 gap-1";
  const statContent = "px-4";
  const statLabel = "text-xs font-medium uppercase tracking-wider";
  const statValue = "text-xl font-mono";
  const statSub = "text-xs text-muted-foreground tabular-nums";
</script>

<div class="flex flex-1 flex-col items-center justify-center px-8">
  <div class="flex w-full max-w-4xl flex-col gap-4">
    <!-- File banner -->
    <div
      class="flex items-center gap-3 rounded-xl border border-success/20 bg-success/5 px-4 py-3"
    >
      <CheckCircleIcon class="size-5 shrink-0 text-success" />
      <FileSpreadsheetIcon class="size-5 shrink-0 text-muted-foreground" />
      <div class="min-w-0 flex-1">
        <span class="truncate text-sm font-medium">
          File Ready: {filename}
        </span>
      </div>
      <Button variant="ghost" size="sm" onclick={onreplace}>
        <RotateCcwIcon class="size-3.5" />
        Replace file
      </Button>
    </div>

    <!-- Import preview card -->
    <Card.Root class="glass-panel gap-0 py-0">
      <!-- Card header -->
      <div class="flex items-center justify-between px-5 py-4">
        <div class="flex items-center gap-2 text-muted-foreground">
          <TableIcon class="size-4" />
          <span class="text-xs font-medium uppercase tracking-wider">
            Import Preview
          </span>
        </div>
        <Badge variant="outline" class="text-muted-foreground">
          <ShieldCheckIcon class="size-3" />
          Verified Kraken Format
        </Badge>
      </div>

      <Separator />

      <!-- Stat cards grid -->
      <div class="grid grid-cols-4 gap-px p-4">
        <!-- Total Rows -->
        <Card.Root class={[statCard, "bg-muted/30 rounded-lg"]}>
          <Card.Header class={statHeader}>
            <div class="flex items-center gap-2">
              <ArrowRightLeftIcon class="size-3.5 text-muted-foreground" />
              <Card.Description class={statLabel}>Total Rows</Card.Description>
            </div>
            <Card.Title class={statValue}>{summary.total_trades}</Card.Title>
          </Card.Header>
          <Card.Content class={statContent}>
            <span class={statSub}>
              {summary.buys} buys · {summary.sells} sells
            </span>
          </Card.Content>
        </Card.Root>

        <!-- Volume -->
        <Card.Root class={[statCard, "bg-muted/30 rounded-lg"]}>
          <Card.Header class={statHeader}>
            <div class="flex items-center gap-2">
              <CoinsIcon class="size-3.5 text-muted-foreground" />
              <Card.Description class={statLabel}>Volume</Card.Description>
            </div>
            <Card.Title class={statValue}>
              {displayAmount(summary.spent).toLocaleString("en-US", {
                style: "currency",
                currency: "EUR",
                maximumFractionDigits: 0,
              })}
            </Card.Title>
          </Card.Header>
          <Card.Content class={statContent}>
            <span class={statSub}>
              ~{displayAmount(summary.fees).toLocaleString("en-US", {
                style: "currency",
                currency: "EUR",
                maximumFractionDigits: 2,
              })} in total fees
            </span>
          </Card.Content>
        </Card.Root>

        <!-- Date Range -->
        <Card.Root class={[statCard, "bg-muted/30 rounded-lg"]}>
          <Card.Header class={statHeader}>
            <div class="flex items-center gap-2">
              <CalendarIcon class="size-3.5 text-muted-foreground" />
              <Card.Description class={statLabel}>Date Range</Card.Description>
            </div>
            <Card.Title class={cn(statValue, "text-base")}>
              {#if summary.date_range}
                {formatDateShort(summary.date_range[0])} &ndash; {formatDateShort(
                  summary.date_range[1],
                )}
              {:else}
                &mdash;
              {/if}
            </Card.Title>
          </Card.Header>
          <Card.Content class={statContent}>
            <span class={statSub}>
              {#if summary.date_range}
                {formatDateYear(summary.date_range[0])} fiscal year
              {/if}
            </span>
          </Card.Content>
        </Card.Root>

        <!-- New Entries (highlighted) -->
        <Card.Root
          class={[
            statCard,
            "glow-top-right rounded-lg border-primary/20 shadow-[0_0_15px] shadow-primary/5",
          ]}
        >
          <Card.Header class={statHeader}>
            <div class="flex items-center gap-2">
              <SparklesIcon class="size-3.5 text-primary" />
              <Card.Description class={[statLabel, "text-primary"]}>
                New Entries
              </Card.Description>
            </div>
            <Card.Title class={statValue}>{summary.total_trades}</Card.Title>
          </Card.Header>
          <Card.Content class={statContent}>
            <span class={statSub}>
              {#if summary.unknown > 0}
                {summary.unknown} unrelated
              {:else}
                Ready to import
              {/if}
            </span>
          </Card.Content>
        </Card.Root>
      </div>

      <!-- Duplicate warning -->
      {#if existingCount > 0}
        <div class="px-4 pb-4">
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
        </div>
      {/if}

      <!-- CTA button -->
      <div class="flex flex-col items-center gap-3 px-5 pb-5">
        <Button
          size="lg"
          onclick={onconfirm}
          disabled={confirming}
          class="w-full"
        >
          {#if confirming}
            <LoaderCircleIcon class="size-4 animate-spin" />
            Processing...
          {:else}
            <SparklesIcon class="size-4" />
            Process Data
          {/if}
        </Button>
        <p class="text-center text-xs text-muted-foreground">
          By clicking Process Data, {summary.total_trades} new transaction items will
          be added to your ledger. This action cannot be undone immediately.
        </p>
      </div>
    </Card.Root>
  </div>
</div>
