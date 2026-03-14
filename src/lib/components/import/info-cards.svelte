<script lang="ts">
  import {
    FileQuestionMarkIcon,
    ShieldIcon,
    ExternalLinkIcon,
  } from "@lucide/svelte";
  import * as Card from "$lib/components/ui/card";
</script>

<div class="grid grid-cols-3 gap-4">
  <!-- Kraken -->
  <Card.Root class="glass-panel gap-3 py-5 shadow-none">
    <Card.Header class="gap-1 px-5">
      <div class="flex items-center gap-2 text-muted-foreground">
        <div
          class="flex size-8 shrink-0 items-center justify-center rounded-full bg-purple-500/10 text-purple-400"
        >
          <FileQuestionMarkIcon class="size-4" />
        </div>
        <Card.Description class="tracking-wide text-foreground">
          Export from Kraken
        </Card.Description>
      </div>
    </Card.Header>
    <Card.Content class="px-5">
      <p class="text-sm leading-relaxed text-muted-foreground">
        Go to History, click Export, and select
        <span class="text-foreground">Ledgers</span> as the export type. Export as
        CSV.
      </p>
      <a
        href="https://support.kraken.com/articles/208267878-how-to-export-your-account-history"
        target="_blank"
        rel="noopener noreferrer"
        class="mt-2 inline-flex items-center gap-1 text-sm text-primary hover:underline"
      >
        Export guide
        <ExternalLinkIcon class="size-3" />
      </a>
    </Card.Content>
  </Card.Root>

  <!-- Coinbase -->
  <Card.Root class="glass-panel gap-3 py-5 shadow-none">
    <Card.Header class="gap-1 px-5">
      <div class="flex items-center gap-2 text-muted-foreground">
        <div
          class="flex size-8 shrink-0 items-center justify-center rounded-full bg-blue-500/10 text-blue-400"
        >
          <FileQuestionMarkIcon class="size-4" />
        </div>
        <Card.Description class="tracking-wide text-foreground">
          Export from Coinbase
        </Card.Description>
      </div>
    </Card.Header>
    <Card.Content class="px-5">
      <p class="text-sm leading-relaxed text-muted-foreground">
        Open the Taxes section, go to Transaction history, and click
        <span class="text-foreground">Generate report</span>. Download the CSV
        file.
      </p>
      <a
        href="https://help.coinbase.com/en/coinbase/taxes/tools/transaction-history"
        target="_blank"
        rel="noopener noreferrer"
        class="mt-2 inline-flex items-center gap-1 text-sm text-primary hover:underline"
      >
        Export guide
        <ExternalLinkIcon class="size-3" />
      </a>
    </Card.Content>
  </Card.Root>

  <!-- Privacy -->
  <Card.Root class="glass-panel gap-3 py-5 shadow-none">
    <Card.Header class="gap-1 px-5">
      <div class="flex items-center gap-2 text-muted-foreground">
        <div
          class="flex size-8 shrink-0 items-center justify-center rounded-full bg-muted text-muted-foreground"
        >
          <ShieldIcon class="size-4 fill-muted-foreground" />
        </div>
        <Card.Description class="tracking-wide text-foreground">
          Your data stays local
        </Card.Description>
      </div>
    </Card.Header>
    <Card.Content class="px-5">
      <p class="text-sm leading-relaxed text-muted-foreground">
        Your data is processed <span class="text-foreground"
          >entirely locally</span
        >
        on your machine. No financial information is ever sent to external servers
        or cloud storage.
      </p>
    </Card.Content>
  </Card.Root>
</div>
