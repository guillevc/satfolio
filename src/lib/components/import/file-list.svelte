<script lang="ts">
  import { FileSpreadsheetIcon, Trash2Icon } from "@lucide/svelte";
  import * as AlertDialog from "$lib/components/ui/alert-dialog";
  import { Button } from "$lib/components/ui/button";
  import type { ImportRecord, Provider } from "$lib/types/bindings";

  interface Props {
    files: ImportRecord[];
    onremove: (id: number) => void;
  }

  let { files, onremove }: Props = $props();

  const providerMeta: Record<
    Provider,
    { label: string; initial: string; classes: string }
  > = {
    kraken: {
      label: "Kraken",
      initial: "K",
      classes: "bg-purple-500/20 text-purple-400 border border-purple-500/30",
    },
    coinbase: {
      label: "Coinbase",
      initial: "C",
      classes: "bg-blue-500/20 text-blue-400 border border-blue-500/30",
    },
  };

  function formatDateShort(iso: string): string {
    const d = new Date(iso);
    const month = d.toLocaleDateString("en-US", { month: "short" });
    const day = String(d.getDate()).padStart(2, "0");
    const year = String(d.getFullYear()).slice(2);
    return `${month} ${day} '${year}`;
  }

  function formatDateRange(from: string | null, to: string | null): string {
    if (!from || !to) return "—";
    return `${formatDateShort(from)} – ${formatDateShort(to)}`;
  }

  function formatRelativeTime(iso: string): string {
    const now = Date.now();
    const then = new Date(iso).getTime();
    const diffSec = Math.round((now - then) / 1000);

    if (diffSec < 60) return "just now";
    const diffMin = Math.round(diffSec / 60);
    if (diffMin < 60) return `${diffMin}m ago`;
    const diffHrs = Math.round(diffMin / 60);
    if (diffHrs < 24) return `${diffHrs}h ago`;
    const diffDays = Math.round(diffHrs / 24);
    if (diffDays < 30) return `${diffDays}d ago`;
    const diffMonths = Math.round(diffDays / 30);
    return `${diffMonths}mo ago`;
  }
</script>

<div class="glass-panel overflow-hidden rounded-xl">
  <!-- Header -->
  <div
    class="grid grid-cols-12 gap-4 border-b border-white/[0.06] bg-white/[0.03] px-4 py-2.5 text-xs font-medium tracking-wider text-muted-foreground uppercase"
  >
    <div class="col-span-3">Filename</div>
    <div class="col-span-2">Source</div>
    <div class="col-span-2 text-right">Trades</div>
    <div class="col-span-3 text-right">Date Range</div>
    <div class="col-span-1 text-right">Imported</div>
    <div class="col-span-1 text-center">Actions</div>
  </div>

  <!-- Body -->
  <div class="max-h-[60vh] overflow-y-auto">
    {#each files as file (file.id)}
      {@const provider = providerMeta[file.provider]}
      <div
        class="grid grid-cols-12 items-center gap-4 border-b border-white/[0.04] px-4 py-3 transition-colors last:border-b-0 hover:bg-white/[0.02]"
      >
        <!-- Filename -->
        <div class="col-span-3 flex min-w-0 items-center gap-2.5">
          <div
            class="flex size-8 shrink-0 items-center justify-center rounded-md bg-muted/50"
          >
            <FileSpreadsheetIcon class="size-4 text-muted-foreground" />
          </div>
          <span class="truncate font-mono text-sm">{file.filename}</span>
        </div>

        <!-- Source / Provider -->
        <div class="col-span-2 flex items-center gap-2">
          <div
            class="flex size-6 shrink-0 items-center justify-center rounded-full text-xs font-bold {provider.classes}"
          >
            {provider.initial}
          </div>
          <span class="text-sm text-muted-foreground">{provider.label}</span>
        </div>

        <!-- Trades -->
        <div class="col-span-2 text-right font-mono text-sm">
          {file.trade_count.toLocaleString()}
        </div>

        <!-- Date Range -->
        <div
          class="col-span-3 text-right font-mono text-sm text-muted-foreground"
        >
          {formatDateRange(file.date_from, file.date_to)}
        </div>

        <!-- Imported (relative) -->
        <div
          class="col-span-1 text-right font-mono text-xs text-muted-foreground"
        >
          {formatRelativeTime(file.imported_at)}
        </div>

        <!-- Actions -->
        <div class="col-span-1 flex justify-center">
          <AlertDialog.Root>
            <AlertDialog.Trigger>
              {#snippet child({ props })}
                <Button
                  variant="ghost"
                  size="icon"
                  class="size-8 text-muted-foreground hover:text-destructive"
                  {...props}
                >
                  <Trash2Icon class="size-4" />
                </Button>
              {/snippet}
            </AlertDialog.Trigger>
            <AlertDialog.Content>
              <AlertDialog.Header>
                <AlertDialog.Title
                  >Remove &ldquo;{file.filename}&rdquo;?</AlertDialog.Title
                >
                <AlertDialog.Description>
                  This will permanently remove the import record and its {file.trade_count}
                  associated trades from the database. This action cannot be undone.
                </AlertDialog.Description>
              </AlertDialog.Header>
              <AlertDialog.Footer>
                <AlertDialog.Cancel>Cancel</AlertDialog.Cancel>
                <AlertDialog.Action onclick={() => onremove(file.id)}>
                  Remove
                </AlertDialog.Action>
              </AlertDialog.Footer>
            </AlertDialog.Content>
          </AlertDialog.Root>
        </div>
      </div>
    {/each}
  </div>

  <!-- Footer -->
  <div
    class="border-t border-white/[0.06] bg-white/[0.03] px-4 py-2.5 text-xs text-muted-foreground"
  >
    Showing all imported sources
  </div>
</div>
