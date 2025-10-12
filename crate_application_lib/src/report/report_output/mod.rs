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

use super::{OutputToString, ReportError, ReportErrorImpl, ReportOutputData};
use build_html::{
    Container, ContainerType, Html, HtmlContainer, HtmlElement, HtmlPage, HtmlTag, Table,
    TableCell, TableCellType, TableRow,
};
pub use options::*;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, strum::Display, PartialOrd, Ord)]
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
    txt_filepath: Option<PathBuf>,
    html_filepath: Option<PathBuf>,
    pdf_filepath: Option<PathBuf>,
}

impl<'a, 'b> ReportOutput<'a, 'b> {
    /// Create a new ReportOutput instance
    pub fn new(options: ReportOutputOptions<'b>, report_data: &'a ReportOutputData) -> Self {
        Self {
            options,
            report_data,
            txt_filepath: None,
            html_filepath: None,
            pdf_filepath: None,
        }
    }

    fn generate_txt(&self) -> Result<Vec<u8>, ReportErrorImpl> {
        let mut content: String = self.options.title().to_string() + "\n\n";
        content.push_str(&self.report_data.output_to_string(4));
        Ok(content.into_bytes())
    }

    fn generate_html(&self) -> Result<Vec<u8>, ReportErrorImpl> {
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

        let signatures = match self.options.explicit_electoral_board_members().len() {
            0 => (0..(self.options.nb_electoral_board()))
                .map(|n| format!("Member {}", n + 1))
                .collect::<Vec<_>>(),
            _ => self
                .options
                .explicit_electoral_board_members()
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

        content.add_header(1, self.options.title());
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
        let pdf_options = self.options.pdf_options().unwrap();
        let browser = pdf_options.browser()?;
        let file_path = format!(
            "file://{}",
            self.html_filepath
                .as_ref()
                .unwrap()
                .canonicalize()
                .unwrap()
                .to_str()
                .unwrap()
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
    pub fn generate(&mut self) -> Result<(), ReportError> {
        for output_type in self.options.output_types().iter() {
            let filepath = self.options.dir().join(format!(
                "{}.{}",
                self.options.filename_without_extension(),
                output_type
            ));
            let content = match output_type {
                ReportOutputType::Txt => {
                    let res = self.generate_txt()?;
                    self.txt_filepath = Some(filepath.clone());
                    res
                }
                ReportOutputType::Html => {
                    let res = self.generate_html()?;
                    self.html_filepath = Some(filepath.clone());
                    res
                }
                ReportOutputType::Pdf => {
                    let res = self.generate_pdf()?;
                    self.pdf_filepath = Some(filepath.clone());
                    res
                }
            };
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
        let options = ReportOutputOptionsBuilder::default()
            .add_output_type(ReportOutputType::Txt)
            .directory(dir.as_path())
            .filename_without_extension("test_report")
            .title("Test Report")
            .logo_bytes(&[])
            .nb_electoral_board(3)
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
    }

    #[test]
    fn generate_html_report() {
        let dir = PathBuf::from(".").join("test_temp_dir");
        let now: String = Local::now().format("%Y%m%d_%H%M%S").to_string();
        let filenname = format!("test_report_{}", now);
        let options = ReportOutputOptionsBuilder::default()
            .add_output_type(ReportOutputType::Html)
            .directory(dir.as_path())
            .filename_without_extension(filenname.as_str())
            .title("Test Report")
            .nb_electoral_board(3)
            .build()
            .unwrap();

        let report_data = test_sample();

        let mut report_output = ReportOutput::new(options, &report_data);
        let res_gen = report_output.generate();
        assert!(res_gen.is_ok());
    }

    #[test]
    fn generate_html_report_with_logo() {
        let dir = PathBuf::from(".").join("test_temp_dir");
        let now: String = Local::now().format("%Y%m%d_%H%M%S").to_string();
        let filenname = format!("test_report_with_logo_{}", now);
        let logo_bytes = test_logo();
        let options = ReportOutputOptionsBuilder::default()
            .add_output_type(ReportOutputType::Html)
            .directory(dir.as_path())
            .filename_without_extension(filenname.as_str())
            .title("Test Report")
            .logo_bytes(&logo_bytes)
            .nb_electoral_board(3)
            .build()
            .unwrap();

        let report_data = test_sample();

        let mut report_output = ReportOutput::new(options, &report_data);
        let res_gen = report_output.generate();
        assert!(res_gen.is_ok());
    }

    #[test]
    fn generate_pdf_report_with_logo() {
        let dir = PathBuf::from(".").join("test_temp_dir");
        let now: String = Local::now().format("%Y%m%d_%H%M%S").to_string();
        let filenname = format!("test_report_with_logo_{}", now);
        let logo_bytes = test_logo();
        let chrome_path = PathBuf::from(".").join("test_data").join("chrome.exe.txt");
        let options = ReportOutputOptionsBuilder::default()
            .add_output_type(ReportOutputType::Pdf)
            .directory(dir.as_path())
            .filename_without_extension(filenname.as_str())
            .title("Test Report")
            .logo_bytes(&logo_bytes)
            .nb_electoral_board(3)
            .pdf_options(
                PDFReportOptionsBuilder::default()
                    .path_to_browser(&chrome_path)
                    .build()
                    .unwrap(),
            )
            .build()
            .unwrap();

        let report_data = test_sample();

        let mut report_output = ReportOutput::new(options, &report_data);
        let res_gen = report_output.generate();
        assert!(res_gen.is_ok(), "{:?}", res_gen.err());
    }
}
