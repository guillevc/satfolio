<script lang="ts">
	interface Props {
		margin: number;
	}

	let { margin }: Props = $props();

	let label = $derived(margin > 20 ? 'Safe' : margin > 0 ? 'Tight' : 'At Risk');
	let color = $derived(margin > 20 ? 'var(--success)' : margin > 0 ? 'var(--primary)' : 'var(--destructive)');

	function computeArc(pct: number): string {
		if (pct <= 0) return '';
		const startAngle = 150;
		const totalSweep = 240;
		const endAngle = startAngle + (pct / 100) * totalSweep;
		const r = 50;
		const cx = 60;
		const cy = 60;

		const toRad = (deg: number) => (deg * Math.PI) / 180;
		const x1 = cx + r * Math.cos(toRad(startAngle));
		const y1 = cy + r * Math.sin(toRad(startAngle));
		const x2 = cx + r * Math.cos(toRad(endAngle));
		const y2 = cy + r * Math.sin(toRad(endAngle));
		const largeArc = endAngle - startAngle > 180 ? 1 : 0;

		return `M ${x1} ${y1} A ${r} ${r} 0 ${largeArc} 1 ${x2} ${y2}`;
	}

	let arcPath = $derived(computeArc(margin));
	let trackPath = $derived(computeArc(100));
</script>

<div class="glass-panel flex flex-col items-center gap-3 p-5">
	<h3 class="self-start text-sm font-semibold">Safety Margin</h3>

	<svg viewBox="0 0 120 90" class="w-40">
		<!-- Track -->
		<path
			d={trackPath}
			fill="none"
			stroke="currentColor"
			stroke-opacity="0.1"
			stroke-width="8"
			stroke-linecap="round"
		/>
		<!-- Active arc -->
		<path
			d={arcPath}
			fill="none"
			stroke={color}
			stroke-width="8"
			stroke-linecap="round"
		/>
		<!-- Center text -->
		<text x="60" y="55" text-anchor="middle" fill="currentColor" font-size="16" font-weight="bold">
			{margin}%
		</text>
		<text x="60" y="70" text-anchor="middle" fill={color} font-size="9" font-weight="600">
			{label}
		</text>
	</svg>
</div>
