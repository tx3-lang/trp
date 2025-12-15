use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use askama::Template;
use clap::Parser;

use crate::mapper::{self, LanguageContext};
use crate::openrpc::OpenRpc;
use crate::resolver::{ResolvedField, ResolvedType};

#[derive(Parser, Debug)]
pub struct GenArgs {
    /// Path to the OpenRPC spec
    #[arg(long, default_value = "specs/trp.json")]
    pub openrpc: PathBuf,
    /// Comma separated list of languages to generate (ts, python, go, rust)
    #[arg(long, value_delimiter = ',')]
    pub lang: Vec<String>,
    /// Output directory
    #[arg(long, default_value = "bindings")]
    pub out: PathBuf,
    /// Clean output directory before generating
    #[arg(long, default_value_t = false)]
    pub clean: bool,
}

pub fn run(args: GenArgs) -> Result<()> {
    if args.clean && args.out.exists() {
        fs::remove_dir_all(&args.out)
            .with_context(|| format!("failed to clean output directory {}", args.out.display()))?;
    }

    fs::create_dir_all(&args.out)
        .with_context(|| format!("failed to create output directory {}", args.out.display()))?;

    let spec = load_openrpc(&args.openrpc)?;
    let metadata = BindingMetadata::from_openrpc(&spec);
    let resolved = crate::resolver::resolve_components(&spec)?;

    for lang in &args.lang {
        let lang = lang.to_lowercase();
        let ctx = mapper::build_context(&resolved, &lang);
        let rendered = render_language(&lang, &resolved, &ctx, &metadata)?;
        let lang_dir = args.out.join(&lang);
        fs::create_dir_all(&lang_dir)
            .with_context(|| format!("failed to create directory {}", lang_dir.display()))?;
        for GeneratedFile { name, content } in rendered {
            let file_path = lang_dir.join(name);
            fs::write(&file_path, content)
                .with_context(|| format!("failed to write {}", file_path.display()))?;
        }
    }

    Ok(())
}

fn load_openrpc(path: &Path) -> Result<OpenRpc> {
    let data =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    let spec: OpenRpc = serde_json::from_str(&data)
        .with_context(|| format!("failed to parse OpenRPC spec from {}", path.display()))?;
    Ok(spec)
}

fn render_language(
    lang: &str,
    types: &[ResolvedType],
    ctx: &LanguageContext,
    metadata: &BindingMetadata,
) -> Result<Vec<GeneratedFile>> {
    match lang {
        "ts" | "typescript" => {
            let ts_template = TsTemplate {
                types,
                ctx,
                metadata,
            };
            let types = render_template(ts_template)?;
            let package = render_template(TsPackageTemplate {
                types,
                ctx,
                metadata,
            })?;
            Ok(vec![
                GeneratedFile::new("types.ts", types),
                GeneratedFile::new("package.json", package),
            ])
        }
        "python" => {
            let python_template = PythonTemplate {
                types,
                ctx,
                metadata,
            };
            let types = render_template(python_template)?;
            let pyproject = render_template(PythonProjectTemplate {
                types,
                ctx,
                metadata,
            })?;
            Ok(vec![
                GeneratedFile::new("types.py", types),
                GeneratedFile::new("pyproject.toml", pyproject),
            ])
        }
        "go" => {
            let go_template = GoTemplate {
                types,
                ctx,
                metadata,
            };
            let types = render_template(go_template)?;
            let go_mod = render_template(GoModuleTemplate {
                types,
                ctx,
                metadata,
            })?;
            Ok(vec![
                GeneratedFile::new("types.go", types),
                GeneratedFile::new("go.mod", go_mod),
            ])
        }
        "rust" => {
            let rust_template = RustTemplate {
                types,
                ctx,
                metadata,
            };
            let types = render_template(rust_template)?;
            let cargo = render_template(RustCargoTemplate {
                types,
                ctx,
                metadata,
            })?;
            Ok(vec![
                GeneratedFile::new("types.rs", types),
                GeneratedFile::new("Cargo.toml", cargo),
            ])
        }
        _ => anyhow::bail!("unsupported language: {}", lang),
    }
}

fn render_template<T: Template>(template: T) -> Result<String> {
    template.render().context("failed to render template")
}

