<script lang="ts">
  import { onMount } from "svelte";
  import { updater, type UpdateState } from "$lib/updater.svelte";
  import * as Dialog from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import { Progress } from "$lib/components/ui/progress";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import Download from "@lucide/svelte/icons/download";
  import ExternalLink from "@lucide/svelte/icons/external-link";
  import RotateCcw from "@lucide/svelte/icons/rotate-ccw";
  import ArrowUpCircle from "@lucide/svelte/icons/arrow-up-circle";
  import CircleAlert from "@lucide/svelte/icons/circle-alert";
  import CircleCheck from "@lucide/svelte/icons/circle-check";
  import Loader from "@lucide/svelte/icons/loader-circle";
  import RefreshCw from "@lucide/svelte/icons/refresh-cw";

  const visibleStates: UpdateState[] = [
    "prompt",
    "downloading",
    "success",
    "error",
  ];

  let open = $derived(visibleStates.includes(updater.state));

  // Keep the last visible state so content stays rendered during the close animation
  let displayState: UpdateState = $state("idle");
  $effect(() => {
    if (visibleStates.includes(updater.state)) {
      displayState = updater.state;
    }
  });

  function onOpenChange(value: boolean) {
    if (!value) {
      updater.dismiss();
    }
  }

  onMount(() => {
    updater.check();
  });
</script>

<Dialog.Root {open} {onOpenChange}>
  <Dialog.Content>
    {#if displayState === "prompt"}
      <Dialog.Header>
        <Dialog.Title class="flex items-center gap-2">
          <ArrowUpCircle class="size-5 text-primary" />
          Update Available
        </Dialog.Title>
        <Dialog.Description>
          {#if updater.currentVersion}
            A new version is available: {updater.currentVersion} → {updater.version}
          {:else}
            Version {updater.version} is available.
          {/if}
        </Dialog.Description>
        <Button
          variant="link"
          class="h-auto justify-start px-0 text-muted-foreground"
          onclick={() =>
            openUrl(
              `https://github.com/guillevc/satfolio/releases/tag/v${updater.version}`,
            )}
        >
          Release notes
          <ExternalLink class="size-3.5" />
        </Button>
      </Dialog.Header>

      <Dialog.Footer>
        <Button variant="outline" onclick={() => updater.dismiss()}>
          Later
        </Button>
        <Button onclick={() => updater.startInstall()}>
          <Download class="size-4" />
          Install
        </Button>
      </Dialog.Footer>
    {:else if displayState === "downloading"}
      <Dialog.Header>
        <Dialog.Title class="flex items-center gap-2">
          <Loader class="size-5 animate-spin text-primary" />
          Downloading Update
        </Dialog.Title>
        <Dialog.Description>
          {#if typeof updater.progress === "number" && updater.progress > 0}
            {updater.progress}% complete{updater.formattedTotalBytes
              ? ` · ${updater.formattedTotalBytes}`
              : ""}
          {:else if updater.progress === null}
            Downloading…
          {:else}
            Preparing update…
          {/if}
        </Dialog.Description>
      </Dialog.Header>

      <Progress value={updater.progress} class="h-2" />

      <Dialog.Footer>
        <Button variant="outline" onclick={() => updater.dismiss()}>
          Cancel
        </Button>
      </Dialog.Footer>
    {:else if displayState === "success"}
      <Dialog.Header>
        <Dialog.Title class="flex items-center gap-2">
          <CircleCheck class="size-5 text-emerald-500" />
          Update Installed
        </Dialog.Title>
        <Dialog.Description>
          Version {updater.version} is installed and ready. Restart to apply the update.
        </Dialog.Description>
      </Dialog.Header>

      <Dialog.Footer>
        <Button onclick={() => updater.restart()}>
          <RotateCcw class="size-4" />
          Restart Now
        </Button>
      </Dialog.Footer>
    {:else if displayState === "error"}
      <Dialog.Header>
        <Dialog.Title class="flex items-center gap-2">
          <CircleAlert class="size-5 text-destructive" />
          Update Failed
        </Dialog.Title>
        <Dialog.Description>
          {updater.errorMessage}
        </Dialog.Description>
      </Dialog.Header>

      <Dialog.Footer>
        <Button variant="outline" onclick={() => updater.dismiss()}>
          Close
        </Button>
        <Button onclick={() => updater.startInstall()}>
          <RefreshCw class="size-4" />
          Retry
        </Button>
      </Dialog.Footer>
    {/if}
  </Dialog.Content>
</Dialog.Root>
