import {
    commands,
    workspace,
    languages,
    DocumentFilter,
    ExtensionContext,
    Uri,
    TextDocument,
    Position,
    Hover,
    CompletionList,
    Location,
} from 'vscode';
import {
    getLanguageService,
} from 'vscode-html-languageservice';

export function activate(context: ExtensionContext) {
    const htmlLS = getLanguageService();

    const virtualDocuments = new Map<Uri, string>();

    const rustFilter: DocumentFilter = {
        scheme: 'file',
        language: 'rust',
        pattern: '**/*.rs',
    };

    const virtualDocumentUriOf = (d: TextDocument, p: Position): Uri => {
        const text = d.getText();
        const isRust = false; // TODO
        const content = isRust ? text /* TODO */ : text;

        const originalUri = d.uri.toString(true/* skip encoding */);
        virtualDocuments.set(Uri.parse(originalUri), content);

        const virtualDocUri = Uri.parse(
            `embedded-content://html/${encodeURIComponent(originalUri)}.${isRust ? 'rs' : 'html'}`
        );
        return virtualDocUri;
    };

    context.subscriptions.push(workspace.registerTextDocumentContentProvider('embedded-content', {
        provideTextDocumentContent: (uri) => {
            const originalUri = uri.path.slice(1).slice(0, uri.path.lastIndexOf('.') - 1);
            const decoded = decodeURIComponent(originalUri);
            return virtualDocuments.get(Uri.parse(decoded));
        }
    }));

    context.subscriptions.push(languages.registerHoverProvider(rustFilter, {
        async provideHover(document, position) {
            const hovers = await commands.executeCommand<Hover[]>(
                'vscode.executeHoverProvider',
                virtualDocumentUriOf(document, position),
                position
            );
            return (hovers?.length) ? hovers[0] : null;
        }
    }));

    context.subscriptions.push(languages.registerCompletionItemProvider(rustFilter, {
        async provideCompletionItems(d, p, _callcellation_token, completion_context) {
            return await commands.executeCommand<CompletionList>(
                'vscode.executeCompletionItemProvider',
                virtualDocumentUriOf(d, p),
                p,
                completion_context.triggerCharacter
            );
        }
    }, '<', '>'));

    context.subscriptions.push(languages.registerDefinitionProvider(rustFilter, {
        async provideDefinition(d, p) {
            return await commands.executeCommand<Location[]>(
                'vscode.executeDefinitionProvider',
                virtualDocumentUriOf(d, p),
                p
            );
        }
    }));

    context.subscriptions.push(languages.registerLinkedEditingRangeProvider());
}
