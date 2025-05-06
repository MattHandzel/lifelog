use anyhow::{Context, Result};
use lifelog_core::LifelogMacroMetaDataType;
use maplit::hashmap;
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

#[derive(Debug, Serialize, Deserialize)]
struct DataTypeDefinition {
    ident: String,
    fields: Vec<(String, String)>,
    #[serde(default)]
    variants: Vec<String>,
    metadata_type: LifelogMacroMetaDataType,
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    output_paths: HashMap<String, PathBuf>,
}

trait CodeGenerator {
    fn generate(&self, types: &[DataTypeDefinition]) -> Result<String>;
}

struct TypeScriptGenerator;
struct ProtobufGenerator;

impl CodeGenerator for TypeScriptGenerator {
    fn generate(&self, types: &[DataTypeDefinition]) -> Result<String> {
        let mut output = String::from("// Auto‐generated types\n\n");

        for dtype in types {
            // enum as TS string‐union
            if !dtype.variants.is_empty() {
                let union = dtype
                    .variants
                    .iter()
                    .map(|v| format!("\"{}\"", v))
                    .collect::<Vec<_>>()
                    .join(" | ");
                output.push_str(&format!("export type {} = {};\n\n", dtype.ident, union));
                continue;
            }

            // struct as interface
            output.push_str(&format!("export interface {} {{\n", dtype.ident));
            for (field_name, field_type) in &dtype.fields {
                let ts_type = match field_type.as_str() {
                    "bool" => "boolean".into(),
                    "i8" | "i16" | "i32" | "i64" | "isize" => "number".into(),
                    "u8" | "u16" | "u32" | "u64" | "usize" => "number".into(),
                    "f32" | "f64" => "number".into(),
                    "char" | "String" | "&str" => "string".into(),
                    "DateTime<Utc>" => "Date".into(),
                    "::lifelog_core::uuid::Uuid" => "string".into(),
                    "::lifelog_core::chrono::DateTime<::lifelog_core::chrono::Utc>" => {
                        "Date".into()
                    }
                    "::lifelog_core::chrono::NaiveDate" => "Date".into(),
                    "::lifelog_core::chrono::NaiveDateTime" => "Date".into(),
                    "std::path::PathBuf" | "PathBuf" => "string".into(),
                    "std::net::IpAddr" | "std::net::SocketAddr" => "string".into(),
                    "std::collections::HashMap<String,String>"
                    | "std::collections::BTreeMap<String,String>" => "Record<string,string>".into(),
                    "Vec<u8>" => "Uint8Array".into(),
                    "Vec<String>" => "string[]".into(),
                    "Vec<i32>" => "number[]".into(),
                    "Option<String>" => "string | null".into(),
                    "Option<bool>" => "boolean | null".into(),
                    "Option<i32>" => "number | null".into(),
                    "serde_json::Value" => "any".into(),
                    t if types.iter().any(|d| d.ident == t) => t.to_string(),
                    t if t.starts_with('(') => "any".into(),
                    t if t.contains("Vec<") => {
                        let inner = t.trim_start_matches("Vec<").trim_end_matches('>');
                        if inner == "u8" {
                            "Uint8Array".into()
                        } else {
                            format!("{}[]", map_ts_type(inner))
                        }
                    }
                    t if t.contains("Option<") => {
                        let inner = t.trim_start_matches("Option<").trim_end_matches('>');
                        format!("{} | null", map_ts_type(inner))
                    }
                    t if t.contains("HashMap<") || t.contains("BTreeMap<") => {
                        "Record<string, any>".into()
                    }
                    _ => {
                        println!("Warning: unsupported TS type `{}`", field_type);
                        "any".into()
                    }
                };
                output.push_str(&format!("  {}: {};\n", field_name, ts_type));
            }
            output.push_str("}\n\n");
        }

        Ok(output)
    }
}

