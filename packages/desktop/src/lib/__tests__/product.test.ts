import { beforeEach, describe, expect, it } from 'vitest';
import { readProductView, writeProductView } from '../product';

describe('product view persistence', () => {
  beforeEach(() => window.localStorage.clear());

  it('defaults to welcome', () => {
    expect(readProductView()).toBe('welcome');
  });

  it('round-trips lab and notebook', () => {
    writeProductView('lab');
    expect(readProductView()).toBe('lab');
    writeProductView('notebook');
    expect(readProductView()).toBe('notebook');
  });

  it('ignores unknown stored values', () => {
    window.localStorage.setItem('mars.product', 'bogus');
    expect(readProductView()).toBe('welcome');
  });

  it('survives storage failures', () => {
    const spy = vi.spyOn(Storage.prototype, 'getItem').mockImplementation(() => {
      throw new Error('denied');
    });
    expect(readProductView()).toBe('welcome');
    spy.mockRestore();

    const setSpy = vi.spyOn(Storage.prototype, 'setItem').mockImplementation(() => {
      throw new Error('denied');
    });
    expect(() => writeProductView('lab')).not.toThrow();
    setSpy.mockRestore();
  });
});
