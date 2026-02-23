<script lang="ts">
	import * as ToggleGroup from '$lib/components/ui/toggle-group';

	type Range = '1D' | '1W' | '1M' | '3M' | '1Y' | 'ALL';

	let range: Range = $state('1M');

	const ranges: Range[] = ['1D', '1W', '1M', '3M', '1Y', 'ALL'];

	const chartPoints = [
		{ x: 0, price: 35000, bep: 35000 },
		{ x: 60, price: 32800, bep: 35000 },
		{ x: 120, price: 38500, bep: 36750 },
		{ x: 180, price: 42000, bep: 38500 },
		{ x: 240, price: 56000, bep: 42500 },
		{ x: 300, price: 64230, bep: 42500 },
	];

	let pricePath = $derived(
		'M ' + chartPoints.map((p) => `${p.x},${200 - ((p.price - 25000) / 45000) * 180}`).join(' L ')
	);
	let bepPath = $derived(
		'M ' + chartPoints.map((p) => `${p.x},${200 - ((p.bep - 25000) / 45000) * 180}`).join(' L ')
	);

	const trades: { x: number; type: 'buy' | 'sell' }[] = [
		{ x: 0, type: 'buy' },
		{ x: 60, type: 'sell' },
		{ x: 120, type: 'buy' },
		{ x: 180, type: 'buy' },
	];
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
		<svg viewBox="0 0 300 200" class="h-full w-full" preserveAspectRatio="none">
			<defs>
				<linearGradient id="priceGrad" x1="0" y1="0" x2="0" y2="1">
					<stop offset="0%" stop-color="var(--success)" stop-opacity="0.3" />
					<stop offset="100%" stop-color="var(--success)" stop-opacity="0" />
				</linearGradient>
			</defs>

			<!-- Price area fill -->
			<path
				d="{pricePath} L 300,200 L 0,200 Z"
				fill="url(#priceGrad)"
			/>

			<!-- BEP step line -->
			<path
				d={bepPath}
				fill="none"
				stroke="var(--primary)"
				stroke-width="2"
				stroke-dasharray="4 4"
				vector-effect="non-scaling-stroke"
			/>

			<!-- Price line -->
			<path
				d={pricePath}
				fill="none"
				stroke="var(--success)"
				stroke-width="2"
				vector-effect="non-scaling-stroke"
			/>

			<!-- Trade dots -->
			{#each trades as trade (trade.x)}
				{@const point = chartPoints.find((p) => p.x === trade.x)}
				{#if point}
					<circle
						cx={trade.x}
						cy={200 - ((point.price - 25000) / 45000) * 180}
						r="4"
						fill={trade.type === 'buy' ? 'var(--success)' : 'var(--destructive)'}
						vector-effect="non-scaling-stroke"
					/>
				{/if}
			{/each}
		</svg>
	</div>
</div>
