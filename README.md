<div align="center">
    <h1>
        UIBeam
    </h1>
    <p>
        A lightweight, JSX-style Web UI library for Rust
    </p>
</div>

<div align="right">
    <a href="https://github.com/ohkami-rs/uibeam/blob/main/LICENSE"><img alt="License" src="https://img.shields.io/crates/l/uibeam.svg" /></a>
    <a href="https://github.com/ohkami-rs/uibeam/actions"><img alt="CI status" src="https://github.com/ohkami-rs/uibeam/actions/workflows/CI.yml/badge.svg"/></a>
    <a href="https://crates.io/crates/uibeam"><img alt="crates.io" src="https://img.shields.io/crates/v/uibeam" /></a>
</div>

- `UI!` : JSX-style template syntax with compile-time checks
- `Beam` : Component System based on Rust structs

## Features

- Supports client component via island architecture in Wasm. (See *Client Component* section below)
- Simply organized API and codebase.
- Emits efficient template rendering avoiding redundant memory allocations as smartly as possible.
- HTML completions and hovers in `UI!` by VSCode extension. ( search "uibeam" from extension marketplace )

![](https://github.com/ohkami-rs/uibeam/raw/HEAD/support/vscode/assets/completion.png)

## Usage

```toml
[dependencies]
uibeam = "0.4"
```

### `UI!` syntax

```rust
use uibeam::UI;

fn main() {
    let user_name = "foo";

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

### unsafely insert HTML string

**raw string literal** ( `r#"..."#` ) or **unsafe block** contents are rendered *without HTML-escape* :

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

### conditional & iterative rendering

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

## `Beam` - Component with Rust struct and JSX-like syntax

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
        <Layout title="admin page">  // title: ("admin page").into()
            <AdminPage />  // children: (AdminPage {}).render()
        </Layout>
    };

    println!("{}", uibeam::shoot(ui));
}
```

## Client Component - WASM islands

### overview

**`#[client]`** makes `Beam` a _*WASM island*_ : initially rendered in server, sent with serialized props, and hydrated with deserialized props in client.

`Signal`, `computed`, `effect` are available in them.

### note

Currently UIBeam's client component system is built upon [`Preact`](https://preactjs.com).
This may be rewritten in pure Rust in the future, but may not because of potential reduction in the final .wasm size.

### usage

