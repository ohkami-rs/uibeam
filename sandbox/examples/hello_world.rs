#![feature(default_field_values)]

use uibeam::{UI, Beam};

struct Hello {
    user_name: String,
    style: Option<String> = None,
}
impl Beam for Hello {
    fn render(self) -> UI {
        let style = self.style.unwrap_or_else(|| format!("\
            color: red; \
            font-size: 20px; \
        "));

        UI! {
            <p class="hello" style={style}>
                "Welcome to the world of uibeam!"
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

fn main() {
    println!("{}", uibeam::shoot(Hello {
        user_name: "uibeam".to_string(),
        ..
    }.render()));
}