impl CodeGenerator for ProtobufGenerator {
    fn generate(&self, types: &[DataTypeDefinition]) -> Result<String> {
        let mut output = String::from(
            r#"syntax = "proto3";
package lifelog;

import "google/protobuf/timestamp.proto";
import "google/protobuf/any.proto";
import "google/protobuf/wrappers.proto";

"#,
        );

        for dtype in types {
            // enum as proto enum
            if !dtype.variants.is_empty() {
                output.push_str(&format!("enum {} {{\n", dtype.ident));
                for (i, v) in dtype.variants.iter().enumerate() {
                    output.push_str(&format!("  {} = {};\n", v, i));
                }
                output.push_str("}\n\n");
                continue;
            }

            // struct as message
            output.push_str(&format!("message {} {{\n", dtype.ident));
            let mut field_number = 1;
            for (field_name, field_type) in &dtype.fields {
                let (pb_type, repeated): (&str, bool) = match field_type.as_str() {
                    "bool" => ("bool", false),
                    "i8" | "i16" | "i32" => ("int32", false),
                    "i64" | "isize" => ("int64", false),
                    "u8" | "u16" | "u32" => ("uint32", false),
                    "u64" | "usize" => ("uint64", false),
                    "f32" => ("float", false),
                    "f64" => ("double", false),
                    "char" | "String" | "&str" => ("string", false),
                    "PathBuf" => ("string", false),
                    "DateTime<Utc>"
                    | "::lifelog_core::chrono::DateTime<::lifelog_core::chrono::Utc>"
                    | "::lifelog_core::chrono::NaiveDate"
                    | "::lifelog_core::chrono::NaiveDateTime" => {
                        ("google.protobuf.Timestamp", false)
                    }
                    "::lifelog_core::uuid::Uuid" => ("string", false),
                    "std::net::IpAddr" | "std::net::SocketAddr" => ("string", false),
                    "std::collections::HashMap<String,String>"
                    | "std::collections::BTreeMap<String,String>" => ("map<string,string>", false),
                    "Vec<u8>" => ("bytes", false),
                    "Vec<String>" => ("string", true),
                    "Vec<i32>" => ("int32", true),
                    "Option<String>" => ("google.protobuf.StringValue", false),
                    "Option<bool>" => ("google.protobuf.BoolValue", false),
                    "Option<i32>" => ("google.protobuf.Int32Value", false),
                    "serde_json::Value" => ("google.protobuf.Any", false),
                    t if t.starts_with('(') => ("string", false),
                    t if t.contains("Vec<") => {
                        let inner = t.trim_start_matches("Vec<").trim_end_matches('>');
                        (map_protobuf_type(inner), true)
                    }
                    t if t.contains("Option<") => {
                        let inner = t.trim_start_matches("Option<").trim_end_matches('>');
                        (
                            // e.g. google.protobuf.StringValue
                            &*format!("google.protobuf.{}Value", map_wrapper_type(inner)),
                            false,
                        )
                    }
                    t if t.contains("HashMap<") || t.contains("BTreeMap<") => {
                        // parse key/value
                        let parts: Vec<&str> = t.trim_end_matches('>').split('<').collect();
                        let kv: Vec<&str> = parts[1].split(',').map(str::trim).collect();
                        let key = if types.iter().any(|d| d.ident == kv[0]) {
                            kv[0]
                        } else {
                            map_protobuf_type(kv[0])
                        };
                        let val = if types.iter().any(|d| d.ident == kv[1]) {
                            kv[1]
                        } else {
                            map_protobuf_type(kv[1])
                        };
                        // leak to 'static str and then coerce to &str
                        let leaked: &'static mut str =
                            Box::leak(format!("map<{},{}>", key, val).into_boxed_str());
                        (&*leaked, false)
                    }
                    t if types.iter().any(|d| d.ident == t) => (t, false),
                    other => panic!("Unsupported Protobuf type `{}`", other),
                };

                if repeated {
                    output.push_str(&format!(
                        "  repeated {} {} = {};\n",
                        pb_type, field_name, field_number
                    ));
                } else {
                    output.push_str(&format!(
                        "  {} {} = {};\n",
                        pb_type, field_name, field_number
                    ));
                }
                field_number += 1;
            }
            output.push_str("}\n\n");
        }

        // top‐level container for all Data‐annotated types
        output.push_str("message LifelogData {\n");
        let mut idx = 1;
        for dtype in types
            .iter()
            .filter(|d| matches!(d.metadata_type, LifelogMacroMetaDataType::Data))
        {
            output.push_str(&format!(
                "  repeated {} {} = {};\n",
                dtype.ident,
                dtype.ident.to_lowercase(),
                idx,
            ));
            idx += 1;
        }
        output.push_str("}\n");

        Ok(output)
    }
}

