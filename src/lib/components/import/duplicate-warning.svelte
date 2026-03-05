<script lang="ts">
  import { AlertTriangleIcon } from "@lucide/svelte";
  import * as Dialog from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";

  interface Props {
    filename: string;
    onclose: () => void;
  }

  let { filename, onclose }: Props = $props();
</script>

<Dialog.Header>
  <Dialog.Title>Duplicate File</Dialog.Title>
  <Dialog.Description>
    A file with this name has already been imported.
  </Dialog.Description>
</Dialog.Header>

<div
  class="flex items-start gap-3 rounded-lg border border-yellow-500/20 bg-yellow-500/5 px-4 py-3"
>
  <AlertTriangleIcon class="mt-0.5 size-5 shrink-0 text-yellow-500" />
  <div class="text-sm">
    <p class="font-medium text-yellow-500">
      &ldquo;{filename}&rdquo; already exists
    </p>
    <p class="mt-1 text-muted-foreground">
      Rename the file before importing to avoid confusion. Duplicate detection
      by content hash will be supported in a future update.
    </p>
  </div>
</div>

<Dialog.Footer>
  <Button variant="outline" onclick={onclose}>Close</Button>
</Dialog.Footer>
