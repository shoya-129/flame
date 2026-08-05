const vscode = require('vscode');
const { existsSync, writeFileSync, unlinkSync } = require('fs');
const { dirname, join, delimiter } = require('path');
const { homedir } = require('os');
const { execFile } = require('child_process');

function findWorkspaceRoot(documentPath) {
    if (vscode.workspace.workspaceFolders && vscode.workspace.workspaceFolders.length > 0) {
        return vscode.workspace.workspaceFolders[0].uri.fsPath;
    }
    return dirname(documentPath);
}

function findCompilerBinary(startPath) {
    // 1. Check user configuration
    try {
        const configPath = vscode.workspace.getConfiguration('flame').get('compilerPath');
        if (configPath && typeof configPath === 'string' && configPath.trim() !== '') {
            if (existsSync(configPath)) return configPath;
        }
    } catch (e) {}

    // 2. Check workspace directories and parents
    let curr = startPath;
    for (let i = 0; i < 8; i++) {
        if (curr) {
            const candidates = [
                join(curr, 'target', 'release', 'flamelang.exe'),
                join(curr, 'target', 'release', 'flame.exe'),
                join(curr, 'bin', 'flamelang.exe'),
                join(curr, 'bin', 'flame.exe'),
                join(curr, 'target', 'debug', 'flamelang.exe'),
                join(curr, 'target', 'debug', 'flame.exe'),
                join(curr, 'target', 'release', 'flamelang'),
                join(curr, 'target', 'release', 'flame'),
                join(curr, 'bin', 'flamelang'),
                join(curr, 'bin', 'flame'),
                join(curr, 'target', 'debug', 'flamelang'),
                join(curr, 'target', 'debug', 'flame'),
            ];

            for (const cand of candidates) {
                if (existsSync(cand)) return cand;
            }

            const parent = dirname(curr);
            if (parent === curr) break;
            curr = parent;
        } else {
            break;
        }
    }

    // 3. Check sibling flame repository / workspace directories
    if (startPath) {
        const parentDir = dirname(startPath);
        const siblingCandidates = [
            join(parentDir, 'flame', 'bin', 'flamelang.exe'),
            join(parentDir, 'flame', 'bin', 'flame.exe'),
            join(parentDir, 'flame', 'target', 'release', 'flamelang.exe'),
            join(parentDir, 'flame', 'target', 'release', 'flame.exe'),
            join(parentDir, 'flame', 'target', 'debug', 'flamelang.exe'),
            join(parentDir, 'flame', 'bin', 'flamelang'),
            join(parentDir, 'flame', 'bin', 'flame'),
            join(parentDir, 'flame', 'target', 'release', 'flamelang'),
            join(parentDir, 'flame', 'target', 'release', 'flame'),
        ];
        for (const cand of siblingCandidates) {
            if (existsSync(cand)) return cand;
        }
    }

    // 4. Check Cargo global binary directory
    const userHome = homedir();
    const cargoCandidates = [
        join(userHome, '.cargo', 'bin', 'flamelang.exe'),
        join(userHome, '.cargo', 'bin', 'flame.exe'),
        join(userHome, '.cargo', 'bin', 'flamelang'),
        join(userHome, '.cargo', 'bin', 'flame'),
    ];
    for (const cand of cargoCandidates) {
        if (existsSync(cand)) return cand;
    }

    // 5. Check NPM global directories (flame installed via npm)
    if (process.platform === 'win32') {
        const appData = process.env.APPDATA || '';
        const localAppData = process.env.LOCALAPPDATA || '';
        const npmCandidates = [
            join(appData, 'npm', 'flame.cmd'),
            join(appData, 'npm', 'flame.exe'),
            join(appData, 'npm', 'flame'),
            join(appData, 'npm', 'flamelang.cmd'),
            join(appData, 'npm', 'flamelang.exe'),
            join(appData, 'npm', 'flamelang'),
            join(localAppData, 'npm', 'flame.cmd'),
            join(localAppData, 'npm', 'flame.exe'),
            join(localAppData, 'npm', 'flame'),
        ];
        for (const cand of npmCandidates) {
            if (existsSync(cand)) return cand;
        }
    } else {
        const unixNpmCandidates = [
            join(userHome, '.npm-global', 'bin', 'flame'),
            join(userHome, '.npm-global', 'bin', 'flamelang'),
            '/usr/local/bin/flame',
            '/usr/local/bin/flamelang',
            '/usr/bin/flame',
            '/usr/bin/flamelang',
        ];
        for (const cand of unixNpmCandidates) {
            if (existsSync(cand)) return cand;
        }
    }

    // 6. Check system PATH entries
    if (process.env.PATH) {
        const pathDirs = process.env.PATH.split(delimiter);
        const cliNames = process.platform === 'win32'
            ? ['flame.cmd', 'flame.exe', 'flame.bat', 'flame', 'flamelang.cmd', 'flamelang.exe', 'flamelang.bat', 'flamelang']
            : ['flame', 'flamelang'];

        for (const pDir of pathDirs) {
            for (const name of cliNames) {
                const fullP = join(pDir, name);
                if (existsSync(fullP)) return fullP;
            }
        }
    }

    // Default fallback to 'flame' CLI (supports npm / cargo / system PATH via shell invocation)
    return 'flame';
}

