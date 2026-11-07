import { imageDataUri, isImageMime, mimeValueToString, selectMime } from '../mimeBundle';

describe('mimeValueToString', () => {
  it('returns string value as-is', () => {
    expect(mimeValueToString('hello')).toBe('hello');
  });

  it('joins string array parts', () => {
    expect(mimeValueToString(['line1', 'line2'])).toBe('line1line2');
  });

  it('returns null for non-string array elements', () => {
    expect(mimeValueToString(['ok', 1])).toBeNull();
  });

  it('returns null for numbers', () => {
    expect(mimeValueToString(42)).toBeNull();
  });

  it('returns null for null', () => {
    expect(mimeValueToString(null)).toBeNull();
  });

  it('returns null for objects', () => {
    expect(mimeValueToString({})).toBeNull();
  });

  it('returns empty string for empty array', () => {
    expect(mimeValueToString([])).toBe('');
  });
});

describe('isImageMime', () => {
  it('recognises png', () => {
    expect(isImageMime('image/png')).toBe(true);
  });

  it('recognises jpeg', () => {
    expect(isImageMime('image/jpeg')).toBe(true);
  });

  it('recognises svg+xml', () => {
    expect(isImageMime('image/svg+xml')).toBe(true);
  });

  it('recognises gif', () => {
    expect(isImageMime('image/gif')).toBe(true);
  });

  it('recognises webp', () => {
    expect(isImageMime('image/webp')).toBe(true);
  });

  it('rejects text/plain', () => {
    expect(isImageMime('text/plain')).toBe(false);
  });

  it('rejects text/html', () => {
    expect(isImageMime('text/html')).toBe(false);
  });
});

describe('selectMime', () => {
  it('prefers image over other mime types', () => {
    const data = {
      'text/plain': 'fallback',
      'image/png': 'iVBORw0KG',
    };
    const selected = selectMime(data);
    expect(selected).toEqual({
      kind: 'image',
      mimeType: 'image/png',
      value: 'iVBORw0KG',
    });
  });

  it('prefers svg over png when both present', () => {
    const data = {
      'image/png': 'png-data',
      'image/svg+xml': '<svg></svg>',
    };
    const selected = selectMime(data);
    expect(selected).toEqual({
      kind: 'image',
      mimeType: 'image/svg+xml',
      value: '<svg></svg>',
    });
  });

  it('selects chart spec from vega-lite mime type', () => {
    const spec = { mark: 'bar' };
    const selected = selectMime({
      'application/vnd.vega-lite.v5+json': spec,
    });
    expect(selected).toEqual({
      kind: 'chart',
      value: JSON.stringify(spec),
    });
  });

  it('stringifies chart spec object', () => {
    const spec = { data: [] };
    const selected = selectMime({
      'application/vnd.vega.v5+json': spec,
    });
    expect(selected).toEqual({
      kind: 'chart',
      value: JSON.stringify(spec),
    });
  });

  it('passes through string chart value', () => {
    const selected = selectMime({
      'application/vnd.vega-lite+json': '{"mark":"line"}',
    });
    expect(selected).toEqual({
      kind: 'chart',
      value: '{"mark":"line"}',
    });
  });

  it('selects html before latex', () => {
    const selected = selectMime({
      'text/latex': 'E = mc^2',
      'text/html': '<p>hi</p>',
    });
    expect(selected).toEqual({ kind: 'html', value: '<p>hi</p>' });
  });

  it('selects latex when no html', () => {
    const selected = selectMime({ 'text/latex': 'E = mc^2' });
    expect(selected).toEqual({ kind: 'latex', value: 'E = mc^2' });
  });

  it('selects markdown before plain text', () => {
    const selected = selectMime({
      'text/plain': 'plain',
      'text/markdown': '# Title',
    });
    expect(selected).toEqual({ kind: 'markdown', value: '# Title' });
  });

  it('selects plain text as fallback', () => {
    const selected = selectMime({ 'text/plain': 'hello' });
    expect(selected).toEqual({ kind: 'text', value: 'hello' });
  });

  it('returns null for empty bundle', () => {
    expect(selectMime({})).toBeNull();
  });

  it('returns null when all values are non-stringifiable', () => {
    expect(selectMime({ 'text/plain': 123 })).toBeNull();
  });
});

describe('imageDataUri', () => {
  it('passes through existing data URIs', () => {
    const uri = 'data:image/png;base64,iVBOR';
    expect(imageDataUri('image/png', uri)).toBe(uri);
  });

  it('wraps svg text in a base64 data URI', () => {
    const uri = imageDataUri('image/svg+xml', '<svg></svg>');
    expect(uri.startsWith('data:image/svg+xml;base64,')).toBe(true);
    expect(atob(uri.split(',')[1])).toBe('<svg></svg>');
  });

  it('wraps svg base64 payload as-is', () => {
    const uri = imageDataUri('image/svg+xml', 'PHN2Zz4=');
    expect(uri).toBe('data:image/svg+xml;base64,PHN2Zz4=');
  });

  it('wraps png base64 payload', () => {
    const uri = imageDataUri('image/png', 'iVBORw0KG');
    expect(uri).toBe('data:image/png;base64,iVBORw0KG');
  });
});
