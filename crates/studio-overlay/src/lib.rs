//! Build-time overlay for an upstream Craft app's binary crate.
//!
//! Each Craft app ships its window glue (file dialogs, preferences, audio, GPU checks) inside its
//! *binary* crate, next to `fn main`, so another crate cannot call it. An adapter crate in
//! `crates/modules/<app>` calls [`generate`] from its `build.rs`: every `.rs` file of the upstream
//! binary is rewritten into `OUT_DIR` so that the adapter can `include!` it as a private module:
//!
//! - the root file loses its inner attributes (`include!` rejects them), `fn main` and its
//!   `#[cfg(test)]` modules;
//! - `mod x;` declarations get an absolute `#[path]` into the rewritten tree;
//! - `crate::` paths gain the adapter's module name (`crate::upstream::…`);
//! - `env!("CARGO_MANIFEST_DIR" | "CARGO_PKG_*")` and relative `include_bytes!`/`include_str!`
//!   paths are resolved against the upstream crate, other build-time `env!` values become `"dev"`.
//!
//! Edits are applied to the original text at syn span positions, so the generated code keeps the
//! upstream formatting and comments and compiler errors stay readable. Upstream files are never
//! modified.

use proc_macro2::{Delimiter, LineColumn, Span, TokenStream, TokenTree};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use syn::spanned::Spanned;
use syn::visit::{self, Visit};

/// What [`generate`] needs to know about the upstream binary crate.
pub struct Overlay<'a> {
    /// The upstream binary crate directory (holds `Cargo.toml` and `src/main.rs`).
    pub crate_dir: &'a Path,
    /// The module the adapter includes the root file into (`mod upstream { include!(…) }`).
    pub module: &'a str,
    /// Where the rewritten tree goes (normally `$OUT_DIR/upstream`).
    pub out_dir: &'a Path,
}

/// Rewrites the binary's `src/` tree into `out_dir` and returns the rewritten `main.rs`.
pub fn generate(overlay: &Overlay) -> io::Result<PathBuf> {
    let src = overlay.crate_dir.join("src");
    let manifest = fs::read_to_string(overlay.crate_dir.join("Cargo.toml"))?;
    let package = PackageInfo::parse(&manifest, overlay.crate_dir);
    let mut files = Vec::new();
    collect_rs(&src, &mut files)?;
    for file in &files {
        let rel = file.strip_prefix(&src).map_err(io::Error::other)?;
        let out = overlay.out_dir.join(rel);
        let text = fs::read_to_string(file)?;
        let is_root = rel == Path::new("main.rs");
        let rewritten = rewrite(&text, file, &src, overlay, &package, is_root)
            .map_err(|e| io::Error::other(format!("{}: {e}", file.display())))?;
        if let Some(dir) = out.parent() {
            fs::create_dir_all(dir)?;
        }
        // Only touch files whose content changed, so cargo does not rebuild needlessly.
        if fs::read_to_string(&out).ok().as_deref() != Some(rewritten.as_str()) {
            fs::write(&out, rewritten)?;
        }
    }
    println!("cargo:rerun-if-changed={}", src.display());
    println!("cargo:rerun-if-changed={}", overlay.crate_dir.join("Cargo.toml").display());
    Ok(overlay.out_dir.join("main.rs"))
}

struct PackageInfo {
    name: String,
    version: String,
    manifest_dir: String,
}

impl PackageInfo {
    /// Reads `name` and `version` from `[package]`; a workspace-inherited version stays `"dev"`.
    fn parse(manifest: &str, dir: &Path) -> Self {
        let mut name = String::from("upstream");
        let mut version = String::from("dev");
        let mut in_package = false;
        for line in manifest.lines() {
            let line = line.trim();
            if line.starts_with('[') {
                in_package = line == "[package]";
                continue;
            }
            if !in_package {
                continue;
            }
            if let Some(v) = quoted_value(line, "name") {
                name = v;
            } else if let Some(v) = quoted_value(line, "version") {
                version = v;
            }
        }
        if version == "dev"
            && let Some(ws) = workspace_version(dir)
        {
            version = ws;
        }
        Self { name, version, manifest_dir: dir.display().to_string() }
    }
}

fn quoted_value(line: &str, key: &str) -> Option<String> {
    let rest = line.strip_prefix(key)?.trim_start().strip_prefix('=')?.trim();
    Some(rest.strip_prefix('"')?.split('"').next()?.to_string())
}

/// `version` from the nearest ancestor's `[workspace.package]`.
fn workspace_version(dir: &Path) -> Option<String> {
    for ancestor in dir.ancestors().skip(1) {
        let Ok(text) = fs::read_to_string(ancestor.join("Cargo.toml")) else { continue };
        if !text.contains("[workspace") {
            continue;
        }
        let mut in_section = false;
        for line in text.lines() {
            let line = line.trim();
            if line.starts_with('[') {
                in_section = line == "[workspace.package]";
                continue;
            }
            if in_section && let Some(v) = quoted_value(line, "version") {
                return Some(v);
            }
        }
        return None;
    }
    None
}

