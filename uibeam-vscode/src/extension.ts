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
    TextDocument as HTMLTextDocument,
} from 'vscode-html-languageservice';
import { ActivateAutoInsertion } from './auto_insertion';
import { findUIInputOffsets } from './find_ui';

function FindUIInputRanges(document: TextDocument): Range[] {
    if (!document.fileName.endsWith('.rs')) {
        return [];
    }

    return findUIInputOffsets(document.getText()).map(([start, end]) => new Range(
        document.positionAt(start),
        document.positionAt(end),
    ));
}

type VirtualDocument = {
    range: Range;
    htmlTextDocument: HTMLTextDocument;
};

export function activate(context: ExtensionContext) {
    const htmlLS = getLanguageService();

    const rustFilter: DocumentFilter = { scheme: 'file', pattern: '**/*.rs', language: 'rust' };

    const virtualDocuments = new Map<Uri, VirtualDocument[]>();

    const virtualDocumentsOf = (document: TextDocument): VirtualDocument[] => {
        if (!document.fileName.endsWith('.rs')) {
            throw new Error('Not a Rust file');
        }

        if (!virtualDocuments.has(document.uri)) {
            const text = document.getText();

            let vdocs: VirtualDocument[] = [];
            for (const range of FindUIInputRanges(document)) {
                const originalUri = document.uri.toString(true/* skip encoding */);
                const virtualUri = `${originalUri}-${range.start.line}:${range.start.character}-${range.end.line}:${range.end.character}`;
                const htmlTextDocument = HTMLTextDocument.create(
                    `embedded-content://html/${virtualUri}.html`,
                    'html',
                    0,
                    text.substring(document.offsetAt(range.start), document.offsetAt(range.end)),
                );
                vdocs.push({ range, htmlTextDocument });
            }

            virtualDocuments.set(document.uri, vdocs);
        }

        return virtualDocuments.get(document.uri)!;
    };

    const getVirtualDocumentAround = (
        position: Position,
        document: TextDocument,
    ): VirtualDocument | null => {
        const vdocs = virtualDocumentsOf(document);
        return vdocs.find((vdoc) => vdoc.range.contains(position)) ?? null;
    };

    context.subscriptions.push(workspace.registerTextDocumentContentProvider('embedded-content', {
        provideTextDocumentContent: (uri) => {
            const virtualDocumentUri = Uri.parse(decodeURIComponent(uri.path.slice(1, uri.path.lastIndexOf('.') - 1)));
            for (const vdocs of virtualDocuments.values()) {
                for (const vdoc of vdocs) {
                    if (vdoc.htmlTextDocument.uri.toString() === virtualDocumentUri.toString()) {
                        return vdoc.htmlTextDocument.getText();
                    }
                }
            }
            return '';
        }
    }));

    context.subscriptions.push(languages.registerHoverProvider(rustFilter, {
        async provideHover(document, position) {
            const vdocUri = getVirtualDocumentAround(position, document)?.htmlTextDocument?.uri;
            if (!vdocUri) {
                return null;
            }

            const hovers = await commands.executeCommand<Hover[]>(
                'vscode.executeHoverProvider',
                vdocUri,
                position
            );
            return (hovers?.length > 0) ? hovers[0] : null;
        }
    }));

    context.subscriptions.push(languages.registerCompletionItemProvider(rustFilter, {
        async provideCompletionItems(document, position, _callcellation_token, completion_context) {
            const vdocUri = getVirtualDocumentAround(position, document)?.htmlTextDocument?.uri;
            if (!vdocUri) {
                return null;
            }

            return await commands.executeCommand<CompletionList>(
                'vscode.executeCompletionItemProvider',
                vdocUri,
                position,
                completion_context.triggerCharacter
            );
        }
    }, '<', '>'));

    context.subscriptions.push(languages.registerDefinitionProvider(rustFilter, {
        async provideDefinition(document, position) {
            const vdocUri = getVirtualDocumentAround(position, document)?.htmlTextDocument?.uri;
            if (!vdocUri) {
                return null;
            }

            return await commands.executeCommand<Location[]>(
                'vscode.executeDefinitionProvider',
                vdocUri,
                position
            );
        }
    }));

    context.subscriptions.push(languages.registerLinkedEditingRangeProvider(rustFilter, {
        async provideLinkedEditingRanges(document, position, _token) {
            const vdoc = getVirtualDocumentAround(position, document);
            if (!vdoc) {
                return null;
            }

            const ranges = htmlLS.findLinkedEditingRanges(
                vdoc.htmlTextDocument,
                position,
                htmlLS.parseHTMLDocument(vdoc.htmlTextDocument),
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

    context.subscriptions.push(ActivateAutoInsertion(rustFilter, async (kind, document, position) => {
        const vdoc = getVirtualDocumentAround(position, document);
        if (!vdoc) {
            return '';
        }

        const textDocument = vdoc.htmlTextDocument;
        const htmlDocument = htmlLS.parseHTMLDocument(textDocument);

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
