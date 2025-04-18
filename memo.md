# UI Beam

A component-based, in-code template engine for Web UI in Rust

## Features

- component-based, in-code
- compile-time checked
- lightweight
- async-ready, no runtime dependency

## Example

```rust
use uibeam::{Beam, UI};

struct Layout {
    title: String,
    child: UI,  // <-- `child` field
}

impl Beam for Layout {
    type Error = std::convert::Infallible;

    async fn render(self) -> Result<UI, Self::Error> {
        Ok(UI! {
            <html>
                <head>
                    <title>{self.title}</title>
                    <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/tailwindcss@2.2.19/dist/tailwind.min.css">
                </head>
                <body class="bg-gray-100">
                    {self.child}
                </body>
            </html>
        })
    }
}

struct AdminPage {}

impl Beam for AdminPage {
    type Error = std::convert::Infallible;

    async fn render(self) -> Result<UI, Self::Error> {
        Ok(UI! {
            <main class="container mx-auto flex-grow py-8 px-4">
                <section class="bg-white shadow-md rounded-lg p-6">
                    <h1 class="text-2xl font-bold text-gray-800 mb-6">
                        "パスワード確認"
                    </h1>
                    <h2 class="text-xl font-semibold text-gray-700 mb-4">
                        "パスワードを確認します。"
                    </h2>
                    <form method="post" action="" class="w-full">
                        <div class="flex flex-col gap-4">
                            <div class="flex flex-col">
                                <label
                                    for="adminPassword"
                                    class="text-gray-700 text-sm font-bold mb-1"
                                >
                                    "パスワード"
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
                                "送信"
                            </button>
                        </div>
                    </form>
                </section>
            </main>
        })
    }
}

struct Email(String);

struct Timestamp(String);

struct ThreadResponse {
    thread_id: String,
    response_id: String,
    response_number: u32,
    author_name: String,
    email: Email,
    posted_at: Timestamp,
    content: String,
}

struct Thread {
    thread_id: String,
    title: String,
    responses: Vec<ThreadResponse>,
}

struct ThreadPage {
    id: String,
    db: Client,
}

impl Beam for ThreadPage {
    type Error = Box<dyn std::error::Error + Send + Sync>;

    async fn render(self) -> Result<UI, Self::Error> {
        let thread = self.db
            .get_thread(&self.id)
            .await
            .map_err(|e| format!("Failed to get thread: {}", e))?;

        Ok(UI! {
            <main class="container mx-auto flex-grow py-8 px-4">
                <section class="bg-white shadow-md rounded-lg p-6 mb-8">
                    <div>
                        <h3 class="text-purple-600 text-xl font-bold mb-4">
                            {format!("{} ({} 件)", thread.title, thread.responses.len())}
                        </h3>
                        {thread.responses.iter().map(|res| UI! {
                            <div
                                id={format!("{}-{}", thread.thread_id, res.rensponse_number)}
                                class="bg-gray-50 p-4 rounded-md"
                            >
                                <div class="flex flex-wrap items-center gap-2 mb-2">
                                    <span class="font-bold">{res.response_number}</span>
                                    <span class="text-gray-700">{res.author_name}</span>
                                    <span class="text-gray-500 text-sm">{res.posted_at}</span>
                                </div>
                                <div class="text-gray-800 whitespace-pre-line">
                                    {res.content}
                                </div>
                            </div>
                        })}
                    </div>
                </section>
            </main>
        })
    }
}

#[tokio::main]
async fn main() {
    // shoot directly

    let html = uibeam::shoot(AdminPage {});
    dbg!(html);

    let html = uibeam::shoot(ThreadPage {
        id: "123".to_string(),
        db: Client::new().await.unwrap(),
    });
    dbg!(html);

    // shoot with `UI!`

    let html = uibeam::shoot(UI! {
        <Layout title="Admin Page">
            <AdminPage />  // <-- `child` field is filled with `AdminPage`
        </Layout>
    });
    dbg!(html);
}
```
