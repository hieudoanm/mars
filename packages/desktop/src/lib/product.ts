export type ProductView = 'welcome' | 'lab' | 'notebook';

const PRODUCT_KEY = 'mars.product';

export const readProductView = (): ProductView => {
  try {
    const raw = window.localStorage.getItem(PRODUCT_KEY);
    return raw === 'lab' || raw === 'notebook' ? raw : 'welcome';
  } catch {
    return 'welcome';
  }
};

export const writeProductView = (view: ProductView): void => {
  try {
    window.localStorage.setItem(PRODUCT_KEY, view);
  } catch {
    // Storage unavailable — the choice simply won't persist.
  }
};
