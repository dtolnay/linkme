use syn::parse::{Parse, ParseStream, Result};
use syn::Path;

pub enum Args {
    None,
    Path(Path),
}

impl Parse for Args {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.is_empty() {
            Ok(Args::None)
        } else {
            input.parse().map(Args::Path)
        }
    }
}
