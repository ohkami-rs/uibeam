// pure utility logics for this extension, not depending on vscode

export function findUIInputRangeOffsets(text: string): [number, number][] {
    let ranges: [number, number][] = [];
    {
        for (const match of text.matchAll(/UI!\s*({|\(|\[)/g)) {
            console.debug('[match]', match);

            // index of the first charactor in `UI!{}`
            const start = match.index! + match[0].length;
            if (start >= text.length) {
                break;
            }

            console.debug(`found UI input at ${start}: '${match[0]}'`);

            // index of the last charactor in `UI!{}`
            let end = start;
            {
                let [OPEN, CLOSE] = match[0].endsWith('{') ?
                    ['{', '}'] : match[0].endsWith('(') ?
                    ['(', ')'] :
                    ['[', ']']
                ;

                let depth = 1;
                while (end < text.length) {
                    console.debug(`depth: ${depth}, end: ${end}, char: '${text[end]}'`);
                    if (text[end] === CLOSE) {
                        depth -= 1;
                    } else if (text[end] === OPEN) {
                        depth += 1;
                    }
                    
                    if (depth === 0) {
                        break;
                    } else {
                        end += 1;
                    }
                }
                if (depth > 0) {
                    console.debug(`not found matching '}' for UI input at ${start}`);
                    continue;
                }
            }

            ranges.push([start, end]);
            console.debug(`pushed input ${start}:${end}: '${text.substring(start, end)}'`);
        }
    }

    return ranges;
}

// This function clears its parts not included in the ranges as whitespaces.
export function clearExcludedFromRanges(
    text: string,
    ranges: [number, number][],
): string {
    let buf = "";

    const mutate_buf = {
        copyFromOriginalText: (from: number, to: number) => {
            buf += text.substring(from, to);
        },
        fillWithWhiteSpace: (from: number, to: number) => {
            for (let i = from; i < to; i++) {
                if (text[i] === '\n') {
                    buf += '\n';
                } else {
                    buf += ' ';
                }
            }
        },
    };

    let lastEnd = 0;
    for (const [start, end] of ranges) {
        mutate_buf.fillWithWhiteSpace(lastEnd, start);
        mutate_buf.copyFromOriginalText(start, end);
        lastEnd = end;
    }
    mutate_buf.fillWithWhiteSpace(lastEnd, text.length);

    return buf;
}
