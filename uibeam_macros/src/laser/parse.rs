use syn::parse::{Parse, ParseStream};
use syn::{Ident, ItemStruct};

pub(super) struct Args {
    pub(super) local: bool,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut args = Args {
            local: false,
        };

        if input.peek(Ident) {
            if input.parse::<Ident>()? == "local" {
                args.local = true;
            }
        }

        if !input.is_empty() {
            return Err(input.error("unexpected input: expected `local`"));
        }

        Ok(args)
    }
}
