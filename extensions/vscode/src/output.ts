import * as vscode from 'vscode';

export type { MarsOutput } from './output/types';
export { marsOutputToNotebookOutput } from './output/encode';
export { notebookOutputToMarsOutput } from './output/decode';

export const appendOutput = (output: vscode.NotebookCellOutput): vscode.NotebookCellOutput => {
  return output;
};
