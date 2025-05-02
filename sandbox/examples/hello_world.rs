use uibeam::{UI, Beam};

struct Hello {
    user_name: String,
    style: Option<String>,
}
impl Beam for Hello {
    fn render(self) -> UI {
        let style = self.style.unwrap_or_else(|| format!("\
            color: red; \
            font-size: 20px; \
        "));

        UI! {
            <p class="hello" style={style}>
                "Welcome to the world of UIBeam!"
                <br>
                "こんにちは"
                <a
                    class="user"
                    style="color: blue;"
                    data-user-id="123"
                    href="https://example-chatapp.com/users/123"
                >
                    "@"{&self.user_name}"!"
                </a>
            </p>
        }
    }
}

struct Counter {
    initial_count: u32,
}
impl Beam for Counter {
    fn render(self) -> UI {
        UI! {
            <div id="counter">
                <a></a>
            </div>
        }
    }
}

fn main() {
    println!("{}", uibeam::shoot(UI! {
        <body>
            <h1>"UIBeam example"</h1>
            <Hello
                user_name="uibeam"
                style={format!("\
                    color: green; \
                    font-size: 30px; \
                    text-decoration: underline; \
                ")}
            >
                // "compile error if you uncomment this line"
                // "because `Hello` doesn't have `children` field"
            </Hello>
        </body>
    }));
}
