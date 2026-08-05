//! Opt-in parsing checks for immutable upstream Quadlet projects.

use std::error::Error;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use quadlet_lens::diagnostic::{Diagnostic, Severity};
use quadlet_lens::model::{NamedQuadletDocument, QuadletDocument, QuadletDocumentSet, QuadletUnitType};
use quadlet_lens::source::SourceId;
use toml::{Table, Value};

#[derive(Debug)]
struct CorpusProject {
    id: String,
    repository: String,
    revision: String,
    files: Vec<CorpusFile>,
}

#[derive(Debug)]
struct CorpusFile {
    path: String,
    blob_sha: String,
    required_sections: Vec<String>,
    required_keys: Vec<String>,
}

impl CorpusProject {
    fn raw_url(&self, file: &CorpusFile) -> String {
        format!(
            "https://raw.githubusercontent.com/{}/{}/{}",
            self.repository, self.revision, file.path
        )
    }
}

#[test]
#[ignore = "downloads immutable public GitHub files; run with cargo ci-real-world-quadlet"]
fn pinned_real_world_units_parse_preserve_and_form_document_sets() -> Result<(), Box<dyn Error>> {
    let mut next_source_id = 1_u32;
    let mut total_files = 0_usize;
    let mut total_model_errors = 0_usize;
    let mut total_graph_errors = 0_usize;

    for project in load_catalog()? {
        let mut documents = Vec::with_capacity(project.files.len());
        let mut project_model_errors = 0_usize;
        for file in &project.files {
            let url = project.raw_url(file);
            let source = download(&url)?;
            let actual_blob_sha = git_blob_sha(&source)?;
            if actual_blob_sha != file.blob_sha {
                return Err(format!(
                    "{}:{}: expected blob {}, received {}",
                    project.id, file.path, file.blob_sha, actual_blob_sha
                )
                .into());
            }
            assert_required_content(&project.id, file, &source)?;

            let unit_type = unit_type(&file.path)?;
            let source_id = SourceId::new(next_source_id);
            next_source_id = next_source_id.checked_add(1).ok_or("source-id overflow")?;
            let parsed = QuadletDocument::parse(unit_type, source_id, source.clone())?;
            if !parsed.syntax().is_valid() {
                return Err(format!(
                    "{}:{}: syntax diagnostics: {}",
                    project.id,
                    file.path,
                    diagnostic_codes(parsed.syntax().diagnostics()).join(", ")
                )
                .into());
            }
            if parsed.syntax().document().render_preserved() != source {
                return Err(format!(
                    "{}:{}: preservation rendering changed source bytes",
                    project.id, file.path
                )
                .into());
            }

            let canonical = parsed.syntax().render_canonical()?;
            let canonical_parsed = QuadletDocument::parse(
                unit_type,
                SourceId::new(next_source_id.checked_add(10_000).ok_or("source-id overflow")?),
                canonical,
            )?;
            if !canonical_parsed.is_valid() {
                return Err(format!(
                    "{}:{}: canonical output was invalid; syntax: {}; model: {}",
                    project.id,
                    file.path,
                    diagnostic_codes(canonical_parsed.syntax().diagnostics()).join(", "),
                    diagnostic_codes(canonical_parsed.model_diagnostics()).join(", ")
                )
                .into());
            }

            let model_errors = error_count(parsed.model_diagnostics());
            if model_errors > 0 {
                return Err(format!(
                    "{}:{}: model diagnostics: {}",
                    project.id,
                    file.path,
                    diagnostic_codes(parsed.model_diagnostics()).join(", ")
                )
                .into());
            }
            project_model_errors += model_errors;
            let (_, document, _) = parsed.into_parts();
            documents.push(NamedQuadletDocument::new(unit_file_name(&file.path)?, document)?);
            total_files += 1;
        }

        let set = QuadletDocumentSet::new(documents)?;
        let project_graph_errors = error_count(set.diagnostics());
        if project_graph_errors > 0 {
            return Err(format!(
                "{}: document-set diagnostics: {}",
                project.id,
                diagnostic_codes(set.diagnostics()).join(", ")
            )
            .into());
        }
        println!(
            "{}: {} files; model errors={}; graph references={}, resolved={}, graph errors={}",
            project.id,
            set.documents().len(),
            project_model_errors,
            set.graph().references().len(),
            set.graph().edges().len(),
            project_graph_errors
        );
        total_model_errors += project_model_errors;
        total_graph_errors += project_graph_errors;
    }

    println!("total: {total_files} files; model errors={total_model_errors}; graph errors={total_graph_errors}");
    Ok(())
}

