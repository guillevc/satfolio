<script lang="ts">
  import { onMount } from "svelte";
  import { appDataDir } from "@tauri-apps/api/path";
  import { Trash2Icon } from "@lucide/svelte";
  import * as AlertDialog from "$lib/components/ui/alert-dialog";
  import { Button } from "$lib/components/ui/button";
  import * as Card from "$lib/components/ui/card";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import { Separator } from "$lib/components/ui/separator";
  import { nukeAllData } from "$lib/api";

  let dbPath = $state("Loading\u2026");
  let nukeDialogOpen = $state(false);

  onMount(async () => {
    try {
      const dir = await appDataDir();
      dbPath = `${dir}betc.db`;
    } catch {
      dbPath = "Unknown";
    }
  });

  async function handleNuke() {
    nukeDialogOpen = false;
    await nukeAllData();
  }
</script>

<div class="flex flex-1 flex-col overflow-auto py-4">
  <div class="flex h-8 items-center px-6">
    <h2 class="text-xl font-semibold">Settings</h2>
  </div>

  <Separator class="mt-4 mb-6" />

  <div class="flex flex-col gap-6 px-6">
    <Card.Root>
      <Card.Header>
        <Card.Title>Trading Pair</Card.Title>
        <Card.Description>
          The base asset and quote currency used for tracking trades.
        </Card.Description>
      </Card.Header>
      <Card.Content class="grid gap-4 sm:grid-cols-2">
        <div class="flex flex-col gap-2">
          <Label>Base asset</Label>
          <Input disabled value="BTC" />
          <p class="text-xs text-muted-foreground">
            Configurable in a future release.
          </p>
        </div>
        <div class="flex flex-col gap-2">
          <Label>Quote currency</Label>
          <Input disabled value="EUR" />
          <p class="text-xs text-muted-foreground">
            Configurable in a future release.
          </p>
        </div>
      </Card.Content>
    </Card.Root>

    <Card.Root>
      <Card.Header>
        <Card.Title>Display</Card.Title>
        <Card.Description>
          How dates and times are shown throughout the app.
        </Card.Description>
      </Card.Header>
      <Card.Content class="grid gap-4 sm:grid-cols-2">
        <div class="flex flex-col gap-2">
          <Label>Date format</Label>
          <Input disabled value="YYYY-MM-DD" />
          <p class="text-xs text-muted-foreground">
            Configurable in a future release.
          </p>
        </div>
        <div class="flex flex-col gap-2">
          <Label>Timezone</Label>
          <Input disabled value="UTC" />
          <p class="text-xs text-muted-foreground">
            Configurable in a future release.
          </p>
        </div>
      </Card.Content>
    </Card.Root>

    <Card.Root>
      <Card.Header>
        <Card.Title>Data</Card.Title>
        <Card.Description>
          Where your trade data is stored on disk.
        </Card.Description>
      </Card.Header>
      <Card.Content>
        <div class="flex flex-col gap-2">
          <Label>Database location</Label>
          <Input disabled value={dbPath} />
          <p class="text-xs text-muted-foreground">
            The SQLite database file used by betc.
          </p>
        </div>
      </Card.Content>
    </Card.Root>

    <Card.Root class="border-destructive/30">
      <Card.Header>
        <Card.Title>Danger Zone</Card.Title>
        <Card.Description>
          Irreversible actions that affect all your data.
        </Card.Description>
      </Card.Header>
      <Card.Content>
        <div class="flex items-center justify-between gap-4">
          <div>
            <p class="text-sm font-medium">Delete all data</p>
            <p class="text-xs text-muted-foreground">
              Permanently remove all imports, trades, and price history. You
              will need to re-import your CSV files to restore data.
            </p>
          </div>
          <AlertDialog.Root bind:open={nukeDialogOpen}>
            <AlertDialog.Trigger>
              {#snippet child({ props })}
                <Button variant="destructive" size="sm" {...props}>
                  <Trash2Icon class="size-4" />
                  Delete All Data
                </Button>
              {/snippet}
            </AlertDialog.Trigger>
            <AlertDialog.Content>
              <AlertDialog.Header>
                <AlertDialog.Title>Delete all data?</AlertDialog.Title>
                <AlertDialog.Description>
                  This will permanently delete all imports, trades, and cached
                  price data from the database. The app will restart
                  automatically. This action cannot be undone.
                </AlertDialog.Description>
              </AlertDialog.Header>
              <AlertDialog.Footer>
                <AlertDialog.Cancel>Cancel</AlertDialog.Cancel>
                <AlertDialog.Action onclick={handleNuke}>
                  Delete Everything
                </AlertDialog.Action>
              </AlertDialog.Footer>
            </AlertDialog.Content>
          </AlertDialog.Root>
        </div>
      </Card.Content>
    </Card.Root>
  </div>
</div>
