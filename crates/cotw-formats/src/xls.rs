//! XLS book: planilhas do jogo (`player_skills.bin`, `player_skill_trees.bin`) guardadas como
//! uma instância ADF `XLSBook`.
//!
//! ```text
//! XLSBook { Sheet[{Cols, Rows, CellIndex[], Name}], Cell[{Type, DataIndex, AttributeIndex}],
//!           StringData[], ValueData[], ... }
//! célula (r, c) = Cell[Sheet.CellIndex[r * Cols + c]]; Type 1 = texto, 2 = número
//! ```

use crate::adf::{Adf, AdfError, Value};

const CELL_TEXT: u32 = 1;
const CELL_NUMBER: u32 = 2;

/// Erros ao montar um XLS book.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum XlsError {
    /// O ADF não tem uma instância `XLSBook`.
    #[error("o arquivo não contém um XLSBook")]
    NotFound,
    /// Falha ao decodificar o ADF.
    #[error(transparent)]
    Adf(#[from] AdfError),
    /// A estrutura não tem o formato esperado.
    #[error("XLSBook com formato inesperado: {0}")]
    Shape(String),
}

/// Valor de uma célula.
#[derive(Debug, Clone, PartialEq)]
pub enum CellValue {
    /// Texto.
    Text(String),
    /// Número.
    Number(f64),
    /// Tipo de célula sem leitura própria (data, booleano...).
    Unsupported(u32),
}

/// Uma planilha.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sheet {
    /// Nome da planilha.
    pub name: String,
    /// Número de colunas.
    pub cols: u32,
    /// Número de linhas.
    pub rows: u32,
    cell_index: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RawCell {
    kind: u32,
    data_index: u32,
}

/// Livro com todas as planilhas e as tabelas de textos e números.
#[derive(Debug, Clone, PartialEq)]
pub struct XlsBook {
    sheets: Vec<Sheet>,
    cells: Vec<RawCell>,
    strings: Vec<String>,
    values: Vec<f64>,
}

impl XlsBook {
    /// Lê o `XLSBook` de um ADF.
    ///
    /// # Errors
    ///
    /// Retorna [`XlsError`] se não houver `XLSBook` ou se a estrutura for diferente da esperada.
    pub fn from_adf(adf: &Adf) -> Result<Self, XlsError> {
        let instance = adf
            .instances()
            .iter()
            .find(|i| i.name == "XLSBook")
            .ok_or(XlsError::NotFound)?;
        Self::from_value(&adf.read_instance(instance)?)
    }

    /// Monta o livro a partir do valor já decodificado da instância.
    ///
    /// # Errors
    ///
    /// Retorna [`XlsError::Shape`] se faltar algum campo ou ele tiver o tipo errado.
    pub fn from_value(root: &Value) -> Result<Self, XlsError> {
        let fields = as_struct(root, "XLSBook")?;
        let sheets = array(fields, "Sheet")?
            .iter()
            .map(read_sheet)
            .collect::<Result<_, _>>()?;
        let cells = array(fields, "Cell")?
            .iter()
            .map(|c| {
                let f = as_struct(c, "Cell")?;
                Ok(RawCell {
                    kind: unsigned(f, "Type")?,
                    data_index: unsigned(f, "DataIndex")?,
                })
            })
            .collect::<Result<_, XlsError>>()?;
        let strings = array(fields, "StringData")?
            .iter()
            .map(|v| match v {
                Value::String(s) => Ok(s.clone()),
                _ => Err(shape("StringData deve conter textos")),
            })
            .collect::<Result<_, _>>()?;
        let values = array(fields, "ValueData")?
            .iter()
            .map(|v| match v {
                Value::Float(x) => Ok(*x),
                _ => Err(shape("ValueData deve conter números")),
            })
            .collect::<Result<_, _>>()?;
        Ok(Self {
            sheets,
            cells,
            strings,
            values,
        })
    }

    /// Planilhas na ordem do arquivo.
    #[must_use]
    pub fn sheets(&self) -> &[Sheet] {
        &self.sheets
    }

    /// Busca uma planilha pelo nome.
    #[must_use]
    pub fn sheet(&self, name: &str) -> Option<&Sheet> {
        self.sheets.iter().find(|s| s.name == name)
    }

