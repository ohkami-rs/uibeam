use ohkami::prelude::*;
use ohkami::serde::Deserialize;
use ohkami::format::Query;
use uibeam::{UI, Beam, Laser, signal};

struct Layout {
    title: String,
    children: UI,
}
impl Beam for &Layout {
    fn render(self) -> UI {
        <html>
            <head>
                <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/tailwindcss@2.2.19/dist/tailwind.min.css" />
                <title>{self.title}</title>
            </head>
            <body>
                {self.children}
            </body>
        </html>
    }
}
impl FangAction for Layout {
    async fn back(&self, res: &mut Response) {
        if res.headers.ContentType().is_some_and(|x| x.starts_with("text/html")) {
            let content = res.drop_content().into_bytes().unwrap();
            res.set_html(uibeam::shoot(self.render()));
        }
    }
}

struct Counter {
    initial_count: i32,
}
impl Laser for Counter {
    fn render(self) -> UI {
        let (count, set_count) = signal(self.initial_count);

        let increment = {
            let count = count.clone();
            move |_| set_count(count() + 1)
        };

        let decrement = {
            let count = count.clone();
            move |_| set_count(count() - 1)
        };

        UI! {
            <div>
                <h1 class="text-2xl font-bold">"Counter: "{count}</h1>
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

async fn index(Query(q): Query<'_, CounterMeta>) -> UI {
    let initial_count = q.init.unwrap_or(0);
    
    UI! {
        <Counter initial_count={initial_count} />
    }
}

async fn _main() {
    Ohkami::new((
        "/".GET(index),
    )).howl("localhost:5000").await;
}

fn main() {
    smol::spawn(_main());
}
