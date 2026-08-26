use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::engine::ast::types::TableName;
use crate::utils::float::Float64;

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, PartialOrd, Eq, Hash)]
pub enum TableDataFieldType {
    // 끝단 Primitive 값
    Integer(i64),
    Float(Float64),
    Boolean(bool),
    String(String),
    Array(Vec<TableDataFieldType>),
    Null,
}

impl TableDataFieldType {
    pub fn type_code(&self) -> isize {
        match self {
            TableDataFieldType::Integer(_) => 1,
            TableDataFieldType::Float(_) => 2,
            TableDataFieldType::Boolean(_) => 3,
            TableDataFieldType::String(_) => 4,
            TableDataFieldType::Array(_) => 5,
            TableDataFieldType::Null => 0,
        }
    }

    /// 값이 힙에 점유하는 대략적인 바이트 수 (#265).
    ///
    /// enum tag + 인라인 값만 세는 것이 아니라, `String`/`Vec`의 힙 버퍼까지
    /// 포함합니다. `TableDataField`/`TableDataRow`의 스택 구조체 오버헤드는
    /// 상수(`ESTIMATED_FIELD_OVERHEAD`)로 합산합니다.
    pub fn estimated_bytes(&self) -> u64 {
        match self {
            TableDataFieldType::Integer(_) => 8,
            TableDataFieldType::Float(_) => 8,
            TableDataFieldType::Boolean(_) => 1,
            TableDataFieldType::Null => 0,
            TableDataFieldType::String(value) => value.len() as u64,
            TableDataFieldType::Array(values) => values.iter().map(|e| e.estimated_bytes()).sum(),
        }
    }

    pub fn to_array(self) -> Self {
        Self::Array(vec![self])
    }

    pub fn push(&mut self, value: Self) {
        #[allow(clippy::single_match)]
        match self {
            TableDataFieldType::Array(array) => array.push(value),
            _ => {}
        }
    }

    pub fn is_null(&self) -> bool {
        self.type_code() == 0
    }

    pub fn is_array(&self) -> bool {
        self.type_code() == 5
    }
}

impl ToString for TableDataFieldType {
    fn to_string(&self) -> String {
        #[allow(unstable_name_collisions)]
        match self {
            TableDataFieldType::Integer(value) => value.to_string(),
            TableDataFieldType::Float(value) => value.to_string(),
            TableDataFieldType::Boolean(value) => value.to_string(),
            TableDataFieldType::String(value) => value.to_owned(),
            TableDataFieldType::Array(value) => value
                .iter()
                .map(|e| e.to_string())
                .intersperse(", ".to_owned())
                .collect(),
            TableDataFieldType::Null => "NULL".into(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TableDataField {
    pub table_name: TableName,
    pub column_name: String,
    pub data: TableDataFieldType,
}

impl TableDataField {
    pub fn to_array(self) -> Self {
        Self {
            table_name: self.table_name,
            column_name: self.column_name,
            data: self.data.to_array(),
        }
    }

    pub fn push(&mut self, value: TableDataFieldType) {
        #[allow(clippy::single_match)]
        match &mut self.data {
            TableDataFieldType::Array(array) => array.push(value),
            _ => {}
        }
    }

    /// 필드 하나가 힙에 점유하는 대략적인 바이트 수 (#265).
    /// `column_name` 문자열과 값의 estimated_bytes를 합산합니다.
    pub fn estimated_bytes(&self) -> u64 {
        self.column_name.len() as u64 + self.data.estimated_bytes()
    }
}

/// 행 하나의 힙 메모리를 추정할 때 사용하는 상수 오버헤드 (#265).
/// `TableDataRow { fields: Vec<TableDataField> }`의 Vec 버퍼 및 field 구조체들의
/// 스택 공간을 대략으로 반영합니다.
pub const ESTIMATED_FIELD_OVERHEAD: u64 = 32;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TableDataRow {
    pub fields: Vec<TableDataField>,
}

impl TableDataRow {
    /// 행 하나가 힙에 점유하는 대략적인 바이트 수 (#265).
    /// 각 필드의 estimated_bytes + 필드 개수 만큼 오버헤드를 합산합니다.
    pub fn estimated_bytes(&self) -> u64 {
        self.fields.iter().map(|f| f.estimated_bytes()).sum::<u64>()
            + self.fields.len() as u64 * ESTIMATED_FIELD_OVERHEAD
    }
}
