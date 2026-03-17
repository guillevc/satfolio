<script lang="ts">
  import { AlertCircleIcon } from "@lucide/svelte";
  import { Spinner } from "$lib/components/ui/spinner";
  import { previewImport, confirmImport } from "$lib/api";
  import { getQuote } from "$lib/stores/config.svelte";
  import { loadDashboard } from "$lib/stores/dashboard.svelte";
  import { loadTrades } from "$lib/stores/trades.svelte";
  import {
    importedFiles,
    addImport,
    deleteImport,
  } from "$lib/stores/imported-files.svelte";
  import type { ImportPreview as ImportPreviewData } from "$lib/types/bindings";
  import * as Dialog from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import { Separator } from "$lib/components/ui/separator";
  import DropZone from "./drop-zone.svelte";
  import ImportPreview from "./import-preview.svelte";
  import DuplicateWarning from "./duplicate-warning.svelte";
  import FileList from "./file-list.svelte";
  import InfoCards from "./info-cards.svelte";

  type DialogState =
    | { step: "closed" }
    | { step: "preview"; path: string; preview: ImportPreviewData }
    | { step: "confirming"; path: string; preview: ImportPreviewData }
    | { step: "duplicate-file"; path: string; filename: string }
    | { step: "all-duplicates"; path: string; count: number }
    | { step: "error"; path: string; message: string };

  let dialogState: DialogState = $state({ step: "closed" });
  let dialogOpen = $derived(dialogState.step !== "closed");
  let hasFiles = $derived(importedFiles.list.length > 0);
  let loading = $state(false);

  function handleOpenChange(open: boolean) {
    if (!open && dialogState.step === "confirming") return;
    if (!open) dialogState = { step: "closed" };
  }

  async function handleFileSelected(path: string) {
    if (dialogOpen || loading) return;

    loading = true;
    try {
      const preview = await previewImport(path, getQuote());

      if (preview.exact_file_duplicate) {
        const filename = path.split("/").pop() ?? path;
        dialogState = { step: "duplicate-file", path, filename };
        return;
      }

      if (
        preview.duplicate_trades === preview.summary.total_trades &&
        preview.duplicate_trades > 0
      ) {
        dialogState = {
          step: "all-duplicates",
          path,
          count: preview.duplicate_trades,
        };
        return;
      }

      dialogState = { step: "preview", path, preview };
    } catch (e) {
      const message =
        e && typeof e === "object" && "message" in e
          ? String((e as { message: string }).message)
          : String(e);
      dialogState = { step: "error", path, message };
    } finally {
      loading = false;
    }
  }

  async function handleConfirm() {
    if (dialogState.step !== "preview") return;
    const { path, preview } = dialogState;
    dialogState = { step: "confirming", path, preview };
    try {
      const result = await confirmImport(path, getQuote());
      addImport(result.import);
      dialogState = { step: "closed" };
      loadDashboard();
      loadTrades();
    } catch (e) {
      const message =
        e && typeof e === "object" && "message" in e
          ? String((e as { message: string }).message)
          : String(e);
      dialogState = { step: "error", path, message };
    }
  }

  function handleCancel() {
    dialogState = { step: "closed" };
  }

  async function handleRemove(id: number) {
    await deleteImport(id);
    loadDashboard();
    loadTrades();
  }
</script>

<div class="flex h-8 flex-1 flex-col overflow-auto py-4">
  {#if loading}
    <div class="flex flex-1 flex-col items-center justify-center gap-3">
      <Spinner class="size-8 text-primary" />
      <p class="text-sm text-muted-foreground">Reading file...</p>
    </div>
  {:else if hasFiles}
    <div class="flex items-center px-6">
      <h2 class="h-8 text-xl font-semibold">Import</h2>
    </div>

    <Separator class="mt-4 mb-6" />

    <div class="flex flex-col gap-6 px-6">
      <DropZone
        onfileselected={handleFileSelected}
        compact
        disabled={dialogOpen}
      />

      <div class="flex flex-col gap-4">
        <div class="flex items-center gap-3">
          <h2 class="text-lg font-semibold">Imported Files</h2>
          <span
            class="rounded-md bg-muted px-2 py-0.5 font-mono text-xs text-muted-foreground"
          >
            {importedFiles.list.length}
          </span>
        </div>
        <FileList files={importedFiles.list} onremove={handleRemove} />
      </div>

      <InfoCards />
    </div>
  {:else}
    <DropZone onfileselected={handleFileSelected} disabled={dialogOpen} />
  {/if}
</div>

<Dialog.Root open={dialogOpen} onOpenChange={handleOpenChange}>
  <Dialog.Content
    class="sm:max-w-2xl"
    showCloseButton={dialogState.step !== "confirming"}
  >
    {#if dialogState.step === "preview" || dialogState.step === "confirming"}
      <ImportPreview
        path={dialogState.path}
        preview={dialogState.preview}
        confirming={dialogState.step === "confirming"}
        onconfirm={handleConfirm}
        oncancel={handleCancel}
      />
    {:else if dialogState.step === "duplicate-file"}
      <DuplicateWarning
        mode="file"
        filename={dialogState.filename}
        onclose={handleCancel}
      />
    {:else if dialogState.step === "all-duplicates"}
      <DuplicateWarning
        mode="trades"
        count={dialogState.count}
        onclose={handleCancel}
      />
    {:else if dialogState.step === "error"}
      <Dialog.Header>
        <div class="flex items-center gap-3">
          <div
            class="flex size-9 shrink-0 items-center justify-center rounded-full bg-destructive/10"
          >
            <AlertCircleIcon class="size-5 text-destructive" />
          </div>
          <Dialog.Title>Import Failed</Dialog.Title>
        </div>
        <Dialog.Description class="mt-2">
          {dialogState.message}
        </Dialog.Description>
      </Dialog.Header>
      <Dialog.Footer>
        <Button variant="outline" onclick={handleCancel}>Close</Button>
      </Dialog.Footer>
    {/if}
  </Dialog.Content>
</Dialog.Root>
