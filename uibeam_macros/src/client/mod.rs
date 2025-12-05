use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{ItemImpl, ItemStruct, LitStr};

pub(super) fn expand(args: TokenStream, input: TokenStream) -> syn::Result<TokenStream> {
    let is_island_boundary = args.parse::<Ident>().is_ok_and(|i| i == "island");
    let mut impl_beam = syn::parse2::<ItemImpl>(input)?;

    let Some(ImplItem::Fn(fn_render)) = impl_beam.first_mut() else {
        return Err(syn::Error::new(
            impl_beam.span(),
            "wrong impl block for `Beam`",
        ));
    };

    // Avoid generating code with `#[cfg(hydrate)]` that emits warning on crate user's side.
    //
    // You may know it can be resolved by, e.g., setting `[lints.rust] unexpected_cfgs = { level = "warn", check-cfg = ["cfg(hydrate)"] }`,
    // but it requires crate user's cooperation.
    fn_render.block =
        if option_env!("RUSTFLAGS").is_some_and(|flags| flags.contains("--cfg hydrate")) {
            let mut stmts = &fn_render.block.stmts;
            parse_quote! ({
                use ::uibeam::client::ClientContext as _;
                #(#stmts)*
            })
        } else {
            parse_quote!({
                use ::uibeam::client::ClientContext as _;
                ::uibeam::UI! {
                    <div></div>
                }
            })
        };

    Ok(impl_beam.into_token_stream())
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
