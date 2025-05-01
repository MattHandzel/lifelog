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
                    "::lifelog_core::uuid::Uuid" => "string",
                    "::lifelog_core::chrono::DateTime<::lifelog_core::chrono::Utc>" => "Date",
                    "std::path::PathBuf" => "string",
                    "f32" => "number",
                    "u32" => "number",
                    "u8" => "number",
                    "String" => "string",
                    "Option<bool>" => "boolean | null",
                    t if t.starts_with('(') => "any",
                    _ => panic!("Unsupported type: {}", field_type),
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
        let mut output = String::from("syntax = \"proto3\";\npackage = \"lifelog\"\n\n");

        for dtype in types {
            output.push_str(&format!("message {} {{\n", dtype.ident));
            let mut field_number = 1;

            for (field_name, field_type) in &dtype.fields {
                let pb_type = match field_type.as_str() {
                    "::lifelog_core::uuid::Uuid" => "string",
                    "::lifelog_core::chrono::DateTime<::lifelog_core::chrono::Utc>" => {
                        "google.protobuf.Timestamp"
                    }
                    "std::path::PathBuf" => "string",
                    "f32" => "float",
                    "u32" => "uint32",
                    "u8" => "uint8",
                    "String" => "string",
                    "Option<bool>" => "bool",
                    t if t.starts_with('(') => "string",
                    _ => "string",
                };

                output.push_str(&format!(
                    "\t{} {} = {};\n",
                    pb_type, field_name, field_number
                ));
                field_number += 1;
            }
            output.push_str("}\n\n");
        }
        // Rest of the messages we want

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
            fs::write(path, content)
                .with_context(|| format!("Failed to write {}", path.display()))?;
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=.");
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

    let metadata_files = find_metadata_files(Path::new("."))?;
    let types = parse_metadata_files(&metadata_files)?;
    generate_and_write(&types, &generators, &config.output_paths)?;

    Ok(())
}