fn load_catalog() -> Result<Vec<CorpusProject>, Box<dyn Error>> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/real-world/corpus.toml");
    let table = std::fs::read_to_string(path)?.parse::<Table>()?;
    let projects = table
        .get("projects")
        .and_then(Value::as_array)
        .ok_or("catalog projects array missing")?;
    projects
        .iter()
        .map(|value| {
            let table = value.as_table().ok_or("catalog project must be a table")?;
            Ok(CorpusProject {
                id: string(table, "id")?.to_owned(),
                repository: string(table, "repository")?.to_owned(),
                revision: string(table, "revision")?.to_owned(),
                files: files(table)?,
            })
        })
        .collect()
}

fn files(table: &Table) -> Result<Vec<CorpusFile>, Box<dyn Error>> {
    table
        .get("files")
        .and_then(Value::as_array)
        .ok_or("catalog field `files` must be an array")?
        .iter()
        .map(|value| {
            let table = value.as_table().ok_or("catalog file must be a table")?;
            Ok(CorpusFile {
                path: string(table, "path")?.to_owned(),
                blob_sha: string(table, "blob_sha")?.to_owned(),
                required_sections: strings(table, "required_sections")?,
                required_keys: strings(table, "required_keys")?,
            })
        })
        .collect()
}

fn string<'a>(table: &'a Table, field: &str) -> Result<&'a str, Box<dyn Error>> {
    table
        .get(field)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("catalog field `{field}` must be a non-empty string").into())
}

fn strings(table: &Table, field: &str) -> Result<Vec<String>, Box<dyn Error>> {
    table
        .get(field)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("catalog field `{field}` must be an array"))?
        .iter()
        .map(|value| {
            value
                .as_str()
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned)
                .ok_or_else(|| format!("catalog field `{field}` must contain non-empty strings").into())
        })
        .collect()
}

fn assert_required_content(project_id: &str, file: &CorpusFile, source: &str) -> Result<(), Box<dyn Error>> {
    for section in &file.required_sections {
        let header = format!("[{section}]");
        if !source.lines().any(|line| line.trim() == header) {
            return Err(format!("{project_id}:{}: required section `{header}` was not found", file.path).into());
        }
    }
    for key in &file.required_keys {
        if !source.lines().any(|line| {
            line.split_once('=')
                .is_some_and(|(candidate, _)| candidate.trim() == key)
        }) {
            return Err(format!("{project_id}:{}: required key `{key}` was not found", file.path).into());
        }
    }
    Ok(())
}

fn unit_type(path: &str) -> Result<QuadletUnitType, Box<dyn Error>> {
    let extension = Path::new(path)
        .extension()
        .and_then(|value| value.to_str())
        .ok_or_else(|| format!("unit path `{path}` has no UTF-8 extension"))?;
    QuadletUnitType::from_extension(extension)
        .ok_or_else(|| format!("unit path `{path}` has unsupported extension `.{extension}`").into())
}

fn unit_file_name(path: &str) -> Result<String, Box<dyn Error>> {
    Path::new(path)
        .file_name()
        .and_then(|value| value.to_str())
        .map(ToOwned::to_owned)
        .ok_or_else(|| format!("unit path `{path}` has no UTF-8 basename").into())
}

fn diagnostic_codes(diagnostics: &[Diagnostic]) -> Vec<&str> {
    diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code().as_str())
        .collect()
}

fn error_count(diagnostics: &[Diagnostic]) -> usize {
    diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity() == Severity::Error)
        .count()
}

fn download(url: &str) -> Result<String, Box<dyn Error>> {
    let output = Command::new("curl")
        .args([
            "--disable",
            "--fail",
            "--location",
            "--proto",
            "=https",
            "--proto-redir",
            "=https",
            "--silent",
            "--show-error",
            "--max-time",
            "30",
            "--max-filesize",
            "1048576",
            url,
        ])
        .output()?;
    if !output.status.success() {
        return Err(format!(
            "curl failed for {url} with {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    Ok(String::from_utf8(output.stdout)?)
}

fn git_blob_sha(source: &str) -> Result<String, Box<dyn Error>> {
    let mut child = Command::new("git")
        .args(["hash-object", "--stdin"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("git stdin unavailable")?
        .write_all(source.as_bytes())?;
    let output = child.wait_with_output()?;
    if !output.status.success() {
        return Err(format!(
            "git hash-object failed with {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    Ok(String::from_utf8(output.stdout)?.trim().to_owned())
}
