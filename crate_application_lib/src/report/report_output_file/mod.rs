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

mod options;

use std::path::PathBuf;

use crate::canonicalize_path_os_dependent;

use super::{OutputToString, ReportError, ReportErrorImpl, ReportOutputData};
use build_html::{
    Container, ContainerType, Html, HtmlContainer, HtmlElement, HtmlPage, HtmlTag, Table,
    TableCell, TableCellType, TableRow,
};
pub use options::*;
use tracing::{error, info};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, strum::Display, PartialOrd, Ord)]
pub enum ReportOutputFileType {
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
    .content p {
        font-size: 90%;
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
        font-size: 90%;
    }
"#;

/// Struct to handle report generation and output
#[derive(Debug)]
pub struct ReportOutputFile<'a> {
    options: ReportOutputFileOptions,
    report_data: &'a ReportOutputData,
    txt_filepath: Option<PathBuf>,
    html_filepath: Option<PathBuf>,
    pdf_filepath: Option<PathBuf>,
}

impl<'a> ReportOutputFile<'a> {
    /// Create a new ReportOutput instance
    pub fn new(options: ReportOutputFileOptions, report_data: &'a ReportOutputData) -> Self {
        Self {
            options,
            report_data,
            txt_filepath: None,
            html_filepath: None,
            pdf_filepath: None,
        }
    }

    fn generate_txt(&self) -> Result<Vec<u8>, ReportErrorImpl> {
        let mut content: String = self.report_data.output_to_string(4);
        content.push_str("\n\nSignatures:\n\n");
        content.push_str(&self.options.signatures().join("\n\n"));
        Ok(content.into_bytes())
    }

