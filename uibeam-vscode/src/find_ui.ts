export function findUIInputOffsets(text: string): [number, number][] {
    let ranges: [number, number][] = [];
    {
        console.debug(/UI!\s*{/g.exec(text));
        for (const match of text.matchAll(/UI!\s*{/g)) {
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
                let depth = 1;
                while (end < text.length) {
                    console.debug(`depth: ${depth}, end: ${end}, char: '${text[end]}'`);
                    if (text[end] === '}') {
                        depth -= 1;
                    } else if (text[end] === '{') {
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
