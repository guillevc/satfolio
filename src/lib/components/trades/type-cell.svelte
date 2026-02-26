<script lang="ts">
  import { ArrowDownLeft, ArrowUpRight } from "@lucide/svelte";

  let { value }: { value: string } = $props();
  const buy = $derived(value === "Buy");
</script>

<span
  class="inline-flex items-center gap-1.5 {buy
    ? 'text-success'
    : 'text-destructive'}"
>
  {#if buy}
    <ArrowDownLeft class="size-3.5" />
  {:else}
    <ArrowUpRight class="size-3.5" />
  {/if}
  {value}
</span>
