<script lang="ts">
  import { BugIcon } from "@lucide/svelte";
  import { Button } from "$lib/components/ui/button";
  import { updater, type UpdateState } from "$lib/updater.svelte";
  import Row from "./settings-row.svelte";
</script>

<section>
  <h3
    class="mb-3 flex items-center gap-2 text-lg font-semibold text-indigo-400"
  >
    <BugIcon class="size-5" />
    Debug
  </h3>
  <div
    class="divide-y divide-indigo-500/20 overflow-hidden rounded-xl border border-indigo-500/30 bg-indigo-500/5"
  >
    <Row
      label="Update dialog"
      description="Simulate updater states to test the UI."
    >
      <div class="flex flex-wrap gap-1.5">
        {#each ["prompt", "downloading", "success", "error"] as state (state)}
          <Button
            variant="outline"
            size="sm"
            onclick={() => {
              updater.update = {
                version: "0.99.0",
                body: "Simulated release notes for testing.",
                async downloadAndInstall(onEvent: (e: any) => void) {
                  onEvent({
                    event: "Started",
                    data: { contentLength: 1000 },
                  });
                  for (let i = 1; i <= 10; i++) {
                    await new Promise((r) => setTimeout(r, 300));
                    onEvent({
                      event: "Progress",
                      data: { chunkLength: 100 },
                    });
                  }
                  onEvent({ event: "Finished", data: {} });
                },
              } as any;
              updater.state = state as UpdateState;
              if (state === "downloading") updater.progress = 65;
              if (state === "error") updater.errorMessage = "Simulated error";
            }}
          >
            {state}
          </Button>
        {/each}
      </div>
    </Row>
    <Row
      label="Force update check"
      description="Call the real updater API. Set version to 0.1.0 in tauri.conf.json first."
    >
      <Button variant="outline" size="sm" onclick={() => updater.forceCheck()}>
        Check now
      </Button>
    </Row>
  </div>
</section>
