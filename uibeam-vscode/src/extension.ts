import {
    commands,
    workspace,
    DocumentFilter,
    ExtensionContext,
} from 'vscode';
import {
    getLanguageService,
} from 'vscode-html-languageservice';

export function activate(context: ExtensionContext) {
    const htmlLS = getLanguageService();

    const rust: DocumentFilter = {
        scheme: 'file',
        language: 'rust',
        pattern: '**/*.rs',
    };

    context.subscriptions.push(workspace.registerTextDocumentContentProvider(
        '',
        {
            provideTextDocumentContent(uri, token) {
                
            },
        }
    ));
}
