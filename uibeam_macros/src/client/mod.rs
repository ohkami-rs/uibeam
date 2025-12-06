use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_quote, spanned::Spanned};

pub(super) fn expand(args: TokenStream, input: TokenStream) -> syn::Result<TokenStream> {
    let is_island_boundary = syn::parse2::<syn::Ident>(args).is_ok_and(|i| i == "island");
    let impl_beam = syn::parse2::<syn::ItemImpl>(input)?;

    let self_ty = impl_beam.self_ty.clone();
    let self_name = match &*self_ty {
        syn::Type::Path(p) => &p.path.segments.last().unwrap().ident,
        _ => {
            return Err(syn::Error::new(
                self_ty.span(),
                "unsupported type for `Beam` impl block",
            ));
        }
    };
    let hydrater_name = format_ident!("__uibeam_hydrate_{self_name}__");
    let hydrater_name_str = syn::LitStr::new(&hydrater_name.to_string(), hydrater_name.span());

    let impl_island_boundary = is_island_boundary.then(|| {
        let (impl_generics, ty_generics, where_clause) = impl_beam.generics.split_for_impl();
        quote! {
            impl #impl_generics ::uibeam::IslandBoundary for #self_ty #ty_generics #where_clause {}
        }
    });
    
    let impl_beam = {
        let mut impl_beam = impl_beam;

        let Some((_, beam_trait, _)) = impl_beam.trait_.as_mut() else {
            return Err(syn::Error::new(
                impl_beam.span(),
                "wrong impl block for `Beam`",
            ));
        };

        let beam_trait = beam_trait.segments.last_mut().unwrap(/* trait_ is Some */);

        if !beam_trait.arguments.is_none() {
            return Err(syn::Error::new(
                beam_trait.arguments.span(),
                "wrong impl block for `Beam`",
            ));
        }

        beam_trait.arguments = syn::PathArguments::AngleBracketed(parse_quote! {
            <::uibeam::Client>
        });

        let Some(syn::ImplItem::Fn(fn_render)) = impl_beam.items.first_mut() else {
            return Err(syn::Error::new(
                impl_beam.span(),
                "wrong impl block for `Beam`",
            ));
        };

        let mut stmts = fn_render.block.stmts.clone();
        insert_client_directive_to_ui_macros(&mut stmts);

        fn_render.block = if is_island_boundary {
            parse_quote! ({
                use ::uibeam::client_attribute as _;

                #[cfg(hydrate)]
                return {
                    #(#stmts)*
                };

                #[cfg(not(hydrate))]
                return {
                    let props = ::uibeam::client::serialize_props(&self);
                    let dry_ui = { #(#stmts)* };
                    ::uibeam::UI! {
                        <div
                            data-uibeam-hydrater=#hydrater_name_str
                            data-uibeam-props={props}
                        >
                            {dry_ui}
                        </div>
                    }
                }
            })
        } else {
            parse_quote!({
                use ::uibeam::client_attribute as _;
                #(#stmts)*
            })
        };

        impl_beam
    };

    let hydrater = is_island_boundary.then(|| {
        quote! {
            #[cfg(hydrate)]
            #[doc(hidden)]
            #[allow(unused, non_snake_case)]
            pub mod #hydrater_name {
                use super::#self_name;
                use ::uibeam::client::wasm_bindgen;
                use ::uibeam::client::wasm_bindgen::{JsCast, UnwrapThrowExt};

                #[cfg(hydrate)]
                #[doc(hidden)]
                #[allow(unused, non_snake_case)]
                #[wasm_bindgen::prelude::wasm_bindgen]
                pub fn #hydrater_name(
                    props: ::uibeam::client::js_sys::Object,
                    container: ::uibeam::client::web_sys::Node,
                ) {
                    ::uibeam::client::hydrate(
                        ::uibeam::client::VNode::new(
                            ::uibeam::client::NodeType::component::<#self_name>(),
                            props,
                            vec![],
                        ),
                        container,
                    )
                }
            }
        }
    });

    Ok(quote! {
        #impl_island_boundary
        #impl_beam
        #hydrater
    })
}

