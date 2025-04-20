use uibeam::UI;

fn main() {
    let name = "world";
    let style = "color: red; font-size: 20px;";
    let user_name = "Mr.<script>alert('XSS');</script>";

    println!("{}", uibeam::shoot(UI! {
        <div class="hello" style={style}>
            "Hello "{name}"!"
            <br>
            "こんにちは"
            <a
                class="user"
                style="color: blue;"
                data-user-id="123"
                href="https://example-chatapp.com/users/123"
            >
                "@"{user_name}"!"
            </a>
        </div>
    }));
}
