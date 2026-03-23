<script lang="ts">
  import { ArrowDownLeft, ArrowUpRight, GiftIcon } from "@lucide/svelte";

  let { value }: { value: string } = $props();
  const buy = $derived(value === "Buy");
  const reward = $derived(value === "Reward");
</script>

<span
  class="inline-flex items-center gap-2 {reward
    ? 'text-amber-400'
    : buy
      ? 'text-success'
      : 'text-foreground'}"
>
  <span
    class="inline-flex size-6 items-center justify-center rounded-md {reward
      ? 'bg-amber-400/15'
      : buy
        ? 'bg-success/15'
        : 'bg-foreground/10'}"
  >
    {#if reward}
      <GiftIcon class="size-3.5" />
    {:else if buy}
      <ArrowDownLeft class="size-3.5" />
    {:else}
      <ArrowUpRight class="size-3.5" />
    {/if}
  </span>
  {value}
</span>