fn collect_rs(dir: &Path, out: &mut Vec<PathBuf>) -> io::Result<()> {
    let mut entries: Vec<_> = fs::read_dir(dir)?.collect::<Result<_, _>>()?;
    entries.sort_by_key(|e| e.path());
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            collect_rs(&path, out)?;
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
    Ok(())
}

/// One text replacement, in byte offsets of the original file.
struct Edit {
    start: usize,
    end: usize,
    text: String,
}

fn rewrite(text: &str, file: &Path, src_root: &Path, overlay: &Overlay, package: &PackageInfo, is_root: bool) -> Result<String, String> {
    let ast = syn::parse_file(text).map_err(|e| format!("parse error at {:?}: {e}", e.span().start()))?;
    let index = LineIndex::new(text);
    let mut edits = Vec::new();

    if is_root {
        for attr in ast.attrs.iter().filter(|a| matches!(a.style, syn::AttrStyle::Inner(_))) {
            edits.push(Edit { start: index.start(attr.pound_token.span), end: index.end(attr.bracket_token.span.close()), text: String::new() });
        }
    }
    for item in &ast.items {
        let removable = (is_root && matches!(item, syn::Item::Fn(f) if f.sig.ident == "main")) || is_test_module(item);
        if removable && let Some(end) = item_end(item) {
            edits.push(Edit { start: index.start(item_start(item)), end: index.end(end), text: String::new() });
        }
    }

    let mut visitor = Rewriter { index: &index, file, src_root, overlay, package, edits: Vec::new() };
    visitor.visit_file(&ast);
    edits.extend(visitor.edits);
    apply(text, edits)
}

fn item_start(item: &syn::Item) -> Span {
    match item {
        syn::Item::Fn(f) => f.attrs.first().map(|a| a.pound_token.span).unwrap_or_else(|| first_token(&f.vis, f.sig.constness.map(|t| t.span).or(f.sig.asyncness.map(|t| t.span)).or(f.sig.unsafety.map(|t| t.span)).unwrap_or(f.sig.fn_token.span))),
        syn::Item::Mod(m) => m.attrs.first().map(|a| a.pound_token.span).unwrap_or_else(|| first_token(&m.vis, m.mod_token.span)),
        _ => item.span(),
    }
}

fn first_token(vis: &syn::Visibility, fallback: Span) -> Span {
    match vis {
        syn::Visibility::Public(p) => p.span,
        syn::Visibility::Restricted(r) => r.pub_token.span,
        syn::Visibility::Inherited => fallback,
    }
}

/// The last token of the items we remove (an explicit close token: span joining may degrade to
/// the first token outside a proc-macro).
fn item_end(item: &syn::Item) -> Option<Span> {
    match item {
        syn::Item::Fn(f) => Some(f.block.brace_token.span.close()),
        syn::Item::Mod(m) => match (&m.content, &m.semi) {
            (Some((brace, _)), _) => Some(brace.span.close()),
            (None, Some(semi)) => Some(semi.span),
            (None, None) => None,
        },
        _ => None,
    }
}

fn is_test_module(item: &syn::Item) -> bool {
    let syn::Item::Mod(m) = item else { return false };
    m.attrs.iter().any(|a| a.path().is_ident("cfg") && a.meta.require_list().is_ok_and(|l| l.tokens.to_string() == "test"))
}

/// Applies edits from the end of the file backwards. Edits inside a removed range (a deleted
/// `fn main` keeps its inner rewrites) are dropped first; other overlaps are skipped.
fn apply(text: &str, edits: Vec<Edit>) -> Result<String, String> {
    let removals: Vec<(usize, usize)> = edits.iter().filter(|e| e.text.is_empty() && e.end > e.start).map(|e| (e.start, e.end)).collect();
    let mut edits: Vec<Edit> = edits
        .into_iter()
        .filter(|e| !removals.iter().any(|&(s, end)| (s, end) != (e.start, e.end) && s <= e.start && e.end <= end))
        .collect();
    edits.sort_by(|a, b| b.start.cmp(&a.start).then(b.end.cmp(&a.end)));
    let mut out = text.to_string();
    let mut floor = usize::MAX;
    for edit in edits {
        if edit.end > floor {
            continue;
        }
        if edit.start > edit.end || edit.end > out.len() {
            return Err(format!("bad edit range {}..{}", edit.start, edit.end));
        }
        out.replace_range(edit.start..edit.end, &edit.text);
        floor = edit.start;
    }
    Ok(out)
}

