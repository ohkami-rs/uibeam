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

            #[cfg(target_arch = "wasm32")]
            #[doc(hidden)]
            #[allow(non_snake_case)]
            #[wasm_bindgen::prelude::wasm_bindgen]
            pub fn #hydrater_name(props: #name, container: ::uibeam::laser::web_sys::Node) {
                ::uibeam::laser::hydrate(
                    <#name as ::uibeam::Laser>::render(props).into_vdom(),
                    container
                )
            }
        }
    });

    let beam_impl = if local {
        quote! {
            impl ::uibeam::Beam for #name {
                fn render(self) -> ::uibeam::UI {
                    unreachable!()
                }
            }
        }
    } else {
        quote! {
            impl ::uibeam::Beam for #name
            where
                Self: ::uibeam::laser::serde::Serialize,
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
if (window.__uibeam_initlock__) {
    while (!window.__uibeam_lasers__) await new Promise(resolve => setTimeout(resolve, 100));
} else {
    window.__uibeam_initlock__ = true;
    const { default: init, ..lasers } = await import('./pkg/lasers.js');
    await init();
    window.__uibeam_lasers__ = lasers;
}
(window.__uibeam_lasers__[name])(
    props,
    document.querySelector(`[data-uibeam-laser=${name}]`)
);
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
