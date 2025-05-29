use components::{Layout, Counter};
use uibeam::UI;
use ohkami::prelude::*;
use ohkami::serde::Deserialize;
use ohkami::format::{Query, HTML};

#[derive(Clone)]
struct LayoutFang {
    title: &'static str,
}
impl FangAction for LayoutFang {
    async fn back(&self, res: &mut Response) {
        if res.headers.ContentType().is_some_and(|x| x.starts_with("text/html")) {
            let content = res.drop_content().into_bytes().unwrap();
            let content = std::str::from_utf8(&*content).unwrap();
            res.set_html(uibeam::shoot(UI! {
                <Layout title={self.title}>
                    unsafe {content}
                </Layout>
            }));
        }
    }
}

#[derive(Clone)]
struct Logger;
impl FangAction for Logger {
    async fn fore(&self, req: &mut Request) -> Result<(), Response> {
        println!("{req:?}");
        Ok(())
    }
    async fn back(&self, res: &mut Response) {
        println!("{res:?}");
    }
}

#[derive(Deserialize)]
struct CounterMeta {
    init: Option<i32>,
}

async fn index(Query(q): Query<CounterMeta>) -> HTML<std::borrow::Cow<'static, str>> {
    let initial_count = q.init.unwrap_or(0);
    
    HTML(uibeam::shoot(UI! {
        <main class="display-flex flex-col items-center justify-center min-h-screen bg-gray-100">
            <h1 class="text-4xl font-bold mb-8">"Counter Example"</h1>
            <h2>"Counter 1"</h2>
            <Counter initial_count={initial_count} />
            <hr />
            <h2>"Counter 2"</h2>
            <Counter initial_count={initial_count + 100} />
        </main>
    }))
}

fn main() {
    smol::block_on(async {
        Ohkami::new((
            Logger,
            LayoutFang { title: "Counter Example" },
            "/.uibeam".Dir("./components/.uibeam"),
            "/".GET(index),
        )).howl("localhost:5000").await
    });
}
