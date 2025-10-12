// Copyright © 2025 Denis Morel
//
// This program is free software: you can redistribute it and/or modify it under
// the terms of the GNU General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version.
//
// This program is distributed in the hope that it will be useful, but WITHOUT
// ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
// FOR A PARTICULAR PURPOSE. See the GNU General Public License for more
// details.
//
// You should have received a copy of the GNU General Public License and
// a copy of the GNU General Public License along with this program. If not, see
// <https://www.gnu.org/licenses/>.

use super::super::{ReportError, ReportErrorImpl};
use super::ReportOutputType;
use headless_chrome::Browser;
use rust_ev_system_library::rust_ev_crypto_primitives::prelude::{ByteArray, EncodeTrait};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// Options for report output
#[derive(Debug, Clone)]
pub struct ReportOutputOptions<'a> {
    output_types: Vec<ReportOutputType>,
    dir: &'a Path,
    filename_without_extension: &'a str,
    title: &'a str,
    logo_bytes: &'a [u8],
    nb_electoral_board: usize,
    explicit_electoral_board_members: Vec<&'a str>,
    pdf_options: Option<PDFReportOptions<'a>>,
}

impl<'a> ReportOutputOptions<'a> {
    /// Returns the output types for the report.
    pub fn output_types(&self) -> &[ReportOutputType] {
        &self.output_types
    }

    /// Returns the output directory path.
    pub fn dir(&self) -> &Path {
        self.dir
    }

    /// Returns the filename without extension.
    pub fn filename_without_extension(&self) -> &str {
        self.filename_without_extension
    }

    /// Returns the report title.
    pub fn title(&self) -> &str {
        self.title
    }

    /// Returns the logo bytes.
    pub fn logo_bytes(&self) -> &[u8] {
        self.logo_bytes
    }

    /// Returns the logo bytes as base64 string, if present.
    pub fn logo_base64(&self) -> Option<String> {
        match self.logo_bytes {
            [] => None,
            bytes => Some(ByteArray::from_bytes(bytes).base64_encode().unwrap()),
        }
    }

    /// Returns the number of electoral board members.
    pub fn nb_electoral_board(&self) -> usize {
        self.nb_electoral_board
    }

    /// Returns the explicit electoral board members.
    pub fn explicit_electoral_board_members(&self) -> &[&str] {
        &self.explicit_electoral_board_members
    }

    /// Returns the PDF report options, if present.
    pub fn pdf_options(&self) -> Option<&PDFReportOptions> {
        self.pdf_options.as_ref()
    }
}

/// Builder for [ReportOutputOptions]
///
/// Following rules apply:
/// - If no output type is specified, defaults to [ReportOutputType::Txt]
/// - The output directory, filename without extension and title are mandatory
/// - If neither the number of electoral board members nor the explicit electoral board members are specified,
///   defaults to 2 members with generic names "Member 1", "Member 2"
/// - It is not allowed to specify both the number of electoral board members and the explicit electoral board members
/// - If PDF output type is selected, PDF report options must be specified
#[derive(Debug, Default)]
pub struct ReportOutputOptionsBuilder<'a> {
    output_types: Option<Vec<ReportOutputType>>,
    dir: Option<&'a Path>,
    filename_without_extension: Option<&'a str>,
    title: Option<&'a str>,
    logo_bytes: Option<&'a [u8]>,
    nb_electoral_board: Option<usize>,
    explicit_electoral_board_members: Option<Vec<&'a str>>,
    pdf_options: Option<PDFReportOptions<'a>>,
}

impl<'a> ReportOutputOptionsBuilder<'a> {
    /// Create a new builder instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an output type to the report options
    /// If PDF is added, HTML is also added automatically
    /// Duplicates are ignored
    pub fn add_output_type(mut self, output_type: ReportOutputType) -> Self {
        match self.output_types.as_mut() {
            Some(v) => {
                v.push(output_type);
                v.sort();
                v.dedup();
            }
            None => self.output_types = Some(vec![output_type]),
        }
        if matches!(output_type, ReportOutputType::Pdf) {
            return self.add_output_type(ReportOutputType::Html);
        }
        self
    }

