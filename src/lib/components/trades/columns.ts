import type { ColumnDef } from "@tanstack/table-core";
import { createRawSnippet } from "svelte";
import {
  renderComponent,
  renderSnippet,
} from "$lib/components/ui/data-table/index.js";
import type { EnrichedTrade } from "$lib/types/bindings";
import { providerMeta } from "$lib/utils/provider";
import {
  formatCurrency,
  formatDecimal,
  formatSignedCurrency,
} from "$lib/utils/format";
import { getQuote } from "$lib/stores/config.svelte";
import {
  FIAT_ASSETS,
  isBuy,
  baseAmount,
  quoteAmount,
  pricePerUnit,
} from "$lib/utils/trade";
import SortButton from "./sort-button.svelte";
import TypeCell from "./type-cell.svelte";

// ── Formatters ──────────────────────────────────────────────

export function formatDate(iso: string): { date: string; time: string } {
  const d = new Date(iso);
  const date = d.toLocaleDateString("en-CA"); // YYYY-MM-DD
  const time = d.toLocaleTimeString("en-GB", {
    hour: "2-digit",
    minute: "2-digit",
  });
  return { date, time };
}

function formatBtcSigned(t: EnrichedTrade): string {
  const n = parseFloat(baseAmount(t));
  const prefix = isBuy(t) ? "+" : "-";
  return `${prefix}${formatDecimal(n, 6)}`;
}

// ── Null-safe sorting ───────────────────────────────────────

function nullableSort(
  rowA: { getValue: <T>(id: string) => T },
  rowB: { getValue: <T>(id: string) => T },
  columnId: string,
): number {
  const a = rowA.getValue<number | null>(columnId);
  const b = rowB.getValue<number | null>(columnId);
  if (a === null && b === null) return 0;
  if (a === null) return 1;
  if (b === null) return -1;
  return a - b;
}

// ── Column definitions ──────────────────────────────────────

