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
                class={self.class.unwrap_or("")}
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
            <div>
                <p>
                    "Count: "{*count}
                </p>
                <div>
                    <CounterButton
                        on_click={Box::new(decrement)}
                        class={None}
                    >"-"</CounterButton>
                    <CounterButton
                        on_click={Box::new(increment)}
                        class={None}
                    >"+"</CounterButton>
                </div>
            </div>
        }
    }
}
