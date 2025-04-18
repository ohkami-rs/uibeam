use uibeam::UI;

fn main() {
    let name = "world";

    println!("{}", uibeam::shoot(UI! {
        <div>
            "Hello "{name}"!"
            // <br>
            // こんにちは @{name}!
        </div>
    }));
}