struct Rewriter<'a> {
    index: &'a LineIndex<'a>,
    file: &'a Path,
    src_root: &'a Path,
    overlay: &'a Overlay<'a>,
    package: &'a PackageInfo,
    edits: Vec<Edit>,
}

impl Rewriter<'_> {
    fn replace(&mut self, start: Span, end: Span, text: String) {
        self.edits.push(Edit { start: self.index.start(start), end: self.index.end(end), text });
    }

    /// Where `mod name;` declared in `self.file` lives, mirrored into the output tree.
    fn module_path(&self, name: &str) -> PathBuf {
        let stem = self.file.file_stem().and_then(|s| s.to_str()).unwrap_or_default();
        let dir = self.file.parent().unwrap_or(self.src_root);
        let base = if matches!(stem, "main" | "lib" | "mod") { dir.to_path_buf() } else { dir.join(stem) };
        let flat = base.join(format!("{name}.rs"));
        let nested = base.join(name).join("mod.rs");
        let original = if !flat.exists() && nested.exists() { nested } else { flat };
        let rel = original.strip_prefix(self.src_root).unwrap_or(&original);
        self.overlay.out_dir.join(rel)
    }

    fn env_value(&self, key: &str) -> String {
        match key {
            "CARGO_MANIFEST_DIR" => self.package.manifest_dir.clone(),
            "CARGO_PKG_VERSION" => self.package.version.clone(),
            "CARGO_PKG_NAME" | "CARGO_CRATE_NAME" | "CARGO_BIN_NAME" => self.package.name.clone(),
            _ => "dev".to_string(),
        }
    }

    /// `include_bytes!("../x.png")` is relative to the file that contains it.
    fn absolute_include(&self, literal: &str) -> String {
        let dir = self.file.parent().unwrap_or(self.src_root);
        dir.join(literal).display().to_string()
    }

    /// `include_bytes!(concat!("../dir/", $name, ".svg"))`: makes the leading relative literal
    /// absolute. Returns whether `tokens` had that shape.
    fn absolute_concat_prefix(&mut self, tokens: &TokenStream) -> bool {
        let trees: Vec<TokenTree> = tokens.clone().into_iter().collect();
        let (Some(TokenTree::Ident(concat)), Some(TokenTree::Punct(bang)), Some(TokenTree::Group(args))) = (trees.first(), trees.get(1), trees.get(2)) else {
            return false;
        };
        if concat != "concat" || bang.as_char() != '!' {
            return false;
        }
        let Some(TokenTree::Literal(first)) = args.stream().into_iter().next() else { return false };
        let Ok(lit) = syn::parse_str::<syn::LitStr>(&first.to_string()) else { return false };
        if Path::new(&lit.value()).is_absolute() {
            return true;
        }
        let mut path = self.absolute_include(&lit.value());
        if lit.value().ends_with('/') && !path.ends_with('/') {
            path.push('/');
        }
        self.replace(first.span(), first.span(), format!("{path:?}"));
        true
    }

    /// Rewrites `env!("…")`, relative include paths and `crate::` inside macro arguments.
    fn walk_tokens(&mut self, tokens: TokenStream) {
        let trees: Vec<TokenTree> = tokens.into_iter().collect();
        let mut i = 0;
        while i < trees.len() {
            if let (TokenTree::Ident(ident), Some(TokenTree::Punct(bang)), Some(TokenTree::Group(group))) = (&trees[i], trees.get(i + 1), trees.get(i + 2))
                && bang.as_char() == '!'
                && group.delimiter() != Delimiter::None
            {
                let name = ident.to_string();
                if matches!(name.as_str(), "include_bytes" | "include_str") && self.absolute_concat_prefix(&group.stream()) {
                    i += 3;
                    continue;
                }
                if let Some(literal) = single_string_literal(&group.stream()) {
                    match name.as_str() {
                        "env" => {
                            let value = self.env_value(&literal);
                            self.replace(ident.span(), group.span_close(), format!("{value:?}"));
                            i += 3;
                            continue;
                        }
                        "include_bytes" | "include_str" if !Path::new(&literal).is_absolute() => {
                            let path = self.absolute_include(&literal);
                            self.replace(ident.span(), group.span_close(), format!("{name}!({path:?})"));
                            i += 3;
                            continue;
                        }
                        _ => {}
                    }
                }
            }
            match &trees[i] {
                TokenTree::Group(group) => self.walk_tokens(group.stream()),
                TokenTree::Ident(ident) if ident == "crate" => {
                    if let (Some(TokenTree::Punct(a)), Some(TokenTree::Punct(b))) = (trees.get(i + 1), trees.get(i + 2))
                        && a.as_char() == ':'
                        && b.as_char() == ':'
                    {
                        self.replace(ident.span(), ident.span(), format!("crate::{}", self.overlay.module));
                    }
                }
                _ => {}
            }
            i += 1;
        }
    }
}

