use uibeam::{UI, Beam};

struct ContactList {
    contacts: Vec<Contact>,
}
struct Contact {
    title: &'static str,
    address: &'static str,
    href: &'static str,
}
impl Beam for ContactList {
    fn render(self) -> UI {
        UI! {
            <div class="w-screen flex flex-col items-center">
                <h2 class="text-2xl font-bold my-4">
                    "Contact"
                </h2>
                {self.contacts.iter().map(|c| UI! {
                    <p>
                        {c.title}": "
                        <a
                            class="text-blue-500 hover:underline"
                            href={c.href}
                        >
                            {c.address}
                        </a>
                    </p>
                })}
            </div>
        }
    }
}

pub(super) fn page() -> UI {
    UI! {
        <h1 class="w-screen text-center text-3xl font-bold my-8">
            "Ohkami organization"
        </h1>

        <p class="w-screen text-center text-lg my-8">
            "The intuitive solutions for Rust web development"
        </p>

        <ContactList contacts={[
            Contact {
                title: "GitHub",
                address: "ohkami-rs",
                href: "https://github.com/ohkami-rs",
            },
            Contact {
                title: "Email",
                address: "contact@ohkami.rs",
                href: "mailto:contact@ohkami.rs",
            },
        ]} />

        <p class="w-screen text-right text-sm font-bold my-8">
            <i>"This page is under construction..."</i>
        </p>
    }
}
