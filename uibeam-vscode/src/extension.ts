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
    content: string;
    uri: Uri
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
        
        const uri = Uri.parse(`embedded-content://html/${
            encodeURIComponent(document.uri.toString(true /* skip encoding */))
        }.html`);

        return { ranges, content, uri };
    }
};

export function activate(context: ExtensionContext) {
    const htmlLS = getLanguageService();

    const rustFilter: DocumentFilter = { scheme: 'file', pattern: '**/*.rs', language: 'rust' };

    const virtualHTMLDocuments = new Map<
        string/* Uri.toString(true) // need to be primitive type to avoid comparing object reference */,
        VirtualHTMLDocument
    >();

    const getVirtualHTMLDocumentOf = (document: TextDocument): VirtualHTMLDocument | null => {
        const originalUri = document.uri.toString(true /* skip encoding */);
        if (!virtualHTMLDocuments.has(originalUri)) {
            const vdoc = VirtualHTMLDocument.from(document);
            if (!vdoc) {
                return null;
            }
            virtualHTMLDocuments.set(originalUri, vdoc);
        }
        return virtualHTMLDocuments.get(originalUri)!;
    };

    context.subscriptions.push(workspace.registerTextDocumentContentProvider('embedded-content', {
        provideTextDocumentContent: (uri) => {
            const originalUri = decodeURIComponent(uri.path.substring(1, uri.path.lastIndexOf('.')));
            const vdoc = virtualHTMLDocuments.get(originalUri);
            return vdoc?.content ?? '';
        }
    }));

    context.subscriptions.push(workspace.onDidChangeTextDocument(event => {
        event.contentChanges.forEach(({ rangeOffset, rangeLength, range, text }) => {
            const changedUri = event.document.uri.toString(true /* skip encoding */);
            if (virtualHTMLDocuments.has(changedUri)) {
                // TODO: more efficient way to update
                const newVdoc = VirtualHTMLDocument.from(event.document);
                if (newVdoc) {
                    virtualHTMLDocuments.set(changedUri, newVdoc);
                }
            }
        });
    }));

    context.subscriptions.push(languages.registerHoverProvider(rustFilter, {
        async provideHover(document, position) {
            const vdoc = getVirtualHTMLDocumentOf(document);
            if (!vdoc) {
                return null;
            }
            if (!vdoc.ranges.some(r => r.contains(position))) {
                return null;
            }

            const hovers = await commands.executeCommand<Hover[]>(
                'vscode.executeHoverProvider',
                vdoc.uri,
                position
            );
            return (hovers?.length > 0) ? hovers[0] : null;
        }
    }));

    context.subscriptions.push(languages.registerCompletionItemProvider(rustFilter, {
        async provideCompletionItems(document, position, _callcellation_token, completion_context) {
            const vdoc = getVirtualHTMLDocumentOf(document);
            if (!vdoc) {
                return null;
            }
            if (!vdoc.ranges.some(r => r.contains(position))) {
                return null;
            }

            return await commands.executeCommand<CompletionList>(
                'vscode.executeCompletionItemProvider',
                vdoc.uri,
                position,
                completion_context.triggerCharacter
            );
        }
    }, '<', '>'));

    context.subscriptions.push(languages.registerDefinitionProvider(rustFilter, {
        async provideDefinition(document, position) {
            const vdoc = getVirtualHTMLDocumentOf(document);
            if (!vdoc) {
                return null;
            }
            if (!vdoc.ranges.some(r => r.contains(position))) {
                return null;
            }

            return await commands.executeCommand<Location[]>(
                'vscode.executeDefinitionProvider',
                vdoc.uri,
                position
            );
        }
    }));

    context.subscriptions.push(languages.registerLinkedEditingRangeProvider(rustFilter, {
        async provideLinkedEditingRanges(document, position, _token) {
            const vdoc = getVirtualHTMLDocumentOf(document);
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
        const vdoc = getVirtualHTMLDocumentOf(document);
        if (!vdoc) {
            return '';
        }
        if (!vdoc.ranges.some(r => r.contains(position))) {
            return '';
        }

        const textDocument = HTMLTextDocument.create(vdoc.uri.toString(true /* skip encoding */), 'html', 1, vdoc.content);
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
