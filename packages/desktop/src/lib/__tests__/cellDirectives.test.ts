import { parseCellDirectives } from '../cellDirectives';

describe('parseCellDirectives', () => {
  it('parses // dep: and // deps: forms', () => {
    const directives = parseCellDirectives('// dep: serde@1.0\n// deps: tokio\nlet x = 1;');
    expect(directives.dependencies).toEqual([
      ['serde', '1.0'],
      ['tokio', '*'],
    ]);
  });

  it('parses // timeout:', () => {
    const directives = parseCellDirectives('// timeout: 30s\nloop {}');
    expect(directives.timeoutMillis).toBe(30000);
  });
});
