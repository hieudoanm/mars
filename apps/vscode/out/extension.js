"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.activate = activate;
exports.deactivate = deactivate;
const vscode = __importStar(require("vscode"));
const node_child_process_1 = require("node:child_process");
const node_util_1 = require("node:util");
const execFileAsync = (0, node_util_1.promisify)(node_child_process_1.execFile);
function activate(context) {
    context.subscriptions.push(vscode.commands.registerCommand("mars.exportHtml", () => exportHtml()), vscode.commands.registerCommand("mars.preview", () => exportHtml()), vscode.commands.registerCommand("mars.runNotebook", () => runNotebook()), vscode.commands.registerCommand("mars.openCell", () => openCell()));
}
async function runNotebook() {
    const document = vscode.window.activeTextEditor?.document;
    if (!document || !document.fileName.endsWith(".mars")) {
        void vscode.window.showErrorMessage("Open a .mars notebook first.");
        return;
    }
    const cliPath = vscode.workspace.getConfiguration("mars").get("cliPath", "mars");
    const runArguments = vscode.workspace
        .getConfiguration("mars")
        .get("runArguments", []);
    const output = vscode.window.createOutputChannel("Mars");
    output.show(true);
    output.appendLine(`$ ${cliPath} run ${document.fileName} ${runArguments.join(" ")}`.trim());
    try {
        const result = await execFileAsync(cliPath, ["run", document.fileName, ...runArguments]);
        if (result.stdout)
            output.append(result.stdout);
        if (result.stderr)
            output.append(result.stderr);
        void vscode.window.showInformationMessage("Mars notebook completed.");
    }
    catch (error) {
        const failure = error;
        if (failure.stdout)
            output.append(failure.stdout);
        if (failure.stderr)
            output.append(failure.stderr);
        void vscode.window.showErrorMessage(failure.message ?? "Mars notebook execution failed");
    }
}
async function exportHtml() {
    const document = vscode.window.activeTextEditor?.document;
    if (!document || !document.fileName.endsWith(".mars")) {
        void vscode.window.showErrorMessage("Open a .mars notebook first.");
        return;
    }
    const output = document.fileName.replace(/\.mars$/i, ".html");
    try {
        const cliPath = vscode.workspace.getConfiguration("mars").get("cliPath", "mars");
        await execFileAsync(cliPath, ["export", document.fileName, "html", output]);
        if (vscode.workspace.getConfiguration("mars").get("previewAfterExport", true)) {
            const uri = vscode.Uri.file(output);
            await vscode.commands.executeCommand("vscode.open", uri);
        }
        else {
            void vscode.window.showInformationMessage(`Exported Mars notebook to ${output}`);
        }
    }
    catch (error) {
        const message = error instanceof Error ? error.message : "Mars export failed";
        void vscode.window.showErrorMessage(message);
    }
}
async function openCell() {
    const editor = vscode.window.activeTextEditor;
    if (!editor)
        return;
    const input = await vscode.window.showInputBox({ prompt: "Cell number" });
    const cellNumber = Number(input);
    if (!Number.isInteger(cellNumber) || cellNumber < 1)
        return;
    const text = editor.document.getText();
    const matches = [...text.matchAll(/"cell_type"\s*:/g)];
    const match = matches[cellNumber - 1];
    if (!match || match.index === undefined) {
        void vscode.window.showWarningMessage(`Cell ${cellNumber} was not found.`);
        return;
    }
    const position = editor.document.positionAt(match.index);
    editor.selection = new vscode.Selection(position, position);
    editor.revealRange(new vscode.Range(position, position), vscode.TextEditorRevealType.InCenter);
}
function deactivate() { }
//# sourceMappingURL=extension.js.map