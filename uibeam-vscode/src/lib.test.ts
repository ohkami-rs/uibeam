import { expect, test } from 'vitest';
import { findUIInputRangeOffsets } from './lib';

test('findUIInputRangeOffsets when not found', () => {
    expect(findUIInputRangeOffsets(``)).toEqual([]);

    expect(findUIInputRangeOffsets(`
        struct Hello<'h> {
            name: &'h str,
        }
        
        impl std::fmt::Display for Hello<'_> {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "Hello, {}!", self.name)
            }
        }

        fn main() {
            println!("{}", Hello { name: "world" });
        }
    `)).toEqual([]);
});

test('findUIInputRangeOffsets when empty UI! {}', () => {
    const text = `
        fn main() {
            let ui = UI! {};
        }
    `;

    const offsets = findUIInputRangeOffsets(text);

    expect(offsets).toEqual([
        [47, 47],
    ]);
    expect(text.substring(offsets[0][0], offsets[0][1])).toEqual(
        ''
    );
});

test('findUIInputRangeOffsets only with literals', () => {
    const text = `
        fn main() {
            let ui = UI! {
                <p>
                    "Hello, world!"
                </p>
            };
        }
    `;

    const offsets = findUIInputRangeOffsets(text);

    expect(offsets).toEqual([
        [47, 137],
    ]);
    expect(text.substring(offsets[0][0], offsets[0][1])).toEqual(
        `
                <p>
                    "Hello, world!"
                </p>
            `
    );
});

test('findUIInputRangeOffsets with interpolations', () => {
    const text = `
        fn main() {
            let name = "world";
            let ui = UI! {
                <p>
                    "Hello, "{name}"!"
                </p>
            };
        }
    `;

    const offsets = findUIInputRangeOffsets(text);

    expect(offsets).toEqual([
        [79, 172],
    ]);
    expect(text.substring(offsets[0][0], offsets[0][1])).toEqual(
        `
                <p>
                    "Hello, "{name}"!"
                </p>
            `
    );
});
