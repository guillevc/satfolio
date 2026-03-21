<script lang="ts">
  import { onMount } from "svelte";
  import { appDataDir, join } from "@tauri-apps/api/path";
  import { revealItemInDir } from "@tauri-apps/plugin-opener";
  import {
    CheckIcon,
    CopyIcon,
    FolderOpenIcon,
    Trash2Icon,
  } from "@lucide/svelte";
  import * as AlertDialog from "$lib/components/ui/alert-dialog";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Separator } from "$lib/components/ui/separator";
  import * as ToggleGroup from "$lib/components/ui/toggle-group";
  import { nukeAllData } from "$lib/api";
  import {
    getQuote,
    setQuote,
    type QuoteCurrency,
  } from "$lib/stores/config.svelte";
  import { loadDashboard } from "$lib/stores/dashboard.svelte";
  import { loadTrades } from "$lib/stores/trades.svelte";
  import { systemLocale } from "$lib/utils/locale";
  import Row from "./settings-row.svelte";

  let dbPath = $state("Loading\u2026");
  let nukeDialogOpen = $state(false);
  let copied = $state(false);

  let baseCurrency: QuoteCurrency = $state(getQuote());
  let bitcoinUnit = $state("BTC");

  onMount(async () => {
    try {
      const dir = await appDataDir();
      dbPath = await join(dir, "satfolio.db");
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
  <div class="flex items-center px-6">
    <h2 class="h-8 text-xl font-semibold">Settings</h2>
  </div>

  <Separator class="mt-4 mb-6" />

  <div class="flex flex-col gap-8 px-6">
    <!-- Currency -->
    <section>
      <h3 class="mb-3 text-lg font-semibold">Display</h3>
      <div
        class="divide-y divide-border overflow-hidden rounded-xl border bg-card"
      >
        <Row
          label="Currency"
          description="Currency used to display all prices and values."
        >
          <ToggleGroup.Root
            type="single"
            value={baseCurrency}
            onValueChange={(v) => {
              if (v) {
                const q = v as QuoteCurrency;
                baseCurrency = q;
                setQuote(q);
                loadDashboard();
                loadTrades();
              }
            }}
            variant="outline"
            size="sm"
          >
            <ToggleGroup.Item
              value="EUR"
              class="data-[state=on]:bg-primary data-[state=on]:text-primary-foreground"
              >EUR</ToggleGroup.Item
            >
            <ToggleGroup.Item
              value="USD"
              class="data-[state=on]:bg-primary data-[state=on]:text-primary-foreground"
              >USD</ToggleGroup.Item
            >
            <ToggleGroup.Item
              value="GBP"
              class="data-[state=on]:bg-primary data-[state=on]:text-primary-foreground"
              >GBP</ToggleGroup.Item
            >
          </ToggleGroup.Root>
        </Row>

        <Row
          label="Bitcoin Units"
          description="Display amounts in whole BTC or satoshis."
        >
          <ToggleGroup.Root
            type="single"
            value={bitcoinUnit}
            onValueChange={(v) => {
              if (v) bitcoinUnit = v;
            }}
            variant="outline"
            size="sm"
            disabled
          >
            <ToggleGroup.Item
              value="BTC"
              class="data-[state=on]:bg-primary data-[state=on]:text-primary-foreground"
              >BTC</ToggleGroup.Item
            >
            <ToggleGroup.Item
              value="sats"
              class="data-[state=on]:bg-primary data-[state=on]:text-primary-foreground"
              >sats</ToggleGroup.Item
            >
          </ToggleGroup.Root>
        </Row>

        <Row
          label="Locale"
          description="Detected from system language. Controls number and date formatting."
        >
          <span class="font-mono text-sm">{systemLocale}</span>
        </Row>
      </div>
    </section>

    <!-- Data -->
    <section>
      <h3 class="mb-3 text-lg font-semibold">Data</h3>
      <div
        class="divide-y divide-border overflow-hidden rounded-xl border bg-card"
      >
        <Row
          label="Database location"
          description="SQLite file used by Satfolio."
        >
          <div class="flex items-center gap-2">
            <Input
              disabled
              value={dbPath}
              size={dbPath.length}
              class="font-mono text-xs"
            />
            <Button
              variant="outline"
              size="icon"
              class="size-9"
              disabled={dbPath === "Loading\u2026" || dbPath === "Unknown"}
              onclick={() => {
                navigator.clipboard.writeText(dbPath);
                copied = true;
                setTimeout(() => (copied = false), 2000);
              }}
            >
              {#if copied}
                <CheckIcon class="size-4" />
              {:else}
                <CopyIcon class="size-4" />
              {/if}
            </Button>
            <Button
              variant="outline"
              size="icon"
              class="size-9"
              disabled={dbPath === "Loading\u2026" || dbPath === "Unknown"}
              onclick={() => revealItemInDir(dbPath)}
            >
              <FolderOpenIcon class="size-4" />
            </Button>
          </div>
        </Row>
      </div>
    </section>

    <!-- Danger Zone -->
    <section>
      <h3 class="mb-3 text-lg font-semibold text-destructive">Danger Zone</h3>
      <div
        class="overflow-hidden rounded-xl border border-destructive/30 bg-destructive/5 p-5"
      >
        <div class="flex items-center justify-between gap-4">
          <div class="space-y-1">
            <p class="text-sm font-semibold text-destructive">
              Delete all data
            </p>
            <p class="max-w-md text-xs text-destructive/70">
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
                  price data. Your settings (currency, locale) will be
                  preserved. The app will restart automatically. This action
                  cannot be undone.
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
      </div>
    </section>

    {#if import.meta.env.DEV}
      {#await import("./debug-settings.svelte") then { default: DebugSettings }}
        <DebugSettings />
      {/await}
    {/if}
  </div>
</div>
