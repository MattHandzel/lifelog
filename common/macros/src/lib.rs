use lifelog_types::*;
use proc_macro::TokenStream;
use quote::quote;
use quote::ToTokens;
use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, parse_quote,
    punctuated::Punctuated,
    token::Comma,
    DeriveInput, Field, FieldsNamed, Ident, ItemStruct, Result, Token,
};

struct MacroOptions {
    datatype: LifelogMacroMetaDataType,
}

impl Parse for MacroOptions {
    fn parse(input: ParseStream) -> Result<Self> {
        let type_ident: Ident = input.parse().expect("Failed parsing input to macros");
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

    // Parse the original struct
    let mut struct_ast = parse_macro_input!(item as ItemStruct);

    // Only proceed if it's a named‐field struct
    let named: &mut FieldsNamed = match &mut struct_ast.fields {
        syn::Fields::Named(f) => f,
        _ => {
            return syn::Error::new_spanned(
                &struct_ast.ident,
                "#[lifelog_type] only supports structs with named fields",
            )
            .to_compile_error()
            .into();
        }
    };

    let named: &mut syn::FieldsNamed = match &mut struct_ast.fields {
        syn::Fields::Named(f) => f,
        _ => {
            panic!("Expected named fields")
        }
    };
    // Prepare any extra fields
    let mut extra: Punctuated<Field, Comma> = Punctuated::new();
    if let LifelogMacroMetaDataType::Data = options.datatype {
        named.named.insert(
            0,
            parse_quote! {
                pub uuid: ::uuid::Uuid
            },
        );
        named.named.insert(
            1,
            parse_quote! {
                pub timestamp: ::chrono::DateTime<::chrono::Utc>
            },
        );
    }

    // Prepend extra fields so they appear before the user’s fields,
    // or push them after if you prefer.
    for f in extra.into_iter().rev() {
        named.named.insert(0, f);
    }

    // Write metadata JSON
    let ident = &struct_ast.ident;
    let fields_meta: Vec<(&Ident, String)> = named
        .named
        .iter()
        .filter_map(|f| {
            f.ident.as_ref().map(|i| {
                let ty = &f.ty;
                (i, ty.into_token_stream().to_string().replace(" ", ""))
            })
        })
        .collect();
    let meta = serde_json::json!({
        "ident": ident.to_string(),
        "fields": fields_meta.iter().map(|(i,t)| [i.to_string(), t.clone()]).collect::<Vec<_>>(),
        "metadata_type" : match options.datatype {
            LifelogMacroMetaDataType::Config => "Config",
            LifelogMacroMetaDataType::Data => "Data",
            LifelogMacroMetaDataType::None => "None",
        },
    });
    let out_dir = PathBuf::from(
        env::var("CARGO_MANIFEST_DIR")
            .expect("Unable to get CARGO_MANIFEST_DIR environment variable"),
    );
    let meta_path = out_dir.join(format!(".{}.type.json", &struct_ast.ident));
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&meta_path)
        .expect("could not open metadata file");
    writeln!(file, "{}", meta).expect("failed to write metadata");

    // If Data, implement DataType
    let impl_datatype = if let LifelogMacroMetaDataType::Data = options.datatype {
        let name = &struct_ast.ident;
        Some(quote! {
            impl ::lifelog_types::DataType for #name {
                fn uuid(&self) -> ::uuid::Uuid {
                    self.uuid
                }
                fn timestamp(&self) -> ::chrono::DateTime<::chrono::Utc> {
                    self.timestamp
                }
            }
        })
    } else {
        None
    };

    // Re-emit the struct exactly as parsed (with added fields) plus impl
    let attrs = &struct_ast.attrs;
    let vis = &struct_ast.vis;
    let generics = &struct_ast.generics;
    let fields = &struct_ast.fields;
    let where_clause = &struct_ast.generics.where_clause;

    let expanded = quote! {
        #vis struct #ident #generics #fields #where_clause
        #(#attrs)*

        #impl_datatype
    };

    expanded.into()
}
