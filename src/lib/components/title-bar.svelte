<script lang="ts">
	import { Bitcoin } from '@lucide/svelte';

	interface Props {
		title?: string;
	}

	let { title = 'Dashboard' }: Props = $props();
</script>

<header
	class="flex h-8 items-center border-b border-sidebar-border bg-sidebar px-3 select-none"
	data-tauri-drag-region
>
	<div class="flex items-center gap-2 pl-[70px]">
		<Bitcoin class="size-4 text-primary" />
		<span class="text-xs font-semibold tracking-wide text-sidebar-foreground">betc</span>
		<span class="text-xs text-muted-foreground">/ {title}</span>
	</div>
</header>
