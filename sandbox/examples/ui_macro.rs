use uibeam::UI;

fn main() {
    let name = "world";
    let html = UI! {
        <div>
            Hello {name}! こんにちは @{name}!
        </div>
    };
    println!("{}", html);
}
