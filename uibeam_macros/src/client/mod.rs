use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{ItemStruct, LitStr};

pub(super) fn expand(args: TokenStream, input: TokenStream) -> syn::Result<TokenStream> {
    let local = syn::parse2::<syn::Ident>(args).is_ok_and(|i| i == "local");
    let input: ItemStruct = syn::parse2(input)?;

    let name = &input.ident;
    let hydrater_name = format_ident!("__uibeam_laser_{name}__");
    let hydrater_name_str = LitStr::new(&hydrater_name.to_string(), hydrater_name.span());

    let attribute_marker_impl = {
        quote! {
            impl ::uibeam::laser::Laser_attribute for #name where Self: ::uibeam::Laser {}
        }
    };

    let hydrater = (!local).then(|| {
        quote! {
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
                ) {
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
        }
    });
    // TODO: reject local lasers outside of `#[Laser]` **by compile-time check**
    //       instead of silently not-hydrated
    let beam_impl = if local {
        quote! {
            impl ::uibeam::Beam for #name {
                fn render(self) -> ::uibeam::UI {
                    #[cfg(target_arch = "wasm32")] {
                        unimplemented!("Sorry, Laser is currently not supported on WASM host!");
                    }

                    #[cfg(not(target_arch = "wasm32"))] {
                        <Self as Laser>::render(self)
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
                                unimplemented!("Sorry, Laser is currently not supported on WASM host!");
                            }

                            #[cfg(not(target_arch = "wasm32"))] {
                                let props: String = ::uibeam::laser::serialize_props(&self);
                                let template: ::std::borrow::Cow<'static, str> = ::uibeam::shoot(<Self as Laser>::render(self));

        // TODO: control hydration flow based on visibility on screen (e.g. by `IntersectionObserver`)
                                ::uibeam::UI! {
                                    <div data-uibeam-laser=#hydrater_name_str>
                                        unsafe {template}

                                        <script type="module">
        r#"const name = '"# #hydrater_name_str r#"';"#
        r#"const props = JSON.parse('"# unsafe {props} r#"');"#
r#"
const container = document.querySelector(`[data-uibeam-laser='${name}']:not([data-uibeam-hydration-status])`);
container.setAttribute('data-uibeam-hydration-status', 'INIT');
if (window.__uibeam_initlock__) {
    container.setAttribute('data-uibeam-hydration-status', 'PENDING');
    for (let i=0; i<50 && !window.__uibeam_lasers__; i++) await new Promise(resolve => setTimeout(resolve, 100));
    if (!window.__uibeam_lasers__) {
        container.setAttribute('data-uibeam-hydration-status', 'FAILED');
        throw('`/.uibeam/lasers.js` is not loaded yet. Please check your network connection or the server configuration.');
    }
} else {
    container.setAttribute('data-uibeam-hydration-status', 'LOADING');
    window.__uibeam_initlock__ = true;
    const importmap = document.createElement('script');
    importmap.type = 'importmap';
    importmap.textContent = '{"imports": {"preact": "https://esm.sh/preact", "preact/hooks": "https://esm.sh/preact/hooks?external=preact", "@preact/signals": "https://esm.sh/@preact/signals?external=preact"}}';
    document.head.appendChild(importmap);
    try {
        const { default: init, ...lasers } = await import('/.uibeam/lasers.js');
        await init();
        window.__uibeam_lasers__ = lasers;
    } catch (e) {
        container.setAttribute('data-uibeam-hydration-status', 'FAILED');
        throw(`Failed to load lasers: ${e}`);
    }
}
(window.__uibeam_lasers__[name])(props, container);
container.setAttribute('data-uibeam-hydration-status', 'DONE');
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

        #attribute_marker_impl

        #hydrater

        #beam_impl
    })
}
