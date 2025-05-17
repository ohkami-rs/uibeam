<div align="center">
    <h1>
        UIBeam
    </h1>
    <p>
        A lightweight, JSX-style HTML template engine for Rust
    </p>
</div>

<div align="right">
    <a href="https://github.com/ohkami-rs/uibeam/blob/main/LICENSE"><img alt="License" src="https://img.shields.io/crates/l/uibeam.svg" /></a>
    <a href="https://github.com/ohkami-rs/uibeam/actions"><img alt="CI status" src="https://github.com/ohkami-rs/uibeam/actions/workflows/CI.yml/badge.svg"/></a>
    <a href="https://crates.io/crates/uibeam"><img alt="crates.io" src="https://img.shields.io/crates/v/uibeam" /></a>
</div>

## Features

- `UI!` : JSX-style template syntax with compile-time checks
- `Beam` : Component system
- Simple : Simply organized API and codebase, with zero external dependencies
- Efficient : Emitting efficient codes, avoiding redundant memory allocations as smartly as possible
- Better UX : HTML completions and hovers in `UI!` by VSCode extension ( search by "_uibeam_" from extension marketplace )

![](https://github.com/ohkami-rs/uibeam/raw/HEAD/support/vscode/assets/completion.png)

## Usage

```toml
[dependencies]
uibeam = "0.2.0"
```

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

    println!("{}", uibeam::shoot(ui));
}
```

### Conditional & Iterative rendering

`{}` at node-position in `UI!` can render, in addition to `Display`-able values, any `impl IntoIterator<Item = UI>`. This includes `Option<UI>` or any other iterators yielding `UI`s !

```rust
use uibeam::{UI, Beam};

struct Task {
    id: u64,
    title: String,
    subtasks: Vec<String>,
    completed: bool,
}

fn main() {
    let t = Task {
        id: 42,
        title: "try uibeam".to_string(),
        subtasks: vec![],
        completed: false,
    };

    let ui = UI! {
        <div id={format!("task-{}", t.id)}>
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

    println!("{}", uibeam::shoot(ui));
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
                    <form method="post" action="" class="w-full">
                        <div class="flex flex-col gap-4">
                            <div class="flex flex-col">
                                <label for="adminPassword" class="text-gray-700 text-sm font-bold mb-1">
                                    "password"
                                </label>
                                <input
                                    required
                                    type="password"
                                    id="adminPassword"
                                    name="adminPassword"
                                    class="py-2 px-3 border border-gray-400 rounded focus:outline-none focus:shadow-outline"
                                />
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
    let ui = UI! {
        <Layout title="admin page">  // title: "admin page".into()
            <AdminPage />  // children: (AdminPage {}).render()
        </Layout>
    };

    println!("{}", uibeam::shoot(ui));
}
```

### Unsafely insert HTML string

**raw string literal** ( `r#"..."#` ) or **unsafe block** contents are rendered *without HTML-escape*.

<!-- ignore for `include_str!` -->
```rust,ignore
use uibeam::UI;

fn main() {
    println!("{}", uibeam::shoot(UI! {
        <html>
            <body>
                /* ↓ wrong here: scripts are html-escaped... */

                <script>
                    "console.log('1 << 3 =', 1 << 3);"
                </script>

                <script>
                    {include_str!("index.js")}
                </script>

                /* ↓ scripts are NOT html-escaped, rendered as they are */

                <script>
                    r#"console.log('1 << 3 =', 1 << 3);"#
                </script>

                <script>
                    unsafe {include_str!("index.js")}
                </script>

                <script>
                    unsafe {"console.log('1 << 3 =', 1 << 3);"}
                </script>
            </body>
        </html>
    }));
}
```

## Integrations with web frameworks

Enables `UI` to be returned as a HTML response.

### [Axum](https://github.com/tokio-rs/axum) - by "axum" feature

```rust
use axum::{routing::get, Router};
use uibeam::UI;

async fn handler() -> UI {
    UI! {
        <h1>"Hello, Axum!"</h1>
    }
}

fn app() -> Router {
    Router::new()
        .route("/", get(handler))
}
```

### [Actix Web](https://actix.rs) - by "actix-web" feature

```rust,no_run
use actix_web::{HttpServer, App, web};
use uibeam::UI;

async fn handler() -> UI {
    UI! {
        <h1>"Hello, Actix Web!"</h1>
    }
}

async fn run() -> std::io::Result<()> {
    HttpServer::new(||
        App::new()
            .route("/", web::get().to(handler))
    )
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
```

## License

UIBeam is licensed under [MIT LICENSE](./LICENSE).
