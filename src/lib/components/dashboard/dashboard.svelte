<script lang="ts">
	import StatCards from './stat-cards.svelte';
	import BepChart from './bep-chart.svelte';
	import SafetyGauge from './safety-gauge.svelte';
	import RecentActivity from './recent-activity.svelte';
	import { mockPositionSummary, mockBtcPrice } from '$lib/mock';

	let safetyMargin = $derived(
		Math.round(((mockBtcPrice.price - mockPositionSummary.break_even_price) / mockPositionSummary.break_even_price) * 100)
	);
</script>

<div class="flex-1 overflow-y-auto p-6">
	<div class="flex flex-col gap-6">
		<StatCards />

		<div class="grid grid-cols-[3fr_1fr] gap-6">
		<BepChart />

		<div class="flex flex-col gap-6">
			<SafetyGauge margin={safetyMargin} />
			<RecentActivity />
		</div>
		</div>
	</div>
</div>
