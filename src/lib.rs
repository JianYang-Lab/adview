use anyhow::Result;
use hdf5::types::VarLenUnicode;
use hdf5::File;
use std::io::{self, Write};

// A general struct to read data from an h5ad file
struct DataReader {
    headers: Vec<String>,
    group: hdf5::Group,
    total_rows: usize,
    encoding_types: Vec<String>,
    categorical_data: Vec<Option<(Vec<String>, Vec<u32>)>>,
}

// Implement the DataReader struct
impl DataReader {
    fn new(file: &File, group_name: &str) -> Result<Self> {
        let group = file.group(group_name)?;
        let headers = group.member_names()?;

        let mut encoding_types = Vec::new();
        let mut categorical_data = Vec::new();
        let mut total_rows = 0;

        for (i, name) in headers.iter().enumerate() {
            let encoding_type = match group.dataset(name) {
                Ok(dataset) => {
                    if i == 0 {
                        total_rows = dataset.shape()[0];
                    }
                    dataset
                        .attr("encoding-type")?
                        .read_scalar::<VarLenUnicode>()?
                }
                Err(_) => {
                    let sub_group = group.group(name)?;
                    if i == 0 {
                        total_rows = sub_group.dataset("codes")?.shape()[0];
                    }
                    sub_group
                        .attr("encoding-type")?
                        .read_scalar::<VarLenUnicode>()?
                }
            };

            encoding_types.push(encoding_type.to_string());

            if encoding_type == "categorical" {
                let sub_group = group.group(name)?;
                let categories: Vec<String> = sub_group
                    .dataset("categories")?
                    .read_1d::<VarLenUnicode>()?
                    .iter()
                    .map(|s| s.to_string())
                    .collect();
                let codes = sub_group.dataset("codes")?.read_1d::<u32>()?;
                categorical_data.push(Some((categories, codes.to_vec())));
            } else {
                categorical_data.push(None);
            }
        }

        Ok(Self {
            headers,
            group,
            total_rows,
            encoding_types,
            categorical_data,
        })
    }

    fn read_chunk(&self, start: usize, chunk_size: usize) -> Result<Vec<Vec<String>>> {
        let end = (start + chunk_size).min(self.total_rows);
        let mut chunk_data = Vec::new();

        for (i, (name, encoding_type)) in self.headers.iter().zip(&self.encoding_types).enumerate()
        {
            let data = match encoding_type.as_str() {
                "string-array" => self
                    .group
                    .dataset(name)?
                    .read_slice_1d::<VarLenUnicode, _>(start..end)?
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                "categorical" => {
                    if let Some((categories, codes)) = &self.categorical_data[i] {
                        codes[start..end]
                            .iter()
                            .map(|&code| categories[code as usize].clone())
                            .collect()
                    } else {
                        return Err(anyhow::anyhow!("Categorical data not found"));
                    }
                }
                "array" => self
                    .group
                    .dataset(name)?
                    .read_slice_1d::<i64, _>(start..end)?
                    .iter()
                    .map(|&n| n.to_string())
                    .collect(),
                _ => {
                    return Err(anyhow::anyhow!(
                        "Unsupported encoding-type: {}",
                        encoding_type
                    ))
                }
            };
            chunk_data.push(data);
        }

        Ok(chunk_data)
    }

    fn get_headers(&self) -> &[String] {
        &self.headers
    }

    fn total_rows(&self) -> usize {
        self.total_rows
    }
}

// Show the first n rows of a group
pub fn show_head(file: &File, group_name: &str, lines: usize) -> Result<()> {
    let reader = DataReader::new(file, group_name)?;
    println!("{}", reader.get_headers().join("\t"));
    let chunk_data = reader.read_chunk(0, lines)?;
    for row_idx in 0..lines.min(reader.total_rows()) {
        let row: Vec<String> = chunk_data.iter().map(|col| col[row_idx].clone()).collect();
        println!("{}", row.join("\t"));
    }
    Ok(())
}

/// Aux function to write to stdout, ignoring broken pipe errors
fn pipe_write(content: &str) -> Result<()> {
    if let Err(e) = writeln!(io::stdout(), "{}", content) {
        if e.kind() == io::ErrorKind::BrokenPipe {
            return Ok(());
        }
        return Err(e.into());
    }
    Ok(())
}

// Show all rows of a group
pub fn show_less(file: &File, group_name: &str) -> Result<()> {
    let reader = DataReader::new(file, group_name)?;
    pipe_write(&reader.get_headers().join("\t"))?;

    const CHUNK_SIZE: usize = 1000;
    let mut start = 0;
    while start < reader.total_rows() {
        let chunk_data = reader.read_chunk(start, CHUNK_SIZE)?;
        for row_idx in 0..chunk_data[0].len() {
            let row: Vec<String> = chunk_data.iter().map(|col| col[row_idx].clone()).collect();
            pipe_write(&row.join("\t"))?;
        }
        start += CHUNK_SIZE;
    }
    Ok(())
}

// Show the shapes of obs and var
pub fn show_shapes(file: &File) -> Result<()> {
    let obs_shape = file
        .dataset(&format!(
            "obs/{}",
            file.group("obs")?
                .attr("_index")?
                .read_scalar::<VarLenUnicode>()?
        ))?
        .shape()[0];

    let var_shape = file
        .dataset(&format!(
            "var/{}",
            file.group("var")?
                .attr("_index")?
                .read_scalar::<VarLenUnicode>()?
        ))?
        .shape()[0];

    println!("obs shape: {}", obs_shape);
    println!("var shape: {}", var_shape);

    Ok(())
}

// Show the fields in obs and var
pub fn show_fields(file: &File) -> Result<()> {
    // get obs fields
    let obs_group = file.group("obs")?;
    let obs_fields = obs_group.member_names()?;

    // get type of encoding for each field
    println!("obs fields:");
    for field in &obs_fields {
        let encoding_type = match obs_group.dataset(field) {
            Ok(dataset) => dataset
                .attr("encoding-type")?
                .read_scalar::<VarLenUnicode>()?,
            Err(_) => {
                let group = obs_group.group(field)?;
                group
                    .attr("encoding-type")?
                    .read_scalar::<VarLenUnicode>()?
            }
        };
        println!("\t{} ({})", field, encoding_type);
    }

    // get var fields
    let var_group = file.group("var")?;
    let var_fields = var_group.member_names()?;

    println!("\nvar fields:");
    for field in &var_fields {
        let encoding_type = match var_group.dataset(field) {
            Ok(dataset) => dataset
                .attr("encoding-type")?
                .read_scalar::<VarLenUnicode>()?,
            Err(_) => {
                let group = var_group.group(field)?;
                group
                    .attr("encoding-type")?
                    .read_scalar::<VarLenUnicode>()?
            }
        };
        println!("\t{} ({})", field, encoding_type);
    }

    Ok(())
}
