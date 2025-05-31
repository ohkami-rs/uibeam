#![cfg(feature = "laser")]

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{ItemStruct, LitStr};

pub(super) fn expand(
    args: TokenStream,
    input: TokenStream,
) -> syn::Result<TokenStream> {
    let local = syn::parse2::<syn::Ident>(args).is_ok_and(|i| i == "local");
    let input: ItemStruct = syn::parse2(input)?;

    let name = &input.ident;
    let hydrater_name = format_ident!("__uibeam_laser_{name}__");
    let hydrater_name_str = LitStr::new(&hydrater_name.to_string(), hydrater_name.span());

    let attribute_marker_impl = {
        quote! {
            impl ::uibeam::laser::Laser_attribute for #name {}
        }
    };

    let hydrater = (!local).then(|| quote! {
        #[cfg(target_arch = "wasm32")]
        #[doc(hidden)]
        #[allow(unused)]
        pub mod #hydrater_name {
            use super::#name;
            use ::uibeam::laser::wasm_bindgen;
            use ::uibeam::laser::wasm_bindgen::{JsCast, UnwrapThrowExt};

            #[cfg(target_arch = "wasm32")]
            #[doc(hidden)]
            #[allow(non_snake_case)]
            #[wasm_bindgen::prelude::wasm_bindgen]
            pub fn #hydrater_name(
                props: ::uibeam::laser::js_sys::Object,
                container: ::uibeam::laser::web_sys::Node
            ) {// TODO: not execute hydration when the container is not displayed in window
                ::uibeam::laser::hydrate(
                    ::uibeam::laser::VNode::new(
                        ::uibeam::laser::NodeType::component::<#name>(),
                        props,
                        vec![]
                    ),
                    container
                )
            }
        }
    });
// TODO: reject local lasers outside of `#[Laser]` **by compile-time check**
//       instead of runtime `panic!` (memo: trait Component :> Beam ; <- children)
    let beam_impl = if local {
        quote! {
            impl ::uibeam::Beam for #name {
                fn render(self) -> ::uibeam::UI {
                    #[cfg(target_arch = "wasm32")] {
                        unreachable!();
                    }

                    #[cfg(not(target_arch = "wasm32"))] {
                        panic!("`#[Laser(local)]` can NOT be used outside of a `#[Laser]`")
                    }
                }
            }
        }
    } else {
        quote! {
            impl ::uibeam::Beam for #name
            where
                Self: ::uibeam::laser::serde::Serialize + for<'de> ::uibeam::laser::serde::Deserialize<'de>,
            {
                fn render(self) -> ::uibeam::UI {
                    #[cfg(target_arch = "wasm32")] {
                        unreachable!();
                    }

                    #[cfg(not(target_arch = "wasm32"))] {
                        let props: String = ::uibeam::laser::serialize_props(&self);

                        let template: ::std::borrow::Cow<'static, str> = ::uibeam::shoot(<Self as Laser>::render(self));

                        ::uibeam::UI! {
                            <div
                                data-uibeam-laser=#hydrater_name_str
                            >
                                unsafe {template}

                                <script type="module">
r#"const name = '"#
#hydrater_name_str
r#"';"#
r#"const props = JSON.parse('"#
unsafe {props}
r#"');"#
r#"
const container = document.querySelector(`div[data-uibeam-laser=${name}]:not([data-uibeam-laser-hydrated])`);
if (window.__uibeam_initlock__) {
    for (let i=0; i<42 && !window.__uibeam_lasers__; i++) await new Promise(resolve => setTimeout(resolve, 100));
    if (!window.__uibeam_lasers__) {
        container.setAttribute('data-uibeam-laser', `${name}@FAILED`);
        throw new Error(`/.uibeam/lasers.js` is not loaded yet, please check your network connection or the server configuration.`);
    }
} else {
    window.__uibeam_initlock__ = true;

    const importMap = document.createElement('script');
    importMap.type = 'importmap';
    importMap.textContent = `{"imports": {
        "preact": "https://esm.sh/preact",
        "preact/hooks": "https://esm.sh/preact/hooks?external=preact",
        "@preact/signals": "https://esm.sh/@preact/signals?external=preact"
    }}`;
    document.head.appendChild(importMap);

    const { default: init, ...lasers } = await import('/.uibeam/lasers.js');
    await init();
    window.__uibeam_lasers__ = lasers;
}
container.setAttribute('data-uibeam-laser-hydrated', '');
(window.__uibeam_lasers__[name])(props, container);
"#
                                </script>
                            </div>
                        }
                    }
                }
            }
        }
    };

    Ok(quote! {
        #input

        const _: () = {
            use ::uibeam::laser::wasm_bindgen;
            #[wasm_bindgen::prelude::wasm_bindgen]
            #[::uibeam::consume]
            #input
        };

        #attribute_marker_impl

        #hydrater

        #beam_impl
    })
}
