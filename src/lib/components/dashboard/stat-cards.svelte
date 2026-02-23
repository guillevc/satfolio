<script lang="ts">
	import { TrendingUp, TrendingDown } from '@lucide/svelte';
	import * as Card from '$lib/components/ui/card';
	import { mockPositionSummary, mockBtcPrice } from '$lib/mock';

	let pnl = $derived(mockBtcPrice.price * mockPositionSummary.total_held_btc - mockPositionSummary.total_invested_usd);
	let pnlPct = $derived((pnl / mockPositionSummary.total_invested_usd) * 100);
	let fiatValue = $derived(mockBtcPrice.price * mockPositionSummary.total_held_btc);

	function formatUsd(value: number): string {
		return new Intl.NumberFormat('en-US', { style: 'currency', currency: 'USD', maximumFractionDigits: 0 }).format(value);
	}

	function formatBtc(value: number): string {
		return `${value.toFixed(4)} BTC`;
	}
</script>

<div class="grid grid-cols-4 gap-4">
	<!-- BTC Price -->
	<Card.Root class="glass-panel border-white/[0.08]">
		<Card.Header>
			<Card.Description>BTC Price</Card.Description>
			<Card.Title class="text-2xl font-bold">{formatUsd(mockBtcPrice.price)}</Card.Title>
		</Card.Header>
		<Card.Content>
			<span class={['inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-xs font-medium',
				mockBtcPrice.change_24h >= 0 ? 'bg-success/15 text-success' : 'bg-destructive/15 text-destructive'
			]}>
				{#if mockBtcPrice.change_24h >= 0}
					<TrendingUp class="size-3" />
				{:else}
					<TrendingDown class="size-3" />
				{/if}
				{mockBtcPrice.change_24h > 0 ? '+' : ''}{mockBtcPrice.change_24h.toFixed(1)}%
			</span>
		</Card.Content>
	</Card.Root>

	<!-- Break-Even Price -->
	<Card.Root class="relative overflow-hidden border-primary/20 shadow-[0_0_15px] shadow-primary/5">
		<div class="pointer-events-none absolute right-0 top-0 h-24 w-24 translate-x-1/2 -translate-y-1/2 rounded-full bg-primary/10 blur-2xl"></div>
		<Card.Header class="relative z-10">
			<Card.Description class="text-primary">Break-Even Price</Card.Description>
			<Card.Title class="text-2xl font-bold">{formatUsd(mockPositionSummary.break_even_price)}</Card.Title>
		</Card.Header>
		<Card.Content class="relative z-10">
			<span class="text-xs text-muted-foreground">
				{mockPositionSummary.buy_count} buys, {mockPositionSummary.sell_count} sell
			</span>
		</Card.Content>
	</Card.Root>

	<!-- Unrealized P&L -->
	<Card.Root class="glass-panel border-white/[0.08]">
		<Card.Header>
			<Card.Description>Unrealized P&L</Card.Description>
			<Card.Title class={['text-2xl font-bold', pnl >= 0 ? 'text-success' : 'text-destructive']}>
				{pnl >= 0 ? '+' : ''}{formatUsd(pnl)}
			</Card.Title>
		</Card.Header>
		<Card.Content>
			<span class={['text-xs font-medium', pnl >= 0 ? 'text-success' : 'text-destructive']}>
				{pnl >= 0 ? '+' : ''}{pnlPct.toFixed(1)}%
			</span>
		</Card.Content>
	</Card.Root>

	<!-- Total Held -->
	<Card.Root class="glass-panel border-white/[0.08]">
		<Card.Header>
			<Card.Description>Total Held</Card.Description>
			<Card.Title class="text-2xl font-bold">{formatBtc(mockPositionSummary.total_held_btc)}</Card.Title>
		</Card.Header>
		<Card.Content>
			<span class="text-xs text-muted-foreground">
				≈ {formatUsd(fiatValue)}
			</span>
		</Card.Content>
	</Card.Root>
</div>
