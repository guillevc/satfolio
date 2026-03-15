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
  <div class="flex items-center gap-3">
    <div
      class="flex size-9 shrink-0 items-center justify-center rounded-full bg-yellow-500/10"
    >
      <AlertTriangleIcon class="size-5 text-yellow-500" />
    </div>
    <Dialog.Title>
      {mode === "file" ? "Duplicate File" : "No New Data"}
    </Dialog.Title>
  </div>
  <Dialog.Description class="mt-2">
    {#if mode === "file"}
      &ldquo;{filename}&rdquo; has already been imported (verified by content
      hash).
    {:else}
      All {count} trades in this file already exist in the database.
    {/if}
  </Dialog.Description>
</Dialog.Header>

<Dialog.Footer>
  <Button variant="outline" onclick={onclose}>Close</Button>
</Dialog.Footer>
