<script lang="ts">
  import {
    Search,
    ChevronLeft,
    ChevronRight,
    ChevronsLeft,
    ChevronsRight,
  } from "@lucide/svelte";
  import type { SortingState, PaginationState } from "@tanstack/table-core";
  import {
    getCoreRowModel,
    getSortedRowModel,
    getPaginationRowModel,
  } from "@tanstack/table-core";
  import { createSvelteTable, FlexRender } from "$lib/components/ui/data-table";
  import * as Table from "$lib/components/ui/table";
  import { Input } from "$lib/components/ui/input";
  import { Button } from "$lib/components/ui/button";
  import * as ToggleGroup from "$lib/components/ui/toggle-group";
  import { trades } from "$lib/stores/trades.svelte";
  import {
    columns,
    isBuy,
    baseAmount,
    pricePerUnit,
    formatDate,
  } from "./columns";
  import Separator from "../ui/separator/separator.svelte";

  // ── Filter state ──────────────────────────────────────────

  let searchQuery = $state("");
  let activeFilter: "all" | "buy" | "sell" = $state("all");

  let filteredData = $derived.by(() => {
    if (!trades.rows) return [];
    let rows = trades.rows.filter((r) => r.side !== null);

    if (activeFilter === "buy") rows = rows.filter((r) => isBuy(r));
    else if (activeFilter === "sell") rows = rows.filter((r) => !isBuy(r));

    if (searchQuery.trim()) {
      const q = searchQuery.toLowerCase();
      rows = rows.filter((r) => {
        const { date, time } = formatDate(r.date);
        return (
          date.includes(q) ||
          time.includes(q) ||
          r.provider.includes(q) ||
          baseAmount(r).includes(q) ||
          String(pricePerUnit(r)).includes(q)
        );
      });
    }

    return rows;
  });

  // ── Table state ───────────────────────────────────────────

  let sorting = $state<SortingState>([{ id: "date", desc: true }]);
  let pagination = $state<PaginationState>({ pageIndex: 0, pageSize: 13 });

  // ── Dynamic page size via ResizeObserver ────────────────

  let scrollRef = $state<HTMLElement | null>(null);
  const FALLBACK_ROW_HEIGHT = 37; // text-sm line-height + p-2 + border-b
  const MIN_PAGE_SIZE = 5;

  $effect(() => {
    if (!scrollRef) return;

    const observer = new ResizeObserver(() => {
      const height = scrollRef!.clientHeight;
      const firstRow = scrollRef!.querySelector("tr");
      const rowHeight =
        firstRow?.getBoundingClientRect().height || FALLBACK_ROW_HEIGHT;
      const newPageSize = Math.max(
        MIN_PAGE_SIZE,
        Math.floor(height / rowHeight),
      );
      if (newPageSize !== pagination.pageSize) {
        const firstItem = pagination.pageIndex * pagination.pageSize;
        pagination = {
          pageSize: newPageSize,
          pageIndex: Math.floor(firstItem / newPageSize),
        };
      }
    });

    observer.observe(scrollRef);
    return () => observer.disconnect();
  });

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
    const rows = trades.rows?.filter((r) => r.side !== null);
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

