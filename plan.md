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
            count.set_with_mut(|c| *c += 1);
        };
        let handle_decrement_click = |_| {
            count.set_with_mut(|c| *c -= 1);
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

struct CounterButton<F: impl Fn(MouseEvent)> {
    onclick: F,
    children: UI,
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
            count.set_with_mut(|c| *c += 1);
        };
        let handle_decrement_click = |_| {
            count.set_with_mut(|c| *c -= 1);
        };
        
        // As current behavior does, `UI!` automatically eliminates intermediate whitespaces.
        UI! {
            // Generates island by the Declarative Shadow DOM.
            // `uibeam-counter` will be later `define`d as a custom element by the Wasm & wrapper JS
            // and all its Rust logic run at that time,
            // while the static template itself is immediately rendered when the HTML is loaded.
            // 
            // i.e. the hydration is achieved by the custom element definition.
            // 
            // NOTE: at least in this version, UIBeam does NOT allow island-in-island.
            // `UI! { <MyIsland><AnotherIsland /></MyIsland> }` causes compile error.
            <uibeam-counter props={::uibeam::client::serialize_props(&self)}>
                <template shadowrootmode="open">
                    <div>
                        <p>{count.get()}</p> // <-- dummy signal impl
                        <CounterButton onclick={handle_increment_click}>
                            +
                        </CounterButton> // <-- rendered with `onclick` ignored
                        <CounterButton onclick={handle_decrement_click}>
                            -
                        </CounterButton>
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
    // to avoid redundant allocation, we should just have registration logic as this,
    // instead of like `reactivities: Reactivities` field
    register_reactivities_fn: Box<dyn FnOnce(&mut Reactivities, NodePath)>,
}

// reactivities to be applied into DOM nodes in the hydration
struct Reactivities {
    // effective list of (`NodePath`, `Reactivity`)
}

impl Beam for UI {
    fn register_reactivities(
        self,
        reactivities: &mut Reactivities,
        basepath: NodePath
    ) {
        (self.register_reactivities_fn)(reactivities, basepath)
    }
}
```

```rust
impl Beam for CounterButton {
    fn register_reactivities(
        self,
        reactivities: &mut Reactivities,        
        // NodePath :: [(firstChild|nextSibling)]
        // firstChild :: 0
        // nextSibling :: 1
        // 
        // struct NodePath(Cow<'static, [u8]>);
        // 
        // const fn NodePath::from_static(s: &'static [u8; N]) -> Self {
        //     1. static assert `s` syntax
        //     2. and then return `Self(Cow::Borrow(s))`
        //     3. this enables compile-time error message for wrong path via `const {NodePath::from_static(...)}`
        // }
        // 
        // TODO (next version): effective buffer management, especially when joining with child path
        basepath: NodePath,
    ) -> Reactivities {
        reactivities
            .register(
                basepath.join(const {NodePath::from_static(&[0])}), // <button>
                Reactivity::EventListener("click", self.onclick)
            );
        self.children
            .register_reactivities(
                reactivities,
                basepath.join(const {NodePath::from_static(&[0, 0])}), // TextNode (in <button>)
            )
    }
}
```

```rust
impl Beam for Counter {
    fn register_reactivities(
        self,
        reactivities: &mut Reactivities,
        basepath: NodePath,
    ) -> Reactivities {
        let count = Signal::new(self.initial_count);
        
        let handle_increment_click = |_| {
            count.set_with_mut(|c| *c += 1);
        };
        let handle_decrement_click = |_| {
            count.set_with_mut(|c| *c -= 1);
        };
        
        reactivities
            .register(
                basepath.join(const {NodePath::from_static(&[0, 0, 0])}), // TextNode (in <p>)
                Reactivity::Text(move || {count.get()})
            );
        (CounterButton { onclick: handle_increment_click })
            .register_reactivities(
                reactivities,
                basepath.join(const {NodePath::from_static(&[0, 0, 1])}), // CounterButton (1st)
            );
        (CounterButton { onclick: handle_decrement_click })
            .register_reactivities(
                reactivities,
                basepath.join(const {NodePath::from_static(&[0, 0, 1, 1])}), // CounterButton (2nd)
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
                    let mut reactivities = Reactivities::new();
                    
                    <Counter as ::uibeam::Beam>::register_reactivities(
                        this,
                        &mut reactivities,
                        NodePath::root()
                    );
                    
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
  for (const [name, f] of Object.entries(items)) {
    if (name.startsWith('__hydrate') && typeof f === 'function') {
      f();
    }
  }
})();
```

### Conditional/Iterative Rendering

Builtin components: `If` and `For`

#### Example Source

```rust
struct TodoList {
    user_is_ken: bool,
    items: Vec<String>,
}
impl Beam for TodoList {
    fn render(self) -> UI {
        let items = Signal::new(self.items);
        
        let handle_click_add = |_| {
            items.set_with_mut(|vec| vec.push("TODO".to_string()));
        };
        
        UI! {
            <p>"If/For example"</p>
            // `<If {expr:bool}>`: special syntax just for `If` builtin component
            <If {self.user_is_ken}>
                <p>"Hello, Ken!"</p>
            </If>
            <ul>
                // `<For {expr in expr:IntoIterator}>`: special syntax just for `For` builtin component
                <For {item in items} key={item}>
                    <li>{item}</li>
                </For>
            </ul>
            <button onclick={handle_click_add}>
                "+"
            </button>
        }
    }
}
```

#### `cfg(not(hydrate))`

```rust
impl Beam for TodoList {
    fn render(self) -> UI {
        // dummy signal just for static template rendering
        let items = Signal::new(self.items);
        
        let handle_click_add = |_| {
            items.set_with_mut(|vec| vec.push("TODO".to_string()));
        };
        
        if false {
            const fn assert_eventhandler<E>(f: impl Fn(E)) {}
            const _: () = {
                assert_eventhandler::<::uibeam::client::PointerEvent>(handle_click_add);
            };
        }
        unsafe {
            UI::new_unchecked(
                ["<p>If/For example</p>", "<ul>", "</ul><button>+</button>"],
                [
                    Dynamic::UI({
                        let content: UI = if self.user_is_ken {
                            unsafe {UI::new_unchecked(["<p>Hello, Ken!</p>"], [])}
                        } else {
                            UI::EMPTY
                        };
                        UI! {
                            <uibeam-if> // hydration marker
                            <template shadowrootmode="open">
                                unsafe {::uibeam::shoot(content)}
                            </template>
                            </uibeam-if>
                        }
                    }),
                    Dynamic::Children({
                        let content: UI = std::iter::IntoIterator::into_iter(items).map(|item| UI! {
                            <li>{item}</li>
                        }).collect();
                        UI! {
                            <uibeam-for> // hydration marker
                            <template shadowrootmode="open">
                                unsafe {::uibeam::shoot(content)}
                            </template>
                            </uibeam-for>
                        }
                    })
                ]
            )
        }
    }
}
```

#### `cfg(hydrate)`

```rust
impl Beam for TodoList {
    fn register_reactivities(
        self,
        reactivities: &mut Reactivities,
        basepath: NodePath,
    ) -> Reactivities {
        let items = Signal::new(self.items);
        
        let handle_click_add = |_| {
            items.set_with_mut(|vec| vec.push("TODO".to_string()));
        };
        
        reactivities
            .register(
                basepath.join(const {NodePath::from_static(&[0,1])}), // If
                Reactivity::If {
                    condition_fn: move || -> bool {self.user_is_ken},
                    render_fn: move || UI! {
                        <p>"Hello, Ken!"</p>
                    },
                },
            )
            .register(
                basepath.join(const {NodePath::from_static(&[0, 1, 1, 0])}), // For
                Reactivity::For {
                    iterator_fn: move || {items},
                    key_fn: Some(move |item| {item}),
                    render_fn: move |item| UI! {
                        <li>{item}</li>
                    }
                }
            );
    }
}
```
