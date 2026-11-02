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
const output_1 = require("./output");
const execFileAsync = (0, node_util_1.promisify)(node_child_process_1.execFile);
async function commandExists(command) {
    const checkCommand = process.platform === "win32" ? "where" : "which";
    try {
        await execFileAsync(checkCommand, [command]);
        return true;
    }
    catch {
        return false;
    }
}
class MarsKernelService {
    process = null;
    requestId = 0;
    cliPath;
    notebookPath = null;
    constructor(cliPath) {
        this.cliPath = cliPath;
    }
    async start(notebookPath = null) {
        if (this.process && this.notebookPath === notebookPath) {
            return;
        }
        if (this.process) {
            this.stop();
        }
        // Check if the command exists
        const exists = await commandExists(this.cliPath);
        if (!exists) {
            throw new Error(`Mars CLI not found at '${this.cliPath}'. Please install Mars CLI and ensure it's on your PATH.`);
        }
        const runtime = vscode.workspace
            .getConfiguration("mars")
            .get("runtime", "replay");
        const kernelArguments = ["kernel", "serve"];
        if (runtime === "evcxr") {
            kernelArguments.push("--runtime", "evcxr");
        }
        // Passing the notebook lets the CLI load its declared Cargo dependencies
        // and lockfile, so cells that use external crates compile in the editor.
        if (notebookPath) {
            kernelArguments.push("--notebook", notebookPath);
        }
        console.log(`Starting Mars kernel: ${this.cliPath} ${kernelArguments.join(" ")}`);
        this.process = (0, node_child_process_1.spawn)(this.cliPath, kernelArguments, {
            stdio: ["pipe", "pipe", "pipe"]
        });
        this.notebookPath = notebookPath;
        this.process.on("error", (error) => {
            console.error("Mars kernel process error:", error);
            this.process = null;
        });
        this.process.on("exit", (code) => {
            console.error(`Mars kernel process exited with code ${code}`);
            this.process = null;
        });
        // Wait a bit for the process to start
        await new Promise((resolve) => setTimeout(resolve, 500));
    }
    async execute(code, timeoutMs, notebookPath = null) {
        await this.start(notebookPath);
        const requestId = (++this.requestId).toString();
        const request = {
            id: requestId,
            request: {
                type: "execute",
                source: code,
                timeout_millis: timeoutMs,
            },
        };
        return new Promise((resolve, reject) => {
            if (!this.process?.stdout) {
                reject(new Error("Kernel process not available"));
                return;
            }
            let buffer = "";
            const onData = (chunk) => {
                buffer += chunk.toString();
                const lines = buffer.split("\n");
                buffer = lines.pop() || "";
                for (const line of lines) {
                    if (!line.trim())
                        continue;
                    try {
                        const response = JSON.parse(line);
                        if (response.id === requestId) {
                            this.process?.stdout?.removeListener("data", onData);
                            this.process?.stdout?.removeListener("error", onError);
                            if (response.response.type === "error") {
                                reject(new Error(response.response.message));
                                return;
                            }
                            if (response.response.type === "execution") {
                                const exec = response.response;
                                const failed = exec.outputs.some((output) => output.type === "error");
                                resolve({
                                    status: failed ? "error" : "success",
                                    execution_count: exec.execution_count,
                                    outputs: exec.outputs,
                                });
                            }
                        }
                    }
                    catch (e) {
                        // Invalid JSON, skip
                    }
                }
            };
            const onError = (error) => {
                this.process?.stdout?.removeListener("data", onData);
                this.process?.stdout?.removeListener("error", onError);
                reject(error);
            };
            this.process.stdout.on("data", onData);
            this.process.stdout.on("error", onError);
            // Send the request
            this.process.stdin?.write(JSON.stringify(request) + "\n");
        });
    }
    async reset() {
        if (!this.process) {
            await this.start(this.notebookPath);
        }
        const requestId = (++this.requestId).toString();
        const request = {
            id: requestId,
            request: { type: "reset" },
        };
        return new Promise((resolve, reject) => {
            if (!this.process?.stdout) {
                reject(new Error("Kernel process not available"));
                return;
            }
            let buffer = "";
            const onData = (chunk) => {
                buffer += chunk.toString();
                const lines = buffer.split("\n");
                buffer = lines.pop() || "";
                for (const line of lines) {
                    if (!line.trim())
                        continue;
                    try {
                        const response = JSON.parse(line);
                        if (response.id === requestId) {
                            this.process?.stdout?.removeListener("data", onData);
                            this.process?.stdout?.removeListener("error", onError);
                            if (response.response.type === "error") {
                                reject(new Error(response.response.message));
                            }
                            else {
                                resolve();
                            }
                        }
                    }
                    catch (e) {
                        // Invalid JSON, skip
                    }
                }
            };
            const onError = (error) => {
                this.process?.stdout?.removeListener("data", onData);
                this.process?.stdout?.removeListener("error", onError);
                reject(error);
            };
            this.process.stdout.on("data", onData);
            this.process.stdout.on("error", onError);
            this.process.stdin?.write(JSON.stringify(request) + "\n");
        });
    }
    stop() {
        if (this.process) {
            this.process.kill();
            this.process = null;
        }
    }
}
class MarsNotebookSerializer {
    async deserializeNotebook(content, _token) {
        const text = new TextDecoder().decode(content);
        let marsNotebook;
        try {
            marsNotebook = JSON.parse(text);
        }
        catch {
            // If parsing fails, create an empty notebook
            marsNotebook = {
                format_version: 2,
                metadata: { language: "rust" },
                cells: []
            };
        }
        // Convert Mars cells to VSCode notebook cells
        const cells = marsNotebook.cells.map((marsCell) => {
            const kind = marsCell.cell_type === "code"
                ? vscode.NotebookCellKind.Code
                : vscode.NotebookCellKind.Markup;
            const cell = new vscode.NotebookCellData(kind, marsCell.source, marsCell.cell_type === "code" ? "rust" : "markdown");
            // Preserve metadata (excluding execution_count which is handled separately)
            cell.metadata = {
                ...marsCell.metadata,
                marsId: marsCell.id
            };
            // Restore outputs if they exist
            if (marsCell.outputs && marsCell.outputs.length > 0) {
                cell.outputs = marsCell.outputs.map((output) => this.convertMarsOutput(output));
            }
            return cell;
        });
        const notebookData = new vscode.NotebookData(cells);
        notebookData.metadata = {
            ...marsNotebook.metadata,
            marsFormatVersion: marsNotebook.format_version
        };
        return notebookData;
    }
    async serializeNotebook(data, _token) {
        // Convert VSCode notebook cells back to Mars format
        const cells = data.cells.map((cell) => {
            const marsCell = {
                id: cell.metadata.marsId || this.generateId(),
                cell_type: cell.kind === vscode.NotebookCellKind.Code ? "code" : "markdown",
                source: cell.value,
                metadata: { ...cell.metadata },
                execution_count: null // VSCode handles execution counts separately
            };
            // Remove our internal metadata
            delete marsCell.metadata.marsId;
            // Convert outputs back to Mars format
            if (cell.outputs && cell.outputs.length > 0) {
                marsCell.outputs = cell.outputs.map((output) => this.convertNotebookOutput(output));
            }
            // Only code cells should have execution_count and outputs
            if (marsCell.cell_type !== "code") {
                delete marsCell.execution_count;
                delete marsCell.outputs;
            }
            return marsCell;
        });
        const marsNotebook = {
            format_version: data.metadata.marsFormatVersion || 2,
            metadata: {
                ...data.metadata,
                language: data.metadata.language || "rust"
            },
            cells
        };
        // Remove our internal metadata
        delete marsNotebook.metadata.marsFormatVersion;
        return new TextEncoder().encode(JSON.stringify(marsNotebook, null, 2));
    }
    convertMarsOutput(marsOutput) {
        return (0, output_1.marsOutputToNotebookOutput)(marsOutput);
    }
    convertNotebookOutput(output) {
        return (0, output_1.notebookOutputToMarsOutput)(output);
    }
    generateId() {
        return `cell-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
    }
}
class MarsNotebookController {
    _controller;
    _executionOrder = 0;
    kernelService = null;
    static instance = null;
    constructor() {
        MarsNotebookController.instance = this;
        this._controller = vscode.notebooks.createNotebookController("mars-kernel", "mars-notebook", "Mars Kernel");
        this._controller.supportedLanguages = ["rust"];
        this._controller.supportsExecutionOrder = true;
        this._controller.executeHandler = this._executeAll.bind(this);
    }
    dispose() {
        this._controller.dispose();
        this.kernelService?.stop();
        MarsNotebookController.instance = null;
    }
    async restartKernel() {
        if (this.kernelService) {
            await this.kernelService.reset();
        }
    }
    getKernelService() {
        if (!this.kernelService) {
            const cliPath = vscode.workspace
                .getConfiguration("mars")
                .get("cliPath", "mars");
            this.kernelService = new MarsKernelService(cliPath);
        }
        return this.kernelService;
    }
    async _executeAll(cells, notebook, _controller) {
        for (const cell of cells) {
            await this._executeCell(cell, notebook);
        }
    }
    async _executeCell(cell, notebook) {
        const exec = this._controller.createNotebookCellExecution(cell);
        this._executionOrder++;
        exec.executionOrder = this._executionOrder;
        exec.start(Date.now());
        let success = false;
        try {
            const code = cell.document.getText();
            const cliPath = vscode.workspace
                .getConfiguration("mars")
                .get("cliPath", "mars");
            // Check if Mars CLI is available
            const cliExists = await commandExists(cliPath);
            if (!cliExists) {
                // Fall back to placeholder execution if CLI is not available
                await exec.appendOutput(new vscode.NotebookCellOutput([
                    new vscode.NotebookCellOutputItem(Buffer.from(`Mars CLI not found at '${cliPath}'.\n\nInstall Mars CLI to enable actual code execution.\n\nFor now, showing placeholder output:\n\n${code}`, "utf-8"), "text/plain")
                ]));
                success = true;
            }
            else {
                const kernel = this.getKernelService();
                // Get timeout from configuration (default 30 seconds)
                const timeoutMs = vscode.workspace
                    .getConfiguration("mars")
                    .get("executionTimeout", 30000);
                // Add a message that execution is starting
                await exec.appendOutput(new vscode.NotebookCellOutput([
                    new vscode.NotebookCellOutputItem(Buffer.from("Compiling and executing Rust code (this may take a while)...\n", "utf-8"), "text/plain")
                ]));
                // Execute using the Mars kernel service with timeout
                const result = await Promise.race([
                    kernel.execute(code, timeoutMs, notebook?.uri.scheme === "file" ? notebook.uri.fsPath : null),
                    new Promise((_, reject) => setTimeout(() => reject(new Error(`Execution timeout after ${timeoutMs}ms. Try increasing mars.executionTimeout in settings.`)), timeoutMs))
                ]);
                // Clear previous outputs
                await exec.clearOutput();
                // Add new outputs
                for (const output of result.outputs) {
                    await exec.appendOutput((0, output_1.marsOutputToNotebookOutput)(output));
                }
                success = result.status === "success";
            }
        }
        catch (error) {
            const errorMessage = error instanceof Error ? error.message : String(error);
            await exec.clearOutput();
            await exec.appendOutput(new vscode.NotebookCellOutput([
                new vscode.NotebookCellOutputItem(Buffer.from(`Error: ${errorMessage}\n\nMake sure:\n1. Mars CLI is installed and on PATH\n2. 'mars kernel serve' command is available\n3. Rust toolchain is installed`, "utf-8"), "application/vnd.code.notebook.error")
            ]));
        }
        finally {
            // Always end execution
            exec.end(success, Date.now());
        }
    }
}
function activate(context) {
    // Register notebook serializer
    const serializer = new MarsNotebookSerializer();
    context.subscriptions.push(vscode.workspace.registerNotebookSerializer("mars-notebook", serializer));
    // Register notebook controller for execution
    const controller = new MarsNotebookController();
    context.subscriptions.push(controller);
    // Keep existing CLI commands for compatibility
    context.subscriptions.push(vscode.commands.registerCommand("mars.exportHtml", () => exportHtml()), vscode.commands.registerCommand("mars.exportMarkdown", () => exportMarkdown()), vscode.commands.registerCommand("mars.preview", () => exportHtml()), vscode.commands.registerCommand("mars.runNotebook", () => runNotebook()), vscode.commands.registerCommand("mars.openCell", () => openCell()), vscode.commands.registerCommand("mars.createNotebook", () => createNotebook()), vscode.commands.registerCommand("mars.restartKernel", () => {
        controller.restartKernel().then(() => {
            void vscode.window.showInformationMessage("Mars kernel restarted");
        });
    }));
}
async function createNotebook() {
    const notebookUri = vscode.Uri.parse("untitled:Untitled.mars");
    await vscode.commands.executeCommand("vscode.openWith", notebookUri, "mars-notebook");
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
function activeMarsFilePath() {
    const notebook = vscode.window.activeNotebookEditor?.notebook;
    if (notebook?.uri.scheme === "file" && notebook.uri.fsPath.endsWith(".mars")) {
        return notebook.uri.fsPath;
    }
    const document = vscode.window.activeTextEditor?.document;
    if (document?.fileName.endsWith(".mars"))
        return document.fileName;
    return undefined;
}
async function exportNotebook(format, extension) {
    const filePath = activeMarsFilePath();
    if (!filePath) {
        void vscode.window.showErrorMessage("Open a .mars notebook first.");
        return;
    }
    const output = filePath.replace(/\.mars$/i, `.${extension}`);
    try {
        const cliPath = vscode.workspace.getConfiguration("mars").get("cliPath", "mars");
        await execFileAsync(cliPath, ["export", filePath, format, output]);
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
async function exportHtml() {
    await exportNotebook("html", "html");
}
async function exportMarkdown() {
    await exportNotebook("md", "md");
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