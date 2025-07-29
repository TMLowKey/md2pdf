# Markdown to PDF Converter

A simple Rust tool that converts Markdown files to PDF documents using Chrome's rendering engine.

## Features

- Convert single Markdown files to PDF
- Process entire directories recursively 
- Automatically organize content with proper heading hierarchy
- Dark mode support
- Clean, readable PDF output
- Code block filtering (removes fenced code blocks)

## Installation

Make sure you have Rust installed, then clone and build:

```bash
git clone <repository-url>
cd markdown-to-pdf
cargo build --release
```

## Requirements

- Rust 1.70+
- Chrome or Chromium browser installed on your system

## Usage

### Single File

Convert a single Markdown file:

```bash
cargo run -- --input document.md --output result.pdf
```

### Directory

Process all `.md` files in a directory and combine them into one PDF:

```bash
cargo run -- --input ./my-docs --output combined.pdf
```

### Options

- `--dark-mode`: Use dark theme for the PDF
- `--title "Custom Title"`: Set document title (for directories only)

```bash
cargo run -- --input ./project-docs --output docs.pdf --title "Project Documentation" --dark-mode
```

## How it Works

When processing directories, the tool creates a hierarchical structure:

- Directory names become level 1 headings (`#`)
- File names become level 2 headings (`##`) 
- Content from files starts at level 3 (`###`)

For example, if you have:
```
docs/
├── characters/
│   ├── alice.md
│   └── bob.md
└── plot.md
```

The output will be structured as:
```markdown
# Documentation
# characters
## alice
### (content from alice.md)...
## bob  
### (content from bob.md)...
# Root
## plot
### (content from plot.md)...
```

## Notes

- Code blocks (fenced with triple backticks) are automatically removed from the output
- Files are sorted alphabetically within each directory
- The tool uses Chrome's print-to-PDF functionality for high-quality output
- A4 paper size with reasonable margins is used by default