    /// Valor da célula (`row`, `col`), ou `None` se estiver fora da planilha ou sem dado.
    #[must_use]
    pub fn cell(&self, sheet: &Sheet, row: u32, col: u32) -> Option<CellValue> {
        if row >= sheet.rows || col >= sheet.cols {
            return None;
        }
        let position = row as usize * sheet.cols as usize + col as usize;
        let cell = self.cells.get(*sheet.cell_index.get(position)? as usize)?;
        let index = cell.data_index as usize;
        match cell.kind {
            CELL_TEXT => self.strings.get(index).cloned().map(CellValue::Text),
            CELL_NUMBER => self.values.get(index).copied().map(CellValue::Number),
            other => Some(CellValue::Unsupported(other)),
        }
    }
}

fn shape(message: &str) -> XlsError {
    XlsError::Shape(message.to_owned())
}

fn as_struct<'a>(value: &'a Value, what: &str) -> Result<&'a [(String, Value)], XlsError> {
    match value {
        Value::Struct(fields) => Ok(fields),
        _ => Err(XlsError::Shape(format!("{what} deve ser um struct"))),
    }
}

fn field<'a>(fields: &'a [(String, Value)], name: &str) -> Result<&'a Value, XlsError> {
    fields
        .iter()
        .find(|(n, _)| n == name)
        .map(|(_, v)| v)
        .ok_or_else(|| XlsError::Shape(format!("campo {name} ausente")))
}

fn array<'a>(fields: &'a [(String, Value)], name: &str) -> Result<&'a [Value], XlsError> {
    match field(fields, name)? {
        Value::Array(items) => Ok(items),
        // Arrays vazios de bytes chegam como `Bytes`; qualquer outro tipo é erro.
        Value::Bytes(b) if b.is_empty() => Ok(&[]),
        _ => Err(XlsError::Shape(format!("{name} deve ser um array"))),
    }
}

fn unsigned(fields: &[(String, Value)], name: &str) -> Result<u32, XlsError> {
    match field(fields, name)? {
        Value::Unsigned(n) => {
            u32::try_from(*n).map_err(|_| XlsError::Shape(format!("{name} fora do intervalo")))
        }
        _ => Err(XlsError::Shape(format!("{name} deve ser um inteiro"))),
    }
}

