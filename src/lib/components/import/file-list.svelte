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
  import type { ImportRecord } from "$lib/types/bindings";

  interface Props {
    files: ImportRecord[];
    onremove: (id: number) => void;
  }

  let { files, onremove }: Props = $props();

  let totalTrades = $derived(files.reduce((sum, f) => sum + f.trade_count, 0));

  function formatImportedAt(iso: string): string {
    return new Date(iso).toLocaleDateString("en-US", {
      month: "short",
      day: "numeric",
      hour: "numeric",
      minute: "2-digit",
    });
  }

  function formatDateRange(from: string | null, to: string | null): string {
    if (!from || !to) return "";
    const fmt = (iso: string) =>
      new Date(iso).toLocaleDateString("en-US", {
        month: "short",
        day: "numeric",
      });
    return `${fmt(from)} – ${fmt(to)}`;
  }
</script>

<div class="flex flex-col gap-2 px-6">
  {#each files as file (file.id)}
    <Card.Root class="glass-panel flex-row items-center gap-3 px-4 py-3">
      <FileSpreadsheetIcon class="size-5 shrink-0 text-muted-foreground" />

      <div class="min-w-0 flex-1">
        <p class="truncate text-sm font-medium">{file.filename}</p>
        <p class="text-xs text-muted-foreground">
          {file.trade_count} trades{#if file.date_from && file.date_to}
            &nbsp;&middot; {formatDateRange(file.date_from, file.date_to)}
          {/if}
        </p>
      </div>

      <Badge variant="outline" class="shrink-0 text-muted-foreground">
        <CalendarIcon class="size-3" />
        {formatImportedAt(file.imported_at)}
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
    </Card.Root>
  {/each}

  <!-- Summary line -->
  <p class="px-1 text-xs text-muted-foreground">
    {totalTrades} total trades from {files.length}
    {files.length === 1 ? "file" : "files"}
  </p>
</div>
