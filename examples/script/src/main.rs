use uibeam::{UI, Beam};

struct Page {
    title: String,
    children: UI,
}
impl Beam for Page {
    fn render(self) -> UI {
        UI! {
            <html lang="en">
            <head>
                <meta charset="UTF-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <title>{self.title}</title>
            </head>
            <body>
                {self.children}

                <script>"
                    console.log('1 << 3 =', 1 << 3);
                "</script>// wrong: `'` in script will be html-escaped...

                <script>r#"
                    console.log('1 << 3 =', 1 << 3);
                "#</script>// `'` in script will NOT be html-escaped...

                <script>// wrong: `'` in script will be html-escaped...
                    {include_str!("module.js")}
                </script>

                <script>// `'` in script will NOT be html-escaped...
                    unsafe {include_str!("module.js")}
                </script>

                <script>unsafe {"
                    console.log('1 << 3 =', 1 << 3);
                "}</script>// `'` in script will NOT be html-escaped...
            </body>
            </html>
        }
    }
}

fn main() {
    let html = uibeam::shoot(UI! {
        <Page title="UIBeam with some script">
            <h1>"Hello, script!"</h1>
            <p>"This is a simple example of using script with UIBeam."</p>
        </Page>
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

    assert_eq!(output, include_str!("../expected.html.txt"));
}