    fn generate_html(&self) -> Result<Vec<u8>, ReportErrorImpl> {
        let sections = self.report_data.blocks().iter().map(|b| {
            let mut section_container =
                Container::new(ContainerType::Div).with_header(2, b.title());

            let key_value_entries = b.key_value_entries();
            if !key_value_entries.is_empty() {
                let mut table = Table::new().with_attributes(vec![("class", "key_value_table")]);
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

        let signatures = self.options.signatures();
        let style_row = format!("width:{}%", 100 / signatures.len());

        let mut signature_header_row = TableRow::new();
        for signature in signatures.iter() {
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

        content.add_header(1, self.report_data.metadata().title());
        content.add_html(HtmlElement::new(HtmlTag::ParagraphText).with_raw(
            format!("Date / Time: {}", self.report_data.metadata().date_time()).as_str(),
        ));
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
        let pdf_options = self.options.pdf_options().as_ref().unwrap();
        let browser = pdf_options.browser()?;
        let file_path = format!(
            "file://{}",
            canonicalize_path_os_dependent(self.html_filepath.as_ref().unwrap())
        );
        let tab = browser.new_tab().map_err(|e| ReportErrorImpl::Browser {
            msg: "Error opening tab".to_string(),
            error: e.to_string(),
        })?;

        tab.navigate_to(&file_path)
            .map_err(|e| ReportErrorImpl::Browser {
                msg: format!("Error navigating to {}", file_path),
                error: e.to_string(),
            })?
            .wait_until_navigated()
            .map_err(|e| ReportErrorImpl::Browser {
                msg: format!("Error waiting for navigation to {}", file_path),
                error: e.to_string(),
            })?
            .print_to_pdf(None)
            .map_err(|e| ReportErrorImpl::Browser {
                msg: format!("Error generating PDF from {}", file_path),
                error: e.to_string(),
            })
    }

    /// Generate the reports in the specified formats and write to files
    pub fn generate(&mut self) -> Vec<ReportError> {
        let mut res = vec![];
        for output_type in self.options.output_types().iter() {
            let filepath = self.options.directory().join(format!(
                "{}.{}",
                self.options.filename_without_extension(),
                output_type
            ));
            let content_res = match output_type {
                ReportOutputFileType::Txt => {
                    let res = self.generate_txt();
                    self.txt_filepath = Some(filepath.clone());
                    res
                }
                ReportOutputFileType::Html => {
                    let res = self.generate_html();
                    self.html_filepath = Some(filepath.clone());
                    res
                }
                ReportOutputFileType::Pdf => {
                    let res = self.generate_pdf();
                    self.pdf_filepath = Some(filepath.clone());
                    res
                }
            };
            match content_res {
                Ok(content) => match std::fs::write(&filepath, content) {
                    Ok(_) => info!("Generated report {}: {}", output_type, filepath.display()),
                    Err(e) => {
                        error!(
                            "Error writing {} file {}: {:?}",
                            output_type,
                            filepath.display(),
                            e
                        );
                        res.push(ReportError::from(ReportErrorImpl::IOError {
                            msg: format!("Error writing {} file", filepath.display()),
                            source: e,
                        }));
                    }
                },
                Err(e) => {
                    error!("Error generating {} report: {:?}", output_type, e);
                    res.push(ReportError::from(ReportErrorImpl::ReportError {
                        path: filepath.clone(),
                        source: Box::new(ReportError::from(e)),
                    }));
                }
            }
        }
        res
    }
}

#[cfg(test)]
pub mod test {
    use super::{
        super::report_output_data::{
            ReportOutputDataBlock, ReportOutputDataBlockTitle, ReportOutputDataEntry,
        },
        *,
    };
    use crate::report::report_output_data::ReportOutputDataMetaDataBuilder;
    use chrono::Local;
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
        ReportOutputData::from_vec(
            ReportOutputDataMetaDataBuilder::default()
                .title("Verifier Test Report")
                .date_time(
                    Local::now()
                        .format("%Y-%m-%d %H:%M:%S")
                        .to_string()
                        .as_str(),
                )
                .seed("KT_20250101_TT99")
                .build()
                .unwrap(),
            vec![block1, block2, block3, block4],
        )
    }

    #[test]
    fn generate_txt_report() {
        let dir = std::env::temp_dir();
        let options = ReportOutputFileOptionsBuilder::default()
            .add_output_type(ReportOutputFileType::Txt)
            .directory(dir.as_path())
            .filename_without_extension("test_report")
            .logo_bytes(vec![])
            .nb_electoral_board(3usize)
            .build()
            .unwrap();

        let report_data = test_sample();

        let report_output = ReportOutputFile::new(options, &report_data);
        let res_gen = report_output.generate_txt();
        assert!(res_gen.is_ok());
        let content = String::from_utf8(res_gen.unwrap()).unwrap();
        assert!(content.contains("Verifier Test Report"));
        assert!(content.contains("Key1: Value1"));
        assert!(content.contains("Info1"));
        assert!(content.contains("ResultKey: ResultValue"));
    }

    #[test]
    fn generate_html_report() {
        let dir = PathBuf::from(".").join("test_temp_dir");
        let now: String = Local::now().format("%Y%m%d_%H%M%S").to_string();
        let filenname = format!("test_report_{}", now);
        let options = ReportOutputFileOptionsBuilder::default()
            .add_output_type(ReportOutputFileType::Html)
            .directory(dir.as_path())
            .filename_without_extension(filenname.as_str())
            .nb_electoral_board(3usize)
            .build()
            .unwrap();

        let report_data = test_sample();

        let mut report_output = ReportOutputFile::new(options, &report_data);
        let res_gen = report_output.generate();
        assert!(res_gen.is_empty());
    }

    #[test]
    fn generate_html_report_with_logo() {
        let dir = PathBuf::from(".").join("test_temp_dir");
        let now: String = Local::now().format("%Y%m%d_%H%M%S").to_string();
        let filenname = format!("test_report_with_logo_{}", now);
        let options = ReportOutputFileOptionsBuilder::default()
            .add_output_type(ReportOutputFileType::Html)
            .directory(dir.as_path())
            .filename_without_extension(filenname.as_str())
            .logo_bytes(test_logo())
            .nb_electoral_board(3usize)
            .build()
            .unwrap();

        let report_data = test_sample();

        let mut report_output = ReportOutputFile::new(options, &report_data);
        let res_gen = report_output.generate();
        assert!(res_gen.is_empty());
    }

    #[test]
    fn generate_pdf_report_with_logo() {
        let dir = PathBuf::from(".").join("test_temp_dir");
        let now: String = Local::now().format("%Y%m%d_%H%M%S").to_string();
        let filenname = format!("test_report_with_logo_{}", now);
        let chrome_path = PathBuf::from(".").join("test_data").join("chrome.exe.txt");
        let options = ReportOutputFileOptionsBuilder::default()
            .add_output_type(ReportOutputFileType::Pdf)
            .directory(dir.as_path())
            .filename_without_extension(filenname.as_str())
            .logo_bytes(test_logo())
            .nb_electoral_board(3usize)
            .pdf_options(
                PDFReportOptionsBuilder::default()
                    .path_to_browser(&chrome_path)
                    .build()
                    .unwrap(),
            )
            .build()
            .unwrap();

        let report_data = test_sample();

        let mut report_output = ReportOutputFile::new(options, &report_data);
        let res_gen = report_output.generate();
        assert!(res_gen.is_empty(), "{:?}", res_gen);
    }
}