fn single_string_literal(tokens: &TokenStream) -> Option<String> {
    let mut iter = tokens.clone().into_iter();
    let first = iter.next()?;
    if iter.next().is_some() {
        return None;
    }
    let TokenTree::Literal(lit) = first else { return None };
    match syn::parse_str::<syn::LitStr>(&lit.to_string()) {
        Ok(s) => Some(s.value()),
        Err(_) => None,
    }
}

impl<'ast> Visit<'ast> for Rewriter<'_> {
    fn visit_item_mod(&mut self, m: &'ast syn::ItemMod) {
        if m.content.is_none() && !m.attrs.iter().any(|a| a.path().is_ident("path")) {
            let path = self.module_path(&m.ident.to_string());
            let at = self.index.start(m.attrs.first().map(|a| a.pound_token.span).unwrap_or_else(|| first_token(&m.vis, m.mod_token.span)));
            self.edits.push(Edit { start: at, end: at, text: format!("#[path = {:?}]\n", path.display().to_string()) });
        }
        visit::visit_item_mod(self, m);
    }

    fn visit_path(&mut self, path: &'ast syn::Path) {
        if path.leading_colon.is_none()
            && path.segments.len() > 1
            && let Some(first) = path.segments.first()
            && first.ident == "crate"
        {
            let span = first.ident.span();
            self.replace(span, span, format!("crate::{}", self.overlay.module));
        }
        visit::visit_path(self, path);
    }

    fn visit_use_path(&mut self, path: &'ast syn::UsePath) {
        if path.ident == "crate" {
            let span = path.ident.span();
            self.replace(span, span, format!("crate::{}", self.overlay.module));
        }
        visit::visit_use_path(self, path);
    }

    fn visit_macro(&mut self, mac: &'ast syn::Macro) {
        let name = mac.path.get_ident().map(ToString::to_string).unwrap_or_default();
        if matches!(name.as_str(), "include_bytes" | "include_str") && self.absolute_concat_prefix(&mac.tokens) {
            return;
        }
        if let Some(literal) = single_string_literal(&mac.tokens) {
            let close = mac.delimiter.span().close();
            let start = mac.path.span();
            match name.as_str() {
                "env" => {
                    let value = self.env_value(&literal);
                    self.replace(start, close, format!("{value:?}"));
                    return;
                }
                "include_bytes" | "include_str" if !Path::new(&literal).is_absolute() => {
                    let path = self.absolute_include(&literal);
                    self.replace(start, close, format!("{name}!({path:?})"));
                    return;
                }
                _ => {}
            }
        }
        self.walk_tokens(mac.tokens.clone());
        visit::visit_macro(self, mac);
    }
}

/// Byte offsets of line starts, to turn proc-macro2 line/column positions into byte offsets.
struct LineIndex<'a> {
    text: &'a str,
    starts: Vec<usize>,
}

impl<'a> LineIndex<'a> {
    fn new(text: &'a str) -> Self {
        let mut starts = vec![0];
        starts.extend(text.match_indices('\n').map(|(i, _)| i + 1));
        Self { text, starts }
    }

    fn offset(&self, at: LineColumn) -> usize {
        let line_start = self.starts.get(at.line.saturating_sub(1)).copied().unwrap_or(self.text.len());
        let line = &self.text[line_start..];
        // proc-macro2 columns count characters, not bytes.
        line.char_indices().nth(at.column).map(|(i, _)| line_start + i).unwrap_or(line_start + line.find('\n').unwrap_or(line.len()))
    }

    fn start(&self, span: Span) -> usize {
        self.offset(span.start())
    }

    fn end(&self, span: Span) -> usize {
        self.offset(span.end())
    }
}

/// The usual adapter `build.rs`: rewrites `upstream/<path>` (relative to the repository root)
/// into `$OUT_DIR/upstream`, for `include!(concat!(env!("OUT_DIR"), "/upstream/main.rs"))`.
pub fn build(upstream_crate: &str) {
    let manifest_dir = std::env::var_os("CARGO_MANIFEST_DIR").map(PathBuf::from).unwrap_or_default();
    let Some(root) = manifest_dir.ancestors().find(|p| p.join("upstream").join("repos.conf").is_file()) else {
        panic!("studio-overlay: no repository root above {}", manifest_dir.display());
    };
    let crate_dir = root.join("upstream").join(upstream_crate);
    let out_dir = std::env::var_os("OUT_DIR").map(PathBuf::from).unwrap_or_default().join("upstream");
    if let Err(e) = generate(&Overlay { crate_dir: &crate_dir, module: "upstream", out_dir: &out_dir }) {
        panic!("studio-overlay: {}: {e}", crate_dir.display());
    }
}
