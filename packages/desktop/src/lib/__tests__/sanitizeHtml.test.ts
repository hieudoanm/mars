import { sanitizeHtml } from '../sanitizeHtml';

describe('sanitizeHtml', () => {
  it('allows basic tags', () => {
    const result = sanitizeHtml('<p>Hello</p>');
    expect(result).toContain('Hello');
  });

  it('removes script tags', () => {
    const result = sanitizeHtml('<script>alert(1)</script><p>hi</p>');
    expect(result).not.toContain('script');
    expect(result).toContain('hi');
  });

  it('removes script tags even when minified', () => {
    const result = sanitizeHtml('<script>alert(1)</script>');
    expect(result).not.toContain('script');
    expect(result).toBe('');
  });

  it('removes iframe tags', () => {
    const result = sanitizeHtml('<iframe src="evil"></iframe>');
    expect(result).not.toContain('iframe');
  });

  it('removes object tags', () => {
    const result = sanitizeHtml('<object data="evil"></object>');
    expect(result).not.toContain('object');
  });

  it('removes foreignObject tags', () => {
    const result = sanitizeHtml('<foreignObject><p>content</p></foreignObject>');
    expect(result).not.toContain('foreignObject');
  });

  it('removes form tags', () => {
    const result = sanitizeHtml('<form action="submit"><button>Submit</button></form>');
    expect(result).not.toContain('form');
    expect(result).not.toContain('action');
  });

  it('removes math tags', () => {
    const result = sanitizeHtml('<math>\\frac{1}{2}</math>');
    expect(result).not.toContain('math');
  });

  it('removes template tags', () => {
    const result = sanitizeHtml('<template><div>test</div></template>');
    expect(result).not.toContain('template');
  });

  it('allows svg and svg filters', () => {
    const result = sanitizeHtml('<svg><circle r="10"/></svg>');
    expect(result).toContain('svg');
  });

  it('forbids srcset attribute', () => {
    const result = sanitizeHtml('<img src="good.jpg" srcset="bad.jpg 2x" />');
    expect(result).toContain('good.jpg');
    expect(result).not.toContain('srcset');
  });

  it('forbids style attribute', () => {
    const result = sanitizeHtml('<p style="color: red;">text</p>');
    expect(result).not.toContain('style');
    expect(result).toContain('text');
  });

  it('strips data attributes', () => {
    const result = sanitizeHtml('<p data-foo="bar">text</p>');
    expect(result).not.toContain('data-foo');
    expect(result).toContain('text');
  });

  it('handles empty input', () => {
    const result = sanitizeHtml('');
    expect(result).toBe('');
  });

  it('preserves multiple allowed tags in nested structure', () => {
    const result = sanitizeHtml('<div><p>Hello <strong>world</strong></p></div>');
    expect(result).toContain('Hello');
    expect(result).toContain('world');
    expect(result).toContain('strong');
  });
});
