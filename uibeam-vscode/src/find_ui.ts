import {
    commands,
    ExtensionContext,
    Position,
    Range,
    TextDocument,
    TextEditor,
    Uri,
    ViewColumn,
    window,
    workspace,
} from 'vscode';

export function FindUIInputRanges(document: TextDocument): Range[] {
    if (!document.fileName.endsWith('.rs')) {
        return [];
    }

    const text = document.getText();
    
    let UIs: Range[] = [];
    {
        let UICall: RegExpExecArray | null;
        while ((UICall = /UI!\s*{/g.exec(text)) !== null) {
            // index of the first charactor in `UI!{}`
            const start = UICall.index + UICall[0].length;
            if (start >= text.length) {
                break;
            }

            // index of the last charactor in `UI!{}`
            let end = start;
            {
                let depth = 1;
                while (depth > 0 && (end + 1) < text.length) {
                    switch (text[(end + 1)]) {
                        case '}':
                            depth -= 1;
                            break;
                        case '{':
                            depth += 1;
                            break;
                    }
                    end += 1;
                }
                if (depth !== 0) {
                    // Unmatched braces, skip this UI
                    break;
                }
            }

            UIs.push(new Range(
                document.positionAt(start),
                document.positionAt(end)
            ));
        }
    }

    return UIs;
}