fn insert_client_directive_to_ui_macros(stmts: &mut [syn::Stmt]) {
    stmts.iter_mut().for_each(walk_stmt);

    fn rewrite_macro(mac: &mut syn::Macro) {
        if mac
            .path
            .segments
            .last()
            .is_some_and(|seg| seg.ident == "UI")
        {
            mac.tokens = {
                let mut new_tokens = TokenStream::new();
                {
                    use quote::ToTokens;
                    (quote! { @client; }).to_tokens(&mut new_tokens);
                    mac.tokens.to_tokens(&mut new_tokens);
                }
                new_tokens
            };
        }
    }

    fn walk_stmt(stmt: &mut syn::Stmt) {
        match stmt {
            syn::Stmt::Macro(syn::StmtMacro { mac, .. }) => rewrite_macro(mac),
            syn::Stmt::Expr(expr, _) => walk_expr(expr),
            syn::Stmt::Item(item) => walk_item(item),
            syn::Stmt::Local(syn::Local {
                init: Some(syn::LocalInit { expr, diverge, .. }),
                ..
            }) => {
                walk_expr(expr);
                if let Some((_else, diverge_expr)) = diverge {
                    walk_expr(diverge_expr);
                }
            }
            syn::Stmt::Local(syn::Local { init: None, .. }) => (),
        }
    }

    fn walk_expr(expr: &mut syn::Expr) {
        match expr {
            syn::Expr::Array(syn::ExprArray { elems, .. }) => elems.iter_mut().for_each(walk_expr),
            syn::Expr::Assign(syn::ExprAssign { right, .. }) => walk_expr(right),
            syn::Expr::Async(syn::ExprAsync { block, .. }) => {
                block.stmts.iter_mut().for_each(walk_stmt)
            }
            syn::Expr::Await(syn::ExprAwait { base, .. }) => walk_expr(base),
            syn::Expr::Binary(_) => (),
            syn::Expr::Block(syn::ExprBlock { block, .. }) => {
                block.stmts.iter_mut().for_each(walk_stmt)
            }
            syn::Expr::Break(syn::ExprBreak {
                expr: Some(expr), ..
            }) => walk_expr(expr),
            syn::Expr::Call(syn::ExprCall { args, .. }) => args.iter_mut().for_each(walk_expr),
            syn::Expr::Cast(_) => (),
            syn::Expr::Closure(syn::ExprClosure { body, .. }) => walk_expr(body),
            syn::Expr::Field(syn::ExprField { base, .. }) => walk_expr(base),
            syn::Expr::ForLoop(syn::ExprForLoop { expr, body, .. }) => {
                walk_expr(expr);
                body.stmts.iter_mut().for_each(walk_stmt);
            }
            syn::Expr::Group(syn::ExprGroup { expr, .. }) => walk_expr(expr),
            syn::Expr::If(syn::ExprIf {
                cond,
                then_branch,
                else_branch,
                ..
            }) => {
                walk_expr(cond);
                then_branch.stmts.iter_mut().for_each(walk_stmt);
                if let Some((_else, else_expr)) = else_branch {
                    walk_expr(else_expr);
                }
            }
            syn::Expr::Index(_) => (),
            syn::Expr::Infer(_) => (),
            syn::Expr::Let(syn::ExprLet { expr, .. }) => walk_expr(expr),
            syn::Expr::Lit(_) => (),
            syn::Expr::Loop(syn::ExprLoop { body, .. }) => {
                body.stmts.iter_mut().for_each(walk_stmt)
            }
            syn::Expr::Macro(syn::ExprMacro { mac, .. }) => rewrite_macro(mac),
            syn::Expr::Match(syn::ExprMatch { arms, .. }) => arms.iter_mut().for_each(|arm| {
                walk_expr(&mut arm.body);
            }),
            syn::Expr::MethodCall(syn::ExprMethodCall { args, .. }) => {
                args.iter_mut().for_each(walk_expr)
            }
            syn::Expr::Paren(syn::ExprParen { expr, .. }) => walk_expr(expr),
            syn::Expr::Path(_) => (),
            syn::Expr::Range(_) => (),
            syn::Expr::RawAddr(_) => (),
            syn::Expr::Reference(syn::ExprReference { expr, .. }) => walk_expr(expr),
            syn::Expr::Repeat(_) => (),
            syn::Expr::Return(syn::ExprReturn {
                expr: Some(expr), ..
            }) => walk_expr(expr),
            syn::Expr::Struct(syn::ExprStruct { fields, rest, .. }) => {
                fields.iter_mut().for_each(|field| {
                    walk_expr(&mut field.expr);
                });
                if let Some(rest_expr) = rest {
                    walk_expr(rest_expr);
                }
            }
            syn::Expr::Try(syn::ExprTry { expr, .. }) => walk_expr(expr),
            syn::Expr::TryBlock(syn::ExprTryBlock { block, .. }) => {
                block.stmts.iter_mut().for_each(walk_stmt)
            }
            syn::Expr::Tuple(syn::ExprTuple { elems, .. }) => elems.iter_mut().for_each(walk_expr),
            syn::Expr::Unary(_) => (),
            syn::Expr::Unsafe(syn::ExprUnsafe { block, .. }) => {
                block.stmts.iter_mut().for_each(walk_stmt)
            }
            syn::Expr::Verbatim(_) => (),
            syn::Expr::While(syn::ExprWhile { cond, body, .. }) => {
                walk_expr(cond);
                body.stmts.iter_mut().for_each(walk_stmt);
            }
            syn::Expr::Yield(syn::ExprYield {
                expr: Some(expr), ..
            }) => walk_expr(expr),
            _ => (),
        }
    }
    fn walk_item(item: &mut syn::Item) {
        match item {
            syn::Item::Const(_) => (),
            syn::Item::Enum(_) => (),
            syn::Item::ExternCrate(_) => (),
            syn::Item::Fn(syn::ItemFn { block, .. }) => block.stmts.iter_mut().for_each(walk_stmt),
            syn::Item::ForeignMod(_) => (),
            syn::Item::Impl(syn::ItemImpl { items, .. }) => {
                items.iter_mut().for_each(|impl_item| {
                    if let syn::ImplItem::Fn(syn::ImplItemFn { block, .. }) = impl_item {
                        block.stmts.iter_mut().for_each(walk_stmt);
                    }
                })
            }
            syn::Item::Macro(_) => (),
            syn::Item::Mod(syn::ItemMod {
                content: Some((_, items)),
                ..
            }) => items.iter_mut().for_each(walk_item),
            syn::Item::Static(syn::ItemStatic { expr, .. }) => walk_expr(expr),
            syn::Item::Struct(_) => (),
            syn::Item::Trait(syn::ItemTrait { items, .. }) => {
                items.iter_mut().for_each(|trait_item| {
                    if let syn::TraitItem::Fn(syn::TraitItemFn {
                        default: Some(block),
                        ..
                    }) = trait_item
                    {
                        block.stmts.iter_mut().for_each(walk_stmt);
                    }
                })
            }
            syn::Item::TraitAlias(_) => (),
            syn::Item::Type(_) => (),
            syn::Item::Union(_) => (),
            syn::Item::Use(_) => (),
            syn::Item::Verbatim(_) => (),
            _ => (),
        }
    }
}