fn read_sheet(value: &Value) -> Result<Sheet, XlsError> {
    let f = as_struct(value, "Sheet")?;
    let name = match field(f, "Name")? {
        Value::String(s) => s.clone(),
        _ => return Err(shape("Sheet.Name deve ser texto")),
    };
    let cell_index = array(f, "CellIndex")?
        .iter()
        .map(|v| match v {
            Value::Unsigned(n) => {
                u32::try_from(*n).map_err(|_| shape("CellIndex fora do intervalo"))
            }
            _ => Err(shape("CellIndex deve conter inteiros")),
        })
        .collect::<Result<_, _>>()?;
    Ok(Sheet {
        name,
        cols: unsigned(f, "Cols")?,
        rows: unsigned(f, "Rows")?,
        cell_index,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn st(fields: Vec<(&str, Value)>) -> Value {
        Value::Struct(fields.into_iter().map(|(n, v)| (n.to_owned(), v)).collect())
    }

    fn uns(n: u64) -> Value {
        Value::Unsigned(n)
    }

    fn cell(kind: u64, data: u64) -> Value {
        st(vec![
            ("Type", uns(kind)),
            ("DataIndex", uns(data)),
            ("AttributeIndex", uns(0)),
        ])
    }

    /// Planilha 2×2: [Nome, 3.5] / [Nome, data]; as duas primeiras apontam para a mesma célula.
    fn sample() -> Value {
        st(vec![
            (
                "Sheet",
                Value::Array(vec![st(vec![
                    ("Cols", uns(2)),
                    ("Rows", uns(2)),
                    (
                        "CellIndex",
                        Value::Array(vec![uns(0), uns(1), uns(0), uns(2)]),
                    ),
                    ("Name", Value::String("Skills".into())),
                ])]),
            ),
            (
                "Cell",
                Value::Array(vec![cell(1, 0), cell(2, 0), cell(3, 0)]),
            ),
            (
                "StringData",
                Value::Array(vec![Value::String("Nome".into())]),
            ),
            ("ValueData", Value::Array(vec![Value::Float(3.5)])),
        ])
    }

    #[test]
    fn reads_cells_through_the_index_indirection() {
        let book = XlsBook::from_value(&sample()).unwrap();
        let sheet = book.sheet("Skills").unwrap();
        assert_eq!((sheet.rows, sheet.cols), (2, 2));
        assert_eq!(book.cell(sheet, 0, 0), Some(CellValue::Text("Nome".into())));
        assert_eq!(book.cell(sheet, 0, 1), Some(CellValue::Number(3.5)));
        assert_eq!(book.cell(sheet, 1, 0), Some(CellValue::Text("Nome".into())));
        assert_eq!(book.cell(sheet, 1, 1), Some(CellValue::Unsupported(3)));
        assert_eq!(book.sheets().len(), 1);
        assert!(book.sheet("Outra").is_none());
    }

    #[test]
    fn out_of_range_cells_are_none() {
        let book = XlsBook::from_value(&sample()).unwrap();
        let sheet = book.sheet("Skills").unwrap();
        assert_eq!(book.cell(sheet, 2, 0), None);
        assert_eq!(book.cell(sheet, 0, 2), None);
    }

    #[test]
    fn dangling_indexes_are_none_not_panics() {
        let Value::Struct(mut fields) = sample() else {
            unreachable!()
        };
        fields[3].1 = Value::Array(vec![]); // ValueData vazio: a célula numérica fica sem dado
        fields[2].1 = Value::Array(vec![]);
        let book = XlsBook::from_value(&Value::Struct(fields)).unwrap();
        let sheet = book.sheet("Skills").unwrap();
        assert_eq!(book.cell(sheet, 0, 0), None);
        assert_eq!(book.cell(sheet, 0, 1), None);
    }

    #[test]
    fn rejects_unexpected_shapes() {
        assert!(matches!(
            XlsBook::from_value(&uns(1)),
            Err(XlsError::Shape(_))
        ));
        let Value::Struct(mut fields) = sample() else {
            unreachable!()
        };
        fields.retain(|(n, _)| n != "Cell");
        assert!(matches!(
            XlsBook::from_value(&Value::Struct(fields)),
            Err(XlsError::Shape(m)) if m.contains("Cell")
        ));
    }

    #[test]
    fn rejects_wrong_element_types() {
        for (field_name, bad) in [
            ("StringData", Value::Array(vec![uns(1)])),
            ("ValueData", Value::Array(vec![uns(1)])),
            ("Cell", Value::Array(vec![uns(1)])),
            ("Sheet", Value::Array(vec![uns(1)])),
            ("Sheet", uns(1)),
        ] {
            let Value::Struct(mut fields) = sample() else {
                unreachable!()
            };
            fields.iter_mut().find(|(n, _)| n == field_name).unwrap().1 = bad;
            assert!(
                matches!(
                    XlsBook::from_value(&Value::Struct(fields)),
                    Err(XlsError::Shape(_))
                ),
                "{field_name}"
            );
        }
    }

    #[test]
    fn rejects_malformed_sheet_fields() {
        for (inner, bad) in [
            ("Name", uns(1)),
            ("CellIndex", Value::Array(vec![Value::Float(1.0)])),
            ("Cols", Value::Float(1.0)),
            ("CellIndex", Value::Array(vec![uns(u64::MAX)])),
        ] {
            let Value::Struct(mut fields) = sample() else {
                unreachable!()
            };
            let Value::Array(sheets) = &mut fields[0].1 else {
                unreachable!()
            };
            let Value::Struct(sheet) = &mut sheets[0] else {
                unreachable!()
            };
            sheet.iter_mut().find(|(n, _)| n == inner).unwrap().1 = bad;
            assert!(
                XlsBook::from_value(&Value::Struct(fields)).is_err(),
                "{inner}"
            );
        }
    }

    #[test]
    fn empty_byte_array_counts_as_empty_array() {
        let Value::Struct(mut fields) = sample() else {
            unreachable!()
        };
        fields[3].1 = Value::Bytes(vec![]);
        assert!(XlsBook::from_value(&Value::Struct(fields)).is_ok());
    }
}