    /// Set the output directory
    pub fn set_dir(mut self, dir: &'a Path) -> Self {
        self.dir = Some(dir);
        self
    }

    /// Set the filename without extension
    pub fn set_filename_without_extension(mut self, filename: &'a str) -> Self {
        self.filename_without_extension = Some(filename);
        self
    }

    /// Set the report title
    pub fn set_title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }

    /// Set the logo bytes
    pub fn set_logo_bytes(mut self, logo_bytes: &'a [u8]) -> Self {
        self.logo_bytes = Some(logo_bytes);
        self
    }

    /// Set the number of electoral board members
    pub fn set_nb_electoral_board(mut self, nb: usize) -> Self {
        self.nb_electoral_board = Some(nb);
        self
    }

    /// Add an explicit electoral board member
    pub fn add_explicit_electoral_board_members(mut self, member: &'a str) -> Self {
        match self.explicit_electoral_board_members.as_mut() {
            Some(v) => v.push(member),
            None => self.explicit_electoral_board_members = Some(vec![member]),
        }
        self
    }

    /// Set the PDF report options
    pub fn set_pdf_options(mut self, pdf_options: PDFReportOptions<'a>) -> Self {
        self.pdf_options = Some(pdf_options);
        self
    }

    /// Build the [ReportOutputOptions] from the builder
    pub fn build(self) -> Result<ReportOutputOptions<'a>, ReportError> {
        self.build_impl().map_err(ReportError::from)
    }

    fn build_impl(self) -> Result<ReportOutputOptions<'a>, ReportErrorImpl> {
        let dir = match self.dir {
            Some(dir) => {
                if dir.is_dir() {
                    dir
                } else {
                    return Err(ReportErrorImpl::ReportOutputOptions(
                        "The output directory is not a directory".to_string(),
                    ));
                }
            }
            None => {
                return Err(ReportErrorImpl::ReportOutputOptions(
                    "The output directory is not set".to_string(),
                ));
            }
        };
        let filename_without_extension = match self.filename_without_extension {
            Some(f) => f,
            None => {
                return Err(ReportErrorImpl::ReportOutputOptions(
                    "The output filename without extension is not set".to_string(),
                ));
            }
        };
        let title = match self.title {
            Some(t) => t,
            None => {
                return Err(ReportErrorImpl::ReportOutputOptions(
                    "The report title is not set".to_string(),
                ));
            }
        };
        let logo_bytes = match self.logo_bytes {
            Some(b) => b,
            None => &[],
        };
        let (nb_electoral_board, explicit_electoral_board_members) = match (
            self.nb_electoral_board,
            self.explicit_electoral_board_members,
        ) {
            (Some(n), None) => (n, vec![]),
            (None, Some(v)) => (v.len(), v),
            (None, None) => (2, vec![]),
            _ => {
                return Err(ReportErrorImpl::ReportOutputOptions(
                    "It is not allowed to set both the number of electoral board members and the explicit electoral board members".to_string(),
                ));
            }
        };
        let output_types = self.output_types.unwrap_or(vec![ReportOutputType::Txt]);
        if output_types
            .iter()
            .any(|t| matches!(&t, ReportOutputType::Pdf))
            && self.pdf_options.is_none()
        {
            return Err(ReportErrorImpl::ReportOutputOptions(
                "PDF report options must be set when PDF output type is selected".to_string(),
            ));
        }
        Ok(ReportOutputOptions {
            output_types,
            dir,
            filename_without_extension,
            title,
            logo_bytes,
            nb_electoral_board,
            explicit_electoral_board_members,
            pdf_options: self.pdf_options,
        })
    }
}

/// Options specific to PDF report generation
#[derive(Debug, Clone)]
pub struct PDFReportOptions<'a> {
    path_to_browser: &'a Path,
}

