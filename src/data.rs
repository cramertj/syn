use super::*;
use delimited::Delimited;

ast_struct! {
    /// An enum variant.
    pub struct Variant {
        /// Attributes tagged on the variant.
        pub attrs: Vec<Attribute>,

        /// Name of the variant.
        pub ident: Ident,

        /// Content stored in the variant.
        pub fields: Fields,

        /// Explicit discriminant, e.g. `Foo = 1`
        pub discriminant: Option<(Token![=], Expr)>,
    }
}

ast_enum_of_structs! {
    /// Data stored within an enum variant or struct.
    pub enum Fields {
        /// Named fields of a struct or struct variant such as `Point { x: f64,
        /// y: f64 }`.
        pub Named(FieldsNamed {
            pub brace_token: token::Brace,
            pub fields: Delimited<Field, Token![,]>,
        }),

        /// Unnamed fields of a tuple struct or tuple variant such as `Some(T)`.
        pub Unnamed(FieldsUnnamed {
            pub paren_token: token::Paren,
            pub fields: Delimited<Field, Token![,]>,
        }),

        /// Unit struct or unit variant such as `None`.
        pub Unit,
    }
}

ast_struct! {
    /// A field of a struct or enum variant.
    pub struct Field {
        /// Attributes tagged on the field.
        pub attrs: Vec<Attribute>,

        /// Visibility of the field.
        pub vis: Visibility,

        /// Name of the field, if any.
        ///
        /// Fields of tuple structs have no names.
        pub ident: Option<Ident>,

        pub colon_token: Option<Token![:]>,

        /// Type of the field.
        pub ty: Type,
    }
}

ast_enum_of_structs! {
    /// Visibility level of an item.
    pub enum Visibility {
        /// Public, i.e. `pub`.
        pub Public(VisPublic {
            pub pub_token: Token![pub],
        }),

        /// Crate-visible, i.e. `pub(crate)`.
        pub Crate(VisCrate {
            pub pub_token: Token![pub],
            pub paren_token: token::Paren,
            pub crate_token: Token![crate],
        }),

        /// Restricted, e.g. `pub(self)` or `pub(super)` or `pub(in some::module)`.
        pub Restricted(VisRestricted {
            pub pub_token: Token![pub],
            pub paren_token: token::Paren,
            pub in_token: Option<Token![in]>,
            pub path: Box<Path>,
        }),

        /// Inherited, i.e. private.
        pub Inherited,
    }
}

#[cfg(feature = "parsing")]
pub mod parsing {
    use super::*;

    use synom::Synom;

    impl Synom for Variant {
        named!(parse -> Self, do_parse!(
            attrs: many0!(Attribute::parse_outer) >>
            id: syn!(Ident) >>
            fields: alt!(
                syn!(FieldsNamed) => { Fields::Named }
                |
                syn!(FieldsUnnamed) => { Fields::Unnamed }
                |
                epsilon!() => { |_| Fields::Unit }
            ) >>
            disr: option!(tuple!(punct!(=), syn!(Expr))) >>
            (Variant {
                ident: id,
                attrs: attrs,
                fields: fields,
                discriminant: disr,
            })
        ));

