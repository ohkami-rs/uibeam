use uibeam::{UI, Beam, Signal, callback};
use uibeam::client::PointerEvent;

pub struct Layout {
    pub title: String,
    pub children: UI,
}
impl Beam for Layout {
    fn render(self) -> UI {
        UI! {
            <html>
                <head>
                    <link rel="stylesheet" href="/.uibeam/tailwind.css" />
                    <title>{&*self.title}</title>
                </head>
                <body>
                    {self.children}
                </body>
            </html>
        }
    }
}

struct CounterButton {
    on_click: Box<dyn Fn(PointerEvent)>,
    children: UI,
    /// additional classes to modify default style
    class: Option<&'static str>,
}
#[uibeam::client] // client component, but not Serialize/Deserialize and not at island boundary
impl Beam for CounterButton {
    fn render(self) -> UI {
        UI! {
            <button
                class={format!(
                    "cursor-pointer w-[32px] py-1 text-white rounded-md {}",
                    self.class.unwrap_or("")
                )}
                onclick={self.on_click}
            >
                {self.children}
            </button>
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Counter {
    pub initial_count: i32,
}
#[uibeam::client(island)] // client component at island boundary
impl Beam for Counter {
    fn render(self) -> UI {
        let count = Signal::new(self.initial_count);

        let increment = callback!([count], |_| {
            count.set(*count + 1);
        });

        let decrement = callback!([count], |_| {
            count.set(*count - 1);
        });

        UI! {
            <div class="w-[144px]">
                <p class="text-2xl font-bold text-center">
                    "Count: "{*count}
                </p>
                <div class="text-center">
                    <CounterButton
                        on_click={Box::new(decrement)}
                        class="bg-red-500"
                    >"-"</CounterButton>
                    <CounterButton
                        on_click={Box::new(increment)}
                        class="bg-blue-500"
                    >"+"</CounterButton>
                </div>
            </div>
        }
    }
}
