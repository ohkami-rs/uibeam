//! A collection of runtime JS functions defined to batch FFI calls for better performance.

use wasm_bindgen::prelude::*;

#[wasm_bindgen(inline_js = r#"
export function collectNodes(root, paths) {
    return paths.map((path) => {
        let node = root;
        for (let i=0; i < path.length; i++) {
            node = (path.charCodeAt(i) === 48) ? node.firstChild : node.nextSibling;
        }
        return node;
    })
}

const delegatedEventTypesRegitry = new WeakMap();
export function delegateEvent(root, eventType, wasmCallbackByID) {
    let delegatedEventTypes = delegatedEventTypesRegitry.get(root);
    if (!delegatedEventTypes) {
        delegatedEventTypes = new Set();
        delegatedEventTypesRegitry.set(root, delegatedEventTypes);
    }
    if (delegatedEventTypes.has(eventType)) return;
    
    delegatedEventTypes.add(eventType);
    root.addEventListener(eventType, (event) => {
        let propagationStopped = false;
        const originalStopPropagation = event.stopPropagation;
        event.stopPropagation = function() {
            propagationStopped = true;
            originalStopPropagation.call(this);
        }
        const originalStopImmediatePropagation = event.stopImmediatePropagation;
        event.stopImmediatePropagation = function() {
            propagationStopped = true;
            originalStopImmediatePropagation.call(this);
        }
        
        const eventidName = `data-uibeam-${eventType}-id`;
        let target = event.target;
        while (target && target !== root.parentNode) {
            const eventid = target.getAttribute(eventidName);
            if (eventid) {
                Object.defineProperty(event, 'currentTarget', {
                    configurable: true,
                    value: target,
                });
                
                try {
                    wasmCallbackByID(parseInt(eventid), event);
                } catch (err) {
                    console.error(`[uibeam] on${eventType} failed: ${err}`);
                } finally {
                    delete event.currentTarget;
                    if (propagationStopped) {
                        return;
                    }
                }
            }
            if (target === root) break;
            target = target.parentNode;
        }
    })
}

export function registerIsland(tagName, hydrateByWasm) {
    if (!customElements.get(tagName)) {
        customElements.define(tagName, class extends HTMLElement {
            constructor() {
                super();
                this.cleanup = null;
            }
            connectedCallback() {
                try {
                    this.cleanup = hydrateByWasm(this, this.getAttribute('props') || '{}');
                } catch (err) {
                    console.error(`[uibeam] <${tagName}> hydration failed: ${err}`);
                }
            }
            disconnectedCallback() {
                if (this.cleanup) {
                    this.cleanup();
                    this.cleanup = null;
                }
            }
        });
    }
}
"#)]
extern "C" {
    /// ## Params
    /// 
    /// - `root`: Island root `Node`.
    /// - `paths`: A list of encoded DOM paths from `root`.
    /// 
    /// The encoded DOM path is a `string` of format `(0|1)+` where:
    /// 
    /// - `0` means `.firstChild`
    /// - `1` means `.nextSibling`
    /// 
    /// ## Returns
    /// 
    /// A list of `Node`s corresponded with the `paths`.
    #[wasm_bindgen(js_name = collectNodes)]
    pub fn collect_nodes(root: web_sys::Node, paths: Vec<&'static str>) -> Vec<web_sys::Node>;
    
    /// ## Params
    /// 
    /// - `root`: Island root `Node`.
    /// - `event_type`: An event type to delegate (`string`) (e.g., `click`).
    /// - `wasm_callback_by_id`: A function that identify the wasm callback function by the `u32` id and call it with the `Event`.
    /// 
    /// The `wasm_callback_by_id` signature is:
    /// 
    /// ```txt
    /// (u32, web_sys::Event) -> ()
    /// ```
    #[wasm_bindgen(js_name = delegateEvent)]
    pub fn delegate_event(root: web_sys::Node, event_type: &'static str, wasm_callback_by_id: js_sys::Function);
    
    /// ## Params
    /// 
    /// - `tag_name`: The custom element name.
    /// - `render_by_wasm`: 
    /// 
    /// ```txt
    /// (root: web_sys::Node, serialized_props: String) -> (() -> ())
    /// /* returns cleanup */
    /// ```
    #[wasm_bindgen(js_name = registerIsland)]
    pub fn register_island(tag_name: &'static str, render_by_wasm: js_sys::Function);
}
