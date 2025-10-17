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
use super::ReportOutputFileType;
use derive_builder::{Builder, UninitializedFieldError};
use headless_chrome::Browser;
use rust_ev_system_library::rust_ev_crypto_primitives::prelude::{ByteArray, EncodeTrait};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::info;

/// Options for report output
#[derive(Debug, Clone, Builder)]
#[builder(pattern = "owned", build_fn(skip))]
pub struct ReportOutputFileOptions<'a> {
    #[builder(setter(name = "add_output_type", custom))]
    output_types: Vec<ReportOutputFileType>,
    directory: &'a Path,
    filename_without_extension: &'a str,
    title: &'a str,
    logo_bytes: &'a [u8],
    nb_electoral_board: usize,
    #[builder(setter(name = "add_explicit_electoral_board_member", custom))]
    explicit_electoral_board_members: Vec<&'a str>,
    #[builder(setter(strip_option))]
    pdf_options: Option<PDFReportOptions<'a>>,
}

impl<'a> ReportOutputFileOptions<'a> {
    /// Returns the output types for the report.
    pub fn output_types(&self) -> &[ReportOutputFileType] {
        &self.output_types
    }

    /// Returns the output directory path.
    pub fn dir(&self) -> &Path {
        self.directory
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
    pub fn pdf_options<'b>(&'b self) -> Option<&'b PDFReportOptions<'a>> {
        self.pdf_options.as_ref()
    }

    pub fn signatures(&self) -> Vec<String> {
        match self.explicit_electoral_board_members().len() {
            0 => (0..(self.nb_electoral_board()))
                .map(|n| format!("Member {}", n + 1))
                .collect::<Vec<_>>(),
            _ => self
                .explicit_electoral_board_members()
                .iter()
                .map(|m| m.to_string())
                .collect::<Vec<_>>(),
        }
    }
}

impl<'a> ReportOutputFileOptionsBuilder<'a> {
    /// Add an output type to the report options
    /// If PDF is added, HTML is also added automatically
    /// Duplicates are ignored
    pub fn add_output_type(mut self, output_type: ReportOutputFileType) -> Self {
        match self.output_types.as_mut() {
            Some(v) => {
                v.push(output_type);
                v.sort();
                v.dedup();
            }
            None => self.output_types = Some(vec![output_type]),
        }
        if matches!(output_type, ReportOutputFileType::Pdf) {
            return self.add_output_type(ReportOutputFileType::Html);
        }
        self
    }

    /// Add an explicit electoral board member
    pub fn add_explicit_electoral_board_member(mut self, member: &'a str) -> Self {
        match self.explicit_electoral_board_members.as_mut() {
            Some(v) => v.push(member),
            None => self.explicit_electoral_board_members = Some(vec![member]),
        }
        self
    }

    /// Build the [ReportOutputOptions] from the builder
    ///
    /// Following rules apply:
    /// - If no output type is specified, defaults to [ReportOutputType::Txt]
    /// - The output directory, filename without extension and title are mandatory
    /// - If neither the number of electoral board members nor the explicit electoral board members are specified,
    ///   defaults to 2 members with generic names "Member 1", "Member 2"
    /// - It is not allowed to specify both the number of electoral board members and the explicit electoral board members
    /// - If PDF output type is selected, PDF report options must be specified
    pub fn build(self) -> Result<ReportOutputFileOptions<'a>, ReportError> {
        self.build_impl().map_err(ReportError::from)
    }

    fn build_impl(self) -> Result<ReportOutputFileOptions<'a>, ReportErrorImpl> {
        let dir = match self.directory {
            Some(dir) => {
                if dir.is_file() {
                    return Err(ReportErrorImpl::ReportOutputOptions(
                        "The output directory is a file".to_string(),
                    ));
                } else {
                    if !dir.exists() {
                        fs::create_dir_all(dir).map_err(|e| {
                            ReportErrorImpl::ReportOutputOptions(format!(
                                "Failed to create output directory: {}",
                                e
                            ))
                        })?;
                        info!("Created output directory: {}", dir.display());
                    }
                    dir
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
        let output_types = self.output_types.unwrap_or(vec![ReportOutputFileType::Txt]);
        if output_types
            .iter()
            .any(|t| matches!(&t, ReportOutputFileType::Pdf))
            && self.pdf_options.is_none()
        {
            return Err(ReportErrorImpl::ReportOutputOptions(
                "PDF report options must be set when PDF output type is selected".to_string(),
            ));
        }
        Ok(ReportOutputFileOptions {
            output_types,
            directory: dir,
            filename_without_extension,
            title,
            logo_bytes,
            nb_electoral_board,
            explicit_electoral_board_members,
            pdf_options: self.pdf_options.flatten(),
        })
    }
}

/// Options specific to PDF report generation
#[derive(Debug, Clone, Builder)]
#[builder(pattern = "owned", build_fn(error = "ReportError"))]
pub struct PDFReportOptions<'a> {
    path_to_browser: &'a Path,
    #[builder(default = "true")]
    sandbox: bool,
}

