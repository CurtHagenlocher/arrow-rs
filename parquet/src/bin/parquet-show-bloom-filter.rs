// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

//! Binary file to read bloom filter data from a Parquet file.
//!
//! # Install
//!
//! `parquet-show-bloom-filter` can be installed using `cargo`:
//! ```
//! cargo install parquet --features=cli
//! ```
//! After this `parquet-show-bloom-filter` should be available:
//! ```
//! parquet-show-bloom-filter XYZ.parquet id a
//! ```
//!
//! The binary can also be built from the source code and run as follows:
//! ```
//! cargo run --features=cli --bin parquet-show-bloom-filter -- --file-name XYZ.parquet --column id --values a
//! ```

use clap::Parser;
use parquet::basic::Type;
use parquet::bloom_filter::Sbbf;
use parquet::file::metadata::ColumnChunkMetaData;
use parquet::file::{
    properties::ReaderProperties,
    reader::{FileReader, SerializedFileReader},
    serialized_reader::ReadOptionsBuilder,
};
use std::{fs::File, path::Path};

#[derive(Debug, Parser)]
#[clap(author, version, about("Binary file to read bloom filter data from a Parquet file"), long_about = None)]
struct Args {
    #[clap(help("Path to the parquet file"))]
    file_name: String,
    #[clap(help("Check the bloom filter indexes for the given column. Only string typed columns or columns with an Int32 or Int64 physical type are supported"))]
    column: String,
    #[clap(
        help(
            "Check if the given values match bloom filter, the values will be parsed to the physical type of the column"
        ),
        required = true
    )]
    values: Vec<String>,
}

fn main() {
    let args = Args::parse();
    let file_name = args.file_name;
    let path = Path::new(&file_name);
    let file = File::open(path).expect("Unable to open file");

    let file_reader = SerializedFileReader::new_with_options(
        file,
        ReadOptionsBuilder::new()
            .with_reader_properties(
                ReaderProperties::builder()
                    .set_read_bloom_filter(true)
                    .build(),
            )
            .build(),
    )
    .expect("Unable to open file as Parquet");
    let metadata = file_reader.metadata();
    for (ri, row_group) in metadata.row_groups().iter().enumerate() {
        println!("Row group #{ri}");
        println!("{}", "=".repeat(80));
        if let Some((column_index, column)) = row_group
            .columns()
            .iter()
            .enumerate()
            .find(|(_, column)| column.column_path().string() == args.column)
        {
            let row_group_reader = file_reader
                .get_row_group(ri)
                .expect("Unable to read row group");
            if let Some(sbbf) = row_group_reader.get_column_bloom_filter(column_index) {
                args.values.iter().for_each(|value| {
                    match check_filter(sbbf, value, column) {
                        Ok(present) => {
                            println!(
                                "Value {} is {} in bloom filter",
                                value,
                                if present { "present" } else { "absent" }
                            )
                        }
                        Err(err) => {
                            println!("{err}");
                        }
                    };
                });
            } else {
                println!("No bloom filter found for column {}", args.column);
            }
        } else {
            println!(
                "No column named {} found, candidate columns are: {}",
                args.column,
                row_group
                    .columns()
                    .iter()
                    .map(|c| c.column_path().string())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
    }
}

fn check_filter(sbbf: &Sbbf, value: &String, column: &ColumnChunkMetaData) -> Result<bool, String> {
    match column.column_type() {
        Type::INT32 => {
            let value: i32 = value
                .parse()
                .map_err(|e| format!("Unable to parse value '{value}' to i32: {e}"))?;
            Ok(sbbf.check(&value))
        }
        Type::INT64 => {
            let value: i64 = value
                .parse()
                .map_err(|e| format!("Unable to parse value '{value}' to i64: {e}"))?;
            Ok(sbbf.check(&value))
        }
        Type::BYTE_ARRAY => Ok(sbbf.check(&value.as_str())),
        _ => Err(format!(
            "Unsupported column type for checking bloom filter: {}",
            column.column_type()
        )),
    }
}
