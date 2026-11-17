import { describe, expect, it } from 'vitest';
import { LANGUAGES, isRunnableLanguage, languageDescriptor } from '../languages';

describe('languages', () => {
  it('registers the expected languages', () => {
    const ids = LANGUAGES.map((language) => language.id);
    expect(ids).toEqual(['rust', 'python', 'r', 'julia', 'cpp', 'sql']);
  });

  it('marks rust, python, julia, r, cpp, and sql as runnable', () => {
    for (const id of ['rust', 'python', 'r', 'julia', 'cpp', 'sql']) {
      expect(isRunnableLanguage(id)).toBe(true);
    }
  });

  it('returns undefined for unknown languages', () => {
    expect(languageDescriptor('cobol')).toBeUndefined();
    expect(isRunnableLanguage('cobol')).toBe(false);
    expect(isRunnableLanguage('')).toBe(false);
  });

  it('exposes syntax extensions and completion for rust', () => {
    const rust = languageDescriptor('rust')!;
    expect(rust.syntax).toBeDefined();
    expect(rust.completionSource).toBeDefined();
    expect(rust.lsp).toBe(true);
    expect(rust.editorAssist).toBe(true);
  });

  it('builds syntax extensions for stream languages', () => {
    for (const id of ['python', 'r', 'julia', 'cpp', 'sql']) {
      const descriptor = languageDescriptor(id)!;
      expect(descriptor.syntax!()).toBeDefined();
    }
  });
});
