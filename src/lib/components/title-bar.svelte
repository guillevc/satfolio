<script lang="ts">
  interface Props {
    title?: string;
  }

  let { title = "Dashboard" }: Props = $props();
</script>

<header
  class="flex h-9 items-center border-b border-sidebar-border bg-sidebar px-3"
  data-tauri-drag-region
>
  <div class="flex items-baseline gap-2 pl-[70px]">
    <span class="text-sm font-semibold tracking-wide text-sidebar-foreground"
      >satfolio</span
    >
    <span class="text-sm tracking-wide text-muted-foreground">/</span>
    <span class="text-sm tracking-wide text-muted-foreground">{title}</span>
  </div>
</header>