fn map_ts_type(ty: &str) -> &str {
    match ty {
        "bool" => "boolean",
        "i8" | "i16" | "i32" | "i64" | "isize" => "number",
        "u8" | "u16" | "u32" | "u64" | "usize" => "number",
        "f32" | "f64" => "number",
        "char" | "String" | "&str" => "string",
        _ => "any",
    }
}

fn map_protobuf_type(ty: &str) -> &str {
    match ty {
        "bool" => "bool",
        "i8" | "i16" | "i32" => "int32",
        "i64" | "isize" => "int64",
        "u8" | "u16" | "u32" => "uint32",
        "u64" | "usize" => "uint64",
        "f32" => "float",
        "f64" => "double",
        "char" | "String" | "&str" => "string",
        _ => "string",
    }
}

fn map_wrapper_type(ty: &str) -> &str {
    match ty {
        "bool" => "Bool",
        "i8" | "i16" | "i32" => "Int32",
        "i64" => "Int64",
        "u8" | "u16" | "u32" => "UInt32",
        "u64" => "UInt64",
        "f32" => "Float",
        "f64" => "Double",
        "String" => "String",
        _ => "String",
    }
}

fn find_metadata_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
        let name = entry.file_name().to_string_lossy();
        if name.contains(".type.json") {
            files.push(entry.path().to_path_buf());
        }
    }
    Ok(files)
}

fn parse_metadata_files(files: &[PathBuf]) -> Result<Vec<DataTypeDefinition>> {
    let mut types = HashMap::new();
    for file in files {
        let content = fs::read_to_string(file)?;
        for line in content.lines() {
            let dtype: DataTypeDefinition = serde_json::from_str(line)
                .with_context(|| format!("failed to parse {}", file.display()))?;
            types.insert(dtype.ident.clone(), dtype);
        }
    }
    Ok(types.into_values().collect())
}

fn generate_and_write(
    types: &[DataTypeDefinition],
    generators: &[(String, Box<dyn CodeGenerator>)],
    output_paths: &HashMap<String, PathBuf>,
) -> Result<()> {
    for (lang, gen) in generators {
        if let Some(path) = output_paths.get(lang) {
            let out = gen.generate(types)?;
            println!("Writing {}", path.display());
            fs::write(path, out).with_context(|| format!("failed to write {}", path.display()))?;
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=../../");
    let config = Config {
        output_paths: hashmap! {
            "typescript".into() => PathBuf::from("../../interface/src/auto_generated_types.ts"),
            "protobuf".into()  => PathBuf::from("../../proto/lifelog_types.proto"),
        },
    };

    let gens: Vec<(String, Box<dyn CodeGenerator>)> = vec![
        ("typescript".into(), Box::new(TypeScriptGenerator)),
        ("protobuf".into(), Box::new(ProtobufGenerator)),
    ];

    let mut files = find_metadata_files(Path::new(".."))?;
    files.sort();
    let types = parse_metadata_files(&files)?;
    generate_and_write(&types, &gens, &config.output_paths)?;
    Ok(())
}
