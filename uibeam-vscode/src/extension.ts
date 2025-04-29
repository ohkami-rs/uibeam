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
    LinkedEditingRanges,
    Range,
} from 'vscode';
import {
    getLanguageService,
    TokenType,
    TextDocument as HTMLTextDocument,
    HTMLDocument,
} from 'vscode-html-languageservice';
import { activateAutoInsertion } from './auto_insertion';

export function activate(context: ExtensionContext) {
    const htmlLS = getLanguageService();

    const virtualDocuments = new Map<Uri, string>();

    const rustFilter: DocumentFilter = {
        scheme: 'file',
        language: 'rust',
        pattern: '**/*.rs',
    };

    const HTMLTextDocumentFrom = (document: TextDocument): HTMLTextDocument => {
        return {
            ...document,
            uri: document.uri.toString()
        }
    };

    const HTMLDocumentFrom = (document: TextDocument): HTMLDocument => {
        return htmlLS.parseHTMLDocument(HTMLTextDocumentFrom(document));
    };

    const isInsideRustRegion = (
        text: string,
        offset: number,
    ): boolean => {
        return false;
        // const s = htmlLS.createScanner(text);

        // let token = s.scan();
        // while (token !== TokenType.EOS) {
        //     if (s.getTokenOffset() <= offset && offset <= s.getTokenEnd()) {
        //         /** TODO: improve the logic */

        //         const text = s.getTokenText();
        //         return text.startsWith('{') && text.endsWith('}');
        //     }
        //     token = s.scan();
        // }

        // return false;
    };

    const virtualDocumentUriOf = (document: TextDocument, position: Position): Uri => {
        const text = document.getText();
        const insideRust = isInsideRustRegion(text, document.offsetAt(position));
        const content = insideRust ? text /* TODO */ : text;

        const originalUri = document.uri.toString(true/* skip encoding */);
        virtualDocuments.set(Uri.parse(originalUri), content);

        const virtualDocUri = Uri.parse(
            `embedded-content://rust/${encodeURIComponent(originalUri)}.${insideRust ? 'rs' : 'html'}`
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
        async provideCompletionItems(document, position, _callcellation_token, completion_context) {
            return await commands.executeCommand<CompletionList>(
                'vscode.executeCompletionItemProvider',
                virtualDocumentUriOf(document, position),
                position,
                completion_context.triggerCharacter
            );
        }
    }, '<', '>'));

    context.subscriptions.push(languages.registerDefinitionProvider(rustFilter, {
        async provideDefinition(document, position) {
            return await commands.executeCommand<Location[]>(
                'vscode.executeDefinitionProvider',
                virtualDocumentUriOf(document, position),
                position
            );
        }
    }));

    context.subscriptions.push(languages.registerLinkedEditingRangeProvider(rustFilter, {
        async provideLinkedEditingRanges(document, position, _token) {
            const text = document.getText();
            const insideRust = isInsideRustRegion(text, document.offsetAt(position));
            if (insideRust) return;

            const ranges = htmlLS.findLinkedEditingRanges(
                HTMLTextDocumentFrom(document),
                position,
                HTMLDocumentFrom(document),
            );
            if (!ranges) return;

            return new LinkedEditingRanges(ranges.map((r) => new Range(
                r.start.line,
                r.start.character,
                r.end.line,
                r.end.character
            )));
        } 
    }));

    context.subscriptions.push(activateAutoInsertion(rustFilter, async (kind, document, position) => {
        const insideRust = isInsideRustRegion(document.getText(), document.offsetAt(position));
        if (insideRust) return '';

        const htmlDocument = HTMLDocumentFrom(document);
        const textDocument = HTMLTextDocumentFrom(document);

        switch (kind) {
            case 'autoQuote':
                return htmlLS.doQuoteComplete(textDocument, position, htmlDocument) ?? '';
            case 'autoClose':
                return htmlLS.doTagComplete(textDocument, position, htmlDocument) ?? '';
            default:
                throw new Error(kind satisfies never);
        }
    }));
}
