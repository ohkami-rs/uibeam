use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{ItemImpl, ItemStruct, LitStr};

pub(super) fn expand(args: TokenStream, input: TokenStream) -> syn::Result<TokenStream> {
    let is_island_boundary = args.parse::<Ident>().is_ok_and(|i| i == "island");
    let impl_beam = syn::parse2::<ItemImpl>(input)?;

    let self_ty = impl_beam.self_ty.as_ref();
    let self_name = match self_ty {
        syn::Type::Path(p) => &p.path.segments.last().unwrap().ident,
        _ => {
            return Err(syn::Error::new(
                self_ty.span(),
                "unsupported type for `Beam` impl block",
            ));
        }
    };
    let hydrater_name = format_ident!("__uibeam_hydrate_{self_name}__");
    let hydrater_name_str = LitStr::new(&hydrater_name.to_string(), hydrater_name.span());

    let impl_beam = {
        let mut impl_beam = impl_beam;

        let Some(ImplItem::Fn(fn_render)) = impl_beam.first_mut() else {
            return Err(syn::Error::new(
                impl_beam.span(),
                "wrong impl block for `Beam`",
            ));
        };

        let mut stmts = fn_render.block.stmts.clone();
        insert_client_directive_to_ui_macros(&mut stmts);

        fn_render.block = if island_boundary {
            parse_quote! ({
                use ::uibeam::client::ClientContext as _;

                #[cfg(hydrate)]
                return {
                    #(#stmts)*
                };

                #[cfg(not(hydrate))]
                return {
                    let props = ::uibeam::client::serialize_props(&self);
                    ::uibeam::UI! {
                        <div
                            data-uibeam-hydrater=#hydrater_name_str
                            data-uibeam-props={props}
                        ></div>
                    }
                }
            })
        } else {
            parse_quote!({
                use ::uibeam::client::ClientContext as _;
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
        #impl_beam
        #hydrater
    })
}

fn insert_client_directive_to_ui_macros(stmts: &mut [Stmt]) {
    stmts.iter_mut().for_each(walk_stmt);

    fn rewrite_macro(mac: &mut Macro) {
        if mac
            .path
            .segments
            .last()
            .map_or(false, |seg| seg.ident == "UI")
        {
            mac.tokens = {
                let mut new_tokens = TokenStream::new();
                (quote! { @client; }).to_tokens(&mut new_tokens);
                mac.tokens.to_tokens(&mut new_tokens);
                new_tokens
            };
        }
    }

    fn walk_stmt(stmt: &mut Stmt) {
        match stmt {
            Stmt::Macro(StmtMacro { mac, .. }) => rewrite_macro(mac),
            Stmt::Expr(expr) => walk_expr(expr),
            Stmt::Item(item) => walk_item(item),
            Stmt::Local(Local {
                init: Some(LocalInit { expr, diverge, .. }),
                ..
            }) => {
                walk_expr(expr);
                if let Some((_else, diverge_expr)) = diverge {
                    walk_expr(diverge_expr);
                }
            }
            Stmt::Local(Local { init: None, .. }) => (),
        }
    }

    fn walk_expr(expr: &mut Expr) {
        match expr {
            Expr::Array(ExprArray { elems, .. }) => elems.iter_mut().for_each(walk_expr),
            Expr::Assign(ExprAssign { right, .. }) => walk_expr(right),
            Expr::Async(ExprAsync { block, .. }) => block.stmts.iter_mut().for_each(walk_stmt),
            Expr::Await(ExprAwait { base, .. }) => walk_expr(base),
            Expr::Bunary(_) => (),
            Expr::Block(ExprBlock { block, .. }) => block.stmts.iter_mut().for_each(walk_stmt),
            Expr::Break(ExprBreak {
                expr: Some(expr), ..
            }) => walk_expr(expr),
            Expr::Call(ExprCall { args, .. }) => args.iter_mut().for_each(walk_expr),
            Expr::Cast(_) => (),
            Expr::Closure(ExprClosure { body, .. }) => walk_expr(body),
            Expr::Field(ExprField { base, .. }) => walk_expr(base),
            Expr::ForLoop(ExprForLoop { expr, body, .. }) => {
                walk_expr(expr);
                body.stmts.iter_mut().for_each(walk_stmt);
            }
            Expr::Group(ExprGroup { expr, .. }) => walk_expr(expr),
            Expr::If(ExprIf {
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
            Expr::Index(_) => (),
            Expr::Infer(_) => (),
            Expr::Let(ExprLet { expr, .. }) => walk_expr(expr),
            Expr::Lit(_) => (),
            Expr::Loop(ExprLoop { body, .. }) => body.stmts.iter_mut().for_each(walk_stmt),
            Expr::Macro(ExprMacro { mac, .. }) => rewrite_macro(mac),
            Expr::Match(ExprMatch { arms, .. }) => arms.iter_mut().for_each(|arm| {
                walk_expr(&mut arm.body);
            }),
            Expr::MethodCall(ExprMethodCall { args, .. }) => args.iter_mut().for_each(walk_expr),
            Expr::Paren(ExprParen { expr, .. }) => walk_expr(expr),
            Expr::Path(_) => (),
            Expr::Range(_) => (),
            Expr::RawAddr(_) => (),
            Expr::Reference(ExprReference { expr, .. }) => walk_expr(expr),
            Expr::Repeat(_) => (),
            Expr::Return(ExprReturn {
                expr: Some(expr), ..
            }) => walk_expr(expr),
            Expr::Struct(ExprStruct { fields, rest, .. }) => {
                fields.iter_mut().for_each(|field| {
                    walk_expr(&mut field.expr);
                });
                if let Some(rest_expr) = rest {
                    walk_expr(rest_expr);
                }
            }
            Expr::Try(ExprTry { expr, .. }) => walk_expr(expr),
            Expr::TryBlock(ExprTryBlock { block, .. }) => {
                block.stmts.iter_mut().for_each(walk_stmt)
            }
            Expr::Tuple(ExprTuple { elems, .. }) => elems.iter_mut().for_each(walk_expr),
            Expr::Unary(_) => (),
            Expr::Unsafe(ExprUnsafe { block, .. }) => block.stmts.iter_mut().for_each(walk_stmt),
            Expr::Verbatim(_) => (),
            Expr::While(ExprWhile { cond, block, .. }) => {
                walk_expr(cond);
                block.stmts.iter_mut().for_each(walk_stmt);
            }
            Expr::Yield(ExprYield {
                expr: Some(expr), ..
            }) => walk_expr(expr),
            _ => (),
        }

        fn walk_item(item: &mut Item) {
            match item {
                Const(_) => (),
                Enum(_) => (),
                ExternCrate(_) => (),
                Fn(ItemFn { block, .. }) => block.stmts.iter_mut().for_each(walk_stmt),
                ForeignMod(_) => (),
                Impl(ItemImpl { items, .. }) => items.iter_mut().for_each(|impl_item| {
                    if let ImplItem::Method(ImplItemMethod { block, .. }) = impl_item {
                        block.stmts.iter_mut().for_each(walk_stmt);
                    }
                }),
                Macro(_) => (),
                Mod(ItemMod {
                    content: Some(_, items),
                    ..
                }) => items.iter_mut().for_each(walk_item),
                Static(ItemStatic { expr, .. }) => walk_expr(expr),
                Struct(_) => (),
                Trait(ItemTrait { items, .. }) => items.iter_mut().for_each(|trait_item| {
                    if let TraitItem::Method(TraitItemMethod {
                        default: Some(block),
                        ..
                    }) = trait_item
                    {
                        block.stmts.iter_mut().for_each(walk_stmt);
                    }
                }),
                TraitAlias(_) => (),
                Type(_) => (),
                Union(_) => (),
                Use(_) => (),
                Verbatim(_) => (),
                _ => (),
            }
        }
    }
}
