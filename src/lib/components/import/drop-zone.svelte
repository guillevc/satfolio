<script lang="ts">
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { open } from "@tauri-apps/plugin-dialog";
  import { UploadIcon, FileSpreadsheetIcon } from "@lucide/svelte";
  import * as Card from "$lib/components/ui/card";
  import { Badge } from "$lib/components/ui/badge";
  import InfoCards from "./info-cards.svelte";

  interface Props {
    onfileselected: (path: string) => void;
    disabled?: boolean;
    compact?: boolean;
  }

  let { onfileselected, disabled = false, compact = false }: Props = $props();

  let dragging = $state(false);
  let validationError = $state<string | null>(null);

  function isValidCsv(path: string): boolean {
    return path.toLowerCase().endsWith(".csv");
  }

  function handleFile(path: string) {
    if (!isValidCsv(path)) {
      validationError =
        "Only .csv files are supported. Please select a Kraken CSV export.";
      return;
    }
    validationError = null;
    onfileselected(path);
  }

  async function handleBrowse() {
    const path = await open({
      multiple: false,
      filters: [{ name: "CSV", extensions: ["csv"] }],
    });
    if (path) handleFile(path);
  }

  $effect(() => {
    const unlisten = getCurrentWebview().onDragDropEvent((event) => {
      if (disabled) return;

      if (event.payload.type === "over") {
        dragging = true;
      } else if (event.payload.type === "leave") {
        dragging = false;
      } else if (event.payload.type === "drop") {
        dragging = false;
        const paths = event.payload.paths;
        if (paths.length > 0) {
          handleFile(paths[0]);
        }
      }
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  });
</script>

{#if compact}
  <div class="glass-panel rounded-2xl p-1">
    <button
      type="button"
      onclick={handleBrowse}
      {disabled}
      class={[
        "bg-surface-dark/40 flex w-full items-center gap-4 rounded-xl border-2 border-dashed px-4 py-6 transition-all",
        dragging
          ? "border-primary bg-primary/5"
          : "border-muted-foreground/25 hover:border-primary/50 hover:bg-white/2",
        disabled && "pointer-events-none opacity-50",
      ]}
    >
      <div
        class={[
          "flex size-12 shrink-0 items-center justify-center rounded-full text-primary transition-colors",
          dragging ? "bg-primary/20" : "bg-primary/10",
        ]}
      >
        <UploadIcon class="size-5" />
      </div>
      <div class="text-left">
        <p class="font-semibold text-foreground">
          {dragging ? "Drop your file here" : "Drop your Kraken CSV here"}
        </p>
        <p class="text-sm text-muted-foreground">
          or <span class="text-primary underline underline-offset-2"
            >browse</span
          > to select a file
        </p>
      </div>
      <Badge variant="outline" class="ml-auto shrink-0 text-muted-foreground">
        <FileSpreadsheetIcon class="size-4" />
        Supports .csv files
      </Badge>
    </button>
    {#if validationError}
      <p class="px-4 py-1.5 text-sm text-destructive">{validationError}</p>
    {/if}
  </div>
{:else}
  <div class="flex flex-1 flex-col items-center justify-center px-8">
    <div class="flex w-full max-w-4xl flex-col gap-4">
      <!-- Drop area -->
      <Card.Root class="glass-panel p-1">
        <button
          type="button"
          onclick={handleBrowse}
          {disabled}
          class={[
            "group flex w-full flex-col items-center gap-5 rounded-lg border-2 border-dashed px-16 py-20 transition-all",
            dragging
              ? "scale-[1.02] border-primary bg-primary/5"
              : "border-muted-foreground/25 hover:border-primary/50 hover:bg-white/2",
            disabled && "pointer-events-none opacity-50",
          ]}
        >
          <div
            class={[
              "flex size-16 items-center justify-center rounded-full text-primary transition-colors",
              dragging ? "bg-primary/20" : "bg-primary/10",
            ]}
          >
            <UploadIcon class="size-7" />
          </div>

          <div class="text-center">
            <p class="text-lg font-medium text-foreground">
              {dragging ? "Drop your file here" : "Drop your Kraken CSV here"}
            </p>
            <p class="mt-1 text-sm text-muted-foreground">
              or <span class="text-primary underline underline-offset-2"
                >browse</span
              >
              to select a file
            </p>
          </div>

          <Badge variant="outline" class="text-muted-foreground">
            <FileSpreadsheetIcon class="size-4" />
            Supports .csv files
          </Badge>

          {#if validationError}
            <p class="text-sm text-destructive">{validationError}</p>
          {/if}
        </button>
      </Card.Root>

      <InfoCards />
    </div>
  </div>
{/if}
