mod pages;

use uibeam::{UI, Beam};

struct Layout {
    children: UI,
}
impl Beam for Layout {
    fn render(self) -> UI {
        UI! {
            <html lang="en">
            <head>
                <meta charset="UTF-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <link rel="stylesheet" href="./tailwind.css">
                <title>"Ohkami organization"</title>
            </head>
            <body>
                {self.children}
            </body>
            </html>
        }
    }
}

fn _test() {
    let _ = UI! {
        {"Hello world!"}
    };
}

fn main() -> std::io::Result<()> {
    let pages_dir = std::path::Path::new("pages");

    for (path, page) in pages::pages() {
        let path = path
            .trim_matches('/')
            .with_extension("html");

        let page = UI! {
            <Layout>
                {page()}
            </Layout>
        };

        std::fs::write(
            pages_dir.join(path),
            uibeam::shoot(page).as_bytes()
        )?;
    }

    Ok(())
}
