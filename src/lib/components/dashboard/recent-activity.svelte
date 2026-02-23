<script lang="ts">
	import { ArrowDownLeft, ArrowUpRight, Plus } from '@lucide/svelte';
	import { ScrollArea } from '$lib/components/ui/scroll-area';
	import { mockTrades, type MockTrade } from '$lib/mock';

	function formatDate(dateStr: string): string {
		return new Date(dateStr).toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' });
	}

	function formatUsd(value: number): string {
		return new Intl.NumberFormat('en-US', { style: 'currency', currency: 'USD', maximumFractionDigits: 0 }).format(value);
	}
</script>

{#snippet tradeItem(trade: MockTrade)}
	<div class="flex items-center gap-3 rounded-lg px-3 py-2.5 transition-colors hover:bg-white/5">
		<div class={[
			'flex size-8 items-center justify-center rounded-full',
			trade.type === 'buy' ? 'bg-success/15 text-success' : 'bg-destructive/15 text-destructive',
		]}>
			{#if trade.type === 'buy'}
				<ArrowDownLeft class="size-4" />
			{:else}
				<ArrowUpRight class="size-4" />
			{/if}
		</div>
		<div class="flex-1">
			<p class="text-sm font-medium capitalize">{trade.type} BTC</p>
			<p class="text-xs text-muted-foreground">{formatDate(trade.date)}</p>
		</div>
		<div class="text-right">
			<p class="text-sm font-medium">{trade.amount_btc} BTC</p>
			<p class="text-xs text-muted-foreground">{formatUsd(trade.total_usd)}</p>
		</div>
	</div>
{/snippet}

<div class="glass-panel flex flex-col p-5">
	<div class="mb-3 flex items-center justify-between">
		<h3 class="text-sm font-semibold">Recent Activity</h3>
		<button class="flex size-7 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-white/10 hover:text-foreground">
			<Plus class="size-4" />
		</button>
	</div>

	<ScrollArea class="flex-1">
		<div class="flex flex-col gap-1">
			{#each mockTrades as trade (trade.date)}
				{@render tradeItem(trade)}
			{/each}
		</div>
	</ScrollArea>
</div>
