<script lang="ts">
	import { ChartLine, ScrollText, FlaskConical, Settings } from '@lucide/svelte';
	import * as Tooltip from '$lib/components/ui/tooltip';
	import { Separator } from '$lib/components/ui/separator';

	type View = 'dashboard' | 'ledger' | 'simulator' | 'settings';

	interface Props {
		active: View;
		onnavigate: (view: View) => void;
	}

	let { active, onnavigate }: Props = $props();

	const navItems: { view: View; icon: typeof ChartLine; label: string }[] = [
		{ view: 'dashboard', icon: ChartLine, label: 'Dashboard' },
		{ view: 'ledger', icon: ScrollText, label: 'Ledger' },
		{ view: 'simulator', icon: FlaskConical, label: 'Simulator' },
	];
</script>

{#snippet navItem(view: View, Icon: typeof ChartLine, label: string)}
	<Tooltip.Root>
		<Tooltip.Trigger>
			<button
				class={[
					'flex size-10 items-center justify-center rounded-lg text-muted-foreground transition-all',
					active === view && 'bg-primary/20 text-primary shadow-[0_0_12px_-3px] shadow-primary/40',
					active !== view && 'hover:bg-white/5 hover:text-foreground',
				]}
				onclick={() => onnavigate(view)}
			>
				<Icon class="size-5" />
			</button>
		</Tooltip.Trigger>
		<Tooltip.Content side="right">
			<p>{label}</p>
		</Tooltip.Content>
	</Tooltip.Root>
{/snippet}

<Tooltip.Provider delayDuration={0}>
	<nav class="flex w-16 flex-col items-center gap-1 border-r border-sidebar-border bg-sidebar py-3">
		<div class="flex flex-col items-center gap-1">
			{#each navItems as { view, icon, label } (view)}
				{@render navItem(view, icon, label)}
			{/each}
		</div>

		<Separator class="mx-3 my-2 w-8" />

		{@render navItem('settings', Settings, 'Settings')}
	</nav>
</Tooltip.Provider>
