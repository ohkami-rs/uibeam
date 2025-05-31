use uibeam::{UI, Beam, Laser, Signal, callback};

pub struct Layout {
    pub title: String,
    pub children: UI,
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

#[Laser]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Counter {
    pub initial_count: i32,
}

impl Laser for Counter {
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
                <p class="text-2xl font-bold">"Count: "{*count}</p>
                <button class="bg-blue-500 text-white px-4 py-2 rounded" onclick={increment}>"+"</button>
                <button class="bg-red-500 text-white px-4 py-2 rounded" onclick={decrement}>"-"</button>
            </div>
        }
    }
}
