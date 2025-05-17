use uibeam::{UI, Beam};

struct Hello {
    user_name: String,
    style: Option<String>,
    children: UI,
}
impl Beam for Hello {
    fn render(self) -> UI {
        let style = self.style.unwrap_or_else(|| format!("\
            color: red; \
            font-size: 20px; \
        "));

        UI! {
            <p class="hello" style={style} data-index={-1}>
                "Welcome to the world of UIBeam!"
                <br>
                "こんにちは"
                <a
                    class="user"
                    style="color: blue;"
                    data-user-id=123
                    href="https://example-chatapp.com/users/123"
                >
                    "@"{&self.user_name}"!"
                </a>
                <br>
                {self.children}
            </p>
        }
    }
}

fn main() {
    let html = uibeam::shoot(UI! {
        <body>
            <h1>"UIBeam example"</h1>
            <custom-element id="example" />
            <custom-element2 id="example2">
                <p>"Hello from a child of custom-element2!"</p>
            </custom-element2>
            <Hello
                user_name="uibeam"
                style={format!("\
                    color: green; \
                    font-size: 30px; \
                    text-decoration: underline; \
                ")}
            >
                <p>"[message] this is a test message"</p>
            </Hello>
        </body>
    });

    println!("{html}");
}

#[cfg(test)]
#[test]
fn test_html() {
    let mut output = String::from_utf8(
        std::process::Command::new("cargo").arg("run").output().unwrap().stdout
    ).unwrap();

    // Remove the last newline character of `println!` output
    output.pop();

    assert_eq!(output, include_str!("../expected.html"));
}
