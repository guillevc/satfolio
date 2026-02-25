import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";
import type { AssetAmount } from "./types/bindings";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

/** Convert AssetAmount decimal string to number for display formatting only.
 *  All financial math stays in Rust. */
export function displayAmount(a: AssetAmount): number {
  return parseFloat(a.amount);
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type WithoutChild<T> = T extends { child?: any } ? Omit<T, "child"> : T;
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type WithoutChildren<T> = T extends { children?: any }
  ? Omit<T, "children">
  : T;
export type WithoutChildrenOrChild<T> = WithoutChildren<WithoutChild<T>>;
export type WithElementRef<T, U extends HTMLElement = HTMLElement> = T & {
  ref?: U | null;
};
