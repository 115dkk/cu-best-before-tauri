// Screen state bound to browser history so the Android back gesture/button
// (which Tauri maps to WebView history) walks back through screens and closes
// the bottom sheet before leaving the app.

export type Screen = { name: "home" } | { name: "editor"; sheetId: string };

interface HistoryState {
  screen: Screen;
  modal: boolean;
}

export const nav = $state<{ screen: Screen; modal: boolean }>({
  screen: { name: "home" },
  modal: false,
});

function snapshot(): HistoryState {
  return { screen: $state.snapshot(nav.screen), modal: nav.modal };
}

export function initNav(): void {
  history.replaceState(snapshot(), "");
  window.addEventListener("popstate", (event) => {
    const state = event.state as HistoryState | null;
    nav.screen = state?.screen ?? { name: "home" };
    nav.modal = state?.modal ?? false;
  });
}

export function openEditor(sheetId: string): void {
  nav.screen = { name: "editor", sheetId };
  nav.modal = false;
  history.pushState(snapshot(), "");
}

export function goHome(): void {
  if (nav.modal) {
    // Pop the modal entry and the editor entry together.
    history.go(-2);
  } else if (nav.screen.name !== "home") {
    history.back();
  }
}

export function openModal(): void {
  if (nav.modal) return;
  nav.modal = true;
  history.pushState(snapshot(), "");
}

export function closeModal(): void {
  if (nav.modal) history.back();
}
