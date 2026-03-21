<script lang="ts">
  import {
    FileQuestionMarkIcon,
    ShieldIcon,
    ExternalLinkIcon,
  } from "@lucide/svelte";
  import * as Card from "$lib/components/ui/card";
</script>

<div class="grid grid-cols-2 gap-4">
  <!-- How to export -->
  <Card.Root class="glass-panel gap-3 py-5 shadow-none">
    <Card.Header class="gap-1 px-5">
      <div class="flex items-center gap-2 text-muted-foreground">
        <div
          class="flex size-8 shrink-0 items-center justify-center rounded-full bg-muted text-muted-foreground"
        >
          <FileQuestionMarkIcon class="size-4" />
        </div>
        <Card.Description class="tracking-wide text-foreground">
          How to export your data
        </Card.Description>
      </div>
    </Card.Header>
    <Card.Content class="flex flex-col gap-4 px-5">
      <div class="flex items-start gap-3">
        <div
          class="flex size-7 shrink-0 items-center justify-center rounded-md bg-purple-500/20 text-xs font-bold text-purple-400"
        >
          K
        </div>
        <div>
          <p class="text-sm leading-relaxed text-muted-foreground">
            <span class="font-medium text-foreground">Kraken</span> — Go to
            History, click Export, select Ledgers, and export as CSV.
            <a
              href="https://support.kraken.com/articles/208267878-how-to-export-your-account-history"
              target="_blank"
              rel="noopener noreferrer"
              class="inline-flex items-center gap-0.5 text-primary hover:underline"
            >
              Guide
              <ExternalLinkIcon class="size-3" />
            </a>
          </p>
        </div>
      </div>
      <div class="flex items-start gap-3">
        <div
          class="flex size-7 shrink-0 items-center justify-center rounded-md bg-blue-500/20 text-xs font-bold text-blue-400"
        >
          C
        </div>
        <div>
          <p class="text-sm leading-relaxed text-muted-foreground">
            <span class="font-medium text-foreground">Coinbase</span> — Open
            Taxes, go to Transaction history, click Generate report, and
            download the CSV.
            <a
              href="https://help.coinbase.com/en/coinbase/taxes/tools/transaction-history"
              target="_blank"
              rel="noopener noreferrer"
              class="inline-flex items-center gap-0.5 text-primary hover:underline"
            >
              Guide
              <ExternalLinkIcon class="size-3" />
            </a>
          </p>
        </div>
      </div>
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
        Everything runs on your machine. Your trade data never touches a server.
      </p>
    </Card.Content>
  </Card.Root>
</div>