working example: [examples/counter](https://github.com/ohkami-rs/uibeam/blob/main/examples/counter)

1. Activate `"client"` feature, and add `serde` to your dependencies:

    ```toml
    [dependencies]
    uibeam = { version = "0.4" }
    serde  = { version = "1", features = ["derive"] }
    ```

2. Configure to export all your client components from a specific library crate.
   (e.g. `lib.rs` entrypoint, or another member crate of a workspace)
   
   (There's no problem if including ordinary `Beam`s, not only client ones, in the lib crate.)

   Additionally, specify `crate-type = ["cdylib", "rlib"]` for the crate:

    ```toml
    [lib]
    crate-type = ["cdylib", "rlib"]
    ```
    
   For size optimization, following configuration is recommended:
  
    ```toml
    [package.metadata.wasm-pack.release]
    wasm-opt = ["-Oz"]  # or "-Os"
    
    [profile.release]
    opt-level = 'z'  # or 's'
    lto = true
    codegen-unit = 1
    ```

   See https://rustwasm.github.io/docs/book/game-of-life/code-size.html or other specific documents for more details.
   
3. Define and use your client components:

    ```rust
    /* islands/src/lib.rs */
    
    use uibeam::{UI, Beam};
    use uibeam::{client, Signal, callback};
    use serde::{Serialize, Deserialize};
    
    // Client component located at **island boundary**
    // must be `Serialize + for<'de> Deserialize<'de>`. (see NOTE below)
    #[derive(Serialize, Deserialize)]
    pub struct Counter;
    
    // `#[client]` makes Beam a Wasm island.
    // `(island)` means this beam is **island boundary**.
    #[client(island)]
    impl Beam for Counter {
        fn render(self) -> UI {
            let count = Signal::new(0);
    
            // `callback!` - a thin utility for callbacks over signals.
            let increment = callback!(
                // [dependent_signals, ...]
                [count],
                // closure depending on the signals
                |_| count.set(*count + 1)
            );
            /* << expanded >>
    
            let increment = {
                let count = count.clone();
                move |_| count.set(*count + 1)
            };
            
            */
    
            let decrement = callback!([count], |_| {
                count.set(*count - 1)
            });
    
            UI! {
                <p>"Count: "{*count}</p>
                <button onclick={increment}>"+"</button>
                <button onclick={decrement}>"-"</button>
            }
        }
    }
    ```

    ```rust,ignore
    /* server/src/main.rs */
    
    use islands::Counter;
    use uibeam::UI;
    
    async fn index() -> UI {
        UI! {
            <Counter />
        }
    }
    ```
   
   **NOTE**:
   Client Beam at island boundary must be `Serialize + for<'de> Deserialize<'de>` in order to the Wasm island architecture.
   In contrast, `#[client]` components without `(island)`, e.g. having `children: UI` or `on_something: Box<dyn FnOnce(Event)>`
   as props, can NOT implement `Serialize` nor `Deserialize` and can **only be used internally in `UI!` of another client component**.
   Especially note that client components at **island boundary can't have `children`**.

4. Compile the lib crate into Wasm by `wasm-pack build` with **`RUSTFLAGS='--cfg hydrate'`** and **`--out-name hydrate --target web`**:

    ```sh
    # example when naming the lib crate `islands`

    cd islands
    RUSTFLAGS='--cfg hydrate' wasm-pack build --out-name hydrate --target web
    ```
    ```sh
    # **`--release`** in relase build:
    
    RUSTFLAGS='--cfg hydrate' wasm-pack build --out-name hydrate --target web --release
    ```
  
   **NOTE**:
   All of `hydrate` cfg (not feature!), `hydrate` out-name and `web` target are **required** here.

5. Make sure that your server responds with **a complete HTML consist of one <html></html> containing your page contents**.
   
   Then, setup your server to serve the output directory (default: `pkg`) at **`/.uibeam`** route:
 
    ```rust
    /* axum example */
 
    use axum::Router;
    use tower_http::services::ServeDir;
 
    fn app() -> Router {
        Router::new()
            .nest_service(
                "/.uibeam",
                ServeDir::new("./islands/pkg")
            )
            // ...
    }
    ```

   (as a result, generated `{crate name}/pkg/hydrate.js` is served at `/.uibeam/hydrate.js` route,
   which is automatically loaded together with corresponding .wasm file in the hydration step on browser.)

## Integrations with web frameworks

Enables `UI` to be returned directly as a HTML response.

### [Axum](https://github.com/tokio-rs/axum) - by "axum" feature

```toml
axum = { version = "0.8" }
uibeam = { version = "0.4", features = ["axum"] }
```

```rust,no_run
use axum::{routing::get, Router};
use uibeam::UI;

async fn handler() -> UI {
    UI! {
        <h1>"Hello, Axum!"</h1>
    }
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(handler));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

### [Actix Web](https://actix.rs) - by "actix-web" feature

```toml
actix-web = { version = "4.12" }
uibeam = { version = "0.4", features = ["actix-web"] }
```

```rust,no_run
use actix_web::{HttpServer, App, get};
use uibeam::UI;

#[get("/")]
async fn handler() -> UI {
    UI! {
        <h1>"Hello, Actix Web!"</h1>
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(||
        App::new()
            .service(handler)
    )
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
```

### [Ohkami](https://github.com/ohkami-rs/ohkami) - by "ohkami" feature

- UIBeam *v0.3* is compatible with Ohkami *v0.24*.
- Ohkami's `openapi` feature is supported via UIBeam's `openapi` feature flag.
- UIBeam itself is runtime-agnostic and available with any async runtimes supported by Ohkami.

```toml
[dependencies]
tokio = { version = "1.48", features = ["full"] }
ohkami = { version = "0.24", features = ["rt_tokio"] }
uibeam = { version = "0.4", features = ["ohkami"] }
# when using ohkami's "openapi" feature,
# activate also uibeam's "openapi" feature.
```

```rust,no_run
use ohkami::{Ohkami, Route};
use uibeam::UI;

async fn handler() -> UI {
    UI! {
        <h1>"Hello, Ohkami!"</h1>
    }
}

#[tokio::main]
async fn main() {
    Ohkami::new((
        "/".GET(handler),
    ))
    .howl("localhost:5000")
    .await
}
```

## License

UIBeam is licensed under [MIT LICENSE](./LICENSE).
