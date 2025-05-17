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

fn main() -> std::io::Result<()> {
    let pages_dir = std::path::Path::new("pages");

    for (path, page) in pages::pages() {
        let page = uibeam::shoot(UI! {
            <Layout>
                {page()}
            </Layout>
        });

        let path = pages_dir.join(format!(
            ".{}.html",
            if *path == "/" {"/index"} else {path}
        ));

        // Create the directory if it doesn't exist
        let parent = path.parent().unwrap();
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }

        // Write the HTML to the file
        std::fs::write(path, page.as_bytes())?;
    }

    Ok(())
}
