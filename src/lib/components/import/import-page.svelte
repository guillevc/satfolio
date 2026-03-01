<script lang="ts">
  import {
    CheckCircle2Icon,
    LoaderCircleIcon,
    AlertCircleIcon,
    RotateCcwIcon,
    FolderOpenIcon,
  } from "@lucide/svelte";
  import { previewImport, confirmImport } from "$lib/api";
  import { loadDashboard } from "$lib/stores/dashboard.svelte";
  import { loadTrades } from "$lib/stores/trades.svelte";
  import type { View } from "$lib/types";
  import type { TradesSummary } from "$lib/types/bindings";
  import { Button } from "$lib/components/ui/button";
  import DropZone from "./drop-zone.svelte";
  import ImportPreview from "./import-preview.svelte";

  interface Props {
    onnavigate: (view: View) => void;
  }

  let { onnavigate }: Props = $props();

  type ImportState =
    | { step: "idle" }
    | { step: "previewing"; path: string }
    | { step: "preview"; path: string; summary: TradesSummary }
    | { step: "confirming"; path: string; summary: TradesSummary }
    | { step: "done"; summary: TradesSummary }
    | { step: "error"; path: string | null; message: string };

  let state: ImportState = $state({ step: "idle" });

  async function handleFileSelected(path: string) {
    state = { step: "previewing", path };
    try {
      const summary = await previewImport(path);
      state = { step: "preview", path, summary };
    } catch (e) {
      const message =
        e && typeof e === "object" && "message" in e
          ? String((e as { message: string }).message)
          : String(e);
      state = { step: "error", path, message };
    }
  }

  async function handleConfirm() {
    if (state.step !== "preview") return;
    const { path, summary } = state;
    state = { step: "confirming", path, summary };
    try {
      const result = await confirmImport(path);
      state = { step: "done", summary: result };
      // Brief success feedback, then navigate
      setTimeout(() => {
        loadDashboard();
        loadTrades();
        onnavigate("dashboard");
      }, 1500);
    } catch (e) {
      const message =
        e && typeof e === "object" && "message" in e
          ? String((e as { message: string }).message)
          : String(e);
      state = { step: "error", path, message };
    }
  }

  function handleReplace() {
    state = { step: "idle" };
  }
</script>

<div class="flex flex-1 flex-col overflow-hidden">
  {#if state.step === "idle"}
    <DropZone onfileselected={handleFileSelected} />
  {:else if state.step === "previewing"}
    <div class="flex flex-1 flex-col items-center justify-center gap-3">
      <LoaderCircleIcon class="size-8 animate-spin text-primary" />
      <p class="text-sm text-muted-foreground">Parsing file...</p>
    </div>
  {:else if state.step === "preview" || state.step === "confirming"}
    <ImportPreview
      path={state.path}
      summary={state.summary}
      confirming={state.step === "confirming"}
      onconfirm={handleConfirm}
      onreplace={handleReplace}
    />
  {:else if state.step === "done"}
    <div class="flex flex-1 flex-col items-center justify-center gap-3">
      <CheckCircle2Icon class="size-12 text-success" />
      <p class="text-lg font-medium">
        Imported {state.summary.total_trades} trades!
      </p>
    </div>
  {:else if state.step === "error"}
    <div class="flex flex-1 flex-col items-center justify-center gap-4">
      <AlertCircleIcon class="size-12 text-destructive" />
      <div class="text-center">
        <p class="text-lg font-medium">Import failed</p>
        <p class="mt-1 max-w-md text-sm text-muted-foreground">
          {state.message}
        </p>
      </div>
      <div class="flex gap-3">
        {#if state.path}
          <Button
            variant="outline"
            onclick={() => {
              if (state.step === "error" && state.path)
                handleFileSelected(state.path);
            }}
          >
            <RotateCcwIcon class="size-4" />
            Try Again
          </Button>
        {/if}
        <Button variant="ghost" onclick={handleReplace}>
          <FolderOpenIcon class="size-4" />
          Choose Another File
        </Button>
      </div>
    </div>
  {/if}
</div>
