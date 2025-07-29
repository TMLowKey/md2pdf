use anyhow::{Context, Result};
use base64::Engine;
use clap::Parser;
use headless_chrome::{Browser, LaunchOptions};
use pulldown_cmark::{html, Options, Parser as MdParser};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Parser)]
#[command(name = "markdown-to-pdf")]
#[command(about = "Convert Markdown files or directories to PDF")]
struct Args {
    /// Input Markdown file or directory path
    #[arg(short, long)]
    input: PathBuf,

    /// Output PDF file path
    #[arg(short, long)]
    output: PathBuf,

    /// Enable dark mode theme
    #[arg(long)]
    dark_mode: bool,

    /// Document title for directories
    #[arg(long, default_value = "Documentation")]
    title: String,
}

#[derive(Debug, Clone)]
struct MarkdownFile {
    path: PathBuf,
    content: String,
    name: String,
}

fn collect_markdown_files(dir: &Path) -> Result<BTreeMap<String, Vec<MarkdownFile>>> {
    let mut files_by_dir = BTreeMap::new();

    for entry in WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "md"))
    {
        let path = entry.path();
        let content =
            fs::read_to_string(path).with_context(|| format!("Failed to read file: {:?}", path))?;

        let name = path
            .file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or("Untitled")
            .to_string();

        let parent_dir = path
            .parent()
            .and_then(|p| p.strip_prefix(dir).ok())
            .and_then(|p| p.to_str())
            .unwrap_or("")
            .to_string();

        let dir_key = if parent_dir.is_empty() {
            "Root".to_string()
        } else {
            parent_dir.replace('/', " > ")
        };

        files_by_dir
            .entry(dir_key)
            .or_insert_with(Vec::new)
            .push(MarkdownFile {
                path: path.to_path_buf(),
                content,
                name,
            });
    }

    // Sort files alphabetically within each directory
    for files in files_by_dir.values_mut() {
        files.sort_by(|a, b| a.name.cmp(&b.name));
    }

    Ok(files_by_dir)
}

fn create_combined_markdown(
    files_by_dir: BTreeMap<String, Vec<MarkdownFile>>,
    title: &str,
) -> String {
    let mut combined = String::new();

    combined.push_str(&format!("# {}\n\n", title));

    for (dir_name, files) in files_by_dir {
        if dir_name != "Root" {
            combined.push_str(&format!("# {}\n\n", dir_name));
        }

        for file in files {
            combined.push_str(&format!("## {}\n\n", file.name));

            let processed_content = preprocess_markdown(&file.content);
            combined.push_str(&processed_content);
            combined.push_str("\n\n---\n\n");
        }
    }

    combined
}

fn preprocess_markdown_single_file(markdown: &str) -> String {
    let mut result = String::new();
    let mut in_code_block = false;

    for line in markdown.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }

        if in_code_block {
            continue;
        }

        result.push_str(line);
        result.push('\n');
    }

    result
}

fn preprocess_markdown(markdown: &str) -> String {
    let mut result = String::new();
    let mut in_code_block = false;

    for line in markdown.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }

        if in_code_block {
            continue;
        }

        // Adjust heading levels for proper hierarchy
        if trimmed.starts_with('#') && !in_code_block {
            let hash_count = trimmed.chars().take_while(|&c| c == '#').count();
            let rest_of_line = &trimmed[hash_count..];

            // Add 2 levels to maintain document structure
            let new_line = format!("{}{}", "#".repeat(hash_count + 2), rest_of_line);
            result.push_str(&new_line);
        } else {
            result.push_str(line);
        }
        result.push('\n');
    }

    result
}

