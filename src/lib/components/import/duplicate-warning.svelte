<script lang="ts">
  import { AlertTriangleIcon } from "@lucide/svelte";
  import * as Dialog from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";

  type Props =
    | { mode: "file"; filename: string; count?: never; onclose: () => void }
    | { mode: "trades"; count: number; filename?: never; onclose: () => void };

  let { mode, filename, count, onclose }: Props = $props();
</script>

<Dialog.Header>
  <Dialog.Title>
    {mode === "file" ? "Duplicate File" : "No New Data"}
  </Dialog.Title>
  <Dialog.Description>Nothing new to import.</Dialog.Description>
</Dialog.Header>

<div
  class="flex items-start gap-3 rounded-lg border border-yellow-500/20 bg-yellow-500/5 px-4 py-3"
>
  <AlertTriangleIcon class="mt-0.5 size-5 shrink-0 text-yellow-500" />
  <div class="text-sm">
    {#if mode === "file"}
      <p class="font-medium text-yellow-500">
        &ldquo;{filename}&rdquo; already imported
      </p>
      <p class="mt-1 text-muted-foreground">
        This file has already been imported (verified by content hash).
      </p>
    {:else}
      <p class="font-medium text-yellow-500">
        All {count} trades already exist
      </p>
      <p class="mt-1 text-muted-foreground">
        All {count} trades in this file already exist in the database.
      </p>
    {/if}
  </div>
</div>

<Dialog.Footer>
  <Button variant="outline" onclick={onclose}>Close</Button>
</Dialog.Footer>
