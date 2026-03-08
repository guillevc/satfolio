<script lang="ts">
  import {
    ArrowRightLeftIcon,
    CalendarIcon,
    CoinsIcon,
    SparklesIcon,
    InfoIcon,
    TableIcon,
    ShieldCheckIcon,
  } from "@lucide/svelte";
  import * as Card from "$lib/components/ui/card";
  import * as Dialog from "$lib/components/ui/dialog";
  import { Badge } from "$lib/components/ui/badge";
  import { Button } from "$lib/components/ui/button";
  import { Separator } from "$lib/components/ui/separator";
  import { Spinner } from "$lib/components/ui/spinner";
  import { cn, displayAmount } from "$lib/utils";
  import type { ImportPreview } from "$lib/types/bindings";

  interface Props {
    path: string;
    preview: ImportPreview;
    confirming: boolean;
    onconfirm: () => void;
    oncancel: () => void;
  }

  let { path, preview, confirming, onconfirm, oncancel }: Props = $props();

  let summary = $derived(preview.summary);
  let filename = $derived(path.split("/").pop() ?? path);
  let newTradeCount = $derived(summary.total_trades - preview.duplicate_trades);
  let hasOverlap = $derived(preview.duplicate_trades > 0);

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

<Dialog.Header>
  <Dialog.Title class="flex items-center gap-2">
    <TableIcon class="size-4" />
    Import Preview
  </Dialog.Title>
  <Dialog.Description class="flex items-center gap-2">
    {filename}
    <Badge variant="outline" class="text-muted-foreground">
      <ShieldCheckIcon class="size-3" />
      Verified Kraken Format
    </Badge>
  </Dialog.Description>
</Dialog.Header>

<Separator />

<!-- Stat cards grid: 3 columns top row -->
<div class="grid grid-cols-3 gap-2">
  <!-- Total Rows -->
  <Card.Root class={[statCard, "rounded-lg bg-muted/30"]}>
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
  <Card.Root class={[statCard, "rounded-lg bg-muted/30"]}>
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
  <Card.Root class={[statCard, "rounded-lg bg-muted/30"]}>
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
</div>

<!-- New Entries (highlighted, full width) -->
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
    <Card.Title class={statValue}>{newTradeCount}</Card.Title>
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

<!-- Overlap info banner -->
{#if hasOverlap}
  <div
    class="flex items-start gap-3 rounded-lg border border-blue-500/20 bg-blue-500/5 px-4 py-3"
  >
    <InfoIcon class="mt-0.5 size-5 shrink-0 text-blue-500" />
    <div class="text-sm">
      <p class="font-medium text-blue-500">Partial overlap detected</p>
      <p class="mt-1 text-muted-foreground">
        {preview.duplicate_trades} of {summary.total_trades} trades already exist
        and will be skipped. {newTradeCount} new trades will be imported.
      </p>
    </div>
  </div>
{/if}

<!-- CTA -->
<Dialog.Footer class="flex-col gap-3 sm:flex-col">
  <div class="flex gap-2">
    <Button variant="outline" onclick={oncancel} disabled={confirming}>
      Cancel
    </Button>
    <Button onclick={onconfirm} disabled={confirming} class="flex-1">
      {#if confirming}
        <Spinner />
        Processing...
      {:else}
        <SparklesIcon class="size-4" />
        Process Data
      {/if}
    </Button>
  </div>
  <p class="text-center text-xs text-muted-foreground">
    By clicking Process Data, {newTradeCount} new transaction items will be added
    to your ledger.
  </p>
</Dialog.Footer>