fn markdown_to_html(markdown: &str, dark_mode: bool) -> String {
    let processed_markdown = preprocess_markdown_single_file(markdown);

    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_TASKLISTS);

    let parser = MdParser::new_ext(&processed_markdown, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    let theme = if dark_mode {
        "background-color: #1a1a1a; color: #e0e0e0;"
    } else {
        "background-color: white; color: black;"
    };

    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Markdown to PDF</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            line-height: 1.6;
            max-width: 800px;
            margin: 0 auto;
            padding: 20px;
            {theme}
        }}
        
        h1, h2, h3, h4, h5, h6 {{
            margin-top: 1.5em;
            margin-bottom: 0.5em;
        }}
        
        h1 {{ font-size: 2em; border-bottom: 2px solid #eee; padding-bottom: 0.3em; }}
        h2 {{ font-size: 1.5em; border-bottom: 1px solid #eee; padding-bottom: 0.3em; }}
        
        code {{
            background-color: {code_bg};
            padding: 2px 4px;
            border-radius: 3px;
            font-family: 'Courier New', monospace;
        }}
        
        pre {{
            background-color: {code_bg};
            padding: 15px;
            border-radius: 5px;
            overflow-x: auto;
        }}
        
        pre code {{
            background-color: transparent;
            padding: 0;
        }}
        
        blockquote {{
            border-left: 4px solid #ddd;
            margin: 0;
            padding-left: 20px;
            color: #666;
        }}
        
        table {{
            border-collapse: collapse;
            width: 100%;
            margin: 1em 0;
        }}
        
        th, td {{
            border: 1px solid #ddd;
            padding: 8px 12px;
            text-align: left;
        }}
        
        th {{
            background-color: {header_bg};
            font-weight: bold;
        }}
        
        img {{
            max-width: 100%;
            height: auto;
        }}
        
        ul, ol {{
            margin: 1em 0;
            padding-left: 2em;
        }}
        
        li {{
            margin: 0.5em 0;
        }}
    </style>
</head>
<body>
{html_output}
</body>
</html>"#,
        theme = theme,
        code_bg = if dark_mode { "#2d2d2d" } else { "#f5f5f5" },
        header_bg = if dark_mode { "#3a3a3a" } else { "#f9f9f9" },
        html_output = html_output
    )
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    if !args.input.exists() {
        anyhow::bail!("Input path does not exist: {:?}", args.input);
    }

    let html_content = if args.input.is_file() {
        // Single file mode
        if args.input.extension().map_or(true, |ext| ext != "md") {
            anyhow::bail!("File must have .md extension: {:?}", args.input);
        }

        println!("Reading markdown file: {:?}", args.input);
        let markdown_content = fs::read_to_string(&args.input)
            .with_context(|| format!("Failed to read file: {:?}", args.input))?;

        println!("Converting markdown to HTML...");
        let processed_markdown = preprocess_markdown_single_file(&markdown_content);
        markdown_to_html(&processed_markdown, args.dark_mode)
    } else if args.input.is_dir() {
        // Directory mode
        println!("Scanning for markdown files in: {:?}", args.input);
        let files_by_dir = collect_markdown_files(&args.input)?;

        if files_by_dir.is_empty() {
            anyhow::bail!("No .md files found in directory");
        }

        let total_files: usize = files_by_dir.values().map(|v| v.len()).sum();
        println!(
            "Found {} markdown files in {} directories",
            total_files,
            files_by_dir.len()
        );

        for (dir, files) in &files_by_dir {
            println!("  📁 {}: {} files", dir, files.len());
            for file in files {
                println!("    📄 {}", file.name);
            }
        }

        println!("Combining all files into single document...");
        let combined_markdown = create_combined_markdown(files_by_dir, &args.title);

        println!("Converting combined markdown to HTML...");
        markdown_to_html(&combined_markdown, args.dark_mode)
    } else {
        anyhow::bail!("Input path is neither file nor directory: {:?}", args.input);
    };

    println!("Starting Chrome for PDF generation...");
    let browser = Browser::new(
        LaunchOptions::default_builder()
            .headless(true)
            .build()
            .expect("Could not configure Chrome"),
    )
    .context("Failed to start Chrome. Make sure Chrome or Chromium is installed.")?;

    let tab = browser.new_tab().context("Failed to create new tab")?;

    println!("Loading HTML content...");
    let data_uri = format!(
        "data:text/html;charset=utf-8;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(&html_content)
    );
    tab.navigate_to(&data_uri)
        .context("Failed to load HTML content")?;

    tab.wait_until_navigated()
        .context("Page navigation timeout")?;

    println!("Generating PDF: {:?}", args.output);
    let pdf_data = tab
        .print_to_pdf(Some(headless_chrome::types::PrintToPdfOptions {
            landscape: Some(false),
            display_header_footer: Some(false),
            print_background: Some(true),
            scale: Some(1.0),
            paper_width: Some(8.27),  // A4 width in inches
            paper_height: Some(11.7), // A4 height in inches
            margin_top: Some(0.4),
            margin_bottom: Some(0.4),
            margin_left: Some(0.4),
            margin_right: Some(0.4),
            page_ranges: None,
            ignore_invalid_page_ranges: Some(false),
            header_template: None,
            footer_template: None,
            prefer_css_page_size: Some(false),
            transfer_mode: None,
            generate_document_outline: Some(false),
            generate_tagged_pdf: Some(false),
        }))
        .context("Failed to generate PDF")?;

    fs::write(&args.output, pdf_data)
        .with_context(|| format!("Failed to save PDF: {:?}", args.output))?;

    println!("✅ PDF successfully created: {:?}", args.output);
    Ok(())
}