#[derive(Debug)]
struct BindingMetadata {
    version: String,
}

impl BindingMetadata {
    fn from_openrpc(spec: &OpenRpc) -> Self {
        let version = spec
            .info
            .as_ref()
            .and_then(|info| info.version.clone())
            .unwrap_or_else(|| "0.1.0".to_string());

        Self { version }
    }
}

struct GeneratedFile {
    name: String,
    content: String,
}

impl GeneratedFile {
    fn new(name: &str, content: String) -> Self {
        Self {
            name: name.to_string(),
            content,
        }
    }
}

#[derive(Template)]
#[template(path = "ts/types.askama", escape = "none")]
struct TsTemplate<'a> {
    types: &'a [ResolvedType],
    ctx: &'a LanguageContext,
    metadata: &'a BindingMetadata,
}

#[derive(Template)]
#[template(path = "ts/package.askama", escape = "none")]
struct TsPackageTemplate<'a> {
    types: &'a [ResolvedType],
    ctx: &'a LanguageContext,
    metadata: &'a BindingMetadata,
}

#[derive(Template)]
#[template(path = "python/types.askama", escape = "none")]
struct PythonTemplate<'a> {
    types: &'a [ResolvedType],
    ctx: &'a LanguageContext,
    metadata: &'a BindingMetadata,
}

#[derive(Template)]
#[template(path = "python/pyproject.askama", escape = "none")]
struct PythonProjectTemplate<'a> {
    types: &'a [ResolvedType],
    ctx: &'a LanguageContext,
    metadata: &'a BindingMetadata,
}

#[derive(Template)]
#[template(path = "go/types.askama", escape = "none")]
struct GoTemplate<'a> {
    types: &'a [ResolvedType],
    ctx: &'a LanguageContext,
    metadata: &'a BindingMetadata,
}

#[derive(Template)]
#[template(path = "go/go_mod.askama", escape = "none")]
struct GoModuleTemplate<'a> {
    types: &'a [ResolvedType],
    ctx: &'a LanguageContext,
    metadata: &'a BindingMetadata,
}

#[derive(Template)]
#[template(path = "rust/types.askama", escape = "none")]
struct RustTemplate<'a> {
    types: &'a [ResolvedType],
    ctx: &'a LanguageContext,
    metadata: &'a BindingMetadata,
}

#[derive(Template)]
#[template(path = "rust/cargo.askama", escape = "none")]
struct RustCargoTemplate<'a> {
    types: &'a [ResolvedType],
    ctx: &'a LanguageContext,
    metadata: &'a BindingMetadata,
}

mod filters {
    use crate::resolver::ResolvedField;

    pub fn length(value: &Vec<ResolvedField>) -> Result<usize, askama::Error> {
        Ok(value.len())
    }
}

// Helper methods exposed to templates
impl ResolvedField {
    pub fn ts_name(&self) -> String {
        mapper::ts::field_name(&self.name)
    }

    pub fn python_name(&self) -> String {
        mapper::python::field_name(&self.name)
    }

    pub fn go_name(&self) -> String {
        mapper::go::field_name(&self.name)
    }

    pub fn rust_name(&self) -> String {
        mapper::rust::field_name(&self.name)
    }

    pub fn ts_type(&self, ctx: &LanguageContext) -> String {
        mapper::ts::map_type(&self.schema, ctx).maybe_optional(self.required, ctx)
    }

    pub fn python_type(&self, ctx: &LanguageContext) -> String {
        mapper::python::map_type(&self.schema, ctx).maybe_optional(self.required, ctx)
    }

    pub fn go_type(&self, ctx: &LanguageContext) -> String {
        mapper::go::map_type(&self.schema, ctx)
    }

    pub fn rust_type(&self, ctx: &LanguageContext) -> String {
        mapper::rust::map_type(&self.schema, ctx).maybe_optional(self.required, ctx)
    }
}

trait OptionalRendering {
    fn maybe_optional(self, required: bool, ctx: &LanguageContext) -> String;
}

impl OptionalRendering for String {
    fn maybe_optional(self, required: bool, ctx: &LanguageContext) -> String {
        if required {
            self
        } else {
            ctx.wrap_optional(&self)
        }
    }
}
