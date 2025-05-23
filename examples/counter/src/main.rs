use ohkami::prelude::*;
use ohkami::serde::Deserialize;
use ohkami::format::{Query, HTML};
use uibeam::{UI, Beam, Laser, signal};

struct Layout {
    title: std::sync::Arc<String>,
    children: UI,
}
impl Beam for Layout {
    fn render(self) -> UI {
        UI! {
            <html>
                <head>
                    <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/tailwindcss@2.2.19/dist/tailwind.min.css" />
                    <title>{&*self.title}</title>
                </head>
                <body>
                    {self.children}
                </body>
            </html>
        }
    }
}
impl Layout {
    fn fang_with_title(title: &str) -> impl FangAction {
        #[derive(Clone)]
        struct Fang {
            title: String,
        }

        impl FangAction for Fang {
            async fn back(&self, res: &mut Response) {
                if res.headers.ContentType().is_some_and(|x| x.starts_with("text/html")) {
                    let content = res.drop_content().into_bytes().unwrap();
                    res.set_html(uibeam::shoot(UI! {
                        <Layout title={self.title.to_string()}>
                            unsafe {std::str::from_utf8(&*content).unwrap()}
                        </Layout>
                    }));
                }
            }
        }

        Fang {
            title: title.to_string(),
        }
    }
}

#[Laser]
#[derive(serde::Serialize)]
struct Counter {
    initial_count: i32,
}
impl Laser for Counter {
    fn render(self) -> UI {
        let (count, set_count) = signal(self.initial_count);

        let increment = {
            let (count, set_count) = (count.clone(), set_count.clone());
            move |_| set_count(count() + 1)
        };

        let decrement = {
            let (count, set_count) = (count.clone(), set_count.clone());
            move |_| set_count(count() - 1)
        };

        UI! {
            <div>
                <h1 class="text-2xl font-bold">"Counter: "{count()}</h1>
                <button class="bg-blue-500 text-white px-4 py-2 rounded" onClick={increment}>"+"</button>
                <button class="bg-red-500 text-white px-4 py-2 rounded" onClick={decrement}>"-"</button>
            </div>
        }
    }
}

#[derive(Deserialize)]
struct CounterMeta {
    init: Option<i32>,
}

async fn index(Query(q): Query<CounterMeta>) -> HTML<std::borrow::Cow<'static, str>> {
    let initial_count = q.init.unwrap_or(0);
    
    HTML(uibeam::shoot(UI! {
        <Counter initial_count={initial_count} />
    }))
}

async fn _main() {
    Ohkami::new((
        Layout::fang_with_title("Counter Example"),
        "/".GET(index),
    )).howl("localhost:5000").await;
}

fn main() {
    smol::block_on(_main());
}