function execCompilerJson(args, cwd) {
    return new Promise((resolve) => {
        const compiler = findCompilerBinary(cwd);
        if (!compiler) {
            resolve(null);
            return;
        }

        let options = { cwd, maxBuffer: 10 * 1024 * 1024 };
        if (process.platform === 'win32' || !compiler.includes('/') && !compiler.includes('\\')) {
            options.shell = true;
        }

        execFile(compiler, args, options, (error, stdout) => {
            if (error && !stdout) {
                resolve(null);
                return;
            }

            try {
                resolve(JSON.parse(stdout));
            } catch (e) {
                resolve(null);
            }
        });
    });
}

async function runCheck(document, position) {
    if (!document || document.uri.fsPath.endsWith('.fmi') || document.uri.fsPath.endsWith('.tmp.fm')) return null;
    const workspaceRoot = findWorkspaceRoot(document.uri.fsPath);

    // Write unsaved content to a temporary file so the compiler sees exactly what the user is typing
    const tempFilePath = document.uri.fsPath + '.tmp.fm';
    try {
        writeFileSync(tempFilePath, document.getText());
    } catch (e) {
        return null;
    }

    const args = ['check', tempFilePath, '--json'];
    if (position) {
        args.push('--line', String(position.line + 1), '--col', String(position.character + 1));
    }

    const result = await execCompilerJson(args, workspaceRoot);

    // Clean up temporary file
    try {
        if (existsSync(tempFilePath)) {
            unlinkSync(tempFilePath);
        }
    } catch (e) {}

    return result;
}

function toCompletionItem(entry) {
    let kind = vscode.CompletionItemKind.Function;
    switch (entry.kind) {
        case 'plugin':
        case 'module':
            kind = vscode.CompletionItemKind.Module;
            break;
        case 'keyword':
            kind = vscode.CompletionItemKind.Keyword;
            break;
        case 'annotation':
            kind = vscode.CompletionItemKind.Function;
            break;
        case 'struct':
            kind = vscode.CompletionItemKind.Struct;
            break;
        case 'property':
            kind = vscode.CompletionItemKind.Property;
            break;
        case 'method':
            kind = vscode.CompletionItemKind.Method;
            break;
        case 'variable':
            kind = vscode.CompletionItemKind.Variable;
            break;
        default:
            kind = vscode.CompletionItemKind.Function;
            break;
    }

    const item = new vscode.CompletionItem(entry.label, kind);
    item.detail = entry.detail || '';
    if (entry.documentation) {
        item.documentation = new vscode.MarkdownString(entry.documentation);
    }
    return item;
}

