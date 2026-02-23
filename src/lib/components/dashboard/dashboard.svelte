<script lang="ts">
	import { onMount } from 'svelte';
	import { dashboard, loadDashboard } from '$lib/stores/dashboard.svelte';
	import StatCards from './stat-cards.svelte';
	import BepChart from './bep-chart.svelte';

	onMount(loadDashboard);
</script>

<div class="flex flex-1 flex-col gap-6 overflow-y-auto p-6">
	{#if dashboard.loading}
		<div class="flex flex-1 items-center justify-center">
			<span class="text-muted-foreground text-sm">Loading…</span>
		</div>
	{:else if dashboard.error}
		<div class="flex flex-1 items-center justify-center">
			<span class="text-destructive text-sm">{dashboard.error}</span>
		</div>
	{:else if dashboard.summary && dashboard.candles && dashboard.bepSnaps}
		<StatCards summary={dashboard.summary} candles={dashboard.candles} />
		<BepChart bepSnaps={dashboard.bepSnaps} candles={dashboard.candles} />
	{/if}
</div>
