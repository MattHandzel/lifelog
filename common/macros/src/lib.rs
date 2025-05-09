// src/lib.rs
use lifelog_core::LifelogMacroMetaDataType;
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use serde_json::json;
use std::{env, fs::OpenOptions, io::Write, path::PathBuf};
use syn::{
    parse::Parse, parse::ParseStream, parse_macro_input, parse_quote,
    AngleBracketedGenericArguments, Fields, GenericArgument, Ident, Item, PathArguments, Result,
    Type,
};

/// `#[lifelog_type(...)]` accepts `Data`, `Config`, or else = `None`
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
    // 1) parse args + keep original tokens
    let options = parse_macro_input!(attr as MacroOptions);
    let original: proc_macro2::TokenStream = item.clone().into();
    let ast = parse_macro_input!(item as Item);

    // 2) extract ident, fields, variants
    let (ident_str, mut fields_meta, variants_meta) = match &ast {
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
                .into()
        }
    };

    // 3) inject uuid/timestamp into JSON metadata for `Data`
    if let LifelogMacroMetaDataType::Data = options.datatype {
        fields_meta.insert(
            0,
            (
                "timestamp".to_string(),
                "::lifelog_core::chrono::DateTime<::lifelog_core::chrono::Utc>".to_string(),
            ),
        );
        fields_meta.insert(
            0,
            ("uuid".to_string(), "::lifelog_core::uuid::Uuid".to_string()),
        );
    }

    // 4) write .type.json
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

    // 5) re-emit with struct impls or enum impls
    let expanded = match ast {
        Item::Struct(mut s) => {
            // only named structs
            let named = if let Fields::Named(n) = &mut s.fields {
                n
            } else {
                return syn::Error::new_spanned(&s.ident, "only named structs")
                    .to_compile_error()
                    .into();
            };

            // inject Rust-side uuid+timestamp for Data
            if let LifelogMacroMetaDataType::Data = options.datatype {
                named.named.insert(
                    0,
                    parse_quote! {

                        //#[serde(serialize_with = "lifelog_core::serialize_uuids", deserialize_with = "lifelog_core::deserialize_uuids")]
                        pub uuid: ::lifelog_core::uuid::Uuid
                    },
                );
                named.named.insert(
                    1,
                    parse_quote! {


                        pub timestamp: ::lifelog_core::chrono::DateTime<::lifelog_core::chrono::Utc>
                    },
                );
            }

            let ident = &s.ident;

            // DataType impl
            let impl_datatype = if let LifelogMacroMetaDataType::Data = options.datatype {
                quote! {
                    impl ::lifelog_core::DataType for #ident {
                        fn uuid(&self) -> ::lifelog_core::uuid::Uuid { self.uuid }
                        fn timestamp(&self) -> ::lifelog_core::chrono::DateTime<::lifelog_core::chrono::Utc> {
                            self.timestamp
                        }
                    }
                }
            } else {
                quote! {}
            };

            // From<proto> -> Self
            let from_fields = named.named.iter().map(|f| {
                let name = f.ident.as_ref().unwrap();
                // uuid
                if name == "uuid" {
                    return quote! {
                        uuid: ::lifelog_core::uuid::Uuid::parse_str(&p.uuid).expect("invalid uuid")
                    };
                }
                // timestamp
                if name == "timestamp" {
                    return quote! {
                        timestamp: {
                            let ts = p.timestamp.unwrap_or_default();
                            ::lifelog_core::chrono::DateTime::<::lifelog_core::chrono::Utc>::from_utc(
                                ::lifelog_core::chrono::NaiveDateTime::from_timestamp(ts.seconds, ts.nanos as u32),
                                ::lifelog_core::chrono::Utc,
                            )
                        }
                    };
                }

                // Vec<Enum> or Vec<String>
                if let Type::Path(typepath) = &f.ty {
                    let seg = typepath.path.segments.last().unwrap();
                    // TODO: REfactor, this code is so bad
if seg.ident == "DateTime" {
                if let PathArguments::AngleBracketed(args) = &seg.arguments {
                    if let Some(GenericArgument::Type(Type::Path(inner))) = args.args.first() {
                        // inner is a path; check its last segment == "Utc"
                        if inner.path.segments.last().unwrap().ident == "Utc" {
                            return quote! {
                                #name: {
                                    let ts = p.#name.unwrap_or_default();
                                    ::lifelog_core::chrono::DateTime::<::lifelog_core::chrono::Utc>::from_utc(
                                        ::lifelog_core::chrono::NaiveDateTime::from_timestamp(ts.seconds, ts.nanos as u32),
                                        ::lifelog_core::chrono::Utc,
                                    )
                                }
                            };
                        }
                    }
                }
            }
                    let seg = typepath.path.segments.last().unwrap().ident.to_string();
                    // Check to see if it a Vec<u8> (aka bytes) vector
                    if seg == "Vec" {
                        if let PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }) =
                            &typepath.path.segments[0].arguments
                        {
                            if let Some(GenericArgument::Type(Type::Path(inner))) = args.first() {
                                let inner_ident = &inner.path.segments.last().unwrap().ident;
                                if inner_ident == "u8" {
                                    return quote! { #name: p.#name.to_vec() };
                                }
                            }
                        }
                    }
                    if typepath.path.segments.len() == 1 && typepath.path.segments[0].ident == "Vec" {
                        if let PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }) =
                            &typepath.path.segments[0].arguments
                        {
                            if let Some(GenericArgument::Type(Type::Path(inner))) = args.first() {
                                let inner_ident = &inner.path.segments.last().unwrap().ident;
                                if inner_ident == "u8" {
                                    return quote! { #name: p.#name.to_vec() };
                                }
                                return quote! {
                                    #name: p.#name.into_iter().map(|s| s.parse().unwrap()).collect()
                                };
                            }
                        }
                    }

                    if seg == "PathBuf" {
                        return quote! { #name: std::path::PathBuf::from(p.#name) }
                    } else if seg.contains("Config") {
                        return quote! { #name: p.#name.expect(concat!("missing ", stringify!(#name))).into() }
                    } else if seg == "HashMap" {
                        return quote! {
                            #name: p.#name.into_iter()
                                .map(|(k,v)| (k, v.into()))
                                .collect::<std::collections::BTreeMap<_,_>>()
                        }
                }

                    // Option<Enum>
                    if typepath.path.segments.len() == 1 && typepath.path.segments[0].ident == "Option" {
                        if let PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }) =
                            &typepath.path.segments[0].arguments
                        {
                            if let Some(GenericArgument::Type(Type::Path(inner))) = args.first() {
                                let _inner_ident = &inner.path.segments.last().unwrap().ident;
                                return quote! {
                                    #name: p.#name.map(|s| s.parse().unwrap())
                                };
                            }
                        }
                    }
                }
                // fallback
                quote! { #name: p.#name.into() }
            });
            let from_impl = quote! {
                impl From<lifelog_proto::#ident> for #ident {
                    fn from(p: lifelog_proto::#ident) -> Self {
                        #ident { #(#from_fields),* }
                    }
                }
            };

            // Self -> proto
            let into_fields = named.named.iter().map(|f| {
                let name = f.ident.as_ref().unwrap();

    if let Type::Path(type_path) = &f.ty {
        if let Some(seg) = type_path.path.segments.last() {
            if seg.ident == "DateTime" {
                if let PathArguments::AngleBracketed(args) = &seg.arguments {
                    if let Some(GenericArgument::Type(Type::Path(inner))) = args.args.first() {
                        if inner.path.segments.last().unwrap().ident == "Utc" {
                            return quote! {
                                #name: Some(::prost_types::Timestamp {
                                    seconds: s.#name.timestamp(),
                                    nanos: 1000 * (s.#name.timestamp_subsec_nanos() / 1000) as i32,
                                })
                            };
                        }
                    }
                }
            }
        }
    }
            
                // uuid
                if name == "uuid" {
                    return quote! { uuid: s.uuid.to_string() };
                }
                // timestamp
                if name == "timestamp" {
                    return quote! {
                        timestamp: Some(::prost_types::Timestamp {
                            seconds: s.timestamp.timestamp(),
                            nanos: 1000 * (s.timestamp.timestamp_subsec_nanos() / 1000) as i32,
                        })
                    };
                }
                // Vec<Enum> -> Vec<String>
                if let Type::Path(typepath) = &f.ty {
                    if typepath.path.segments.len() == 1 && typepath.path.segments[0].ident == "Vec"
                    {
                        if let PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                            args,
                            ..
                        }) = &typepath.path.segments[0].arguments
                        {
                            if let Some(GenericArgument::Type(Type::Path(_inner))) = args.first() {
                                let inner_ident = &_inner.path.segments.last().unwrap().ident;
                                if inner_ident == "u8" {
                                    return quote! { #name: s.#name.into() }; // Vec<u8> to bytes
                                }
                                return quote! {
                                    #name: s.#name.iter().map(|e| e.to_string()).collect()
                                };
                            }
                        }
                    }

                    if typepath.path.segments[typepath.path.segments.len() - 1]
                        .ident
                        .to_string()
                        .contains("Config")
                    {
                        // Vec<String> -> Vec<Enum>
                        return quote! {
                            #name: Some(s.#name.into())
                        };
                    }

                    // Option<Enum> -> Option<String>
                    if typepath.path.segments.len() == 1
                        && typepath.path.segments[0].ident == "Option"
                    {
                        if let PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                            args,
                            ..
                        }) = &typepath.path.segments[0].arguments
                        {
                            if let Some(GenericArgument::Type(Type::Path(_inner))) = args.first() {
                                return quote! {
                                    #name: s.#name.as_ref().map(|e| e.to_string())
                                };
                            }
                        }
                    }
                    let seg = typepath.path.segments.last().unwrap().ident.to_string();

                    if seg == "PathBuf" {
                        return quote! { #name: s.#name.display().to_string() };
                    } else if seg.ends_with("Config") {
                        return quote! { #name: Some(s.#name.into()) };
                    } else {
                        return quote! { #name: s.#name.into() };
                    }
                }
                // fallback
                quote! { #name: s.#name.into() }
            });
            let into_impl = quote! {
                impl From<#ident> for lifelog_proto::#ident {
                    fn from(s: #ident) -> Self {
                        lifelog_proto::#ident { #(#into_fields),* }
                    }
                }
            };

            // re-emit struct + all impls
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

        Item::Enum(e) => {
            let name = &e.ident;
            let variants: Vec<&Ident> = e.variants.iter().map(|v| &v.ident).collect();

            quote! {
                // re-emit enum
                #original

                // parse from string
                impl std::str::FromStr for #name {
                    type Err = ();
                    fn from_str(input: &str) -> Result<#name, Self::Err> {
                        match input {
                            #(
                                stringify!(#variants) => Ok(#name::#variants),
                            )*
                            _ => Err(()),
                        }
                    }
                }

                // to string
                impl std::fmt::Display for #name {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        let s = match self {
                            #(
                                #name::#variants => stringify!(#variants),
                            )*
                        };
                        write!(f, "{}", s)
                    }
                }
            }
        }

        _ => unreachable!(),
    };

    expanded.into()
}
