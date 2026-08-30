// Product/location lists and labels, fetched once from the backend so the
// frontend never hand-copies domain tables.
import { api, type CatalogItem, type Location, type Product } from "./api";

export const catalog = $state<{
  products: CatalogItem<Product>[];
  locations: CatalogItem<Location>[];
  ready: boolean;
}>({ products: [], locations: [], ready: false });

export async function loadCatalog(): Promise<void> {
  const c = await api.catalog();
  catalog.products = c.products;
  catalog.locations = c.locations;
  catalog.ready = true;
}

export function productLabel(key: Product): string {
  return catalog.products.find((p) => p.key === key)?.label ?? key;
}

export function locationLabel(key: Location): string {
  return catalog.locations.find((l) => l.key === key)?.label ?? key;
}
