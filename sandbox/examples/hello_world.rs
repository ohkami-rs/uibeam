use uibeam::UI;

fn main() {
    let name = "world";
    let style = "color: red; font-size: 20px;";

    println!("{}", uibeam::shoot(UI! {
        <div class="hello" style={style}>
            "Hello "{name}"!"
            <br/>
            "こんにちは" //@{name}!
        </div>
    }));
}
