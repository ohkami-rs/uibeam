import { expect, test } from 'vitest';
import { findUIInputOffsets } from './find_ui';

test('findUIInputOffsets when not found', () => {
    expect(findUIInputOffsets(``)).toEqual([]);

    expect(findUIInputOffsets(`
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

test('findUIInputOffsets when empty UI! {}', () => {
    const text = `
        fn main() {
            let ui = UI! {};
        }
    `;

    const offsets = findUIInputOffsets(text);

    expect(offsets).toEqual([
        [47, 47],
    ]);
    expect(text.substring(offsets[0][0], offsets[0][1])).toEqual(
        ''
    );
});

test('findUIInputOffsets only with literals', () => {
    const text = `
        fn main() {
            let ui = UI! {
                <p>
                    "Hello, world!"
                </p>
            };
        }
    `;

    const offsets = findUIInputOffsets(text);

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

test('findUIInputOffsets with interpolations', () => {
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

    const offsets = findUIInputOffsets(text);

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
