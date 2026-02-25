<script lang="ts">
  import { onMount } from "svelte";
  import { Search, ArrowDownLeft, ArrowUpRight } from "@lucide/svelte";
  import * as Table from "$lib/components/ui/table";
  import { Input } from "$lib/components/ui/input";
  import * as Pagination from "$lib/components/ui/pagination";
  import * as ToggleGroup from "$lib/components/ui/toggle-group";
  import { trades, loadTrades } from "$lib/stores/trades.svelte";
  import type { EnrichedTrade } from "$lib/types/bindings";

  const PAGE_SIZE = 25;
  const QUOTE_ASSET = "EUR";

  let searchQuery: string = $state("");
  let activeFilter: "all" | "buy" | "sell" = $state("all");
  let currentPage: number = $state(1);

  onMount(loadTrades);

  // ── Trade helpers ───────────────────────────────────────

  function isBuy(t: EnrichedTrade): boolean {
    return t.spent.asset === QUOTE_ASSET;
  }

  function baseAmount(t: EnrichedTrade): string {
    return isBuy(t) ? t.received.amount : t.spent.amount;
  }

  function quoteAmount(t: EnrichedTrade): string {
    return isBuy(t) ? t.spent.amount : t.received.amount;
  }

  function pricePerUnit(t: EnrichedTrade): number {
    const units = parseFloat(baseAmount(t));
    if (units === 0) return 0;
    return parseFloat(quoteAmount(t)) / units;
  }

  // ── Formatters ─────────────────────────────────────────

  const fiatFmt = new Intl.NumberFormat("en-US", {
    style: "currency",
    currency: "USD",
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  });

  function formatDate(iso: string): { date: string; time: string } {
    const d = new Date(iso);
    const date = d.toLocaleDateString("en-CA"); // YYYY-MM-DD
    const time = d.toLocaleTimeString("en-GB", {
      hour: "2-digit",
      minute: "2-digit",
    });
    return { date, time };
  }

  function formatBtc(t: EnrichedTrade): string {
    const n = parseFloat(baseAmount(t));
    const prefix = isBuy(t) ? "+" : "-";
    const formatted = n.toFixed(6).replace(/0+$/, "").replace(/\.$/, "");
    return `${prefix}${formatted}`;
  }

  function formatFiat(value: string | number): string {
    return fiatFmt.format(
      typeof value === "number" ? value : parseFloat(value),
    );
  }

  function formatPnl(value: string): string {
    const n = parseFloat(value);
    const formatted = fiatFmt.format(Math.abs(n));
    return n >= 0 ? `+${formatted}` : `-${formatted}`;
  }

  // ── Derived state ──────────────────────────────────────

  let filteredRows = $derived.by(() => {
    if (!trades.rows) return [];
    let rows = trades.rows;

    if (activeFilter === "buy") {
      rows = rows.filter((r) => isBuy(r));
    } else if (activeFilter === "sell") {
      rows = rows.filter((r) => !isBuy(r));
    }

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

  let totalPages = $derived(
    Math.max(1, Math.ceil(filteredRows.length / PAGE_SIZE)),
  );

  let paginatedRows = $derived(
    filteredRows.slice((currentPage - 1) * PAGE_SIZE, currentPage * PAGE_SIZE),
  );

  let summaryStats = $derived.by(() => {
    const rows = trades.rows;
    if (!rows || rows.length === 0) return null;
    const buys = rows.filter((r) => isBuy(r)).length;
    const sells = rows.filter((r) => !isBuy(r)).length;
    const firstDate = formatDate(rows[0].date).date;
    const lastDate = formatDate(rows[rows.length - 1].date).date;
    return { total: rows.length, buys, sells, firstDate, lastDate };
  });

  // ── Handlers (reset page on filter/search change) ──────

  function handleFilterChange(value: string | undefined) {
    if (value) {
      activeFilter = value as "all" | "buy" | "sell";
      currentPage = 1;
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
      <h2 class="text-lg font-semibold">Transactions</h2>

      <div class="flex items-center gap-3">
        <div class="relative">
          <Search
            class="text-muted-foreground pointer-events-none absolute left-2.5 top-1/2 size-4 -translate-y-1/2"
          />
          <Input
            type="text"
            placeholder="Search trades…"
            bind:value={searchQuery}
            oninput={() => (currentPage = 1)}
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
    <div class="glass-panel flex-1 overflow-y-auto">
      <Table.Root>
        <Table.Header>
          <Table.Row class="border-b border-white/5 hover:bg-transparent">
            <Table.Head class="w-40">Date</Table.Head>
            <Table.Head class="w-24">Type</Table.Head>
            <Table.Head class="w-32 text-right">Amount (BTC)</Table.Head>
            <Table.Head class="w-32 text-right">Price/BTC</Table.Head>
            <Table.Head class="w-28 text-right">Fees</Table.Head>
            <Table.Head class="w-32 text-right">Total</Table.Head>
            <Table.Head class="w-32 text-right">Running BEP</Table.Head>
            <Table.Head class="w-32 text-right">Realized P&L</Table.Head>
          </Table.Row>
        </Table.Header>
        <Table.Body>
          {#each paginatedRows as row, i (row.date + row.spent.amount + row.received.amount + i)}
            {@const buy = isBuy(row)}
            {@const dt = formatDate(row.date)}
            <Table.Row class="border-b border-white/5">
              <Table.Cell class="font-mono tabular-nums">
                <span>{dt.date}</span>
                <span class="text-muted-foreground ml-2">{dt.time}</span>
              </Table.Cell>

              <Table.Cell>
                <span
                  class="inline-flex items-center gap-1.5 {buy
                    ? 'text-success'
                    : 'text-destructive'}"
                >
                  {#if buy}
                    <ArrowDownLeft class="size-3.5" />
                  {:else}
                    <ArrowUpRight class="size-3.5" />
                  {/if}
                  {buy ? "Buy" : "Sell"}
                </span>
              </Table.Cell>

              <Table.Cell
                class="text-right font-mono tabular-nums {buy
                  ? 'text-success'
                  : 'text-destructive'}"
              >
                {formatBtc(row)}
              </Table.Cell>

              <Table.Cell class="text-right font-mono tabular-nums">
                {formatFiat(pricePerUnit(row))}
              </Table.Cell>

              <Table.Cell
                class="text-muted-foreground text-right font-mono tabular-nums"
              >
                {formatFiat(row.fee.amount)}
              </Table.Cell>

              <Table.Cell class="text-right font-mono tabular-nums">
                {formatFiat(quoteAmount(row))}
              </Table.Cell>

              <Table.Cell
                class="text-right font-mono tabular-nums text-amber-400"
              >
                {row.bep ? formatFiat(row.bep.amount) : "–"}
              </Table.Cell>

              <Table.Cell class="text-right font-mono tabular-nums">
                {#if row.pnl}
                  <span
                    class={parseFloat(row.pnl.amount) >= 0
                      ? "text-success"
                      : "text-destructive"}
                  >
                    {formatPnl(row.pnl.amount)}
                  </span>
                {:else}
                  <span class="text-muted-foreground">–</span>
                {/if}
              </Table.Cell>
            </Table.Row>
          {:else}
            <Table.Row>
              <Table.Cell
                colspan={8}
                class="text-muted-foreground py-8 text-center"
              >
                No trades match your search.
              </Table.Cell>
            </Table.Row>
          {/each}
        </Table.Body>
      </Table.Root>
    </div>

    <!-- Footer -->
    {#if filteredRows.length > 0}
      <div class="mt-3 flex items-center justify-between">
        <span class="text-muted-foreground text-xs">
          Showing {Math.min(
            filteredRows.length,
            (currentPage - 1) * PAGE_SIZE + 1,
          )}–{Math.min(filteredRows.length, currentPage * PAGE_SIZE)} of {filteredRows.length}
          trades
        </span>

        {#if totalPages > 1}
          <Pagination.Root
            count={filteredRows.length}
            perPage={PAGE_SIZE}
            bind:page={currentPage}
            siblingCount={1}
          >
            {#snippet children({ pages })}
              <Pagination.Content>
                <Pagination.Item>
                  <Pagination.Previous />
                </Pagination.Item>
                {#each pages as page (page.key)}
                  {#if page.type === "ellipsis"}
                    <Pagination.Item>
                      <Pagination.Ellipsis />
                    </Pagination.Item>
                  {:else}
                    <Pagination.Item>
                      <Pagination.Link
                        {page}
                        isActive={currentPage === page.value}
                      >
                        {page.value}
                      </Pagination.Link>
                    </Pagination.Item>
                  {/if}
                {/each}
                <Pagination.Item>
                  <Pagination.Next />
                </Pagination.Item>
              </Pagination.Content>
            {/snippet}
          </Pagination.Root>
        {/if}
      </div>
    {/if}
  {/if}
</div>
