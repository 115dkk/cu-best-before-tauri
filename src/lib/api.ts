// Thin, typed wrappers over the Tauri commands defined in
// src-tauri/src/commands.rs. Shapes mirror docs/BACKEND-CONTRACT.md exactly.
// Every display string (labels) comes from the backend view layer; nothing is
// computed here.
import { invoke } from "@tauri-apps/api/core";

export type Product = "onigiri" | "gimbap" | "lunchbox" | "sandwich" | "burger";
export type Location = "store" | "walk_in";

/** ISO local datetime without zone, e.g. "2026-08-30T14:00:00". */
export type LocalDateTime = string;

export interface CatalogItem<K extends string> {
  key: K;
  label: string;
}

/** Products and locations in display order, with their Korean labels. */
export interface Catalog {
  products: CatalogItem<Product>[];
  locations: CatalogItem<Location>[];
}

export interface EntryView {
  at: LocalDateTime;
  quantity: number;
  /** "8/30 14시" — identical to the text in the exported PNG. */
  label: string;
}

export type SectionView = Record<Product, EntryView[]>;

/** Sheet as returned by the backend; can be sent back verbatim to `save_sheet`. */
export interface SheetView {
  id: string;
  created_at: LocalDateTime;
  /** "8/30 (일) 오전 8:02" */
  created_label: string;
  updated_at: LocalDateTime;
  sections: Record<Location, SectionView>;
}

export interface SheetSummary {
  id: string;
  created_at: LocalDateTime;
  created_label: string;
  updated_at: LocalDateTime;
  entry_count: number;
  total_quantity: number;
}

export interface TimeOption {
  at: LocalDateTime;
  hour: number;
  label: string;
}

export interface DateOption {
  /** ISO date, e.g. "2026-08-30". */
  date: string;
  label: string;
  /** True only for a date inserted to keep an already-past entry editable. */
  past: boolean;
  times: TimeOption[];
}

export interface SlotOptions {
  product: Product;
  dates: DateOption[];
}

export interface ExportResult {
  path: string;
  file_name: string;
  bytes: number;
}

export const api = {
  catalog: () => invoke<Catalog>("catalog"),
  listSheets: () => invoke<SheetSummary[]>("list_sheets"),
  createSheet: () => invoke<SheetView>("create_sheet"),
  getSheet: (id: string) => invoke<SheetView>("get_sheet", { id }),
  /** Returns the normalized sheet (sorted, duplicates merged, updated_at bumped). */
  saveSheet: (sheet: SheetView) => invoke<SheetView>("save_sheet", { sheet }),
  deleteSheet: (id: string) => invoke<void>("delete_sheet", { id }),
  /** `include` keeps the slot of an entry being edited selectable even if it is past. */
  slotOptions: (product: Product, include?: LocalDateTime) =>
    invoke<SlotOptions>("slot_options", { product, include: include ?? null }),
  exportSheet: (id: string) => invoke<ExportResult>("export_sheet", { id }),
};

/** Commands reject with a plain string (Display of core::Error). */
export function errorMessage(e: unknown): string {
  if (typeof e === "string") return e;
  if (e instanceof Error) return e.message;
  return String(e);
}
