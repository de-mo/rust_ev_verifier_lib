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

use std::path::Path;

use super::{OutputToString, ReportError, ReportErrorImpl, ReportOutputData};
use build_html::{
    Container, ContainerType, Html, HtmlContainer, HtmlElement, HtmlPage, HtmlTag, Table,
    TableCell, TableCellType, TableRow,
};
use rust_ev_system_library::rust_ev_crypto_primitives::prelude::{ByteArray, EncodeTrait};

#[derive(Debug, Clone, Default, PartialEq, Eq, strum::Display, PartialOrd, Ord)]
pub enum ReportOutputType {
    #[default]
    #[strum(to_string = "txt")]
    Txt,
    #[strum(to_string = "html")]
    Html,
    #[strum(to_string = "pdf")]
    Pdf,
}

const STYLE: &str = r#"
    html {
    font-family: Arial, Helvetica, sans-serif;
    }
    .content {
        position:relative;
    }
    .logo {
        position: absolute;
        top: 0;
        right: 0;
        max-height: 100px;
    }
    .key_value_table {
        border-collapse: collapse;
    }
    .key_value_table th, .key_value_table td {
        border: 1px solid #ddd;
        padding: 8px;
        vertical-align: top;
    }
"#;

/// Struct to handle report generation and output
#[derive(Debug)]
pub struct ReportOutput<'a, 'b> {
    options: ReportOutputOptions<'b>,
    report_data: &'a ReportOutputData,
}

impl<'a, 'b> ReportOutput<'a, 'b> {
    /// Create a new ReportOutput instance
    pub fn new(options: ReportOutputOptions<'b>, report_data: &'a ReportOutputData) -> Self {
        Self {
            options,
            report_data,
        }
    }

    fn generate_txt(&self) -> Result<Vec<u8>, ReportErrorImpl> {
        let mut content: String = self.options.title.to_string() + "\n\n";
        content.push_str(&self.report_data.output_to_string(4));
        Ok(content.into_bytes())
    }

    fn gernerate_html(&self) -> Result<Vec<u8>, ReportErrorImpl> {
        let sections = self.report_data.blocks().iter().map(|b| {
            let mut section_container =
                Container::new(ContainerType::Div).with_header(2, b.title());

            let key_value_entries = b.key_value_entries();
            if !key_value_entries.is_empty() {
                let mut table = Table::new().with_attributes(vec![("clas", "key_value_table")]);
                for (key, value) in key_value_entries.iter() {
                    table.add_body_row(vec![key, value]);
                }
                section_container.add_table(table);
            }

            for elem in b.only_value_entries().iter() {
                section_container.add_html(
                    HtmlElement::new(HtmlTag::ParagraphText)
                        .with_attribute("style", "white-space:pre")
                        .with_raw(elem),
                );
            }
            section_container
        });

        let signatures = match self.options.explicit_electoral_board_members.len() {
            0 => (0..(self.options.nb_electoral_board))
                .map(|n| format!("Member {}", n + 1))
                .collect::<Vec<_>>(),
            _ => self
                .options
                .explicit_electoral_board_members
                .iter()
                .map(|m| m.to_string())
                .collect::<Vec<_>>(),
        };
        let style_row = format!("width:{}%", 100 / signatures.len());

        let mut signature_header_row = TableRow::new();
        for signature in signatures {
            signature_header_row.add_cell(
                TableCell::new(TableCellType::Header)
                    .with_attributes(vec![("style", style_row.as_str())])
                    .with_raw(signature.as_str()),
            )
        }
        let signature_table = Table::new()
            .with_attributes(vec![("style", "width: 100%")])
            .with_custom_header_row(signature_header_row);

        let signature_container = Container::new(ContainerType::Div)
            .with_header(2, "Signatures")
            .with_table(signature_table);

        let mut content =
            Container::new(ContainerType::Div).with_attributes(vec![("class", "content")]);

        if let Some(logo_base64) = self.options.logo_base64() {
            let logo = HtmlElement::new(HtmlTag::Image)
                .with_attribute(
                    "src",
                    format!("data:image/png;base64,{}", logo_base64).as_str(),
                )
                .with_attribute("alt", "Logo")
                .with_attribute("class", "logo");
            content.add_html(logo);
        }

        content.add_header(1, self.options.title);
        for section in sections {
            content.add_container(section);
        }
        content.add_container(signature_container);

        Ok(HtmlPage::new()
            .with_style(STYLE)
            .with_container(content)
            .to_html_string()
            .into_bytes())
    }

