<script lang="ts">
  import {
    FileSpreadsheetIcon,
    Trash2Icon,
    CalendarIcon,
  } from "@lucide/svelte";
  import * as Card from "$lib/components/ui/card";
  import * as AlertDialog from "$lib/components/ui/alert-dialog";
  import { Badge } from "$lib/components/ui/badge";
  import { Button } from "$lib/components/ui/button";
  import type { ImportedFile } from "$lib/stores/imported-files.svelte";

  interface Props {
    files: ImportedFile[];
    onremove: (id: string) => void;
  }

  let { files, onremove }: Props = $props();

  let totalTrades = $derived(
    files.reduce((sum, f) => sum + f.summary.total_trades, 0),
  );

  function formatDate(date: Date): string {
    return date.toLocaleDateString("en-US", {
      month: "short",
      day: "numeric",
      hour: "numeric",
      minute: "2-digit",
    });
  }

  function formatDateRange(range: [string, string] | null): string {
    if (!range) return "";
    const fmt = (iso: string) =>
      new Date(iso).toLocaleDateString("en-US", {
        month: "short",
        day: "numeric",
      });
    return `${fmt(range[0])} – ${fmt(range[1])}`;
  }
</script>

<div class="flex flex-col gap-2 px-6">
  {#each files as file (file.id)}
    <Card.Root class="glass-panel flex-row items-center gap-3 px-4 py-3">
      <FileSpreadsheetIcon class="size-5 shrink-0 text-muted-foreground" />

      <div class="min-w-0 flex-1">
        <p class="truncate text-sm font-medium">{file.filename}</p>
        <p class="text-xs text-muted-foreground">
          {file.summary.total_trades} trades{#if file.summary.date_range}
            &nbsp;&middot; {formatDateRange(file.summary.date_range)}
          {/if}
        </p>
      </div>

      <Badge variant="outline" class="shrink-0 text-muted-foreground">
        <CalendarIcon class="size-3" />
        {formatDate(file.importedAt)}
      </Badge>

      <AlertDialog.Root>
        <AlertDialog.Trigger>
          {#snippet child({ props })}
            <Button
              variant="ghost"
              size="icon"
              class="size-8 shrink-0"
              {...props}
            >
              <Trash2Icon class="size-4 text-muted-foreground" />
            </Button>
          {/snippet}
        </AlertDialog.Trigger>
        <AlertDialog.Content>
          <AlertDialog.Header>
            <AlertDialog.Title
              >Remove &ldquo;{file.filename}&rdquo;?</AlertDialog.Title
            >
            <AlertDialog.Description>
              This removes the file from the import list only. Trades already
              processed remain in the database. Full cascade deletion will be
              available in a future update.
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
    </Card.Root>
  {/each}

  <!-- Summary line -->
  <p class="px-1 text-xs text-muted-foreground">
    {totalTrades} total trades from {files.length}
    {files.length === 1 ? "file" : "files"}
  </p>
</div>
