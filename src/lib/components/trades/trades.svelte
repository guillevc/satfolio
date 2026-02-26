<script lang="ts">
  import { onMount } from "svelte";
  import { Search } from "@lucide/svelte";
  import type { SortingState, PaginationState } from "@tanstack/table-core";
  import {
    getCoreRowModel,
    getSortedRowModel,
    getPaginationRowModel,
  } from "@tanstack/table-core";
  import { createSvelteTable, FlexRender } from "$lib/components/ui/data-table";
  import * as Table from "$lib/components/ui/table";
  import { ScrollArea } from "$lib/components/ui/scroll-area";
  import { Input } from "$lib/components/ui/input";
  import { Button } from "$lib/components/ui/button";
  import * as ToggleGroup from "$lib/components/ui/toggle-group";
  import { trades, loadTrades } from "$lib/stores/trades.svelte";
  import {
    columns,
    isBuy,
    baseAmount,
    pricePerUnit,
    formatDate,
  } from "./columns";

  onMount(loadTrades);

  // ── Filter state ──────────────────────────────────────────

  let searchQuery = $state("");
  let activeFilter: "all" | "buy" | "sell" = $state("all");

  let filteredData = $derived.by(() => {
    if (!trades.rows) return [];
    let rows = trades.rows;

    if (activeFilter === "buy") rows = rows.filter((r) => isBuy(r));
    else if (activeFilter === "sell") rows = rows.filter((r) => !isBuy(r));

    if (searchQuery.trim()) {
      const q = searchQuery.toLowerCase();
      rows = rows.filter((r) => {
        const { date, time } = formatDate(r.date);
        return (
          date.includes(q) ||
          time.includes(q) ||
          baseAmount(r).includes(q) ||
          String(pricePerUnit(r)).includes(q)
        );
      });
    }

    return rows;
  });

  // ── Table state ───────────────────────────────────────────

  let sorting = $state<SortingState>([{ id: "date", desc: true }]);
  let pagination = $state<PaginationState>({ pageIndex: 0, pageSize: 15 });

  const table = createSvelteTable({
    get data() {
      return filteredData;
    },
    columns,
    state: {
      get sorting() {
        return sorting;
      },
      get pagination() {
        return pagination;
      },
    },
    getCoreRowModel: getCoreRowModel(),
    getSortedRowModel: getSortedRowModel(),
    getPaginationRowModel: getPaginationRowModel(),
    onSortingChange: (updater) => {
      sorting = typeof updater === "function" ? updater(sorting) : updater;
    },
    onPaginationChange: (updater) => {
      pagination =
        typeof updater === "function" ? updater(pagination) : updater;
    },
  });

  // ── Summary stats (from raw data, not filtered) ──────────

  let summaryStats = $derived.by(() => {
    const rows = trades.rows;
    if (!rows || rows.length === 0) return null;
    const buys = rows.filter((r) => isBuy(r)).length;
    const sells = rows.filter((r) => !isBuy(r)).length;
    const firstDate = formatDate(rows[0].date).date;
    const lastDate = formatDate(rows[rows.length - 1].date).date;
    return { total: rows.length, buys, sells, firstDate, lastDate };
  });

  // ── Pagination derived values ─────────────────────────────

  let total = $derived(filteredData.length);
  let start = $derived(
    total > 0 ? pagination.pageIndex * pagination.pageSize + 1 : 0,
  );
  let end = $derived(
    Math.min((pagination.pageIndex + 1) * pagination.pageSize, total),
  );

  // ── Handlers ──────────────────────────────────────────────

  function handleFilterChange(value: string | undefined) {
    if (value) {
      activeFilter = value as "all" | "buy" | "sell";
    }
  }
</script>

