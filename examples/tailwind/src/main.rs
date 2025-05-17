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
                <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/tailwindcss@2.2.19/dist/tailwind.min.css">
                <title>"Ohkami organization"</title>
            </head>
            <body>
                {self.children}
            </body>
            </html>
        }
    }
}

fn main() -> std::io::Result<()> {
    let pages_dir = std::path::Path::new("pages");

    for (path, page) in pages::pages() {
        let page = uibeam::shoot(UI! {
            <Layout>
                {page()} /* resolved in https://github.com/ohkami-rs/uibeam/pull/77 */
            </Layout>
        });

        let path = pages_dir.join(format!(
            ".{}.html",
            if *path == "/" {"/index"} else {path}
        ));

        let parent = path.parent().unwrap();
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(path, page.as_bytes())?;
    }

    Ok(())
}
