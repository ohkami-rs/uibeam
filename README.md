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

## Examples

### `UI!` syntax

```rust
use uibeam::UI;

fn main() {
    let user_name = "foo".to_string();

    let style = "
        color: red; \
        font-size: 20px; \
    ";
    
    let ui: UI = UI! {
        <p class="hello" style={style}>
            "Welcome to the world of UIBeam!"
            <br>
            "こんにちは"
            <a
                class="user"
                style="color: blue;"
                data-user-id="123"
                href="https://example-chatapp.com/users/123"
            >
                "@"{user_name}"!"
            </a>
        </p>
    };

    dbg!(ui);
}
```

### Conditional & Iterating rendering

`{}` at node-position in `UI!` can render, in addition to string or integer, any `impl IntoIterator<Item = UI>`. This includes `Option<UI>` or any iterator yielding `UI` !

```rust
use uibeam::{UI, Beam};

struct Task {
    id: u64,
    title: String,
    subtasks: Vec<String>,
    completed: bool,
}
impl Beam for T

fn main() {
    let t = Task {
        id: 42,
        title: "try uibeam".to_string(),
        subtasks: vec![],
        completed: false,
    };

    let task_ui = UI! {
        <div id={format!("task-{t.id}")}>
            <h2>{t.title}</h2>

            <h3>"subtasks"</h3>
            <ul>
                {t.subtasks.iter().map(|s| UI! {
                    <li>{s}</li>
                })}
            </ul>

            {t.completed.then_some(UI! {
                <i><strong>"completed"</strong></i>
            })}
        </div>
    };

    dbg!(task_ui);
}
```

### Components with `Beam`

```rust
use uibeam::{Beam, UI};

struct Layout {
    title: String,
    children: UI,  // `children` field
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
                    {self.children}
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
