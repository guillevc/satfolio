<script lang="ts">
  import { AlertCircleIcon, RotateCcwIcon } from "@lucide/svelte";
  import { Spinner } from "$lib/components/ui/spinner";
  import { previewImport, confirmImport } from "$lib/api";
  import { loadDashboard } from "$lib/stores/dashboard.svelte";
  import { loadTrades } from "$lib/stores/trades.svelte";
  import {
    importedFiles,
    addImportedFile,
    removeImportedFile,
    isFilenameDuplicate,
  } from "$lib/stores/imported-files.svelte";
  import type { TradesSummary } from "$lib/types/bindings";
  import * as Dialog from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import DropZone from "./drop-zone.svelte";
  import ImportPreview from "./import-preview.svelte";
  import DuplicateWarning from "./duplicate-warning.svelte";
  import FileList from "./file-list.svelte";

  type DialogState =
    | { step: "closed" }
    | { step: "preview"; path: string; summary: TradesSummary }
    | { step: "confirming"; path: string; summary: TradesSummary }
    | { step: "duplicate"; path: string; filename: string }
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

    if (isFilenameDuplicate(path)) {
      const filename = path.split("/").pop() ?? path;
      dialogState = { step: "duplicate", path, filename };
      return;
    }

    loading = true;
    try {
      const summary = await previewImport(path);
      dialogState = { step: "preview", path, summary };
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
    const { path, summary } = dialogState;
    dialogState = { step: "confirming", path, summary };
    try {
      const result = await confirmImport(path);
      addImportedFile(path, result);
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

  function handleRemove(id: string) {
    removeImportedFile(id);
  }
</script>

<div class="flex flex-1 flex-col gap-4 overflow-auto py-6">
  {#if loading}
    <div class="flex flex-1 flex-col items-center justify-center gap-3">
      <Spinner class="size-8 text-primary" />
      <p class="text-sm text-muted-foreground">Parsing file...</p>
    </div>
  {:else if hasFiles}
    <DropZone
      onfileselected={handleFileSelected}
      compact
      disabled={dialogOpen}
    />
    <FileList files={importedFiles.list} onremove={handleRemove} />
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
        summary={dialogState.summary}
        confirming={dialogState.step === "confirming"}
        onconfirm={handleConfirm}
        oncancel={handleCancel}
      />
    {:else if dialogState.step === "duplicate"}
      <DuplicateWarning
        filename={dialogState.filename}
        onclose={handleCancel}
      />
    {:else if dialogState.step === "error"}
      <Dialog.Header>
        <Dialog.Title>Import Failed</Dialog.Title>
        <Dialog.Description>{dialogState.message}</Dialog.Description>
      </Dialog.Header>
      <div class="flex flex-col items-center gap-4 py-4">
        <AlertCircleIcon class="size-12 text-destructive" />
      </div>
      <Dialog.Footer>
        <Button
          variant="outline"
          onclick={() => {
            if (dialogState.step === "error")
              handleFileSelected(dialogState.path);
          }}
        >
          <RotateCcwIcon class="size-4" />
          Try Again
        </Button>
        <Button variant="ghost" onclick={handleCancel}>Cancel</Button>
      </Dialog.Footer>
    {/if}
  </Dialog.Content>
</Dialog.Root>
