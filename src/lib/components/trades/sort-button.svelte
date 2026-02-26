<script lang="ts">
  import { ArrowUp, ArrowDown, ArrowUpDown } from "@lucide/svelte";
  import { Button } from "$lib/components/ui/button";

  let {
    label,
    sorted = false,
    onclick,
  }: {
    label: string;
    sorted: false | "asc" | "desc";
    onclick: (e: Event) => void;
  } = $props();
</script>

<Button variant="ghost" {onclick}>
  {label}
  {#if sorted === "asc"}
    <ArrowUp class="ml-2 size-3.5" />
  {:else if sorted === "desc"}
    <ArrowDown class="ml-2 size-3.5" />
  {:else}
    <ArrowUpDown class="ml-2 size-3.5" />
  {/if}
</Button>