    fn generate_pdf(&self) -> Result<Vec<u8>, ReportErrorImpl> {
        // Placeholder for PDF generation logic
        unimplemented!();
    }

    /// Generate the reports in the specified formats and write to files
    pub fn generate(&self) -> Result<(), ReportError> {
        for output_type in &self.options.output_types {
            let content = match output_type {
                ReportOutputType::Txt => self.generate_txt()?,
                ReportOutputType::Html => self.gernerate_html()?,
                ReportOutputType::Pdf => self.generate_pdf()?,
            };
            let filepath = self.options.dir.join(format!(
                "{}.{}",
                self.options.filename_without_extension, output_type
            ));
            std::fs::write(&filepath, content).map_err(|e| {
                ReportError::from(ReportErrorImpl::IOError {
                    msg: format!("Error writing {} file {}", output_type, filepath.display()),
                    source: e,
                })
            })?;
        }
        Ok(())
    }
}

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
    pdf_options: Option<PDFReportOptions>,
}

impl<'a> ReportOutputOptions<'a> {
    /// Get the logo bytes
    pub fn logo_base64(&self) -> Option<String> {
        match self.logo_bytes {
            [] => None,
            bytes => Some(ByteArray::from_bytes(bytes).base64_encode().unwrap()),
        }
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
    pdf_options: Option<PDFReportOptions>,
}

impl<'a> ReportOutputOptionsBuilder<'a> {
    /// Create a new builder instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an output type to the report options
    pub fn add_output_type(mut self, output_type: ReportOutputType) -> Self {
        match self.output_types.as_mut() {
            Some(v) => {
                v.push(output_type);
                v.sort();
                v.dedup();
            }
            None => self.output_types = Some(vec![output_type]),
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
    pub fn set_pdf_options(mut self, pdf_options: PDFReportOptions) -> Self {
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
        if self.output_types.is_some()
            && self
                .output_types
                .as_ref()
                .unwrap()
                .iter()
                .any(|t| matches!(&t, ReportOutputType::Pdf))
            && self.pdf_options.is_none()
        {
            return Err(ReportErrorImpl::ReportOutputOptions(
                "PDF report options must be set when PDF output type is selected".to_string(),
            ));
        }
        Ok(ReportOutputOptions {
            output_types: self.output_types.unwrap_or(vec![ReportOutputType::Txt]),
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
pub struct PDFReportOptions {}

/// Builder for [PDFReportOptions]
#[derive(Debug, Default)]
pub struct PDFReportOptionsBuilder {}

#[cfg(test)]
mod test {
    use chrono::Local;

    use super::{
        super::report_output_data::{
            ReportOutputDataBlock, ReportOutputDataBlockTitle, ReportOutputDataEntry,
        },
        *,
    };
    use std::path::PathBuf;

    const VALUE_MULTILINE: &str = r"This is a value
that spans multiple
lines.";

    const VALUE_MULTILINE_2: &str = r"This is a second value
that spans multiple
lines.";

    const VALUE_MULTILINE_3: &str = r"This is a third value
that spans multiple
lines.";

    pub fn test_logo() -> Vec<u8> {
        std::fs::read(PathBuf::from("").join("test_data").join("test_logo.png")).unwrap()
    }

    pub fn test_sample() -> ReportOutputData {
        let block1 = ReportOutputDataBlock::new_with_tuples(
            ReportOutputDataBlockTitle::Fingerprints,
            &[
                ("Key1".to_string(), "Value1".to_string()),
                ("Key2".to_string(), "Value2".to_string()),
            ],
        );
        let block2 = ReportOutputDataBlock::new_with_strings(
            ReportOutputDataBlockTitle::Information,
            &["Info1".to_string(), "Info2".to_string()],
        );
        let mut block3 =
            ReportOutputDataBlock::new(ReportOutputDataBlockTitle::VerificationResults);
        block3.push(ReportOutputDataEntry::KeyValue((
            "ResultKey".to_string(),
            "ResultValue".to_string(),
        )));
        block3.push(ReportOutputDataEntry::OnlyValue(
            VALUE_MULTILINE.to_string(),
        ));
        let mut block4 = ReportOutputDataBlock::new(
            ReportOutputDataBlockTitle::VerificationErrors("test".to_string()),
        );
        block4.push(ReportOutputDataEntry::OnlyValue(
            VALUE_MULTILINE_2.to_string(),
        ));
        block4.push(ReportOutputDataEntry::OnlyValue(
            VALUE_MULTILINE_3.to_string(),
        ));
        ReportOutputData::from_vec(vec![block1, block2, block3, block4])
    }

    #[test]
    fn generate_txt_report() {
        let dir = std::env::temp_dir();
        let options = ReportOutputOptionsBuilder::new()
            .add_output_type(ReportOutputType::Txt)
            .set_dir(dir.as_path())
            .set_filename_without_extension("test_report")
            .set_title("Test Report")
            .set_logo_bytes(&[])
            .set_nb_electoral_board(3)
            .build()
            .unwrap();

        let report_data = test_sample();

        let report_output = ReportOutput::new(options, &report_data);
        let res_gen = report_output.generate_txt();
        assert!(res_gen.is_ok());
        let content = String::from_utf8(res_gen.unwrap()).unwrap();
        assert!(content.contains("Test Report"));
        assert!(content.contains("Key1: Value1"));
        assert!(content.contains("Info1"));
        assert!(content.contains("ResultKey: ResultValue"));
        assert!(content.contains("Just a value"));
    }

    #[test]
    fn generate_html_report() {
        let dir = PathBuf::from(".").join("test_temp_dir");
        let now: String = Local::now().format("%Y%m%d_%H%M%S").to_string();
        let filenname = format!("test_report_{}", now);
        let options = ReportOutputOptionsBuilder::new()
            .add_output_type(ReportOutputType::Html)
            .set_dir(dir.as_path())
            .set_filename_without_extension(filenname.as_str())
            .set_title("Test Report")
            .set_nb_electoral_board(3)
            .build()
            .unwrap();

        let report_data = test_sample();

        let report_output = ReportOutput::new(options, &report_data);
        let res_gen = report_output.generate();
        assert!(res_gen.is_ok());
    }

    #[test]
    fn generate_html_report_with_logo() {
        let dir = PathBuf::from(".").join("test_temp_dir");
        let now: String = Local::now().format("%Y%m%d_%H%M%S").to_string();
        let filenname = format!("test_report_with_logo_{}", now);
        let logo_bytes = test_logo();
        let options = ReportOutputOptionsBuilder::new()
            .add_output_type(ReportOutputType::Html)
            .set_dir(dir.as_path())
            .set_filename_without_extension(filenname.as_str())
            .set_title("Test Report")
            .set_logo_bytes(&logo_bytes)
            .set_nb_electoral_board(3)
            .build()
            .unwrap();

        let report_data = test_sample();

        let report_output = ReportOutput::new(options, &report_data);
        let res_gen = report_output.generate();
        assert!(res_gen.is_ok());
    }
}

#[cfg(test)]
mod test_builder {
    use super::*;
    use std::path::PathBuf;

    fn test_dir() -> PathBuf {
        std::env::temp_dir()
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
        let builder = ReportOutputOptionsBuilder::new()
            .add_output_type(ReportOutputType::Pdf)
            .set_dir(dir.as_path())
            .set_filename_without_extension("report2")
            .set_title("Another Report")
            .set_logo_bytes(&[])
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
        builder = builder.add_output_type(ReportOutputType::Txt);
        assert_eq!(
            builder.output_types,
            Some(vec![ReportOutputType::Txt, ReportOutputType::Pdf])
        );
        builder = builder.add_output_type(ReportOutputType::Txt);
        assert_eq!(
            builder.output_types,
            Some(vec![ReportOutputType::Txt, ReportOutputType::Pdf])
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
