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
                <div class="w-[144px]">
                    <p class="text-2xl font-bold text-center">
                        "Count: "{*count}
                    </p>
                    <div class="text-center">
                        <button class="cursor-pointer bg-red-500  w-[32px] py-1 text-white rounded-md" onclick={decrement}>"-"</button>
                        <button class="cursor-pointer bg-blue-500 w-[32px] py-1 text-white rounded-md" onclick={increment}>"+"</button>
                    </div>
                </div>
            </div>
        }
    }
}
