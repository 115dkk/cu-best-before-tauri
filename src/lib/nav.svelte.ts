// Screen + slot-picker state bound to browser history (ADR-0007): the Android
// back gesture is mapped by Tauri to WebView history, so each pushed entry is
// one "back" step — the picker closes first, then the editor returns home.

import type { LocalDateTime, Product } from "./api";

export type Screen = { name: "home" } | { name: "editor"; sheetId: string };

/** What the slot picker is editing: a product, and the slot of an existing entry when editing. */
export interface PickerTarget {
  product: Product;
  at?: LocalDateTime;
}

interface HistoryState {
  screen: Screen;
  picker: PickerTarget | null;
}

export const nav = $state<{ screen: Screen; picker: PickerTarget | null }>({
  screen: { name: "home" },
  picker: null,
});

function snapshot(): HistoryState {
  return { screen: $state.snapshot(nav.screen), picker: $state.snapshot(nav.picker) };
}

export function initNav(): void {
  history.replaceState(snapshot(), "");
  window.addEventListener("popstate", (event) => {
    const state = event.state as HistoryState | null;
    nav.screen = state?.screen ?? { name: "home" };
    nav.picker = state?.picker ?? null;
  });
}

export function openEditor(sheetId: string): void {
  nav.screen = { name: "editor", sheetId };
  nav.picker = null;
  history.pushState(snapshot(), "");
}

export function goHome(): void {
  if (nav.picker) {
    // Picker entry sits directly above the editor entry.
    history.go(-2);
  } else if (nav.screen.name !== "home") {
    history.back();
  }
}

export function openPicker(target: PickerTarget): void {
  if (nav.picker) return;
  nav.picker = target;
  history.pushState(snapshot(), "");
}

export function closePicker(): void {
  if (nav.picker) history.back();
}
