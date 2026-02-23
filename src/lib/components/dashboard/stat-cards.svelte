<script lang="ts">
	import { TrendingUp, TrendingDown } from '@lucide/svelte';
	import * as Card from '$lib/components/ui/card';
	import { cn } from '$lib/utils';
	import { mockPositionSummary, mockBtcPrice } from '$lib/mock';

	let pnl = $derived(mockBtcPrice.price * mockPositionSummary.total_held_btc - mockPositionSummary.total_invested_usd);
	let pnlPct = $derived((pnl / mockPositionSummary.total_invested_usd) * 100);
	let fiatValue = $derived(mockBtcPrice.price * mockPositionSummary.total_held_btc);
	let tradeCount = $derived(mockPositionSummary.buy_count + mockPositionSummary.sell_count);

	function formatUsd(value: number): string {
		return new Intl.NumberFormat('en-US', { style: 'currency', currency: 'USD', maximumFractionDigits: 0 }).format(value);
	}

	function formatUsdFull(value: number): string {
		return new Intl.NumberFormat('en-US', { style: 'currency', currency: 'USD', minimumFractionDigits: 2, maximumFractionDigits: 2 }).format(value);
	}

	function formatBtc(value: number): string {
		return `${parseFloat(value.toFixed(4))} BTC`;
	}

	const cardRoot = 'gap-1 py-4 shadow-none';
	const cardHeader = 'px-4 gap-1';
	const cardContent = 'px-4';
	const cardLabel = 'text-xs font-medium uppercase tracking-wider';
	const cardValue = 'text-xl font-mono';
	const cardSub = 'text-xs text-muted-foreground tabular-nums';
</script>

<div class="grid grid-cols-4 gap-4">
	<!-- BTC Price -->
	<Card.Root class={[cardRoot, 'glass-panel']}>
		<Card.Header class={cardHeader}>
			<Card.Description class={cardLabel}>BTC Price</Card.Description>
			<Card.Title class={cardValue}>{formatUsd(mockBtcPrice.price)}</Card.Title>
		</Card.Header>
		<Card.Content class={cardContent}>
			<span class={cn(cardSub, 'inline-flex items-center gap-1', mockBtcPrice.change_24h >= 0 ? 'text-success' : 'text-destructive')}>
				{#if mockBtcPrice.change_24h >= 0}
					<TrendingUp class="size-3.5" />
				{:else}
					<TrendingDown class="size-3.5" />
				{/if}
				{mockBtcPrice.change_24h > 0 ? '+' : ''}{mockBtcPrice.change_24h.toFixed(1)}% (24h)
			</span>
		</Card.Content>
	</Card.Root>

	<!-- Break-Even Price -->
	<Card.Root class={[cardRoot, 'glow-top-right border-primary/20 shadow-[0_0_15px] shadow-primary/5']}>
		<Card.Header class={cardHeader}>
			<Card.Description class={[cardLabel, 'text-primary']}>Break-Even Price</Card.Description>
			<Card.Title class={cardValue}>{formatUsd(mockPositionSummary.break_even_price)}</Card.Title>
		</Card.Header>
		<Card.Content class={cardContent}>
			<span class={cardSub}>{tradeCount} trades</span>
		</Card.Content>
	</Card.Root>

	<!-- Unrealized P&L -->
	<Card.Root class={[cardRoot, 'glass-panel']}>
		<Card.Header class={cardHeader}>
			<Card.Description class={cardLabel}>Unrealized P&L</Card.Description>
			<Card.Title class={[cardValue, pnl >= 0 ? 'text-success' : 'text-destructive']}>
				{pnl >= 0 ? '+' : ''}{formatUsd(pnl)}
			</Card.Title>
		</Card.Header>
		<Card.Content class={cardContent}>
			<span class={cardSub}>
				{pnl >= 0 ? '+' : ''}{pnlPct.toFixed(2)}%
			</span>
		</Card.Content>
	</Card.Root>

	<!-- Total Held -->
	<Card.Root class={[cardRoot, 'glass-panel']}>
		<Card.Header class={cardHeader}>
			<Card.Description class={cardLabel}>Total Held</Card.Description>
			<Card.Title class={cardValue}>{formatBtc(mockPositionSummary.total_held_btc)}</Card.Title>
		</Card.Header>
		<Card.Content class={cardContent}>
			<span class={cardSub}>{formatUsdFull(fiatValue)}</span>
		</Card.Content>
	</Card.Root>
</div>
