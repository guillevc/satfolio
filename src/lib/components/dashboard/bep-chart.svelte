<script lang="ts">
	import * as ToggleGroup from '$lib/components/ui/toggle-group';
	import type { BepSnapshot, Candle } from '$lib/types/bindings';

	let {
		bepSnaps,
		candles,
	}: {
		bepSnaps: Record<string, BepSnapshot>;
		candles: Candle[];
	} = $props();

	type Range = '1W' | '1M' | '3M' | '1Y' | 'ALL';

	let range: Range = $state('ALL');

	const ranges: Range[] = ['1W', '1M', '3M', '1Y', 'ALL'];

	const VIEWBOX_W = 300;
	const VIEWBOX_H = 200;
	const PAD = 10;

	let filteredCandles = $derived.by(() => {
		if (range === 'ALL') return candles;
		const now = new Date();
		const cutoff = new Date(now);
		if (range === '1W') cutoff.setDate(now.getDate() - 7);
		else if (range === '1M') cutoff.setMonth(now.getMonth() - 1);
		else if (range === '3M') cutoff.setMonth(now.getMonth() - 3);
		else if (range === '1Y') cutoff.setFullYear(now.getFullYear() - 1);
		const cutoffStr = cutoff.toISOString().slice(0, 10);
		return candles.filter((c) => c.date >= cutoffStr);
	});

	/** Merge candle close prices with BEP snapshots, aligned by date */
	let chartData = $derived.by(() => {
		const fc = filteredCandles;
		if (fc.length === 0) return [];

		// Build a sorted list of BEP values — carry forward last known BEP
		const snapDates = Object.keys(bepSnaps).sort();
		let lastBep: number | null = null;

		return fc.map((c) => {
			const date = c.date;
			// Find the most recent BEP snapshot at or before this date
			for (let i = snapDates.length - 1; i >= 0; i--) {
				if (snapDates[i] <= date) {
					const snap = bepSnaps[snapDates[i]];
					lastBep = snap.bep ? parseFloat(snap.bep) : null;
					break;
				}
			}
			return {
				date,
				price: parseFloat(c.close),
				bep: lastBep,
			};
		});
	});

	/** Dates where BEP changed (i.e. a trade happened) */
	let tradeDots = $derived.by(() => {
		const data = chartData;
		const dots: { idx: number; type: 'buy' | 'sell' }[] = [];
		for (let i = 1; i < data.length; i++) {
			if (data[i].bep !== data[i - 1].bep && data[i].bep !== null) {
				// BEP went up → likely a buy at higher price; went down → sell
				const type = (data[i].bep ?? 0) > (data[i - 1].bep ?? 0) ? 'buy' : 'sell';
				dots.push({ idx: i, type });
			}
		}
		// Also mark the first point if BEP exists (first trade)
		if (data.length > 0 && data[0].bep !== null) {
			dots.unshift({ idx: 0, type: 'buy' });
		}
		return dots;
	});

	/** Scale helpers */
	let scaleInfo = $derived.by(() => {
		const data = chartData;
		if (data.length === 0) return { minY: 0, maxY: 1, scaleX: (_i: number) => 0, scaleY: (_v: number) => 0 };

		const allValues = data.flatMap((d) => (d.bep !== null ? [d.price, d.bep] : [d.price]));
		const minY = Math.min(...allValues) * 0.95;
		const maxY = Math.max(...allValues) * 1.05;
		const rangeY = maxY - minY || 1;

		const scaleX = (i: number) => (data.length > 1 ? (i / (data.length - 1)) * VIEWBOX_W : VIEWBOX_W / 2);
		const scaleY = (v: number) => VIEWBOX_H - PAD - ((v - minY) / rangeY) * (VIEWBOX_H - PAD * 2);

		return { minY, maxY, scaleX, scaleY };
	});

	let pricePath = $derived.by(() => {
		const data = chartData;
		const { scaleX, scaleY } = scaleInfo;
		if (data.length === 0) return '';
		return 'M ' + data.map((d, i) => `${scaleX(i)},${scaleY(d.price)}`).join(' L ');
	});

	let bepPath = $derived.by(() => {
		const data = chartData;
		const { scaleX, scaleY } = scaleInfo;
		if (data.length === 0) return '';
		// Only draw segments where BEP is known
		const segments: string[] = [];
		let inSegment = false;
		for (let i = 0; i < data.length; i++) {
			if (data[i].bep !== null) {
				segments.push(`${inSegment ? 'L' : 'M'} ${scaleX(i)},${scaleY(data[i].bep!)}`);
				inSegment = true;
			} else {
				inSegment = false;
			}
		}
		return segments.join(' ');
	});
</script>

{#snippet legendPill(color: string, label: string)}
	<div class="flex items-center gap-1.5">
		<span class="size-2 rounded-full" style="background-color: {color}"></span>
		<span class="text-xs text-muted-foreground">{label}</span>
	</div>
{/snippet}

<div class="glass-panel flex min-h-0 flex-1 flex-col gap-4 p-5">
	<div class="flex items-center justify-between">
		<div class="flex items-center gap-4">
			<h3 class="text-sm font-semibold">Performance vs BEP</h3>
			<div class="flex items-center gap-3">
				{@render legendPill('var(--primary)', 'BEP')}
				{@render legendPill('var(--success)', 'BTC Price')}
			</div>
		</div>
		<ToggleGroup.Root type="single" value={range} onValueChange={(v) => { if (v) range = v as Range; }}>
			{#each ranges as r (r)}
				<ToggleGroup.Item value={r} class="h-7 px-2 text-xs">
					{r}
				</ToggleGroup.Item>
			{/each}
		</ToggleGroup.Root>
	</div>

	<div class="relative min-h-0 flex-1">
		{#if chartData.length > 0}
			<svg viewBox="0 0 {VIEWBOX_W} {VIEWBOX_H}" class="h-full w-full" preserveAspectRatio="none">
				<defs>
					<linearGradient id="priceGrad" x1="0" y1="0" x2="0" y2="1">
						<stop offset="0%" stop-color="var(--success)" stop-opacity="0.3" />
						<stop offset="100%" stop-color="var(--success)" stop-opacity="0" />
					</linearGradient>
				</defs>

				<!-- Price area fill -->
				<path
					d="{pricePath} L {VIEWBOX_W},{VIEWBOX_H} L 0,{VIEWBOX_H} Z"
					fill="url(#priceGrad)"
				/>

				<!-- BEP step line -->
				{#if bepPath}
					<path
						d={bepPath}
						fill="none"
						stroke="var(--primary)"
						stroke-width="2"
						stroke-dasharray="4 4"
						vector-effect="non-scaling-stroke"
					/>
				{/if}

				<!-- Price line -->
				<path
					d={pricePath}
					fill="none"
					stroke="var(--success)"
					stroke-width="2"
					vector-effect="non-scaling-stroke"
				/>

				<!-- Trade dots -->
				{#each tradeDots as dot (dot.idx)}
					{@const d = chartData[dot.idx]}
					<circle
						cx={scaleInfo.scaleX(dot.idx)}
						cy={scaleInfo.scaleY(d.price)}
						r="4"
						fill={dot.type === 'buy' ? 'var(--success)' : 'var(--destructive)'}
						vector-effect="non-scaling-stroke"
					/>
				{/each}
			</svg>
		{:else}
			<div class="flex h-full items-center justify-center">
				<span class="text-muted-foreground text-sm">No price data available</span>
			</div>
		{/if}
	</div>
</div>
