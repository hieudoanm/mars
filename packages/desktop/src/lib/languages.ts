import type { Extension } from '@codemirror/state';
import type { CompletionSource } from '@codemirror/autocomplete';
import { rust } from '@codemirror/lang-rust';
import { python } from '@codemirror/lang-python';
import { StreamLanguage } from '@codemirror/language';
import { r as rMode } from '@codemirror/legacy-modes/mode/r';
import { julia as juliaMode } from '@codemirror/legacy-modes/mode/julia';
import { cpp as cppMode } from '@codemirror/legacy-modes/mode/clike';
import { sqlite as sqlMode } from '@codemirror/legacy-modes/mode/sql';
import { rustCompletionSource } from './rustCompletions';

/// Single source of truth for notebook languages. Mirrors the backend
/// registry in `crates/mars-kernel/src/runtime.rs` — keep the two in sync.
export interface LanguageDescriptor {
  id: string;
  label: string;
  /// Whether cells in this language can be executed.
  runnable: boolean;
  /// rust-analyzer LSP features (hover, diagnostics, goto-definition).
  lsp?: boolean;
  /// CodeMirror syntax mode; absent means plain text editing.
  syntax?: () => Extension;
  /// Bracket coloring, sticky scroll, and indentation markers.
  editorAssist?: boolean;
  /// Language-aware autocomplete source.
  completionSource?: CompletionSource;
}

export const LANGUAGES: readonly LanguageDescriptor[] = [
  {
    id: 'rust',
    label: 'Rust',
    runnable: true,
    lsp: true,
    syntax: rust,
    editorAssist: true,
    completionSource: rustCompletionSource,
  },
  {
    id: 'python',
    label: 'Python',
    runnable: true,
    syntax: python,
    editorAssist: true,
  },
  {
    id: 'r',
    label: 'R',
    runnable: true,
    syntax: () => StreamLanguage.define(rMode),
    editorAssist: true,
  },
  {
    id: 'julia',
    label: 'Julia',
    runnable: true,
    syntax: () => StreamLanguage.define(juliaMode),
    editorAssist: true,
  },
  {
    id: 'cpp',
    label: 'C++',
    runnable: true,
    syntax: () => StreamLanguage.define(cppMode),
  },
  {
    id: 'sql',
    label: 'SQL',
    runnable: true,
    syntax: () => StreamLanguage.define(sqlMode),
    editorAssist: true,
  },
];

export const languageDescriptor = (id: string): LanguageDescriptor | undefined =>
  LANGUAGES.find((language) => language.id === id);

export const isRunnableLanguage = (id: string): boolean =>
  languageDescriptor(id)?.runnable === true;