<div class="flex flex-1 flex-col overflow-hidden p-6">
  {#if trades.loading}
    <div class="flex flex-1 items-center justify-center">
      <span class="text-muted-foreground text-sm">Loading…</span>
    </div>
  {:else if trades.error}
    <div class="flex flex-1 items-center justify-center">
      <span class="text-destructive text-sm">{trades.error}</span>
    </div>
  {:else if trades.rows}
    <!-- Header -->
    <div class="mb-4 flex items-center justify-between gap-4">
      <h2 class="text-lg font-semibold">Trades</h2>

      <div class="flex items-center gap-3">
        <div class="relative">
          <Search
            class="text-muted-foreground pointer-events-none absolute left-2.5 top-1/2 size-4 -translate-y-1/2"
          />
          <Input
            type="text"
            placeholder="Search trades…"
            bind:value={searchQuery}
            class="h-8 w-56 pl-8 text-sm"
          />
        </div>

        <ToggleGroup.Root
          type="single"
          value={activeFilter}
          onValueChange={handleFilterChange}
          variant="outline"
          size="sm"
        >
          <ToggleGroup.Item value="all">All</ToggleGroup.Item>
          <ToggleGroup.Item value="buy">Buys</ToggleGroup.Item>
          <ToggleGroup.Item value="sell">Sells</ToggleGroup.Item>
        </ToggleGroup.Root>
      </div>
    </div>

    <!-- Summary line -->
    {#if summaryStats}
      <p class="text-muted-foreground mb-3 text-xs">
        {summaryStats.total} trades ·
        <span class="text-success">{summaryStats.buys} buys</span> ·
        <span class="text-destructive">{summaryStats.sells} sells</span> ·
        First: {summaryStats.firstDate} · Latest: {summaryStats.lastDate}
      </p>
    {/if}

    <!-- Table -->
    <div
      class="glass-panel flex min-h-0 flex-1 flex-col **:data-[slot=table-container]:overflow-visible"
    >
      {#snippet colgroup()}
        {#each ["15%", "7%", "12%", "12%", "9%", "11%", "14%", "14%"] as w}
          <col style:width={w} />
        {/each}
      {/snippet}

      <!-- Fixed header -->
      <Table.Root class="table-fixed">
        {@render colgroup()}
        <Table.Header>
          {#each table.getHeaderGroups() as headerGroup (headerGroup.id)}
            <Table.Row class="border-b border-white/5 hover:bg-transparent">
              {#each headerGroup.headers as header (header.id)}
                <Table.Head>
                  {#if !header.isPlaceholder}
                    <FlexRender
                      content={header.column.columnDef.header}
                      context={header.getContext()}
                    />
                  {/if}
                </Table.Head>
              {/each}
            </Table.Row>
          {/each}
        </Table.Header>
      </Table.Root>

      <!-- Scrollable body -->
      <ScrollArea class="min-h-0 flex-1" orientation="vertical">
        <Table.Root class="table-fixed">
          {@render colgroup()}
          <Table.Body>
            {#each table.getRowModel().rows as row (row.id)}
              <Table.Row class="border-b border-white/5">
                {#each row.getVisibleCells() as cell (cell.id)}
                  <Table.Cell class="last:pr-6">
                    <FlexRender
                      content={cell.column.columnDef.cell}
                      context={cell.getContext()}
                    />
                  </Table.Cell>
                {/each}
              </Table.Row>
            {:else}
              <Table.Row>
                <Table.Cell
                  colspan={columns.length}
                  class="text-muted-foreground py-8 text-center"
                >
                  No trades match your search.
                </Table.Cell>
              </Table.Row>
            {/each}
          </Table.Body>
        </Table.Root>

        <!-- Footer -->
        {#if total > 0}
          <div class="flex items-center justify-between px-4 py-3">
            <span class="text-muted-foreground text-xs">
              Showing {start}–{end} of {total} trades
            </span>
            <div class="flex items-center gap-2">
              <span class="text-muted-foreground text-xs">
                Page {table.getState().pagination.pageIndex + 1} of {table.getPageCount()}
              </span>
              <Button
                variant="outline"
                size="sm"
                onclick={() => table.previousPage()}
                disabled={!table.getCanPreviousPage()}
              >
                Previous
              </Button>
              <Button
                variant="outline"
                size="sm"
                onclick={() => table.nextPage()}
                disabled={!table.getCanNextPage()}
              >
                Next
              </Button>
            </div>
          </div>
        {/if}
      </ScrollArea>
    </div>
  {/if}
</div>
