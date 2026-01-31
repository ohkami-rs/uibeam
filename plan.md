## `client` Feature Strategy

### Example Source Code

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
            count.set_mut(|c| *c -= 1);
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

### Key Idea

#### `cfg(not(hydrate))`

```rust
struct UI {
    template: Cow<'static, str>,
    island_tag_names: Vec<&'static str>,
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
            count.set_mut(|c| *c -= 1);
        };
        
        UI! {
            // Generates island by the Declarative Shadow DOM.
            // `uibeam-counter` will be later `define`d as a custom element by the Wasm & wrapper JS
            // and all its Rust logic run at that time,
            // while the static template itself is immediately rendered when the HTML is loaded.
            // 
            // i.e. the hydration is achieved by the custom element definition.
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
            count.set_mut(|c| *c -= 1);
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
pub fn __hydrate_Counter() {
    ::uibeam::client::runtime::register_island(
        "uibeam-counter",        
        Closure::<dyn Fn(Node, String) -> Function>::new(
            |shadow_root, serialized_props| -> ::uibeam::client::js_sys::Function {
                let this = ::uibeam::client::deserialize_props::<Counter>(&serialize_props);
                let scope = ::uibeam::client::EffectScope::new(move || {
                    let reactivities = <Counter as ::uibeam::Beam>::render(this).reactivities;
                    let event_types = ::std::iter::Iterator::zip(
                        // bulk-convert `NodePath`s (encoded to strings) to DOM Nodes in ahead
                        // to mimize Rust-JS FFI cost.
                        ::uibeam::client::runtime::collect_nodes(shadow_root, reactivities.path_list()),
                        reactivities.reactivities()
                    ).filter_map(|(target_node, r)| r.apply_in_island(shadow_root, target_node));
                    // bulk-delegate events 
                    // to minimize Rust-JS FFI cost.
                    ::uibeam::client::runtime::delegate_events(event_types);
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

```js
(async () => {
  const { default: init, ...items } = await import('/.uibeam/hydrate.js');
  await init();
  for (const [name, f] of Object.entires(items)) {
    if name.startsWith('__hydrate') && typeof f === 'function' {
      f();
    }
  }
})();
```
