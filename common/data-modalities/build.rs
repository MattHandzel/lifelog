use anyhow::{Context, Result};
use lifelog_core::LifelogMacroMetaDataType;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

#[derive(Debug, Serialize, Deserialize)]
struct DataTypeDefinition {
    ident: String,
    fields: Vec<(String, String)>,
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
        let mut output = String::from("// Auto-generated types\n\n");

        for dtype in types {
            output.push_str(&format!("export interface {} {{\n", dtype.ident));
            for (field_name, field_type) in &dtype.fields {
                let ts_type = match field_type.as_str() {
                    // Primitive types
                    "bool" => "boolean".to_string(),
                    "i8" | "i16" | "i32" | "i64" | "isize" => "number".to_string(),
                    "u8" | "u16" | "u32" | "u64" | "usize" => "number".to_string(),
                    "f32" | "f64" => "number".to_string(),
                    "char" => "string".to_string(),
                    "String" => "string".to_string(),
                    "&str" => "string".to_string(),

                    // Common Rust types
                    "::lifelog_core::uuid::Uuid" => "string".to_string(),
                    "::lifelog_core::chrono::DateTime<::lifelog_core::chrono::Utc>" => {
                        "Date".to_string()
                    }
                    "::lifelog_core::chrono::NaiveDate" => "Date".to_string(),
                    "::lifelog_core::chrono::NaiveDateTime" => "Date".to_string(),
                    "std::path::PathBuf" => "string".to_string(),
                    "std::net::IpAddr" => "string".to_string(),
                    "std::net::SocketAddr" => "string".to_string(),
                    "std::collections::HashMap<String, String>" => {
                        "Record<string, string>".to_string()
                    }
                    "std::collections::BTreeMap<String, String>" => {
                        "Record<string, string>".to_string()
                    }
                    "Vec<u8>" => "Uint8Array".to_string(),
                    "Vec<String>" => "string[]".to_string(),
                    "Vec<i32>" => "number[]".to_string(),
                    "Option<String>" => "string | null".to_string(),
                    "Option<bool>" => "boolean | null".to_string(),
                    "Option<i32>" => "number | null".to_string(),
                    "Result<String, String>" => "string".to_string(), // Simplified
                    "serde_json::Value" => "any".to_string(),
                    // Check to see if the type is a custom data type
                    t if types.iter().any(|d| d.ident == t) => t.to_string(),

                    // Tuples and complex types
                    t if t.starts_with('(') => "any".to_string(),
                    t if t.contains("Vec<") => {
                        let inner = t.replace("Vec<", "").replace('>', "");
                        match inner.as_str() {
                            "u8" => "Uint8Array".to_string(),
                            _ => format!("{}[]", map_ts_type(&inner)),
                        }
                    }
                    t if t.contains("Option<") => {
                        let inner = t.replace("Option<", "").replace('>', "");
                        format!("{} | null", map_ts_type(&inner))
                    }
                    t if t.contains("HashMap<") || t.contains("BTreeMap<") => {
                        "Record<string, any>".to_string()
                    }
                    _ => {
                        println!("Warning: Unsupported TypeScript type: {}", field_type);
                        "any".to_string()
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
        let mut output = String::from("syntax = \"proto3\";\npackage lifelog;\n\n");
        output.push_str("import \"google/protobuf/timestamp.proto\";\n");
        output.push_str("import \"google/protobuf/any.proto\";\n");
        output.push_str("import \"google/protobuf/wrappers.proto\";\n\n");

        for dtype in types {
            output.push_str(&format!("message {} {{\n", dtype.ident));
            let mut field_number = 1;

            for (field_name, field_type) in &dtype.fields {
                let (pb_type, is_repeated) = match field_type.as_str() {
                    // Primitive types
                    "bool" => ("bool".to_string(), false),
                    "i8" | "i16" | "i32" => ("int32".to_string(), false),
                    "i64" | "isize" => ("int64".to_string(), false),
                    "u8" | "u16" | "u32" => ("uint32".to_string(), false),
                    "u64" | "usize" => ("uint64".to_string(), false),
                    "f32" => ("float".to_string(), false),
                    "f64" => ("double".to_string(), false),
                    "char" => ("string".to_string(), false),
                    "String" => ("string".to_string(), false),
                    "&str" => ("string".to_string(), false),

                    // Common Rust types
                    "::lifelog_core::uuid::Uuid" => ("string".to_string(), false),
                    "::lifelog_core::chrono::DateTime<::lifelog_core::chrono::Utc>" => {
                        ("google.protobuf.Timestamp".to_string(), false)
                    }
                    "::lifelog_core::chrono::NaiveDate" => {
                        ("google.protobuf.Timestamp".to_string(), false)
                    }
                    "::lifelog_core::chrono::NaiveDateTime" => {
                        ("google.protobuf.Timestamp".to_string(), false)
                    }
                    "std::path::PathBuf" => ("string".to_string(), false),
                    "std::net::IpAddr" => ("string".to_string(), false),
                    "std::net::SocketAddr" => ("string".to_string(), false),
                    "std::collections::HashMap<String, String>" => {
                        ("map<string, string>".to_string(), false)
                    }
                    "std::collections::BTreeMap<String, String>" => {
                        ("map<string, string>".to_string(), false)
                    }
                    "Vec<u8>" => ("bytes".to_string(), false),
                    "Vec<String>" => ("string".to_string(), true),
                    "Vec<i32>" => ("int32".to_string(), true),
                    "Option<String>" => ("google.protobuf.StringValue".to_string(), false),
                    "Option<bool>" => ("google.protobuf.BoolValue".to_string(), false),
                    "Option<i32>" => ("google.protobuf.Int32Value".to_string(), false),
                    "Result<String, String>" => ("string".to_string(), false),
                    "serde_json::Value" => ("google.protobuf.Any".to_string(), false),

                    // Complex types
                    t if t.starts_with('(') => ("string".to_string(), false),
                    t if t.contains("Vec<") => {
                        let inner = t.replace("Vec<", "").replace('>', "");
                        (map_protobuf_type(&inner).to_string(), true)
                    }
                    t if t.contains("Option<") => {
                        let inner = t.replace("Option<", "").replace('>', "");
                        (
                            format!("google.protobuf.{}Value", map_wrapper_type(&inner)),
                            false,
                        )
                    }
                    t if t.contains("HashMap<") || t.contains("BTreeMap<") => {
                        let parts: Vec<&str> = t.split('<').collect();
                        if parts.len() > 1 {
                            let inner = parts[1].replace('>', "");
                            let key_value: Vec<&str> = inner.split(',').collect();
                            if key_value.len() == 2 {
                                let key_type = map_protobuf_type(key_value[0].trim());
                                let value_type = map_protobuf_type(key_value[1].trim());
                                (format!("map<{}, {}>", key_type, value_type), false)
                            } else {
                                ("map<string, string>".to_string(), false)
                            }
                        } else {
                            ("map<string, string>".to_string(), false)
                        }
                    }
                    // Check to see if the type is a custom data type
                    t if types.iter().any(|d| d.ident == t) => (t.to_string(), false),
                    _ => {
                        println!("Warning: Unsupported Protobuf type: {}", field_type);
                        ("string".to_string(), false)
                    }
                };

                if is_repeated {
                    output.push_str(&format!(
                        "\trepeated {} {} = {};\n",
                        pb_type, field_name, field_number
                    ));
                } else {
                    output.push_str(&format!(
                        "\t{} {} = {};\n",
                        pb_type, field_name, field_number
                    ));
                }
                field_number += 1;
            }
            output.push_str("}\n\n");
        }

        // Main LifelogData message
        output.push_str("message LifelogData {\n");
        let mut field_number = 1;
        for dtype in types {
            match dtype.metadata_type {
                LifelogMacroMetaDataType::Data => output.push_str(&format!(
                    "\trepeated {} {} = {};\n",
                    dtype.ident,
                    dtype.ident.to_lowercase(),
                    field_number
                )),
                _ => println!("Skipping non-data type: {}", dtype.ident),
            }
            field_number += 1;
        }
        output.push_str("}\n\n");

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
        if entry
            .file_name()
            .to_str()
            .expect("Unable to convert file name to string while walking through directory")
            .contains(".type.json")
        {
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
                .with_context(|| format!("Failed to parse line in {}", file.display()))?;
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
    for (lang, generator) in generators {
        if let Some(path) = output_paths.get(lang) {
            let content = generator.generate(types)?;
            println!("Writing to {}", path.display());
            fs::write(path, content)
                .with_context(|| format!("Failed to write {}", path.display()))?;
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=../../");
    let config = Config {
        output_paths: maplit::hashmap! {
            "typescript".into() => PathBuf::from("../../interface/src/auto_generated_types.ts"),
            "protobuf".into() => PathBuf::from("../../proto/lifelog_types.proto"),
        },
    };

    let generators: Vec<(String, Box<dyn CodeGenerator>)> = vec![
        ("typescript".into(), Box::new(TypeScriptGenerator)),
        ("protobuf".into(), Box::new(ProtobufGenerator)),
    ];

    let mut metadata_files = find_metadata_files(Path::new(".."))?;
    metadata_files.sort();
    let types = parse_metadata_files(&metadata_files)?;
    generate_and_write(&types, &generators, &config.output_paths)?;

    Ok(())
}