        fn description() -> Option<&'static str> {
            Some("enum variant")
        }
    }

    impl Synom for FieldsNamed {
        named!(parse -> Self, map!(
            braces!(call!(Delimited::parse_terminated_with, Field::parse_named)),
            |(brace, fields)| FieldsNamed {
                brace_token: brace,
                fields: fields,
            }
        ));
    }

    impl Synom for FieldsUnnamed {
        named!(parse -> Self, map!(
            parens!(call!(Delimited::parse_terminated_with, Field::parse_unnamed)),
            |(paren, fields)| FieldsUnnamed {
                paren_token: paren,
                fields: fields,
            }
        ));
    }

    impl Field {
        named!(pub parse_named -> Self, do_parse!(
            attrs: many0!(Attribute::parse_outer) >>
            vis: syn!(Visibility) >>
            id: syn!(Ident) >>
            colon: punct!(:) >>
            ty: syn!(Type) >>
            (Field {
                ident: Some(id),
                vis: vis,
                attrs: attrs,
                ty: ty,
                colon_token: Some(colon),
            })
        ));

        named!(pub parse_unnamed -> Self, do_parse!(
            attrs: many0!(Attribute::parse_outer) >>
            vis: syn!(Visibility) >>
            ty: syn!(Type) >>
            (Field {
                ident: None,
                colon_token: None,
                vis: vis,
                attrs: attrs,
                ty: ty,
            })
        ));
    }

    impl Synom for Visibility {
        named!(parse -> Self, alt!(
            do_parse!(
                pub_token: keyword!(pub) >>
                other: parens!(keyword!(crate)) >>
                (Visibility::Crate(VisCrate {
                    pub_token: pub_token,
                    paren_token: other.0,
                    crate_token: other.1,
                }))
            )
            |
            do_parse!(
                pub_token: keyword!(pub) >>
                other: parens!(keyword!(self)) >>
                (Visibility::Restricted(VisRestricted {
                    pub_token: pub_token,
                    paren_token: other.0,
                    in_token: None,
                    path: Box::new(other.1.into()),
                }))
            )
            |
            do_parse!(
                pub_token: keyword!(pub) >>
                other: parens!(keyword!(super)) >>
                (Visibility::Restricted(VisRestricted {
                    pub_token: pub_token,
                    paren_token: other.0,
                    in_token: None,
                    path: Box::new(other.1.into()),
                }))
            )
            |
            do_parse!(
                pub_token: keyword!(pub) >>
                other: parens!(do_parse!(
                    in_tok: keyword!(in) >>
                    restricted: call!(Path::parse_mod_style) >>
                    (in_tok, restricted)
                )) >>
                (Visibility::Restricted(VisRestricted {
                    pub_token: pub_token,
                    paren_token: other.0,
                    in_token: Some((other.1).0),
                    path: Box::new((other.1).1),
                }))
            )
            |
            keyword!(pub) => { |tok| {
                Visibility::Public(VisPublic {
                    pub_token: tok,
                })
            } }
            |
            epsilon!() => { |_| Visibility::Inherited }
        ));

        fn description() -> Option<&'static str> {
            Some("visibility qualifier, e.g. `pub`")
        }
    }
}

#[cfg(feature = "printing")]
mod printing {
    use super::*;
    use quote::{ToTokens, Tokens};

    impl ToTokens for Variant {
        fn to_tokens(&self, tokens: &mut Tokens) {
            tokens.append_all(&self.attrs);
            self.ident.to_tokens(tokens);
            self.fields.to_tokens(tokens);
            if let Some((ref eq_token, ref disc)) = self.discriminant {
                eq_token.to_tokens(tokens);
                disc.to_tokens(tokens);
            }
        }
    }

    impl ToTokens for FieldsNamed {
        fn to_tokens(&self, tokens: &mut Tokens) {
            self.brace_token.surround(tokens, |tokens| {
                self.fields.to_tokens(tokens);
            });
        }
    }

    impl ToTokens for FieldsUnnamed {
        fn to_tokens(&self, tokens: &mut Tokens) {
            self.paren_token.surround(tokens, |tokens| {
                self.fields.to_tokens(tokens);
            });
        }
    }

    impl ToTokens for Field {
        fn to_tokens(&self, tokens: &mut Tokens) {
            tokens.append_all(&self.attrs);
            self.vis.to_tokens(tokens);
            if let Some(ref ident) = self.ident {
                ident.to_tokens(tokens);
                TokensOrDefault(&self.colon_token).to_tokens(tokens);
            }
            self.ty.to_tokens(tokens);
        }
    }

    impl ToTokens for VisPublic {
        fn to_tokens(&self, tokens: &mut Tokens) {
            self.pub_token.to_tokens(tokens)
        }
    }

    impl ToTokens for VisCrate {
        fn to_tokens(&self, tokens: &mut Tokens) {
            self.pub_token.to_tokens(tokens);
            self.paren_token.surround(tokens, |tokens| {
                self.crate_token.to_tokens(tokens);
            })
        }
    }

    impl ToTokens for VisRestricted {
        fn to_tokens(&self, tokens: &mut Tokens) {
            self.pub_token.to_tokens(tokens);
            self.paren_token.surround(tokens, |tokens| {
                // XXX: If we have a path which is not "self" or "super",
                // automatically add the "in" token.
                self.in_token.to_tokens(tokens);
                self.path.to_tokens(tokens);
            });
        }
    }
}
