<div align="center">
    <h1>UI Beam</h1>
    A component-based, in-source template engine for Web UI in Rust
</div>

## Features

- `UI!` : HTML-like template syntax in Rust source, compile-time checked
- `Beam` : Component support
- simple : Simply organized codebase with zero external dependencies
- efficient : Emitting efficient code, avoiding redundant memory allocations as smartly as possible
- providing [VSCode extension](./support/vscode)

## Example

```rust
use uibeam::{Beam, UI};

struct Layout {
    title: String,
    child: UI,  // <-- `child` field
}

impl Beam for Layout {
    fn render(self) -> UI {
        UI! {
            <html>
                <head>
                    <title>{self.title}</title>
                    <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/tailwindcss@2.2.19/dist/tailwind.min.css">
                </head>
                <body class="bg-gray-100">
                    {self.child}
                </body>
            </html>
        }
    }
}

struct AdminPage {}

impl Beam for AdminPage {
    fn render(self) -> UI {
        UI! {
            <main class="container mx-auto flex-grow py-8 px-4">
                <section class="bg-white shadow-md rounded-lg p-6">
                    <h1 class="text-2xl font-bold text-gray-800 mb-6">
                        "Password"
                    </h1>
                    <h2 class="text-xl font-semibold text-gray-700 mb-4">
                        "Enter password."
                    </h2>
                    <form method="post" action="" class="w-full">
                        <div class="flex flex-col gap-4">
                            <div class="flex flex-col">
                                <label
                                    for="adminPassword"
                                    class="text-gray-700 text-sm font-bold mb-1"
                                >
                                    "password"
                                </label>
                                <input
                                    required
                                    type="password"
                                    id="adminPassword"
                                    name="adminPassword"
                                    class="py-2 px-3 border border-gray-400 rounded focus:outline-none focus:shadow-outline"
                                </input>
                            </div>
                        </div>
                        <div class="mt-6">
                            <button
                                type="submit"
                                class="bg-purple-500 hover:bg-purple-700 text-white py-2 px-4 rounded focus:outline-none focus:shadow-outline"
                            >
                                "Send"
                            </button>
                        </div>
                    </form>
                </section>
            </main>
        }
    }
}

fn main() {
    // shoot directly

    let html = uibeam::shoot(AdminPage {});
    dbg!(html);

    let html = uibeam::shoot(Layout {
        title: "admin page".to_string(),
        children: uibeam::shoot(AdminPage {}),
    });
    dbg!(html);

    // shoot with `UI!`

    let html = uibeam::shoot(UI! {
        <Layout title="admin page">  // `title` is filled with `"admin page".into()`
            <AdminPage />  // `children` is filled with `AdminPage {}`
        </Layout>
    });
    dbg!(html);
}
```