impl From<UninitializedFieldError> for ReportError {
    fn from(ufe: UninitializedFieldError) -> Self {
        ReportErrorImpl::ReportOutputOptions(format!("{ufe:?}")).into()
    }
}

impl<'a> PDFReportOptions<'a> {
    /// Returns a headless browser instance based on the options
    ///
    /// In test mode, uses the fetcher to get a headless browser installation.
    #[cfg(feature = "fetch")]
    pub(super) fn browser(&self) -> Result<Browser, ReportErrorImpl> {
        let fetcher_options = headless_chrome::FetcherOptions::default()
            .with_install_dir(Some(PathBuf::from(".").join("test_temp_dir")));
        Browser::new(
            headless_chrome::LaunchOptionsBuilder::default()
                .fetcher_options(fetcher_options)
                .sandbox(false)
                .headless(true)
                .build()
                .map_err(|e| ReportErrorImpl::Browser {
                    msg: "Failed to build options for headless chrome".to_string(),
                    error: e.to_string(),
                })?,
        )
        .map_err(|e| ReportErrorImpl::Browser {
            msg: "Failed to launch headless chrome".to_string(),
            error: e.to_string(),
        })
    }

    #[cfg(not(feature = "fetch"))]
    pub(super) fn browser(&self) -> Result<Browser, ReportErrorImpl> {
        Browser::new(
            headless_chrome::LaunchOptionsBuilder::default()
                .path(Some(self.path_to_browser.to_path_buf()))
                .sandbox(self.sandbox)
                .headless(true)
                .build()
                .map_err(|e| ReportErrorImpl::Browser {
                    msg: "Failed to build options for headless browser".to_string(),
                    error: e.to_string(),
                })?,
        )
        .map_err(|e| ReportErrorImpl::Browser {
            msg: "Failed to launch headless browser".to_string(),
            error: e.to_string(),
        })
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
        let pdf_options = PDFReportOptionsBuilder::default()
            .path_to_browser(&chrome_path)
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
        let opts = ReportOutputFileOptionsBuilder::default()
            .add_output_type(ReportOutputFileType::Txt)
            .add_output_type(ReportOutputFileType::Html)
            .directory(dir.as_path())
            .filename_without_extension("report")
            .title("Test Report")
            .logo_bytes(&[1, 2, 3])
            .nb_electoral_board(3)
            .build()
            .unwrap();
        assert_eq!(
            opts.output_types,
            vec![ReportOutputFileType::Txt, ReportOutputFileType::Html]
        );
        assert_eq!(opts.directory, dir.as_path());
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
        let builder = ReportOutputFileOptionsBuilder::default()
            .add_output_type(ReportOutputFileType::Pdf)
            .directory(dir.as_path())
            .filename_without_extension("report2")
            .title("Another Report")
            .logo_bytes(&[])
            .pdf_options(
                PDFReportOptionsBuilder::default()
                    .path_to_browser(&chrome_path)
                    .build()
                    .unwrap(),
            )
            .add_explicit_electoral_board_member("Alice")
            .add_explicit_electoral_board_member("Bob");
        let opts = builder.build().unwrap();
        assert_eq!(opts.nb_electoral_board, 2);
        assert_eq!(opts.explicit_electoral_board_members, vec!["Alice", "Bob"]);
    }

