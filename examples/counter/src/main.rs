use components::{Layout, Counter};
use uibeam::UI;
use ohkami::{Ohkami, Route, FangAction, Request, Response};
use ohkami::claw::param::Query;

#[derive(Clone)]
struct LayoutFang {
    title: &'static str,
}
impl FangAction for LayoutFang {
    async fn back(&self, res: &mut Response) {
        if res.headers.content_type().is_some_and(|x| x.starts_with("text/html")) {
            let content = res.drop_content().into_bytes().unwrap();
            let content = std::str::from_utf8(&*content).unwrap();
            res.set_html(uibeam::shoot(UI! {
                <Layout title={self.title.to_string()}>
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

#[derive(serde::Deserialize)]
struct CounterMeta {
    init: Option<i32>,
}

async fn index(Query(q): Query<CounterMeta>) -> UI {
    let initial_count = q.init.unwrap_or(0);

    UI! {
        <main>
            <h1 class="text-4xl font-bold mb-8 text-center">"Counter Example"</h1>
            <div class="space-y-8">
                {[-100, -10, 0, 10, 100].iter().enumerate().map(|(i, &offset)| UI! {
                    <div class="flex items-center justify-center space-x-4">
                        <div class="w-1/3 min-w-fit grid gap-4 grid-cols-[1fr_144px]">
                            <div class="flex items-center">
                                <p class="text-2xl">"Counter #"{1+i}</p>
                            </div>
                            <div>
                                <Counter initial_count={initial_count + offset} />
                            </div>
                        </div>
                    </div>
                })}
            </div>
        </main>
    }
}

fn main() {
    smol::block_on(async {
        Ohkami::new((
            Logger,
            LayoutFang { title: "Counter Example" },
            "/.uibeam".Mount("./components/pkg"),
            "/".GET(index),
        )).howl("localhost:5000").await
    });
}
