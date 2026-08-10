const vscode = require('vscode');
const { existsSync, writeFileSync, unlinkSync, statSync } = require('fs');
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
    const existing = [];
    const addIfExist = (cand) => { if (cand && existsSync(cand)) existing.push(cand); };

    // 1. Check user configuration
    try {
        const configPath = vscode.workspace.getConfiguration('flame').get('compilerPath');
        if (configPath && typeof configPath === 'string' && configPath.trim() !== '') {
            if (existsSync(configPath)) return configPath;
        }
    } catch (e) { }

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
            for (const cand of candidates) addIfExist(cand);

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
        for (const cand of siblingCandidates) addIfExist(cand);
    }

    // 4. Check Cargo global binary directory
    const userHome = homedir();
    const cargoCandidates = [
        join(userHome, '.cargo', 'bin', 'flamelang.exe'),
        join(userHome, '.cargo', 'bin', 'flame.exe'),
        join(userHome, '.cargo', 'bin', 'flamelang'),
        join(userHome, '.cargo', 'bin', 'flame'),
    ];
    for (const cand of cargoCandidates) addIfExist(cand);

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
        for (const cand of npmCandidates) addIfExist(cand);
    } else {
        const unixNpmCandidates = [
            join(userHome, '.npm-global', 'bin', 'flame'),
            join(userHome, '.npm-global', 'bin', 'flamelang'),
            '/usr/local/bin/flame',
            '/usr/local/bin/flamelang',
            '/usr/bin/flame',
            '/usr/bin/flamelang',
        ];
        for (const cand of unixNpmCandidates) addIfExist(cand);
    }

    // 6. Check system PATH entries
    if (process.env.PATH) {
        const pathDirs = process.env.PATH.split(delimiter);
        const cliNames = process.platform === 'win32'
            ? ['flame.cmd', 'flame.exe', 'flame.bat', 'flame', 'flamelang.cmd', 'flamelang.exe', 'flamelang.bat', 'flamelang']
            : ['flame', 'flamelang'];

        for (const pDir of pathDirs) {
            for (const name of cliNames) {
                addIfExist(join(pDir, name));
            }
        }
    }

    // Return the newest binary by mtime timestamp so recent compiler rebuilds take precedence over older binaries
    if (existing.length > 0) {
        let newest = existing[0];
        let newestTime = 0;
        for (const cand of existing) {
            try {
                const mtime = statSync(cand).mtimeMs;
                if (mtime > newestTime) {
                    newestTime = mtime;
                    newest = cand;
                }
            } catch (e) { }
        }
        return newest;
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
    } catch (e) { }

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
            kind = vscode.CompletionItemKind.Keyword;
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
    if (entry.sortText) {
        item.sortText = entry.sortText;
    }

    // Fix annotation @@ issue by stripping @ from insertText
    if (entry.kind === 'annotation' && entry.label.startsWith('@')) {
        item.insertText = entry.label.substring(1);
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

    const tokenTypes = ['keyword', 'function', 'annotation', 'comment', 'string'];
    const tokenModifiers = ['declaration', 'readonly'];
    const legend = new vscode.SemanticTokensLegend(tokenTypes, tokenModifiers);

    function isIdentStart(ch) {
        return (ch >= 'a' && ch <= 'z') || (ch >= 'A' && ch <= 'Z') || ch === '_';
    }

    function isIdentPart(ch) {
        return isIdentStart(ch) || (ch >= '0' && ch <= '9');
    }

    context.subscriptions.push(
        vscode.languages.registerDocumentSemanticTokensProvider(['flame'], {
            provideDocumentSemanticTokens(document) {
                const tokensBuilder = new vscode.SemanticTokensBuilder(legend);

                const text = document.getText();
                const len = text.length;

                let i = 0;

                /*
                 * Semantic token types:
                 *
                 * 0 = keyword
                 * 1 = function
                 * 2 = annotation
                 * 3 = comment
                 * 4 = string
                 */

                const KEYWORD = 0;
                const FUNCTION = 1;
                const COMMENT = 3;
                const STRING = 4;

                const keywords = new Set([
                    'or',
                    'and',
                    'not',

                    'if',
                    'else',
                    'match',
                    'for',
                    'in',
                    'while',
                    'loop',

                    'break',
                    'continue',
                    'defer',
                    'return',
                    'yield',

                    'await',
                    'async',
                    'thread',

                    'let',
                    'const',
                    'var',

                    'fn',
                    'struct',
                    'enum',
                    'trait',
                    'impl',

                    'export',
                    'import',

                    'mut',
                    'as',
                    'type',
                    'where',

                    'formula',

                    'self',
                    'Self',

                    'true',
                    'false',
                    'nil',

                    'annotation'
                ]);

                function isIdentStart(ch) {
                    return (
                        (ch >= 'a' && ch <= 'z') ||
                        (ch >= 'A' && ch <= 'Z') ||
                        ch === '_'
                    );
                }

                function isIdentPart(ch) {
                    return (
                        isIdentStart(ch) ||
                        (ch >= '0' && ch <= '9')
                    );
                }

                function pushTokenRange(startIdx, endIdx, typeIdx) {
                    let curr = startIdx;

                    while (curr < endIdx) {
                        const nextLine = text.indexOf('\n', curr);

                        if (
                            nextLine === -1 ||
                            nextLine >= endIdx
                        ) {
                            let lineLen = endIdx - curr;

                            if (
                                lineLen > 0 &&
                                text[curr + lineLen - 1] === '\r'
                            ) {
                                lineLen--;
                            }

                            if (lineLen > 0) {
                                const pos = document.positionAt(curr);

                                tokensBuilder.push(
                                    pos.line,
                                    pos.character,
                                    lineLen,
                                    typeIdx,
                                    0
                                );
                            }

                            break;
                        }

                        let lineLen = nextLine - curr;

                        if (
                            lineLen > 0 &&
                            text[curr + lineLen - 1] === '\r'
                        ) {
                            lineLen--;
                        }

                        if (lineLen > 0) {
                            const pos = document.positionAt(curr);

                            tokensBuilder.push(
                                pos.line,
                                pos.character,
                                lineLen,
                                typeIdx,
                                0
                            );
                        }

                        curr = nextLine + 1;
                    }
                }

                /*
                 * Skip whitespace and return the next non-whitespace
                 * character.
                 */
                function skipWhitespace(pos) {
                    while (
                        pos < len &&
                        (
                            text[pos] === ' ' ||
                            text[pos] === '\t' ||
                            text[pos] === '\r' ||
                            text[pos] === '\n'
                        )
                    ) {
                        pos++;
                    }

                    return pos;
                }

                /*
                 * Reads an identifier starting at `start`.
                 */
                function readIdentifier(start) {
                    let end = start;

                    while (
                        end < len &&
                        isIdentPart(text[end])
                    ) {
                        end++;
                    }

                    return end;
                }

                /*
                 * ---------------------------------------------------------
                 * Main scanner
                 * ---------------------------------------------------------
                 */

                while (i < len) {
                    const ch = text[i];
                    const next = i + 1 < len
                        ? text[i + 1]
                        : '';

                    /*
                     * -----------------------------------------------------
                     * 1. Single-line comments
                     * -----------------------------------------------------
                     */

                    if (ch === '/' && next === '/') {
                        const start = i;

                        i += 2;

                        while (
                            i < len &&
                            text[i] !== '\n' &&
                            text[i] !== '\r'
                        ) {
                            i++;
                        }

                        pushTokenRange(
                            start,
                            i,
                            COMMENT
                        );

                        continue;
                    }

                    /*
                     * -----------------------------------------------------
                     * 2. Multi-line comments
                     * -----------------------------------------------------
                     */

                    if (ch === '/' && next === '*') {
                        const start = i;

                        i += 2;

                        while (i < len) {
                            if (
                                text[i] === '*' &&
                                i + 1 < len &&
                                text[i + 1] === '/'
                            ) {
                                i += 2;
                                break;
                            }

                            i++;
                        }

                        pushTokenRange(
                            start,
                            i,
                            COMMENT
                        );

                        continue;
                    }

                    /*
                     * -----------------------------------------------------
                     * 3. Interpolated strings
                     * -----------------------------------------------------
                     */

                    if (
                        ch === '$' &&
                        next === '"'
                    ) {
                        const start = i;

                        i += 2;

                        let segmentStart = start;

                        while (i < len) {
                            /*
                             * Escaped character.
                             */
                            if (text[i] === '\\') {
                                i += 2;
                                continue;
                            }

                            /*
                             * End of interpolated string.
                             */
                            if (text[i] === '"') {
                                i++;

                                pushTokenRange(
                                    segmentStart,
                                    i,
                                    STRING
                                );

                                break;
                            }

                            /*
                             * Template expression:
                             *
                             * ${ ... }
                             */
                            if (text[i] === '{') {
                                if (i > segmentStart) {
                                    pushTokenRange(
                                        segmentStart,
                                        i,
                                        STRING
                                    );
                                }

                                i++;

                                let braceDepth = 1;

                                while (
                                    i < len &&
                                    braceDepth > 0
                                ) {
                                    if (text[i] === '{') {
                                        braceDepth++;
                                        i++;
                                        continue;
                                    }

                                    if (text[i] === '}') {
                                        braceDepth--;

                                        if (braceDepth === 0) {
                                            i++;
                                            segmentStart = i;
                                            break;
                                        }

                                        i++;
                                        continue;
                                    }

                                    /*
                                     * Highlight keywords inside
                                     * interpolation expressions.
                                     */
                                    if (isIdentStart(text[i])) {
                                        const wordStart = i;
                                        const wordEnd =
                                            readIdentifier(i);

                                        const word =
                                            text.slice(
                                                wordStart,
                                                wordEnd
                                            );

                                        if (
                                            keywords.has(word)
                                        ) {
                                            const pos =
                                                document.positionAt(
                                                    wordStart
                                                );

                                            tokensBuilder.push(
                                                pos.line,
                                                pos.character,
                                                wordEnd - wordStart,
                                                KEYWORD,
                                                0
                                            );
                                        }

                                        i = wordEnd;
                                        continue;
                                    }

                                    i++;
                                }

                                continue;
                            }

                            i++;
                        }

                        continue;
                    }

                    /*
                     * -----------------------------------------------------
                     * 4. Double quoted strings
                     * -----------------------------------------------------
                     */

                    if (ch === '"') {
                        const start = i;

                        i++;

                        while (i < len) {
                            if (text[i] === '\\') {
                                i += 2;
                                continue;
                            }

                            if (
                                text[i] === '"' ||
                                text[i] === '\n' ||
                                text[i] === '\r'
                            ) {
                                if (text[i] === '"') {
                                    i++;
                                }

                                break;
                            }

                            i++;
                        }

                        pushTokenRange(
                            start,
                            i,
                            STRING
                        );

                        continue;
                    }

                    /*
                     * -----------------------------------------------------
                     * 5. Single quoted strings
                     * -----------------------------------------------------
                     */

                    if (ch === '\'') {
                        const start = i;

                        i++;

                        while (i < len) {
                            if (text[i] === '\\') {
                                i += 2;
                                continue;
                            }

                            if (
                                text[i] === '\'' ||
                                text[i] === '\n' ||
                                text[i] === '\r'
                            ) {
                                if (text[i] === '\'') {
                                    i++;
                                }

                                break;
                            }

                            i++;
                        }

                        pushTokenRange(
                            start,
                            i,
                            STRING
                        );

                        continue;
                    }

                    /*
                     * -----------------------------------------------------
                     * 6. @ annotations
                     *
                     * @native
                     * @native.test
                     * @std.test.foo
                     *
                     * Annotation names are KEYWORDS.
                     * They are NOT functions.
                     * -----------------------------------------------------
                     */

                    if (ch === '@') {
                        let cursor = i + 1;

                        /*
                         * @ must be followed by an identifier.
                         */
                        if (
                            cursor < len &&
                            isIdentStart(text[cursor])
                        ) {
                            /*
                             * First annotation component.
                             */
                            const firstStart = cursor;

                            cursor = readIdentifier(cursor);

                            const firstPos =
                                document.positionAt(
                                    firstStart
                                );

                            tokensBuilder.push(
                                firstPos.line,
                                firstPos.character,
                                cursor - firstStart,
                                KEYWORD,
                                0
                            );

                            /*
                             * Additional components:
                             *
                             * @native.test.foo
                             */
                            while (
                                cursor < len &&
                                text[cursor] === '.' &&
                                cursor + 1 < len &&
                                isIdentStart(text[cursor + 1])
                            ) {
                                cursor++;

                                const segmentStart =
                                    cursor;

                                cursor =
                                    readIdentifier(
                                        cursor
                                    );

                                const segmentPos =
                                    document.positionAt(
                                        segmentStart
                                    );

                                tokensBuilder.push(
                                    segmentPos.line,
                                    segmentPos.character,
                                    cursor - segmentStart,
                                    KEYWORD,
                                    0
                                );
                            }

                            i = cursor;

                            continue;
                        }

                        /*
                         * A standalone @.
                         */
                        i++;

                        continue;
                    }

                    /*
                     * -----------------------------------------------------
                     * 7. Identifier handling
                     * -----------------------------------------------------
                     */

                    if (isIdentStart(ch)) {
                        const start = i;
                        const end = readIdentifier(i);

                        const word = text.slice(
                            start,
                            end
                        );

                        /*
                         * -------------------------------------------------
                         * KEYWORDS
                         * -------------------------------------------------
                         */

                        if (keywords.has(word)) {
                            const pos =
                                document.positionAt(start);

                            tokensBuilder.push(
                                pos.line,
                                pos.character,
                                end - start,
                                KEYWORD,
                                0
                            );

                            /*
                             * -------------------------------------------------
                             * FUNCTION DECLARATION
                             * -------------------------------------------------
                             *
                             * Only:
                             *
                             *     fn name
                             *
                             * makes `name` a function.
                             *
                             * This is intentionally NOT:
                             *
                             *     identifier(
                             *
                             * because that would incorrectly color
                             * arbitrary identifiers/function calls.
                             */
                            if (word === 'fn') {
                                let cursor =
                                    skipWhitespace(end);

                                /*
                                 * Function name.
                                 */
                                if (
                                    cursor < len &&
                                    isIdentStart(text[cursor])
                                ) {
                                    const functionStart =
                                        cursor;

                                    cursor =
                                        readIdentifier(
                                            cursor
                                        );

                                    const functionPos =
                                        document.positionAt(
                                            functionStart
                                        );

                                    tokensBuilder.push(
                                        functionPos.line,
                                        functionPos.character,
                                        cursor - functionStart,
                                        FUNCTION,
                                        0
                                    );

                                    i = cursor;

                                    continue;
                                }
                            }

                            /*
                             * -------------------------------------------------
                             * ANNOTATION DECLARATION
                             * -------------------------------------------------
                             *
                             * annotation native
                             *
                             * `native` stays KEYWORD.
                             *
                             * It is NEVER FUNCTION.
                             */
                            if (word === 'annotation') {
                                let cursor =
                                    skipWhitespace(end);

                                if (
                                    cursor < len &&
                                    isIdentStart(text[cursor])
                                ) {
                                    const annotationStart =
                                        cursor;

                                    cursor =
                                        readIdentifier(
                                            cursor
                                        );

                                    const annotationPos =
                                        document.positionAt(
                                            annotationStart
                                        );

                                    tokensBuilder.push(
                                        annotationPos.line,
                                        annotationPos.character,
                                        cursor - annotationStart,
                                        KEYWORD,
                                        0
                                    );

                                    i = cursor;

                                    continue;
                                }
                            }

                            i = end;

                            continue;
                        }

                        /*
                         * -------------------------------------------------
                         * NORMAL IDENTIFIER
                         * -------------------------------------------------
                         *
                         * Do NOT mark it as a function just because
                         * it is followed by `(`.
                         *
                         * This is the critical part preventing things
                         * such as annotation names from becoming
                         * function-colored.
                         */
                        i = end;

                        continue;
                    }

                    /*
                     * Everything else.
                     */
                    i++;
                }

                return tokensBuilder.build();
            }
        }, legend)
    );

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
                    } catch (e) { }

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

function deactivate() { }

module.exports = {
    activate,
    deactivate,
};