<script lang="ts">
  import {
    ChartLineIcon,
    UploadIcon,
    SettingsIcon,
    ArrowLeftRightIcon,
  } from "@lucide/svelte";
  import * as Tooltip from "$lib/components/ui/tooltip";
  import { Separator } from "$lib/components/ui/separator";
  import type { View } from "$lib/types";

  interface Props {
    active: View;
    onnavigate: (view: View) => void;
    hasImports: boolean;
  }

  let { active, onnavigate, hasImports }: Props = $props();

  const navItems: { view: View; icon: typeof ChartLineIcon; label: string }[] =
    [
      { view: "dashboard", icon: ChartLineIcon, label: "Dashboard" },
      { view: "trades", icon: ArrowLeftRightIcon, label: "Trades" },
      { view: "import", icon: UploadIcon, label: "Import" },
    ];
</script>

{#snippet navItem(view: View, Icon: typeof ChartLineIcon, label: string)}
  {@const disabled = !hasImports && view !== "import"}
  <Tooltip.Root>
    <Tooltip.Trigger>
      <button
        class={[
          "flex size-10 items-center justify-center rounded-lg transition-all",
          disabled && "cursor-not-allowed text-muted-foreground/30",
          !disabled && "text-muted-foreground",
          !disabled &&
            active === view &&
            "bg-primary/20 text-primary shadow-[0_0_12px_-3px] shadow-primary/40",
          !disabled &&
            active !== view &&
            "hover:bg-white/5 hover:text-foreground",
        ]}
        {disabled}
        onclick={() => onnavigate(view)}
      >
        <Icon class="size-5" />
      </button>
    </Tooltip.Trigger>
    <Tooltip.Content side="right">
      <p>{label}{disabled ? " (import data first)" : ""}</p>
    </Tooltip.Content>
  </Tooltip.Root>
{/snippet}

<Tooltip.Provider delayDuration={0}>
  <nav
    class="flex w-16 flex-col items-center gap-1 border-r border-sidebar-border bg-sidebar py-3"
  >
    <div class="flex flex-col items-center gap-1">
      {#each navItems as { view, icon, label } (view)}
        {@render navItem(view, icon, label)}
      {/each}
    </div>

    <Separator
      class="mx-4 my-2 self-stretch data-[orientation=horizontal]:w-auto"
    />

    {@render navItem("settings", SettingsIcon, "Settings")}
  </nav>
</Tooltip.Provider>
