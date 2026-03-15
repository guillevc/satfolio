<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    label,
    description,
    children,
  }: { label: string; description: string; children: Snippet } = $props();
</script>

<div class="flex items-center justify-between p-4">
  <div class="space-y-0.5">
    <p class="text-sm">{label}</p>
    <p class="text-sm text-muted-foreground">{description}</p>
  </div>
  {@render children()}
</div>