impl<'a> PDFReportOptions<'a> {
    pub(super) fn browser(&self) -> Result<Browser, ReportErrorImpl> {
        if cfg!(test) {
            let fetcher_options = headless_chrome::FetcherOptions::default()
                .with_install_dir(Some(PathBuf::from(".").join("test_temp_dir")));
            return Browser::new(
                headless_chrome::LaunchOptionsBuilder::default()
                    .args(vec![
                        &OsStr::new("--disable-gpu"),
                        &OsStr::new("--no-sandbox"),
                        &OsStr::new("--headless"),
                    ])
                    .fetcher_options(fetcher_options)
                    .build()
                    .map_err(|e| ReportErrorImpl::Browser {
                        msg: "Failed to build options for headless Chrome".to_string(),
                        error: e.to_string(),
                    })?,
            )
            .map_err(|e| ReportErrorImpl::Browser {
                msg: "Failed to launch headless Chrome".to_string(),
                error: e.to_string(),
            });
        }
        todo!("Implement PDF report browser for production with Windows and Linux Support")
    }
}

/// Builder for [PDFReportOptions]
#[derive(Debug, Default)]
pub struct PDFReportOptionsBuilder<'a> {
    path_to_browser: Option<&'a Path>,
}

impl<'a> PDFReportOptionsBuilder<'a> {
    /// Create a new builder instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the path to the browser executable
    pub fn set_path_to_browser(mut self, path: &'a Path) -> Self {
        self.path_to_browser = Some(path);
        self
    }