function activate(context) {
    const diagnostics = vscode.languages.createDiagnosticCollection('flame');
    context.subscriptions.push(diagnostics);

    async function refreshDiagnostics(document) {
        const supportedLanguages = ['flame'];
        if (!document || !supportedLanguages.includes(document.languageId) || document.uri.fsPath.endsWith('.fmi') || document.uri.fsPath.endsWith('.tmp.fm')) return;
        const result = await runCheck(document);
        if (!result) return;

        const mapped = (result.diagnostics || []).map((diag) => {
            const line = Math.max(0, (diag.line || 1) - 1);
            const col = Math.max(0, (diag.column || 1) - 1);
            const severity = diag.severity === 'warning'
                ? vscode.DiagnosticSeverity.Warning
                : diag.severity === 'info'
                    ? vscode.DiagnosticSeverity.Information
                    : vscode.DiagnosticSeverity.Error;
            return new vscode.Diagnostic(
                new vscode.Range(line, col, line, col + 1),
                diag.message,
                severity
            );
        });

        diagnostics.set(document.uri, mapped);
    }

    context.subscriptions.push(
        vscode.workspace.onDidOpenTextDocument(refreshDiagnostics),
        vscode.workspace.onDidSaveTextDocument(refreshDiagnostics),
        vscode.workspace.onDidChangeTextDocument((event) => refreshDiagnostics(event.document))
    );

    if (vscode.window.activeTextEditor) {
        refreshDiagnostics(vscode.window.activeTextEditor.document);
    }

    context.subscriptions.push(vscode.languages.registerCompletionItemProvider(['flame'], {
        async provideCompletionItems(document, position) {
            const result = await runCheck(document, position);
            if (!result) return [];
            return (result.completions || []).map(toCompletionItem);
        }
    }, '.', '@', ':'));

    const tokenTypes = ['keyword', 'function', 'annotation'];
    const tokenModifiers = ['declaration', 'readonly'];
    const legend = new vscode.SemanticTokensLegend(tokenTypes, tokenModifiers);

    function isIdentStart(ch) {
        return (ch >= 'a' && ch <= 'z') || (ch >= 'A' && ch <= 'Z') || ch === '_';
    }

    function isIdentPart(ch) {
        return isIdentStart(ch) || (ch >= '0' && ch <= '9');
    }

    context.subscriptions.push(vscode.languages.registerDocumentSemanticTokensProvider(['flame'], {
        provideDocumentSemanticTokens(document) {
            const tokensBuilder = new vscode.SemanticTokensBuilder(legend);
            const text = document.getText();
            const len = text.length;
            let i = 0;

            while (i < len) {
                const ch = text[i];
                const next = i + 1 < len ? text[i + 1] : '';

                // 1. Single-line comments (// ...)
                if (ch === '/' && next === '/') {
                    i += 2;
                    while (i < len && text[i] !== '\n') {
                        i++;
                    }
                    continue;
                }

                // 2. Multi-line comments (/* ... */)
                if (ch === '/' && next === '*') {
                    i += 2;
                    while (i < len) {
                        if (text[i] === '*' && i + 1 < len && text[i + 1] === '/') {
                            i += 2;
                            break;
                        }
                        i++;
                    }
                    continue;
                }

                // 3. Interpolated strings ($"...")
                if (ch === '$' && next === '"') {
                    i += 2;
                    let braceDepth = 0;
                    while (i < len) {
                        if (braceDepth === 0) {
                            if (text[i] === '\\') {
                                i += 2;
                                continue;
                            }
                            if (text[i] === '"') {
                                i++;
                                break;
                            }
                            if (text[i] === '{') {
                                braceDepth = 1;
                                i++;
                                continue;
                            }
                            i++;
                        } else {
                            // Inside interpolation expression { ... }
                            if (text[i] === '/' && i + 1 < len && text[i + 1] === '/') {
                                i += 2;
                                while (i < len && text[i] !== '\n') i++;
                                continue;
                            }
                            if (text[i] === '/' && i + 1 < len && text[i + 1] === '*') {
                                i += 2;
                                while (i < len) {
                                    if (text[i] === '*' && i + 1 < len && text[i + 1] === '/') {
                                        i += 2;
                                        break;
                                    }
                                    i++;
                                }
                                continue;
                            }
                            if (text[i] === '"') {
                                i++;
                                while (i < len) {
                                    if (text[i] === '\\') { i += 2; continue; }
                                    if (text[i] === '"' || text[i] === '\n') { if (text[i] === '"') i++; break; }
                                    i++;
                                }
                                continue;
                            }
                            if (text[i] === '\'') {
                                i++;
                                while (i < len) {
                                    if (text[i] === '\\') { i += 2; continue; }
                                    if (text[i] === '\'' || text[i] === '\n') { if (text[i] === '\'') i++; break; }
                                    i++;
                                }
                                continue;
                            }
                            if (text[i] === '{') {
                                braceDepth++;
                                i++;
                                continue;
                            }
                            if (text[i] === '}') {
                                braceDepth--;
                                i++;
                                continue;
                            }
                            i++;
                        }
                    }
                    continue;
                }

                // 4. Double quoted strings ("...")
                if (ch === '"') {
                    i++;
                    while (i < len) {
                        if (text[i] === '\\') {
                            i += 2;
                            continue;
                        }
                        if (text[i] === '"' || text[i] === '\n') {
                            if (text[i] === '"') i++;
                            break;
                        }
                        i++;
                    }
                    continue;
                }

                // 5. Single quoted strings ('...')
                if (ch === '\'') {
                    i++;
                    while (i < len) {
                        if (text[i] === '\\') {
                            i += 2;
                            continue;
                        }
                        if (text[i] === '\'' || text[i] === '\n') {
                            if (text[i] === '\'') i++;
                            break;
                        }
                        i++;
                    }
                    continue;
                }

                // 6. Annotation: @identifier (e.g. @test, @std.test)
                if (ch === '@') {
                    if (i + 1 < len && isIdentStart(text[i + 1])) {
                        let curr = i + 1;
                        while (curr < len && isIdentPart(text[curr])) {
                            curr++;
                        }
                        const pos = document.positionAt(i + 1);
                        tokensBuilder.push(pos.line, pos.character, curr - (i + 1), 2, 0);
                        i = curr;
                        while (i < len && text[i] === '.' && i + 1 < len && isIdentStart(text[i + 1])) {
                            const segStart = i + 1;
                            curr = segStart;
                            while (curr < len && isIdentPart(text[curr])) {
                                curr++;
                            }
                            const segPos = document.positionAt(segStart);
                            tokensBuilder.push(segPos.line, segPos.character, curr - segStart, 2, 0);
                            i = curr;
                        }
                        continue;
                    }
                    i++;
                    continue;
                }

                // 7. Keyword "annotation" declaration: annotation <name>
                if (text.startsWith('annotation', i)) {
                    const prevChar = i > 0 ? text[i - 1] : ' ';
                    const nextChar = i + 10 < len ? text[i + 10] : ' ';
                    if (!isIdentPart(prevChar) && !isIdentPart(nextChar)) {
                        const kwPos = document.positionAt(i);
                        tokensBuilder.push(kwPos.line, kwPos.character, 10, 0, 0);
                        i += 10;
                        let look = i;
                        while (look < len && (text[look] === ' ' || text[look] === '\t')) {
                            look++;
                        }
                        if (look < len && isIdentStart(text[look])) {
                            const nameStart = look;
                            while (look < len && isIdentPart(text[look])) {
                                look++;
                            }
                            const namePos = document.positionAt(nameStart);
                            tokensBuilder.push(namePos.line, namePos.character, look - nameStart, 1, 0);
                            i = look;
                        }
                        continue;
                    }
                }

                i++;
            }
            return tokensBuilder.build();
        }
    }, legend));

    context.subscriptions.push(vscode.languages.registerHoverProvider(['flame'], {
        async provideHover(document, position) {
            const result = await runCheck(document, position);
            if (!result || !result.hover) return null;

            const blocks = [];
            if (result.hover.documentation) {
                blocks.push(new vscode.MarkdownString(result.hover.documentation));
            } else if (result.hover.label) {
                blocks.push(new vscode.MarkdownString('`' + result.hover.label + '`'));
            }
            return new vscode.Hover(blocks);
        }
    }));

    context.subscriptions.push(vscode.languages.registerDocumentFormattingEditProvider(['flame'], {
        async provideDocumentFormattingEdits(document) {
            if (document.uri.fsPath.endsWith('.fmi') || document.uri.fsPath.endsWith('.tmp.fm')) return [];
            const workspaceRoot = findWorkspaceRoot(document.uri.fsPath);

            const tempFilePath = document.uri.fsPath + '.fmt.tmp.fm';
            try {
                writeFileSync(tempFilePath, document.getText());
            } catch (e) {
                return [];
            }

            const args = ['format', tempFilePath, '--stdout'];
            const compiler = findCompilerBinary(workspaceRoot);
            if (!compiler) return [];

            return new Promise((resolve) => {
                let options = { cwd: workspaceRoot, maxBuffer: 10 * 1024 * 1024 };
                if (process.platform === 'win32' || !compiler.includes('\\') && !compiler.includes('/')) {
                    options.shell = true;
                }
                execFile(compiler, args, options, (error, stdout) => {
                    try {
                        if (existsSync(tempFilePath)) {
                            unlinkSync(tempFilePath);
                        }
                    } catch (e) {}

                    if (error) {
                        resolve([]);
                        return;
                    }

                    if (stdout) {
                        const fullRange = new vscode.Range(
                            document.positionAt(0),
                            document.positionAt(document.getText().length)
                        );
                        resolve([vscode.TextEdit.replace(fullRange, stdout)]);
                    } else {
                        resolve([]);
                    }
                });
            });
        }
    }));
}

function deactivate() {}

module.exports = {
    activate,
    deactivate,
};
