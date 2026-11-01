import { useEffect, useState } from "react";

export type EditorPreferences = {
  autocomplete: boolean;
  folding: boolean;
};

const DEFAULT_FONT_SIZE = 14;
const DEFAULT_PREFERENCES: EditorPreferences = {
  autocomplete: true,
  folding: true,
};

function readFontSize() {
  const stored = window.localStorage.getItem("mars.editor.fontSize");
  const value = stored ? Number(stored) : DEFAULT_FONT_SIZE;
  return Number.isFinite(value) && value >= 10 && value <= 28 ? value : DEFAULT_FONT_SIZE;
}

function readPreferences(): EditorPreferences {
  try {
    const stored = JSON.parse(window.localStorage.getItem("mars.editor.preferences") ?? "{}") as Partial<EditorPreferences>;
    return {
      autocomplete: stored.autocomplete ?? DEFAULT_PREFERENCES.autocomplete,
      folding: stored.folding ?? DEFAULT_PREFERENCES.folding,
    };
  } catch {
    return DEFAULT_PREFERENCES;
  }
}

export function useEditorPreferences() {
  const [fontSize, setFontSize] = useState(readFontSize);
  const [preferences, setPreferences] = useState(readPreferences);

  useEffect(() => {
    window.localStorage.setItem("mars.editor.fontSize", String(fontSize));
  }, [fontSize]);

  useEffect(() => {
    window.localStorage.setItem("mars.editor.preferences", JSON.stringify(preferences));
  }, [preferences]);

  const updateFontSize = (value: number) => {
    if (Number.isFinite(value) && value >= 10 && value <= 28) setFontSize(value);
  };

  return {
    fontSize,
    updateFontSize,
    preferences,
    setPreferences,
  };
}