    /// Build the [PDFReportOptions] from the builder
    pub fn build(self) -> Result<PDFReportOptions<'a>, ReportError> {
        self.build_impl().map_err(ReportError::from)
    }

    fn build_impl(self) -> Result<PDFReportOptions<'a>, ReportErrorImpl> {
        let path_to_browser = match self.path_to_browser {
            Some(p) => p,
            None => {
                return Err(ReportErrorImpl::ReportOutputOptions(
                    "The path to the browser executable is not set".to_string(),
                ));
            }
        };
        if !path_to_browser.is_file() {
            return Err(ReportErrorImpl::ReportOutputOptions(
                "The path to the browser executable is not a file".to_string(),
            ));
        }
        Ok(PDFReportOptions { path_to_browser })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::path::PathBuf;

    fn test_dir() -> PathBuf {
        std::env::temp_dir()
    }

    #[test]
    fn test_browser_in_test() {
        let chrome_path = PathBuf::from(".").join("test_data").join("chrome.exe.txt");
        let pdf_options = PDFReportOptionsBuilder::new()
            .set_path_to_browser(&chrome_path)
            .build()
            .unwrap();
        let browser_res = pdf_options.browser();
        assert!(browser_res.is_ok());
        let browser = browser_res.unwrap();
        assert!(browser.get_process_id().is_some());
    }

    #[test]
    fn builder_sets_all_fields() {
        let dir = test_dir();
        let builder = ReportOutputOptionsBuilder::new()
            .add_output_type(ReportOutputType::Txt)
            .add_output_type(ReportOutputType::Html)
            .set_dir(dir.as_path())
            .set_filename_without_extension("report")
            .set_title("Test Report")
            .set_logo_bytes(&[1, 2, 3])
            .set_nb_electoral_board(3);
        let opts = builder.build().unwrap();
        assert_eq!(
            opts.output_types,
            vec![ReportOutputType::Txt, ReportOutputType::Html]
        );
        assert_eq!(opts.dir, dir.as_path());
        assert_eq!(opts.filename_without_extension, "report");
        assert_eq!(opts.title, "Test Report");
        assert_eq!(opts.logo_bytes, &[1, 2, 3]);
        assert_eq!(opts.nb_electoral_board, 3);
        assert_eq!(opts.explicit_electoral_board_members, Vec::<&str>::new());
    }

    #[test]
    fn builder_explicit_members_only() {
        let dir = test_dir();
        let chrome_path = PathBuf::from(".").join("test_data").join("chrome.exe.txt");
        let builder = ReportOutputOptionsBuilder::new()
            .add_output_type(ReportOutputType::Pdf)
            .set_dir(dir.as_path())
            .set_filename_without_extension("report2")
            .set_title("Another Report")
            .set_logo_bytes(&[])
            .set_pdf_options(
                PDFReportOptionsBuilder::new()
                    .set_path_to_browser(&chrome_path)
                    .build()
                    .unwrap(),
            )
            .add_explicit_electoral_board_members("Alice")
            .add_explicit_electoral_board_members("Bob");
        let opts = builder.build().unwrap();
        assert_eq!(opts.nb_electoral_board, 2);
        assert_eq!(opts.explicit_electoral_board_members, vec!["Alice", "Bob"]);
    }

    #[test]
    fn builder_defaults() {
        let dir = test_dir();
        let builder = ReportOutputOptionsBuilder::new()
            .set_dir(dir.as_path())
            .set_filename_without_extension("default")
            .set_title("Default Report")
            .set_logo_bytes(&[]);
        let opts = builder.build().unwrap();
        assert_eq!(opts.output_types, vec![ReportOutputType::Txt]);
        assert_eq!(opts.nb_electoral_board, 2);
        assert_eq!(opts.explicit_electoral_board_members, Vec::<&str>::new());
    }

    #[test]
    fn builder_many_output_types() {
        let mut builder = ReportOutputOptionsBuilder::new();
        builder = builder.add_output_type(ReportOutputType::Pdf);
        assert_eq!(
            builder.output_types,
            Some(vec![ReportOutputType::Html, ReportOutputType::Pdf])
        );
        builder = builder.add_output_type(ReportOutputType::Txt);
        assert_eq!(
            builder.output_types,
            Some(vec![
                ReportOutputType::Txt,
                ReportOutputType::Html,
                ReportOutputType::Pdf
            ])
        );
        builder = builder.add_output_type(ReportOutputType::Txt);
        assert_eq!(
            builder.output_types,
            Some(vec![
                ReportOutputType::Txt,
                ReportOutputType::Html,
                ReportOutputType::Pdf
            ])
        );
        builder = builder.add_output_type(ReportOutputType::Html);
        assert_eq!(
            builder.output_types,
            Some(vec![
                ReportOutputType::Txt,
                ReportOutputType::Html,
                ReportOutputType::Pdf
            ])
        );
        let builder = builder.add_output_type(ReportOutputType::Html);
        assert_eq!(
            builder.output_types,
            Some(vec![
                ReportOutputType::Txt,
                ReportOutputType::Html,
                ReportOutputType::Pdf
            ])
        );
    }

    #[test]
    fn builder_missing_dir() {
        let builder = ReportOutputOptionsBuilder::new()
            .set_filename_without_extension("fail")
            .set_title("Fail Report")
            .set_logo_bytes(&[]);
        let err = builder.build();
        assert!(err.is_err());
    }

    #[test]
    fn builder_missing_filename() {
        let dir = test_dir();
        let builder = ReportOutputOptionsBuilder::new()
            .set_dir(dir.as_path())
            .set_title("Fail Report")
            .set_logo_bytes(&[]);
        let err = builder.build();
        assert!(err.is_err());
    }

    #[test]
    fn builder_missing_title() {
        let dir = test_dir();
        let builder = ReportOutputOptionsBuilder::new()
            .set_dir(dir.as_path())
            .set_filename_without_extension("fail")
            .set_logo_bytes(&[]);
        let err = builder.build();
        assert!(err.is_err());
    }

    #[test]
    fn builder_conflicting_nb_and_members() {
        let dir = test_dir();
        let builder = ReportOutputOptionsBuilder::new()
            .set_dir(dir.as_path())
            .set_filename_without_extension("fail")
            .set_title("Fail Report")
            .set_logo_bytes(&[])
            .set_nb_electoral_board(2)
            .add_explicit_electoral_board_members("Alice");
        let err = builder.build();
        assert!(err.is_err());
    }

    #[test]
    fn builder_with_members() {
        let dir = test_dir();
        let builder = ReportOutputOptionsBuilder::new()
            .set_dir(dir.as_path())
            .set_filename_without_extension("fail")
            .set_title("Fail Report")
            .set_logo_bytes(&[])
            .add_explicit_electoral_board_members("Alice")
            .add_explicit_electoral_board_members("Ben")
            .add_explicit_electoral_board_members("Toto");
        let res = builder.build();
        assert!(res.is_ok());
        let opts = res.unwrap();
        assert_eq!(opts.nb_electoral_board, 3);
        assert_eq!(
            opts.explicit_electoral_board_members,
            vec!["Alice", "Ben", "Toto"]
        );
    }
}