    #[test]
    fn builder_defaults() {
        let dir = test_dir();
        let builder = ReportOutputFileOptionsBuilder::default()
            .directory(dir.as_path())
            .filename_without_extension("default")
            .title("Default Report")
            .logo_bytes(&[]);
        let opts = builder.build().unwrap();
        assert_eq!(opts.output_types, vec![ReportOutputFileType::Txt]);
        assert_eq!(opts.nb_electoral_board, 2);
        assert_eq!(opts.explicit_electoral_board_members, Vec::<&str>::new());
    }

    #[test]
    fn builder_many_output_types() {
        let mut builder = ReportOutputFileOptionsBuilder::default();
        builder = builder.add_output_type(ReportOutputFileType::Pdf);
        assert_eq!(
            builder.output_types,
            Some(vec![ReportOutputFileType::Html, ReportOutputFileType::Pdf])
        );
        builder = builder.add_output_type(ReportOutputFileType::Txt);
        assert_eq!(
            builder.output_types,
            Some(vec![
                ReportOutputFileType::Txt,
                ReportOutputFileType::Html,
                ReportOutputFileType::Pdf
            ])
        );
        builder = builder.add_output_type(ReportOutputFileType::Txt);
        assert_eq!(
            builder.output_types,
            Some(vec![
                ReportOutputFileType::Txt,
                ReportOutputFileType::Html,
                ReportOutputFileType::Pdf
            ])
        );
        builder = builder.add_output_type(ReportOutputFileType::Html);
        assert_eq!(
            builder.output_types,
            Some(vec![
                ReportOutputFileType::Txt,
                ReportOutputFileType::Html,
                ReportOutputFileType::Pdf
            ])
        );
        let builder = builder.add_output_type(ReportOutputFileType::Html);
        assert_eq!(
            builder.output_types,
            Some(vec![
                ReportOutputFileType::Txt,
                ReportOutputFileType::Html,
                ReportOutputFileType::Pdf
            ])
        );
    }

    #[test]
    fn builder_missing_dir() {
        let builder = ReportOutputFileOptionsBuilder::default()
            .filename_without_extension("fail")
            .title("Fail Report")
            .logo_bytes(&[]);
        let err = builder.build();
        assert!(err.is_err());
    }

    #[test]
    fn builder_missing_filename() {
        let dir = test_dir();
        let builder = ReportOutputFileOptionsBuilder::default()
            .directory(dir.as_path())
            .title("Fail Report")
            .logo_bytes(&[]);
        let err = builder.build();
        assert!(err.is_err());
    }

    #[test]
    fn builder_missing_title() {
        let dir = test_dir();
        let builder = ReportOutputFileOptionsBuilder::default()
            .directory(dir.as_path())
            .filename_without_extension("fail")
            .logo_bytes(&[]);
        let err = builder.build();
        assert!(err.is_err());
    }

    #[test]
    fn builder_conflicting_nb_and_members() {
        let dir = test_dir();
        let builder = ReportOutputFileOptionsBuilder::default()
            .directory(dir.as_path())
            .filename_without_extension("fail")
            .title("Fail Report")
            .logo_bytes(&[])
            .nb_electoral_board(2)
            .add_explicit_electoral_board_member("Alice");
        let err = builder.build();
        assert!(err.is_err());
    }

    #[test]
    fn builder_with_members() {
        let dir = test_dir();
        let builder = ReportOutputFileOptionsBuilder::default()
            .directory(dir.as_path())
            .filename_without_extension("fail")
            .title("Fail Report")
            .logo_bytes(&[])
            .add_explicit_electoral_board_member("Alice")
            .add_explicit_electoral_board_member("Ben")
            .add_explicit_electoral_board_member("Toto");
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
