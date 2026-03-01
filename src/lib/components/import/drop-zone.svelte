<script lang="ts">
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { open } from "@tauri-apps/plugin-dialog";
  import {
    UploadIcon,
    FileSpreadsheetIcon,
    ShieldCheckIcon,
    BadgeInfoIcon,
    ShieldIcon,
    InfoIcon,
    FileIcon,
    FileQuestionMarkIcon,
    ExternalLinkIcon,
  } from "@lucide/svelte";
  import * as Card from "$lib/components/ui/card";
  import { Badge } from "$lib/components/ui/badge";
  import { Button } from "$lib/components/ui/button";

  interface Props {
    onfileselected: (path: string) => void;
    disabled?: boolean;
  }

  let { onfileselected, disabled = false }: Props = $props();

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
            ? "border-primary bg-primary/5 scale-[1.02]"
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

    <!-- Info cards -->
    <div class="grid grid-cols-2 gap-4">
      <Card.Root class="gap-3 py-5 shadow-none glass-panel">
        <Card.Header class="px-5 gap-1">
          <div class="flex items-center gap-2 text-muted-foreground">
            <div
              class="flex size-8 shrink-0 items-center justify-center rounded-full bg-muted text-muted-foreground"
            >
              <FileQuestionMarkIcon class="size-4" />
            </div>
            <Card.Description class="text-foreground tracking-wide">
              Where to find your file?
            </Card.Description>
          </div>
        </Card.Header>
        <Card.Content class="px-5">
          <p class="text-sm text-muted-foreground leading-relaxed">
            Log in to your Kraken account, navigate to the History tab, and
            select &ldquo;Export&rdquo;. Ensure you select
            <span class="text-foreground">Ledgers</span> as the export type.
          </p>
          <a
            href="https://support.kraken.com/articles/208267878-how-to-export-your-account-history"
            target="_blank"
            rel="noopener noreferrer"
            class="mt-2 inline-flex items-center gap-1 text-sm text-primary hover:underline"
          >
            Go to Kraken export
            <ExternalLinkIcon class="size-3" />
          </a>
        </Card.Content>
      </Card.Root>

      <Card.Root class="gap-3 py-5 shadow-none glass-panel">
        <Card.Header class="px-5 gap-1">
          <div class="flex items-center gap-2 text-muted-foreground">
            <div
              class="flex size-8 shrink-0 items-center justify-center rounded-full bg-muted text-muted-foreground"
            >
              <ShieldIcon class="size-4 fill-muted-foreground" />
            </div>
            <Card.Description class="text-foreground tracking-wide">
              Privacy First
            </Card.Description>
          </div>
        </Card.Header>
        <Card.Content class="px-5">
          <p class="text-sm text-muted-foreground leading-relaxed">
            Your data is processed <span class="text-foreground"
              >entirely locally</span
            >
            on your machine. No financial information is ever sent to external servers
            or cloud storage.
          </p>
        </Card.Content>
      </Card.Root>
    </div>
  </div>
</div>
