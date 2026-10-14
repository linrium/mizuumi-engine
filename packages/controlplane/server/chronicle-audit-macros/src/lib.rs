use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    Expr, ItemFn, LitStr, Token,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
};

struct AuditArgs {
    action: LitStr,
    principal: Option<Expr>,
    fields: Vec<AuditField>,
}

struct AuditField {
    key: syn::Ident,
    value: Expr,
}

impl Parse for AuditField {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let key: syn::Ident = input.parse()?;
        input.parse::<Token![=]>()?;
        let value: Expr = input.parse()?;
        Ok(AuditField { key, value })
    }
}

impl Parse for AuditArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut action: Option<LitStr> = None;
        let mut principal: Option<Expr> = None;
        let mut fields: Vec<AuditField> = Vec::new();

        while !input.is_empty() {
            let ident: syn::Ident = input.parse()?;
            match ident.to_string().as_str() {
                "action" => {
                    input.parse::<Token![=]>()?;
                    action = Some(input.parse()?);
                }
                "principal" => {
                    input.parse::<Token![=]>()?;
                    principal = Some(input.parse()?);
                }
                "fields" => {
                    let content;
                    syn::parenthesized!(content in input);
                    let parsed: Punctuated<AuditField, Token![,]> =
                        content.parse_terminated(AuditField::parse, Token![,])?;
                    fields.extend(parsed.into_iter());
                }
                other => {
                    return Err(syn::Error::new(
                        ident.span(),
                        format!("unknown chronicle audit argument `{other}`"),
                    ));
                }
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        let action = action.ok_or_else(|| {
            syn::Error::new(
                proc_macro2::Span::call_site(),
                "missing required `action = \"...\"` argument",
            )
        })?;

        Ok(AuditArgs {
            action,
            principal,
            fields,
        })
    }
}

#[proc_macro_attribute]
pub fn audit(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AuditArgs);
    let mut func = parse_macro_input!(item as ItemFn);

    let AuditArgs {
        action,
        principal,
        fields,
    } = args;

    let field_setup: Vec<TokenStream2> = fields
        .iter()
        .map(|f| {
            let key = f.key.to_string();
            let expr = &f.value;
            quote! {
                (
                    ::std::string::String::from(#key),
                    ::std::format!("{}", &(#expr)),
                )
            }
        })
        .collect();

    let principal_setup = match principal {
        Some(expr) => quote! {
            ::std::option::Option::Some(::std::format!("{}", &(#expr)))
        },
        None => quote! { ::std::option::Option::<::std::string::String>::None },
    };

    let is_async = func.sig.asyncness.is_some();
    let original_block = func.block.clone();

    let body_wrap = if is_async {
        quote! {
            let __audit_result = async move #original_block.await;
        }
    } else {
        quote! {
            let __audit_result = (|| #original_block)();
        }
    };

    let new_body: TokenStream2 = quote! {
        {
            let __audit_action: &'static str = #action;
            let __audit_principal: ::std::option::Option<::std::string::String> = #principal_setup;
            let __audit_fields: ::std::vec::Vec<(::std::string::String, ::std::string::String)> =
                ::std::vec![#(#field_setup),*];
            let __audit_started = ::chronicle_audit::__rt::now();

            #body_wrap

            let (__audit_status, __audit_error): (&'static str, ::std::option::Option<::std::string::String>) =
                ::chronicle_audit::__rt::outcome(&__audit_result);

            ::chronicle_audit::__rt::record(::chronicle_audit::AuditEvent {
                action: ::std::string::String::from(__audit_action),
                principal: __audit_principal,
                fields: __audit_fields,
                started_at: __audit_started,
                finished_at: ::chronicle_audit::__rt::now(),
                status: ::std::string::String::from(__audit_status),
                error: __audit_error,
            });

            __audit_result
        }
    };

    func.block = syn::parse2(new_body).expect("failed to build chronicle audit function body");

    quote!(#func).into()
}
