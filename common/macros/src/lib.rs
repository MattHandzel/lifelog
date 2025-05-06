// src/lib.rs
use lifelog_core::LifelogMacroMetaDataType;
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use serde_json::json;
use std::{env, fs::OpenOptions, io::Write, path::PathBuf};
use syn::{
    parse::Parse, parse::ParseStream, parse_macro_input, parse_quote, Fields, Ident, Item,
    ItemEnum, ItemStruct, Result,
};

struct MacroOptions {
    datatype: LifelogMacroMetaDataType,
}

impl Parse for MacroOptions {
    fn parse(input: ParseStream) -> Result<Self> {
        let type_ident: Ident = input.parse()?;
        let datatype = match type_ident.to_string().as_str() {
            "Config" => LifelogMacroMetaDataType::Config,
            "Data" => LifelogMacroMetaDataType::Data,
            _ => LifelogMacroMetaDataType::None,
        };
        Ok(MacroOptions { datatype })
    }
}

#[proc_macro_attribute]
pub fn lifelog_type(attr: TokenStream, item: TokenStream) -> TokenStream {
    let options = parse_macro_input!(attr as MacroOptions);
    let original = item.clone();
    let ast = parse_macro_input!(item as Item);

    // build metadata
    let (ident_str, fields_meta, variants_meta) = match &ast {
        Item::Struct(s) => {
            let f = if let Fields::Named(named) = &s.fields {
                named
                    .named
                    .iter()
                    .filter_map(|f| {
                        f.ident.as_ref().map(|i| {
                            let ty = f.ty.to_token_stream().to_string().replace(' ', "");
                            (i.to_string(), ty)
                        })
                    })
                    .collect()
            } else {
                Vec::new()
            };
            (s.ident.to_string(), f, Vec::new())
        }
        Item::Enum(e) => {
            let v = e.variants.iter().map(|v| v.ident.to_string()).collect();
            (e.ident.to_string(), Vec::new(), v)
        }
        _ => {
            return syn::Error::new_spanned(&ast, "only structs or enums")
                .to_compile_error()
                .into();
        }
    };

    let meta = json!({
        "ident": ident_str,
        "fields": fields_meta,
        "variants": variants_meta,
        "metadata_type": match options.datatype {
            LifelogMacroMetaDataType::Config => "Config",
            LifelogMacroMetaDataType::Data   => "Data",
            LifelogMacroMetaDataType::None   => "None",
        }
    });

    let out_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(out_dir.join(format!(".{}.type.json", meta["ident"].as_str().unwrap())))
        .unwrap();
    writeln!(file, "{}", meta).unwrap();

    // re-emit with struct impls or verbatim enum
    let expanded = match ast {
        Item::Struct(mut s) => {
            // named fields only
            let named = if let Fields::Named(n) = &mut s.fields {
                n
            } else {
                return syn::Error::new_spanned(&s.ident, "only named structs")
                    .to_compile_error()
                    .into();
            };

            // inject uuid/timestamp on Data
            if let LifelogMacroMetaDataType::Data = options.datatype {
                named
                    .named
                    .insert(0, parse_quote! { pub uuid: ::lifelog_core::uuid::Uuid });
                named.named.insert(
                    1,
                    parse_quote! { pub timestamp: ::lifelog_core::chrono::DateTime<::lifelog_core::chrono::Utc> },
                );
            }

            let ident = &s.ident;
            let impl_datatype = if let LifelogMacroMetaDataType::Data = options.datatype {
                quote! {
                    impl ::lifelog_core::DataType for #ident {
                        fn uuid(&self) -> ::lifelog_core::uuid::Uuid { self.uuid }
                        fn timestamp(&self) -> ::lifelog_core::chrono::DateTime<::lifelog_core::chrono::Utc> { self.timestamp }
                    }
                }
            } else {
                quote! {}
            };

            // From<proto> → Self
            let from_proto_fields = named.named.iter().map(|f| {
                let n = f.ident.as_ref().unwrap();
                if n == "uuid" {
                    quote! { uuid: ::lifelog_core::uuid::Uuid::parse_str(&p.uuid).expect("invalid uuid") }
                } else if n == "timestamp" {
                    quote! {
                        timestamp: {
                            let ts = p.timestamp.unwrap_or_default();
                            ::lifelog_core::chrono::DateTime::<::lifelog_core::chrono::Utc>::from_utc(
                                ::lifelog_core::chrono::NaiveDateTime::from_timestamp(ts.seconds, ts.nanos as u32),
                                ::lifelog_core::chrono::Utc,
                            )
                        }
                    }
                } else {
                    quote! { #n: p.#n.into() }
                }
            });
            let from_impl = quote! {
                impl From<lifelog_proto::#ident> for #ident {
                    fn from(p: lifelog_proto::#ident) -> Self {
                        #ident { #(#from_proto_fields),* }
                    }
                }
            };

            // Self → proto
            let into_proto_fields = named.named.iter().map(|f| {
                let n = f.ident.as_ref().unwrap();
                if n == "uuid" {
                    quote! { uuid: s.uuid.to_string() }
                } else if n == "timestamp" {
                    quote! {
                        timestamp: Some(::prost_types::Timestamp {
                            seconds: s.timestamp.timestamp(),
                            nanos: s.timestamp.timestamp_subsec_nanos() as i32,
                        })
                    }
                } else {
                    quote! { #n: s.#n.into() }
                }
            });
            let into_impl = quote! {
                impl From<#ident> for lifelog_proto::#ident {
                    fn from(s: #ident) -> Self {
                        lifelog_proto::#ident { #(#into_proto_fields),* }
                    }
                }
            };

            let attrs = &s.attrs;
            let vis = &s.vis;
            let gens = &s.generics;
            let fields = &s.fields;
            let where_clause = &s.generics.where_clause;
            quote! {
                #(#attrs)*
                #vis struct #ident #gens #fields #where_clause

                #impl_datatype
                #from_impl
                #into_impl
            }
        }
        Item::Enum(_) => {
            // emit enums verbatim
            proc_macro2::TokenStream::from(original)
        }
        _ => unreachable!(),
    };

    expanded.into()
}
