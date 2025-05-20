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
        #[wasm_bindgen::prelude::wasm_bindgen]
        #input

        impl #impl_generics ::uibeam::Beam for #name #ty_generics
            #where_clause
        {
            fn render(self) -> ::uibeam::UI {
                let name = ::std::any::type_name::<Self>();

                #[cfg(not(target_arch = "wasm32"))] {
                    let props = ::uibeam::serialize_json(self);

                    ::uibeam::UI! {
                        <div
                            data-uibeam-laser={name}
                            data-uibeam-props={props}
                            /* `data-uibeam-hydrated` must be attached after hydrated on client side */
                        >
                            <script type="module">
                                r#"const name = '"#{}r#"';"#

                                todo!()
                            </script>
                        </div>
                    }
                }
                
                #[cfg(target_arch = "wasm32")] {

                }
            }
        }
    })
}
