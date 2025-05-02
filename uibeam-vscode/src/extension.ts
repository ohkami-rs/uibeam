import {
    commands,
    workspace,
    languages,
    DocumentFilter,
    ExtensionContext,
    Uri,
    TextDocument,
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
import { findUIInputRangeOffsets, clearExcludedFromRanges } from './lib';

type VirtualHTMLDocument = {
    ranges: Range[];
    htmlTextDocument: HTMLTextDocument;
};
const VirtualHTMLDocument = {
    from(document: TextDocument): VirtualHTMLDocument | null {
        if (!document.fileName.endsWith('.rs')) {
            return null;
        }

        const text = document.getText();

        const rangeOffsets = findUIInputRangeOffsets(text);
        if (rangeOffsets.length === 0) {
            return null;
        }

        const ranges = rangeOffsets.map(([start, end]) => new Range(
            document.positionAt(start),
            document.positionAt(end),
        ));

        // The same string as the original text, but with
        // non-UI input parts replaced with whitespaces.
        const content = clearExcludedFromRanges(text, rangeOffsets);
        
        const originalUri = document.uri.toString(true /* skip encoding */);
        const htmlTextDocument = HTMLTextDocument.create(
            `embedded-content://html/${encodeURIComponent(originalUri)}.html`,
            'html',
            0,
            content,
        );

        return { ranges, htmlTextDocument };
    }
};

export function activate(context: ExtensionContext) {
    const htmlLS = getLanguageService();

    const rustFilter: DocumentFilter = { scheme: 'file', pattern: '**/*.rs', language: 'rust' };

    const virtualHTMLDocuments = new Map<Uri, VirtualHTMLDocument>();

    context.subscriptions.push(workspace.registerTextDocumentContentProvider('embedded-content', {
        provideTextDocumentContent: (uri) => {
            const virtualDocumentUri = Uri.parse(decodeURIComponent(uri.path.slice(1, uri.path.lastIndexOf('.') - 1)));
            return virtualHTMLDocuments.get(virtualDocumentUri)?.htmlTextDocument.getText() ?? '';
        }
    }));

    context.subscriptions.push(languages.registerHoverProvider(rustFilter, {
        async provideHover(document, position) {
            const vdoc = virtualHTMLDocuments.get(document.uri);
            if (!vdoc) {
                return null;
            }
            if (!vdoc.ranges.some(r => r.contains(position))) {
                return null;
            }

            const hovers = await commands.executeCommand<Hover[]>(
                'vscode.executeHoverProvider',
                vdoc.htmlTextDocument.uri,
                position
            );
            return (hovers?.length > 0) ? hovers[0] : null;
        }
    }));

    context.subscriptions.push(languages.registerCompletionItemProvider(rustFilter, {
        async provideCompletionItems(document, position, _callcellation_token, completion_context) {
            const vdoc = virtualHTMLDocuments.get(document.uri);
            if (!vdoc) {
                return null;
            }
            if (!vdoc.ranges.some(r => r.contains(position))) {
                return null;
            }

            return await commands.executeCommand<CompletionList>(
                'vscode.executeCompletionItemProvider',
                vdoc.htmlTextDocument.uri,
                position,
                completion_context.triggerCharacter
            );
        }
    }, '<', '>'));

    context.subscriptions.push(languages.registerDefinitionProvider(rustFilter, {
        async provideDefinition(document, position) {
            const vdoc = virtualHTMLDocuments.get(document.uri);
            if (!vdoc) {
                return null;
            }
            if (!vdoc.ranges.some(r => r.contains(position))) {
                return null;
            }

            return await commands.executeCommand<Location[]>(
                'vscode.executeDefinitionProvider',
                vdoc.htmlTextDocument.uri,
                position
            );
        }
    }));

    context.subscriptions.push(languages.registerLinkedEditingRangeProvider(rustFilter, {
        async provideLinkedEditingRanges(document, position, _token) {
            const vdoc = virtualHTMLDocuments.get(document.uri);
            if (!vdoc) {
                return null;
            }
            if (!vdoc.ranges.some(r => r.contains(position))) {
                return null;
            }

            return new LinkedEditingRanges(vdoc.ranges);
        } 
    }));

    context.subscriptions.push(ActivateAutoInsertion(rustFilter, async (kind, document, position) => {
        const vdoc = virtualHTMLDocuments.get(document.uri);
        if (!vdoc) {
            return '';
        }
        if (!vdoc.ranges.some(r => r.contains(position))) {
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
