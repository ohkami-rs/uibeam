use uibeam::{UI, Beam, Laser, signal};

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
        #[cfg(target_arch = "wasm32")]
        uibeam::laser::web_sys::console::log_1(&format!("Counter initialized with count: {}", self.initial_count).into());

        let _signal_creator = signal::<i32>;//(self.initial_count);
        #[cfg(target_arch = "wasm32")]
        uibeam::laser::web_sys::console::log_2(&"Signal creator:".into(), &std::any::type_name_of_val(&_signal_creator).into());

        let _dummy_signal_creator = uibeam::laser::dummy_signal::<i32>;
        #[cfg(target_arch = "wasm32")]
        uibeam::laser::web_sys::console::log_2(&"Dummy signal creator:".into(), &std::any::type_name_of_val(&_dummy_signal_creator).into());
        let _dummy_signal = _dummy_signal_creator();
        #[cfg(target_arch = "wasm32")]
        uibeam::laser::web_sys::console::log_2(&"Dummy signal created:".into(), &_dummy_signal.into());

        //let (count, set_count) = signal(self.initial_count);
//
//        #[cfg(target_arch = "wasm32")]
//        uibeam::laser::web_sys::console::log_1(&"Signal created for count".into());

        let increment = {
            //let (count, set_count) = (count.clone(), set_count.clone());
            move |_| //set_count(count() + 1)
            {
                #[cfg(target_arch = "wasm32")] {
                    uibeam::laser::web_sys::console::log_1(&"Increment button clicked".into());
                    let count = uibeam::laser::web_sys::window()
                        .unwrap()
                        .document()
                        .unwrap()
                        .get_element_by_id("count")
                        .unwrap();
                    let current_count = count.text_content().unwrap();
                    let new_count = current_count.parse::<i32>().unwrap() + 1;
                    count.set_text_content(Some(&new_count.to_string()));
                }
            }
        };

        let decrement = {
            //let (count, set_count) = (count.clone(), set_count.clone());
            move |_| //set_count(count() - 1)
            {
                #[cfg(target_arch = "wasm32")] {
                    uibeam::laser::web_sys::console::log_1(&"Decrement button clicked".into());
                    let count = uibeam::laser::web_sys::window()
                        .unwrap()
                        .document()
                        .unwrap()
                        .get_element_by_id("count")
                        .unwrap();
                    let current_count = count.text_content().unwrap();
                    let new_count = current_count.parse::<i32>().unwrap() - 1;
                    count.set_text_content(Some(&new_count.to_string()));
                }
            }
        };

        UI! {
            <div>
                <h1 class="text-2xl font-bold">"Count: "
                    //{count()}
                    <span id="count">"0"</span>
                </h1>
                <button class="bg-blue-500 text-white px-4 py-2 rounded" onclick={increment}>"+"</button>
                <button class="bg-red-500 text-white px-4 py-2 rounded" onclick={decrement}>"-"</button>
            </div>
        }
    }
}
