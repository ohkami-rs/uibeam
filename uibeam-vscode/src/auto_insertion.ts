// Original source: https://github.com/microsoft/vscode/blob/f0cdaab75f07e11102f3cbd85f47552780c18e84/extensions/html-language-features/client/src/autoInsertion.ts

/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Microsoft Corporation. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

import { window, workspace, Disposable, TextDocument, Position, SnippetString, TextDocumentChangeEvent, TextDocumentChangeReason, TextDocumentContentChangeEvent, languages, DocumentFilter } from 'vscode';

export function activateAutoInsertion(provider: (kind: 'autoQuote' | 'autoClose', document: TextDocument, position: Position) => Thenable<string>, documentSelector: DocumentFilter): Disposable {
  const disposables: Disposable[] = [];
  workspace.onDidChangeTextDocument(onDidChangeTextDocument, null, disposables);

  let anyIsEnabled = false;
  const isEnabled = {
    'autoQuote': false,
    'autoClose': false
  };
  updateEnabledState();
  window.onDidChangeActiveTextEditor(updateEnabledState, null, disposables);

  let timeout: NodeJS.Timeout | undefined = undefined;

  disposables.push({
    dispose: () => {
      clearTimeout(timeout);
    }
  });

  function updateEnabledState() {
    anyIsEnabled = false;
    const editor = window.activeTextEditor;
    if (!editor) {
      return;
    }
    const document = editor.document;
    if (languages.match(documentSelector, document) === 0) {
      return;
    }

    const configurations = workspace.getConfiguration(undefined, document.uri);
    isEnabled['autoQuote'] = configurations.get<boolean>('html.autoCreateQuotes') ?? false;
    isEnabled['autoClose'] = configurations.get<boolean>('html.autoClosingTags') ?? false;
    anyIsEnabled = isEnabled['autoQuote'] || isEnabled['autoClose'];
  }

  function onDidChangeTextDocument({ document, contentChanges, reason }: TextDocumentChangeEvent) {
    if (!anyIsEnabled || contentChanges.length === 0 || reason === TextDocumentChangeReason.Undo || reason === TextDocumentChangeReason.Redo) {
      return;
    }
    const activeDocument = window.activeTextEditor && window.activeTextEditor.document;
    if (document !== activeDocument) {
      return;
    }
    if (timeout) {
      clearTimeout(timeout);
    }

    const lastChange = contentChanges[contentChanges.length - 1];
    const lastCharacter = lastChange.text[lastChange.text.length - 1];
    if (isEnabled['autoQuote'] && lastChange.rangeLength === 0 && lastCharacter === '=') {
      doAutoInsert('autoQuote', document, lastChange);
    } else if (isEnabled['autoClose'] && lastChange.rangeLength === 0 && (lastCharacter === '>' || lastCharacter === '/')) {
      doAutoInsert('autoClose', document, lastChange);
    }
  }

  function doAutoInsert(kind: 'autoQuote' | 'autoClose', document: TextDocument, lastChange: TextDocumentContentChangeEvent) {
    const rangeStart = lastChange.range.start;
    const version = document.version;
    timeout = setTimeout(() => {
      const position = new Position(rangeStart.line, rangeStart.character + lastChange.text.length);
      provider(kind, document, position).then(text => {
        // FIXME: Temporary workaround for paste malfunction
        if(kind === 'autoClose' && lastChange.text.trim().endsWith(text.substring(2))) {
          return
        }

        if (text && isEnabled[kind]) {
          const activeEditor = window.activeTextEditor;
          if (activeEditor) {
            const activeDocument = activeEditor.document;
            if (document === activeDocument && activeDocument.version === version) {
              const selections = activeEditor.selections;
              if (selections.length && selections.some(s => s.active.isEqual(position))) {
                activeEditor.insertSnippet(new SnippetString(text), selections.map(s => s.active));
              } else {
                activeEditor.insertSnippet(new SnippetString(text), position);
              }
            }
          }
        }
      });
      timeout = undefined;
    }, 100);
  }
  return Disposable.from(...disposables);
}
