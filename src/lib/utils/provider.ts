import type { Provider } from "$lib/types/bindings";

export const providerMeta: Record<
  Provider,
  { label: string; initial: string; classes: string }
> = {
  kraken: {
    label: "Kraken",
    initial: "K",
    classes: "bg-purple-500/20 text-purple-400 border border-purple-500/30",
  },
  coinbase: {
    label: "Coinbase",
    initial: "C",
    classes: "bg-blue-500/20 text-blue-400 border border-blue-500/30",
  },
};
