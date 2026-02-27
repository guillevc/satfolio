<script lang="ts">
  import { onMount } from "svelte";
  import {
    createChart,
    LineSeries,
    HistogramSeries,
    ColorType,
  } from "lightweight-charts";
  import type { IChartApi, Time } from "lightweight-charts";
  import { RefreshCw } from "@lucide/svelte";
  import { Button } from "$lib/components/ui/button";
  import { Switch } from "$lib/components/ui/switch";
  import * as ToggleGroup from "$lib/components/ui/toggle-group";
  import type { BepSnapshot, Candle } from "$lib/types/bindings";
  import { Label } from "$lib/components/ui/label";

  let {
    bepSnaps,
    candles,
    syncing,
    onrefresh,
  }: {
    bepSnaps: Record<string, BepSnapshot>;
    candles: Candle[];
    syncing: boolean;
    onrefresh: () => void;
  } = $props();

  let showTrades = $state(false);

  type Range = "1M" | "3M" | "1Y" | "3Y" | "5Y" | "ALL";
  let range: Range = $state("1Y");
  const ranges: Range[] = ["1M", "3M", "1Y", "3Y", "5Y", "ALL"];

  let container: HTMLDivElement;
  let chart: IChartApi | undefined;
  let priceSeries: ReturnType<IChartApi["addSeries"]> | undefined;
  let bepSeries: ReturnType<IChartApi["addSeries"]> | undefined;
  let bandsSeries: ReturnType<IChartApi["addSeries"]> | undefined;

  // Crosshair state — null means "show latest values"
  let crosshairData = $state<{
    date: string;
    price: number | null;
    bep: number | null;
  } | null>(null);

  // --- Data transformations ---
  // parseFloat() precision: intentional — canvas pixels don't need 18 decimal places (display-only)
  let marketPrices = $derived(
    candles.map((c) => ({ time: c.date as Time, value: parseFloat(c.close) })),
  );

  let snapDates = $derived(Object.keys(bepSnaps).sort());

  let bepPrices = $derived.by(() => {
    if (snapDates.length === 0) return [];

    let lastBep: number | null = null;
    let si = 0;
    const points: { time: Time; value: number }[] = [];

    for (const c of candles) {
      while (si < snapDates.length && snapDates[si] <= c.date) {
        const snap = bepSnaps[snapDates[si]];
        if (snap.bep) lastBep = parseFloat(snap.bep);
        si++;
      }
      if (lastBep !== null) {
        points.push({ time: c.date as Time, value: lastBep });
      }
    }
    return points;
  });

  // Trade bands: full-height vertical lines at each trade date
  const buyColor = "rgba(251,191,36,0.15)";
  const sellColor = "rgba(148,163,184,0.15)";

  let tradeBands = $derived.by(() => {
    const bands: Array<{ time: Time; value: number; color: string }> = [];

    for (let i = 0; i < snapDates.length; i++) {
      const curr = bepSnaps[snapDates[i]];
      if (!curr.bep) continue;

      bands.push({
        time: snapDates[i] as Time,
        value: 999_999_999,
        color: curr.side === "Buy" ? buyColor : sellColor,
      });
    }

    return bands;
  });

  // Legend: crosshair values or latest from series
  let legend = $derived.by(() => {
    if (crosshairData) return crosshairData;
    const mp = marketPrices;
    const bp = bepPrices;
    if (mp.length === 0)
      return {
        date: "",
        price: null as number | null,
        bep: null as number | null,
      };
    return {
      date: mp[mp.length - 1].time as string,
      price: mp[mp.length - 1].value,
      bep: bp.length > 0 ? bp[bp.length - 1].value : null,
    };
  });

  let legendSpread = $derived(
    legend.price !== null && legend.bep !== null
      ? legend.price - legend.bep
      : null,
  );

  // --- Range → logical range ---
  const rangeDays: Record<Range, number | null> = {
    "1M": 30,
    "3M": 90,
    "1Y": 365,
    "3Y": 1095,
    "5Y": 1825,
    ALL: null,
  };

  // Data sync effects (declared before range effect so data is set first).
  // Read derived values before optional chain — otherwise ?. short-circuits
  // and Svelte never tracks the dependency (effect becomes dead).
  $effect(() => {
    const data = marketPrices;
    priceSeries?.setData(data);
  });

  $effect(() => {
    const data = bepPrices;
    bepSeries?.setData(data);
  });

  function applyShowTrades(show: boolean) {
    bandsSeries?.setData(show ? tradeBands : []);
  }

  function applyRange(r: Range) {
    if (!chart) return;
    const total = marketPrices.length;
    if (total === 0) return;
    const days = rangeDays[r];
    chart.timeScale().setVisibleLogicalRange({
      from: days === null ? -0.5 : Math.max(total - days, 0) - 0.5,
      to: total - 0.5,
    });
  }

  // --- Chart lifecycle ---
  onMount(() => {
    chart = createChart(container, {
      layout: {
        background: { type: ColorType.Solid, color: "transparent" },
        textColor: "#a1a1aa",
        fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif',
      },
      grid: {
        vertLines: { visible: false },
        horzLines: { color: "#27272a" },
      },
      crosshair: {
        vertLine: { color: "#71717a", style: 3 },
        horzLine: { color: "#71717a", style: 3 },
      },
      rightPriceScale: {
        borderVisible: false,
        scaleMargins: { top: 0.05, bottom: 0 },
      },
      timeScale: {
        borderVisible: false,
        minBarSpacing: 0.1,
        fixRightEdge: true,
        shiftVisibleRangeOnNewBar: true,
      },
    });

    // Trade bands — full-height histogram on hidden scale, renders behind lines
    bandsSeries = chart.addSeries(HistogramSeries, {
      priceScaleId: "bands",
      lastValueVisible: false,
      priceLineVisible: false,
    });
    chart.priceScale("bands").applyOptions({
      visible: false,
      scaleMargins: { top: 0, bottom: 0 },
    });

    // BTC price — neutral zinc reference line
    priceSeries = chart.addSeries(LineSeries, {
      color: "#d4d4d8",
      lineWidth: 2,
      lastValueVisible: true,
      priceLineVisible: false,
      crosshairMarkerVisible: true,
      crosshairMarkerRadius: 4,
    });

    // BEP — the hero line, amber dashed
    bepSeries = chart.addSeries(LineSeries, {
      color: "#fbbf24",
      lineWidth: 2,
      lineStyle: 2,
      lastValueVisible: true,
      priceLineVisible: false,
      crosshairMarkerVisible: true,
      crosshairMarkerRadius: 4,
    });

    // Initial data load
    priceSeries.setData(marketPrices);
    bepSeries.setData(bepPrices);

    // Crosshair → legend
    chart.subscribeCrosshairMove((param) => {
      if (!param.point || !param.time) {
        crosshairData = null;
        return;
      }
      const pd = param.seriesData.get(priceSeries!) as
        | { value?: number }
        | undefined;
      const bd = param.seriesData.get(bepSeries!) as
        | { value?: number }
        | undefined;
      crosshairData = {
        date: param.time as string,
        price: pd?.value ?? null,
        bep: bd?.value ?? null,
      };
    });

    applyRange(range);

    // Auto-resize canvas to container
    const observer = new ResizeObserver((entries) => {
      for (const entry of entries) {
        const { width, height } = entry.contentRect;
        chart?.resize(width, height);
      }
    });
    observer.observe(container);

    return () => {
      observer.disconnect();
      chart?.remove();
      chart = undefined;
    };
  });

  function formatPrice(v: number | null): string {
    if (v === null) return "—";
    return v.toLocaleString("en-US", {
      style: "currency",
      currency: "USD",
      minimumFractionDigits: 0,
      maximumFractionDigits: 0,
    });
  }

  function formatSpread(v: number | null): string {
    if (v === null) return "—";
    const sign = v >= 0 ? "+" : "";
    return (
      sign +
      v.toLocaleString("en-US", {
        style: "currency",
        currency: "USD",
        minimumFractionDigits: 0,
        maximumFractionDigits: 0,
      })
    );
  }
