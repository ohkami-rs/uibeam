mod parse;

use proc_macro2::TokenStream;
use quote::quote;

pub(super) fn expand(
    args: TokenStream,
    input: TokenStream,
) -> syn::Result<TokenStream> {
    let parse::Args { local, } = syn::parse2(args)?;
    let input: syn::ItemStruct = syn::parse2(input)?;

    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    Ok(quote! {
        #[cfg_attr(
            target_arch = "wasm32",
            wasm_bindgen::prelude::wasm_bindgen
        )]
        #input

        impl #impl_generics ::uibeam::Beam for #name #ty_generics
            #where_clause
        {
            fn render(self) -> ::uibeam::UI {
                let name = ::std::any::type_name::<Self>();

                #[cfg(not(target_arch = "wasm32"))] {
                    if false {
                        fn is_laser<T: ::uibeam::Laser>()
                        is_laser::<#name #ty_generics>();
                    }

                    let props: String =
                        ::uibeam::serialize_json(&self);

                    let template: ::std::borrow::Cow<'static, str> =
                        ::uibeam::shoot(<Self as ::uibeam::Laser>::render(self));

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
if (!window.__uibeam_lock__) {
    // based on single-threaded nature of JS
    window.__uibeam_lock__ = true;
    const { default: init, ..lasers } = await import('./pkg/lasers.js');
    await init();
    window.__uibeam_lasers__ = lasers;
} else {
    while (!window.__uibeam_lasers__) {
        await new Promise(resolve => setTimeout(resolve, 100));
    }
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

                #[cfg(target_arch = "wasm32")] {
                    unreachable!();
                }
            }
        }
    })
}
