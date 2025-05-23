#![cfg(feature = "laser")]

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{ItemStruct, LitStr, WhereClause};

pub(super) fn expand(
    args: TokenStream,
    input: TokenStream,
) -> syn::Result<TokenStream> {
    let local = syn::parse2::<syn::Ident>(args).is_ok_and(|i| i == "local");
    let input: ItemStruct = syn::parse2(input)?;

    let name = &input.ident;
    let hydrater_name = format_ident!("__uibeam_laser_{name}__");
    let hydrater_name_str = LitStr::new(&hydrater_name.to_string(), hydrater_name.span());

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let attribute_marker_impl = {
        //let mut where_clause = where_clause.cloned();
        //if !local {
        //    if where_clause.is_none() {
        //        where_clause = Some(WhereClause {
        //            where_token: Default::default(),
        //            predicates: Default::default(),
        //        });
        //    }
        //    where_clause.as_mut().unwrap().predicates.push(
        //        syn::parse2(quote! {
        //            Self: ::uibeam::laser::serde::Serialize,
        //        }).unwrap(),
        //    );
        //}

        quote! {
            impl #impl_generics ::uibeam::laser::Laser_attribute for #name #ty_generics
                #where_clause
            {}
        }
    };

    let hydrater = (!local).then(|| quote! {
        #[cfg(target_arch = "wasm32")]
        #[doc(hidden)]
        #[allow(non_snake_case)]
        #[::uibeam::laser::wasm_bindgen::prelude::wasm_bindgen]
        pub fn #hydrater_name #impl_generics(props: #name #ty_generics, container: ::uibeam::laser::web_sys::Node)
            #where_clause
        {
            ::uibeam::laser::hydrate(
                <#name as ::uibeam::Laser>::render(props).into_vdom(),
                container
            )
        }
    });

    let beam_impl = if local {
        quote! {
            impl<L: ::uibeam::Laser> ::uibeam::Beam for L {
                fn render(self) -> ::uibeam::UI {
                    unreachable!()
                }
            }
        }
    } else {
        quote! {
            impl<L: Laser + ::serde::Serialize> ::uibeam::Beam for L {
                fn render(self) -> ::uibeam::UI {
                    #[cfg(target_arch = "wasm32")] {
                        unreachable!();
                    }

                    #[cfg(not(target_arch = "wasm32"))] {
                        let name = #hydrater_name_str;

                        let props: String = ::uibeam::laser::serialize_props(&self);

                        let template: ::std::borrow::Cow<'static, str> = ::uibeam::shoot(<Self as Laser>::render(self));

                        ::uibeam::UI! {
                            <div
                                data-uibeam-laser={name}
                            >
                                unsafe {template}

                                <script type="module">
unsafe {format!("
const name = '{name}';
const props = JSON.parse('{props}');
")}
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
        #[::uibeam::laser::wasm_bindgen::prelude::wasm_bindgen]
        #input

        #attribute_marker_impl

        #hydrater

        #beam_impl
    })
}
