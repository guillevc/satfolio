<script lang="ts">
  import { ArrowDownLeft, ArrowUpRight } from "@lucide/svelte";

  let { value }: { value: string } = $props();
  const buy = $derived(value === "Buy");
</script>

<span
  class="inline-flex items-center gap-2 {buy
    ? 'text-success'
    : 'text-foreground'}"
>
  <span
    class="inline-flex items-center justify-center rounded-md size-6 {buy
      ? 'bg-success/15'
      : 'bg-foreground/10'}"
  >
    {#if buy}
      <ArrowDownLeft class="size-3.5" />
    {:else}
      <ArrowUpRight class="size-3.5" />
    {/if}
  </span>
  {value}
</span>
