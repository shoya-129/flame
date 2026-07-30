const vscode = require('vscode')
const fs = require('fs')
const path = require('path')
const child_process = require('child_process')

function findWorkspaceRoot(documentPath) {
    if (vscode.workspace.workspaceFolders && vscode.workspace.workspaceFolders.length > 0) {
        return vscode.workspace.workspaceFolders[0].uri.fsPath
    }
    return path.dirname(documentPath)
}

function findCompilerBinary(workspaceRoot) {
    // Local development: check for built binaries in the workspace
    if (workspaceRoot) {
        const candidates = [
            path.join(workspaceRoot, 'target', 'debug', 'wrenlang.exe'),
            path.join(workspaceRoot, 'target', 'release', 'wrenlang.exe')
        ]

        for (const candidate of candidates) {
            if (fs.existsSync(candidate)) return candidate
        }
    }

    return 'wren'
}

function execCompilerJson(args, cwd) {
    return new Promise((resolve) => {
        const compiler = findCompilerBinary(cwd)
        if (!compiler) {
            resolve(null)
            return
        }

        let options = { cwd }
        if (process.platform === 'win32' && compiler === 'wren') {
            options.shell = true
        }
        child_process.execFile(compiler, args, options, (error, stdout) => {
            if (error && !stdout) {
                resolve(null)
                return
            }

            try {
                resolve(JSON.parse(stdout))
            } catch (e) {
                resolve(null)
            }
        })
    })
}

async function runCheck(document, position) {
    const workspaceRoot = findWorkspaceRoot(document.uri.fsPath)
    
    // Write unsaved content to a temporary file so the compiler sees exactly what the user is typing
    const tempFilePath = document.uri.fsPath + '.wtmp'
    try {
        fs.writeFileSync(tempFilePath, document.getText())
    } catch (e) {
        return null
    }

    const args = ['check', tempFilePath, '--json']
    if (position) {
        args.push('--line', String(position.line + 1), '--col', String(position.character + 1))
    }
    
    const result = await execCompilerJson(args, workspaceRoot)
    
    // Clean up temporary file
    try {
        if (fs.existsSync(tempFilePath)) {
            fs.unlinkSync(tempFilePath)
        }
    } catch (e) {}

    return result
}

function toCompletionItem(entry) {
    const kind = entry.kind === 'plugin'
        ? vscode.CompletionItemKind.Module
        : entry.kind === 'module'
            ? vscode.CompletionItemKind.Module
            : vscode.CompletionItemKind.Function
    const item = new vscode.CompletionItem(entry.label, kind)
    item.detail = entry.detail || ''
    if (entry.documentation) {
        item.documentation = new vscode.MarkdownString(entry.documentation)
    }
    return item
}

function activate(context) {
    const diagnostics = vscode.languages.createDiagnosticCollection('wren')
    context.subscriptions.push(diagnostics)

    async function refreshDiagnostics(document) {
        if (!document || document.languageId !== 'wren') return
        const result = await runCheck(document)
        if (!result) return

        const mapped = (result.diagnostics || []).map((diag) => {
            const line = Math.max(0, (diag.line || 1) - 1)
            const col = Math.max(0, (diag.column || 1) - 1)
            const severity = diag.severity === 'warning'
                ? vscode.DiagnosticSeverity.Warning
                : diag.severity === 'info'
                    ? vscode.DiagnosticSeverity.Information
                    : vscode.DiagnosticSeverity.Error
            return new vscode.Diagnostic(
                new vscode.Range(line, col, line, col + 1),
                diag.message,
                severity
            )
        })

        diagnostics.set(document.uri, mapped)
    }

    context.subscriptions.push(
        vscode.workspace.onDidOpenTextDocument(refreshDiagnostics),
        vscode.workspace.onDidSaveTextDocument(refreshDiagnostics),
        vscode.workspace.onDidChangeTextDocument((event) => refreshDiagnostics(event.document))
    )

    if (vscode.window.activeTextEditor) {
        refreshDiagnostics(vscode.window.activeTextEditor.document)
    }

    context.subscriptions.push(vscode.languages.registerCompletionItemProvider('wren', {
        async provideCompletionItems(document, position) {
            const result = await runCheck(document, position)
            if (!result) return []
            return (result.completions || []).map(toCompletionItem)
        }
    }, '.', '@'))

    context.subscriptions.push(vscode.languages.registerHoverProvider('wren', {
        async provideHover(document, position) {
            const result = await runCheck(document, position)
            if (!result || !result.hover) return null

            const blocks = [new vscode.MarkdownString('`' + result.hover.label + '`')]
            if (result.hover.documentation) {
                blocks.push(new vscode.MarkdownString(result.hover.documentation))
            }
            return new vscode.Hover(blocks)
        }
    }))
}

function deactivate() {}

module.exports = {
    activate,
    deactivate,
}