export const columns: ColumnDef<EnrichedTrade>[] = [
  // Date
  {
    id: "date",
    accessorFn: (row) => new Date(row.date).getTime(),
    header: ({ column }) =>
      renderComponent(SortButton, {
        label: "Date",
        sorted: column.getIsSorted(),
        onclick: column.getToggleSortingHandler()!,
      }),
    cell: ({ row }) => {
      const { date, time } = formatDate(row.original.date);
      const snippet = createRawSnippet(() => ({
        render: () =>
          `<div class="font-mono tabular-nums"><span>${date}</span><span class="text-muted-foreground ml-2">${time}</span></div>`,
      }));
      return renderSnippet(snippet);
    },
  },

  // Exchange
  {
    id: "exchange",
    accessorFn: (row) => row.provider,
    enableSorting: false,
    header: "Exchange",
    cell: ({ row }) => {
      const p = row.original.provider;
      const meta = providerMeta[p];
      const snippet = createRawSnippet(() => ({
        render: () =>
          `<span class="inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium ${meta.classes}">${meta.label}</span>`,
      }));
      return renderSnippet(snippet);
    },
  },

  // Type
  {
    id: "type",
    accessorFn: (row) => (isBuy(row) ? "Buy" : "Sell"),
    header: "Type",
    enableSorting: false,
    cell: ({ row }) =>
      renderComponent(TypeCell, {
        value: row.getValue("type") as string,
      }),
  },

  // Amount (BTC)
  {
    id: "amount",
    accessorFn: (row) => parseFloat(baseAmount(row)),
    meta: { align: "right" },
    header: ({ column }) =>
      renderComponent(SortButton, {
        label: "BTC",
        sorted: column.getIsSorted(),
        onclick: column.getToggleSortingHandler()!,
      }),
    cell: ({ row }) => {
      const text = formatBtcSigned(row.original);
      const snippet = createRawSnippet(() => ({
        render: () =>
          `<div class="text-right font-mono tabular-nums text-foreground">${text}</div>`,
      }));
      return renderSnippet(snippet);
    },
  },

  // Price/BTC
  {
    id: "price",
    accessorFn: (row) => pricePerUnit(row),
    meta: { align: "right" },
    header: ({ column }) =>
      renderComponent(SortButton, {
        label: "Price/BTC",
        sorted: column.getIsSorted(),
        onclick: column.getToggleSortingHandler()!,
      }),
    cell: ({ row }) => {
      const text = formatCurrency(
        row.getValue("price") as number,
        getQuote(),
        2,
      );
      const snippet = createRawSnippet(() => ({
        render: () =>
          `<div class="text-right font-mono tabular-nums">${text}</div>`,
      }));
      return renderSnippet(snippet);
    },
  },

  // Fees
  {
    id: "fees",
    accessorFn: (row) => parseFloat(row.fee.amount),
    meta: { align: "right" },
    header: ({ column }) =>
      renderComponent(SortButton, {
        label: "Fees",
        sorted: column.getIsSorted(),
        onclick: column.getToggleSortingHandler()!,
      }),
    cell: ({ row }) => {
      const fee = row.original.fee;
      const text = FIAT_ASSETS.has(fee.asset)
        ? formatCurrency(parseFloat(fee.amount), getQuote(), 2)
        : `${formatDecimal(parseFloat(fee.amount), 8)} ${fee.asset}`;
      const snippet = createRawSnippet(() => ({
        render: () =>
          `<div class="text-muted-foreground text-right font-mono tabular-nums">${text}</div>`,
      }));
      return renderSnippet(snippet);
    },
  },

  // Total
  {
    id: "total",
    accessorFn: (row) => parseFloat(quoteAmount(row)),
    meta: { align: "right" },
    header: ({ column }) =>
      renderComponent(SortButton, {
        label: "Total",
        sorted: column.getIsSorted(),
        onclick: column.getToggleSortingHandler()!,
      }),
    cell: ({ row }) => {
      const text = formatCurrency(
        row.getValue("total") as number,
        getQuote(),
        2,
      );
      const snippet = createRawSnippet(() => ({
        render: () =>
          `<div class="text-right font-mono tabular-nums">${text}</div>`,
      }));
      return renderSnippet(snippet);
    },
  },

  // Running BEP
  {
    id: "bep",
    accessorFn: (row) => (row.bep ? parseFloat(row.bep.amount) : null),
    meta: { align: "right" },
    header: ({ column }) =>
      renderComponent(SortButton, {
        label: "Running BEP",
        sorted: column.getIsSorted(),
        onclick: column.getToggleSortingHandler()!,
      }),
    sortingFn: nullableSort,
    cell: ({ row }) => {
      const val = row.getValue("bep") as number | null;
      const text = val !== null ? formatCurrency(val, getQuote(), 2) : "\u2013";
      const cls =
        val !== null
          ? "text-right font-mono tabular-nums text-amber-400"
          : "text-right font-mono tabular-nums text-muted-foreground";
      const snippet = createRawSnippet(() => ({
        render: () => `<div class="${cls}">${text}</div>`,
      }));
      return renderSnippet(snippet);
    },
  },

  // P&L
  {
    id: "pnl",
    accessorFn: (row) => (row.pnl ? parseFloat(row.pnl.amount) : null),
    meta: { align: "right" },
    header: ({ column }) =>
      renderComponent(SortButton, {
        label: "P&L",
        sorted: column.getIsSorted(),
        onclick: column.getToggleSortingHandler()!,
      }),
    sortingFn: nullableSort,
    cell: ({ row }) => {
      const val = row.getValue("pnl") as number | null;
      if (val === null) {
        const snippet = createRawSnippet(() => ({
          render: () =>
            `<div class="text-right font-mono tabular-nums text-muted-foreground">\u2013</div>`,
        }));
        return renderSnippet(snippet);
      }
      const text = formatSignedCurrency(
        parseFloat(row.original.pnl!.amount),
        getQuote(),
        2,
      );
      const color = val >= 0 ? "text-success" : "text-destructive";
      const snippet = createRawSnippet(() => ({
        render: () =>
          `<div class="text-right font-mono tabular-nums ${color}">${text}</div>`,
      }));
      return renderSnippet(snippet);
    },
  },
];
