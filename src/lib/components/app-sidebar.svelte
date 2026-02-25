<script lang="ts">
  import {
    ChartLineIcon,
    FlaskConicalIcon,
    SettingsIcon,
    ArrowLeftRightIcon,
  } from "@lucide/svelte";
  import * as Tooltip from "$lib/components/ui/tooltip";
  import { Separator } from "$lib/components/ui/separator";
  import type { View } from "$lib/types";

  interface Props {
    active: View;
    onnavigate: (view: View) => void;
  }

  let { active, onnavigate }: Props = $props();

  const navItems: { view: View; icon: typeof ChartLineIcon; label: string }[] =
    [
      { view: "dashboard", icon: ChartLineIcon, label: "Dashboard" },
      { view: "trades", icon: ArrowLeftRightIcon, label: "Trades" },
      { view: "simulator", icon: FlaskConicalIcon, label: "Simulator" },
    ];
</script>

{#snippet navItem(view: View, Icon: typeof ChartLineIcon, label: string)}
  <Tooltip.Root>
    <Tooltip.Trigger>
      <button
        class={[
          "flex size-10 items-center justify-center rounded-lg text-muted-foreground transition-all",
          active === view &&
            "bg-primary/20 text-primary shadow-[0_0_12px_-3px] shadow-primary/40",
          active !== view && "hover:bg-white/5 hover:text-foreground",
        ]}
        onclick={() => onnavigate(view)}
      >
        <Icon class="size-5" />
      </button>
    </Tooltip.Trigger>
    <Tooltip.Content side="right">
      <p>{label}</p>
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

    <Separator class="mx-3 my-2 w-8" />

    {@render navItem("settings", SettingsIcon, "Settings")}
  </nav>
</Tooltip.Provider>
