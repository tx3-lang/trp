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
    let resolved = crate::resolver::resolve_components(&spec)?;

    for lang in &args.lang {
        let lang = lang.to_lowercase();
        let ctx = mapper::build_context(&resolved, &lang);
        let rendered = render_language(&lang, &resolved, &ctx)?;
        let lang_dir = args.out.join(&lang);
        fs::create_dir_all(&lang_dir)
            .with_context(|| format!("failed to create directory {}", lang_dir.display()))?;
        let file_path = lang_dir.join(output_file_name(&lang));
        fs::write(&file_path, rendered)
            .with_context(|| format!("failed to write {}", file_path.display()))?;
    }

    Ok(())
}

fn output_file_name(lang: &str) -> &str {
    match lang {
        "ts" | "typescript" => "types.ts",
        "python" => "types.py",
        "go" => "types.go",
        "rust" => "types.rs",
        _ => "types.txt",
    }
}

fn load_openrpc(path: &Path) -> Result<OpenRpc> {
    let data =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    let spec: OpenRpc = serde_json::from_str(&data)
        .with_context(|| format!("failed to parse OpenRPC spec from {}", path.display()))?;
    Ok(spec)
}

fn render_language(lang: &str, types: &[ResolvedType], ctx: &LanguageContext) -> Result<String> {
    match lang {
        "ts" | "typescript" => {
            let rendered = render_template(TsTemplate { types, ctx });
            rendered
        }
        "python" => {
            let rendered = render_template(PythonTemplate { types, ctx });
            rendered
        }
        "go" => {
            let rendered = render_template(GoTemplate { types, ctx });
            rendered
        }
        "rust" => {
            let rendered = render_template(RustTemplate { types, ctx });
            rendered
        }
        _ => anyhow::bail!("unsupported language: {}", lang),
    }
}

fn render_template<T: Template>(template: T) -> Result<String> {
    template.render().context("failed to render template")
}

#[derive(Template)]
#[template(path = "ts/types.askama", escape = "none")]
struct TsTemplate<'a> {
    types: &'a [ResolvedType],
    ctx: &'a LanguageContext,
}

#[derive(Template)]
#[template(path = "python/types.askama", escape = "none")]
struct PythonTemplate<'a> {
    types: &'a [ResolvedType],
    ctx: &'a LanguageContext,
}

#[derive(Template)]
#[template(path = "go/types.askama", escape = "none")]
struct GoTemplate<'a> {
    types: &'a [ResolvedType],
    ctx: &'a LanguageContext,
}

#[derive(Template)]
#[template(path = "rust/types.askama", escape = "none")]
struct RustTemplate<'a> {
    types: &'a [ResolvedType],
    ctx: &'a LanguageContext,
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
