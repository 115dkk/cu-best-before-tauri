// Editing session for one sheet: owns the serial save pipeline so that rapid
// taps never race on a stale snapshot, and addresses entries by their slot
// (`at`), which stays stable across the backend's sort/merge normalization.
import { api, errorMessage, type EntryView, type Location, type Product, type SheetView } from "./api";
import { showToast } from "./toast.svelte";

/** A change applied to a fresh copy of the latest sheet right before saving. */
export type Mutation = (sheet: SheetView) => void;

export function addEntry(location: Location, product: Product, entry: EntryView): Mutation {
  return (sheet) => {
    sheet.sections[location][product].push(entry);
  };
}

export function replaceEntry(location: Location, product: Product, at: string, entry: EntryView): Mutation {
  return (sheet) => {
    const list = sheet.sections[location][product];
    const index = list.findIndex((e) => e.at === at);
    if (index === -1) list.push(entry);
    else list.splice(index, 1, entry);
  };
}

export function removeEntry(location: Location, product: Product, at: string): Mutation {
  return (sheet) => {
    const list = sheet.sections[location][product];
    const index = list.findIndex((e) => e.at === at);
    if (index !== -1) list.splice(index, 1);
  };
}

export class SheetSession {
  sheet = $state<SheetView | null>(null);
  saving = $state(false);
  private queue: Mutation[] = [];
  private flushing = false;

  async load(id: string): Promise<void> {
    this.sheet = await api.getSheet(id);
  }

  /** Queue a change; it is applied to the latest normalized sheet when its turn comes. */
  apply(mutation: Mutation): void {
    this.queue.push(mutation);
    void this.flush();
  }

  private async flush(): Promise<void> {
    if (this.flushing) return;
    this.flushing = true;
    this.saving = true;
    try {
      while (this.queue.length > 0 && this.sheet) {
        const batch = this.queue.splice(0);
        const next = $state.snapshot(this.sheet) as SheetView;
        for (const mutation of batch) mutation(next);
        try {
          this.sheet = await api.saveSheet(next);
        } catch (e) {
          showToast(errorMessage(e), "error");
        }
      }
    } finally {
      this.flushing = false;
      this.saving = false;
    }
  }
}
