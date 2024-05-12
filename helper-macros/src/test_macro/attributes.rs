use proc_macro::{TokenStream, TokenTree};

pub struct Attributes {
    pub servers: u16,
    pub config: String,
}

pub fn parse(tokens: TokenStream) -> Result<Attributes, String> {
    let mut servers = None;
    let mut config = None;
    let mut iter = tokens.into_iter();
    while let Some(token) = iter.next() {
        if let TokenTree::Ident(ident) = token {
            if ident.to_string() == "servers" {
                if let Some(TokenTree::Punct(punct)) = iter.next() {
                    if punct.as_char() == '=' {
                        if let Some(TokenTree::Literal(literal)) = iter.next() {
                            servers =
                                Some(literal.to_string().parse().map_err(|e| {
                                    format!("could not parse servers attribute: {}", e)
                                })?)
                        }
                    }
                }
            } else if ident.to_string() == "config" {
                if let Some(TokenTree::Punct(punct)) = iter.next() {
                    if punct.as_char() == '=' {
                        if let Some(TokenTree::Ident(ident)) = iter.next() {
                            config = Some(ident.to_string());
                        }
                    }
                }
            }
        }
    }
    Ok(Attributes {
        servers: servers.ok_or("servers attribute not found")?,
        config: config.ok_or("config attribute not found")?,
    })
}
