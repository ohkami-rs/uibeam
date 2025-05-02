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

import { writeFileSync } from 'node:fs';
function DEBUG(message: string, panic: boolean = false) {
    console.error(`[uibeam-vscode] ${message}`);
    writeFileSync(
        `${__dirname}/../log.txt`,
        `[uibeam-vscode] ${message}`
    );
    if (panic) {
        throw new Error(message);
    }
}

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
    DEBUG(`[activate]`);

    const htmlLS = getLanguageService();

    const rustFilter: DocumentFilter = { scheme: 'file', pattern: '**/*.rs', language: 'rust' };

    const virtualHTMLDocuments = new Map<
        string/* Uri.toString(true) // need to be a primitive type */,
        VirtualHTMLDocument
    >();

    const getVirtualHTMLDocumentOf = (document: TextDocument): VirtualHTMLDocument | null => {
        const originalUri = document.uri.toString(true /* skip encoding */);
        if (!virtualHTMLDocuments.has(originalUri)) {
            const vdoc = VirtualHTMLDocument.from(document);
            if (!vdoc) {DEBUG(`vdoc is null`, true);
                return null;
            }
            virtualHTMLDocuments.set(originalUri, vdoc);
        }
        return virtualHTMLDocuments.get(originalUri)!;
    };

    context.subscriptions.push(workspace.registerTextDocumentContentProvider('embedded-content', {
        provideTextDocumentContent: (uri) => {
            DEBUG(`[provideTextDocumentContent] uri: ${uri}`);
            const originalUri = decodeURIComponent(uri.path.substring(1, uri.path.lastIndexOf('.')));
            DEBUG(`[provideTextDocumentContent] originalUri: ${originalUri}`);
            const vdoc = virtualHTMLDocuments.get(originalUri);
            DEBUG(`\
                [provideTextDocumentContent] vdoc: ${JSON.stringify(vdoc)} \
                for originalUri: '${originalUri}' \
                where the map: ${JSON.stringify(virtualHTMLDocuments)} \
            `);
            return vdoc?.htmlTextDocument?.getText() ?? '';
        }
    }));

    context.subscriptions.push(languages.registerHoverProvider(rustFilter, {
        async provideHover(document, position) {
            DEBUG(`[provideHover] document: ${document.uri}, position: ${JSON.stringify(position)}`);
            const vdoc = getVirtualHTMLDocumentOf(document);
            if (!vdoc) {DEBUG(`vdoc is null`, true);
                return null;
            }
            if (!vdoc.ranges.some(r => r.contains(position))) {
                return null;
            }
            DEBUG(`[provideHover] vdoc: ${JSON.stringify(vdoc)}`);

            const hovers = await commands.executeCommand<Hover[]>(
                'vscode.executeHoverProvider',
                Uri.parse(vdoc.htmlTextDocument.uri),
                position
            );
            return (hovers?.length > 0) ? hovers[0] : null;
        }
    }));

    context.subscriptions.push(languages.registerCompletionItemProvider(rustFilter, {
        async provideCompletionItems(document, position, _callcellation_token, completion_context) {
            DEBUG(`[provideHover] document: ${document.uri}, position: ${JSON.stringify(position)}`);
            const vdoc = getVirtualHTMLDocumentOf(document);
            if (!vdoc) {DEBUG(`vdoc is null`, true);
                return null;
            }
            if (!vdoc.ranges.some(r => r.contains(position))) {
                return null;
            }
            DEBUG(`[provideHover] vdoc: ${JSON.stringify(vdoc)}`);

            return await commands.executeCommand<CompletionList>(
                'vscode.executeCompletionItemProvider',
                Uri.parse(vdoc.htmlTextDocument.uri),
                position,
                completion_context.triggerCharacter
            );
        }
    }, '<', '>'));

    context.subscriptions.push(languages.registerDefinitionProvider(rustFilter, {
        async provideDefinition(document, position) {
            DEBUG(`[provideHover] document: ${document.uri}, position: ${JSON.stringify(position)}`);
            const vdoc = getVirtualHTMLDocumentOf(document);
            if (!vdoc) {DEBUG(`vdoc is null`, true);
                return null;
            }
            if (!vdoc.ranges.some(r => r.contains(position))) {
                return null;
            }
            DEBUG(`[provideHover] vdoc: ${JSON.stringify(vdoc)}`);

            return await commands.executeCommand<Location[]>(
                'vscode.executeDefinitionProvider',
                Uri.parse(vdoc.htmlTextDocument.uri),
                position
            );
        }
    }));

    context.subscriptions.push(languages.registerLinkedEditingRangeProvider(rustFilter, {
        async provideLinkedEditingRanges(document, position, _token) {
            DEBUG(`[provideHover] document: ${document.uri}, position: ${JSON.stringify(position)}`);
            const vdoc = getVirtualHTMLDocumentOf(document);
            if (!vdoc) {DEBUG(`vdoc is null`, true);
                return null;
            }
            if (!vdoc.ranges.some(r => r.contains(position))) {
                return null;
            }
            DEBUG(`[provideHover] vdoc: ${JSON.stringify(vdoc)}`);

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

    DEBUG(`[activateed]`);
}