<div class="flex flex-1 flex-col overflow-hidden py-4">
  {#if trades.loading}
    <div class="flex flex-1 items-center justify-center">
      <span class="text-sm text-muted-foreground">Loading…</span>
    </div>
  {:else if trades.error}
    <div class="flex flex-1 items-center justify-center">
      <span class="text-sm text-destructive">{trades.error}</span>
    </div>
  {:else if trades.rows}
    <!-- Header -->
    <div class="flex items-center gap-5 px-6">
      <h2 class="text-xl font-semibold">Trades</h2>
      <Separator orientation="vertical" class="min-h-6!" />
      <div class="flex items-center gap-5">
        <div class="relative">
          <Search
            class="pointer-events-none absolute top-1/2 left-2.5 size-4 -translate-y-1/2 text-muted-foreground"
          />
          <Input
            type="text"
            placeholder="Search trades…"
            bind:value={searchQuery}
            class="h-8 w-56 pl-8 text-sm"
          />
        </div>
        <Separator orientation="vertical" class="min-h-6!" />
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

    <Separator class="my-4" />

    <!-- Summary line -->
    {#if summaryStats}
      <p class="mb-4 px-6 font-mono text-xs text-muted-foreground">
        {summaryStats.total} trades ·
        <span class="text-success">{summaryStats.buys} buys</span> ·
        <span class="text-foreground">{summaryStats.sells} sells</span> · First:
        <span class="text-foreground">{summaryStats.firstDate}</span>
        · Latest: <span class="text-foreground">{summaryStats.lastDate}</span>
      </p>
    {/if}

    <!-- Table -->
    <div
      class="glass-panel mx-6 flex min-h-0 flex-1 flex-col **:data-[slot=table-container]:overflow-visible"
    >
      {#snippet colgroup()}
        {#each ["16%", "8%", "6%", "12%", "12%", "12%", "10%", "12%", "12%"] as w, i (i)}
          <col style:width={w} />
        {/each}
      {/snippet}

      <!-- Horizontal scroll wrapper (header + body only, footer stays outside) -->
      <div class="min-h-0 flex-1 overflow-x-auto">
        <div class="flex h-full min-w-[1100px] flex-col">
          <!-- Fixed header -->
          <Table.Root class="min-w-[1100px] table-fixed">
            {@render colgroup()}
            <Table.Header>
              {#each table.getHeaderGroups() as headerGroup (headerGroup.id)}
                <Table.Row class="border-b border-white/5 hover:bg-transparent">
                  {#each headerGroup.headers as header (header.id)}
                    <Table.Head
                      class={`first:pl-4 ${header.column.getCanSort() ? "px-0" : ""} ${(header.column.columnDef.meta as Record<string, string>)?.align === "right" ? "text-end" : ""}`}
                    >
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
          <!-- Body (page size auto-fits, no vertical scroll needed) -->
          <div bind:this={scrollRef} class="min-h-0 flex-1 overflow-hidden">
            <Table.Root class="min-w-[1100px] table-fixed">
              {@render colgroup()}
              <Table.Body>
                {#each table.getRowModel().rows as row (row.id)}
                  <Table.Row class="border-b border-white/5">
                    {#each row.getVisibleCells() as cell (cell.id)}
                      <Table.Cell class="first:pl-6 last:pr-6">
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
          </div>
        </div>
      </div>

      <!-- Footer (outside scroll area so it sticks to the bottom) -->
      {#if total > 0}
        <div
          class="flex items-center justify-between border-t border-white/5 px-4 py-3 tracking-wide"
        >
          <span class="text-xs text-muted-foreground">
            Showing {start}–{end} of {total} trades
          </span>
          <div class="flex items-center gap-1">
            <span class="mr-4 text-xs tracking-wide text-muted-foreground">
              Page {table.getState().pagination.pageIndex + 1} of {table.getPageCount()}
            </span>
            <Button
              variant="outline"
              size="icon"
              class="size-7"
              onclick={() => table.firstPage()}
              disabled={!table.getCanPreviousPage()}
            >
              <ChevronsLeft class="size-4" />
            </Button>
            <Button
              variant="outline"
              size="icon"
              class="size-7"
              onclick={() => table.previousPage()}
              disabled={!table.getCanPreviousPage()}
            >
              <ChevronLeft class="size-4" />
            </Button>
            <Button
              variant="outline"
              size="icon"
              class="size-7"
              onclick={() => table.nextPage()}
              disabled={!table.getCanNextPage()}
            >
              <ChevronRight class="size-4" />
            </Button>
            <Button
              variant="outline"
              size="icon"
              class="size-7"
              onclick={() => table.lastPage()}
              disabled={!table.getCanNextPage()}
            >
              <ChevronsRight class="size-4" />
            </Button>
          </div>
        </div>
      {/if}
    </div>
  {/if}
</div>
