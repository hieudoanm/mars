import { activate } from './extension';

describe('extension', () => {
  it('has activate function', () => {
    expect(typeof activate).toBe('function');
  });
});