</script>

<div class="glass-panel flex min-h-0 flex-1 flex-col gap-4 p-5">
  <div class="flex items-center justify-between">
    <div class="flex items-center gap-2">
      <h3 class="text-sm font-semibold">Performance vs. BEP</h3>
      <Button
        variant="ghost"
        size="icon"
        class="size-7"
        disabled={syncing}
        onclick={onrefresh}
      >
        <RefreshCw class="size-3.5 {syncing ? 'animate-spin' : ''}" />
      </Button>
    </div>
    <div class="flex items-center gap-6">
      <div class="flex items-center gap-2 text-xs">
        <Switch
          id="show-buys-sells-switch"
          checked={showTrades}
          onCheckedChange={(v) => {
            showTrades = v;
            applyShowTrades(v);
          }}
        />
        <Label class="text-xs" for="show-buys-sells-switch"
          >Show buys/sells</Label
        >
      </div>
      <ToggleGroup.Root
        type="single"
        variant="outline"
        size="sm"
        value={range}
        onValueChange={(v) => {
          if (v) {
            range = v as Range;
            applyRange(v as Range);
          }
        }}
      >
        {#each ranges as r (r)}
          <ToggleGroup.Item value={r} class="text-xs font-mono">
            {r}
          </ToggleGroup.Item>
        {/each}
      </ToggleGroup.Root>
    </div>
  </div>

  <div class="relative min-h-0 flex-1">
    {#if candles.length > 0}
      <div
        class="pointer-events-none absolute left-3 top-2 z-10 flex items-center gap-2 text-xs font-mono tracking-wide"
      >
        <span class="flex items-center gap-1">
          <span class="size-2 rounded-full bg-zinc-300"></span>
          <span class="text-muted-foreground font-medium">BTC</span>
          <span class="text-zinc-200 min-w-[9ch]"
            >{formatPrice(legend.price)}</span
          >
        </span>
        {#if legend.bep !== null}
          <span class="flex items-center gap-1">
            <span class="size-2 rounded-full bg-amber-400"></span>
            <span class="text-muted-foreground font-medium">BEP</span>
            <span class="text-amber-400 min-w-[9ch]"
              >{formatPrice(legend.bep)}</span
            >
          </span>
        {/if}
        {#if legendSpread !== null}
          <span class="flex items-center gap-1">
            <span class="text-muted-foreground font-medium">P&L</span>
            <span
              class={legendSpread >= 0 ? "text-emerald-400" : "text-red-400"}
            >
              {formatSpread(legendSpread)}
            </span>
          </span>
        {/if}
      </div>
    {/if}

    <div bind:this={container} class="h-full w-full"></div>

    {#if candles.length === 0}
      <div class="absolute inset-0 flex items-center justify-center">
        <span class="text-muted-foreground text-sm"
          >No price data available</span
        >
      </div>
    {/if}
  </div>
</div>
