## Compiling Strategy

### Source Code

*counter.rs*
```rust
#[derive(Serialize, Deserialize)]
struct Counter {
    initial_count: u32,
}
#[uibeam::client(island)]
impl Beam for Counter {
    fn render(self) -> UI {
        let count = Signal::new(self.initial_count);
        
        let handle_increment_click = |_| {
            count.set_mut(|c| *c += 1);
        };
        let handle_decrement_click = |_| {
            count.set_mut(|c| *c += 1);
        };
        
        UI! {
            <div>
                <p>{count.get()}</p>
                <CounterButton onclick={handle_increment_click}>
                    +
                </CounterButton>
                <CounterButton onclick={handle_decrement_click}>
                    -
                </CounterButton>
            </div>
        }
    }
}

struct CounterButton<T: Display, F: impl Fn(MouseEvent)> {
    onclick: F,
    children: T,
}
impl<T: Display> Beam for CounterButton<T> {
    fn render(self) -> UI {
        UI! {
            <button onclick={self.onclick}>
                {self.children}
            </button>
        }
    }
}
```

### Generated Code

#### `cfg(not(hydrate))`

```rust
impl Beam for Counter {
    fn render(self) -> UI {
        let count = Signal::new(self.initial_count);
        
        let handle_increment_click = |_| {
            count.set_mut(|c| *c += 1);
        };
        let handle_decrement_click = |_| {
            count.set_mut(|c| *c += 1);
        };
        
        UI! {
            // island by Declarative Shadow DOM.
            // `uibeam-counter` is defined by the Wasm & wrapper JS.
            <uibeam-counter>
                <template shadowrootmode="open">
                    <div>
                        <p></p>
                        <button>+</button>
                        <button>-</button>
                    </div>
                </template>
            </uibeam-counter>
        }
    }
}
```

#### `cfg(hydrate)`

```rust
struct UI {
    template: Cow<'static, str>,
    reactivities: Vec<Reactivity>,
}
```

```rust
impl Beam for Counter {
    fn render(self) -> UI {
        let count = Signal::new(self.initial_count);
        
        let handle_increment_click = |_| {
            count.set_mut(|c| *c += 1);
        };
        let handle_decrement_click = |_| {
            count.set_mut(|c| *c += 1);
        };
        
        UI::new_unchecked(
            // static template string (will be never used in many cases)
            "<div><p></p><button>+</button><button>-</button></div>",
            // reactivities to be applied into DOM nodes in the hydration
            [
                Reactivity::Text(
                    NodePath::new(|root| root.first_child().first_child()),
                    move || {count.get()}
                ),
                Reactivity::EventListener(
                    NodePath::new(|root| root.first_child().first_child().next_sibling()),
                    "click",
                    handle_increment_click,
                ),
                Reactivity::EventListener(
                    NodePath::new(|root| root.first_child().first_child().next_sibling().next_sibling()),
                    "click",
                    handle_decrement_click,
                ),
            ]
        )
    }
}
```

```rust
#[wasm_bindgen]
#[doc(hidden)]
#[allow(non_snake_case)]
pub fn register_Counter() {
    ::uibeam::client::runtime::register_island(
        "uibeam-counter",        
        Closure::<dyn Fn(Node, String) -> Function>::new(
            |shadow_root, serialized_props| -> ::uibeam::client::js_sys::Function {
                let this = ::uibeam::client::deserialize_props::<Counter>(&serialize_props);
                let scope = ::uibeam::client::EffectScope::new(move || {
                    let reactivities = <Counter as ::uibeam::Beam>::render(this).reactivities;                
                    ::std::iter::Iterator::zip(
                        // bulk-convert `NodePath`s (encoded to strings) to DOM Nodes in ahead
                        // to mimize Rust-JS FFI cost.
                        ::uibeam::client::runtime::collect_nodes(shadow_root, reactivities.path_list()),
                        reactivities.reactivities()
                    ).for_each(|(target_node, r)| r.apply_in_island(shadow_root, target_node));
                });
                // returns cleanup function
                Closure::<dyn FnOnce()>::new(
                    move || scope.dispose()
                ).into_js_value().unchecked_into::<::uibeam::client::js_sys::Function>()
            }
        ).into_js_value().unchecked_into::<::uibeam::client::js_sys::Function>()
    );
}
```
