#![cfg(test)]

use crate::engine::ast::ddl::create_table::CreateTableQuery;
use crate::engine::ast::types::{Column, DataType, TableName};
use crate::engine::parser::context::ParserContext;
use crate::engine::parser::predule::Parser;

/// 테이블 레벨 복합 PRIMARY KEY 파싱 (#220)
#[test]
pub fn create_table_with_composite_table_level_primary_key() {
    let text = r#"
        CREATE TABLE "test_db".memberships
        (
            user_id INTEGER,
            group_id INTEGER,
            PRIMARY KEY (user_id, group_id)
        );
    "#
    .to_owned();

    let mut parser = Parser::with_string(text).unwrap();

    let expected = CreateTableQuery::builder()
        .set_table(TableName::new(
            Some("test_db".to_owned()),
            "memberships".to_owned(),
        ))
        .add_column(
            Column::builder()
                .set_name("user_id".to_owned())
                .set_data_type(DataType::Int)
                .build(),
        )
        .add_column(
            Column::builder()
                .set_name("group_id".to_owned())
                .set_data_type(DataType::Int)
                .build(),
        )
        .set_primary_key(vec!["user_id".to_owned(), "group_id".to_owned()])
        .build();

    assert_eq!(
        parser.parse(ParserContext::default()).unwrap(),
        vec![expected],
    );
}

/// 단일 컬럼 테이블 레벨 PRIMARY KEY (#220)
#[test]
pub fn create_table_with_single_column_table_level_primary_key() {
    let text = r#"
        CREATE TABLE "test_db".t
        (
            a INTEGER,
            PRIMARY KEY (a)
        );
    "#
    .to_owned();

    let mut parser = Parser::with_string(text).unwrap();

    let expected = CreateTableQuery::builder()
        .set_table(TableName::new(Some("test_db".to_owned()), "t".to_owned()))
        .add_column(
            Column::builder()
                .set_name("a".to_owned())
                .set_data_type(DataType::Int)
                .build(),
        )
        .set_primary_key(vec!["a".to_owned()])
        .build();

    assert_eq!(
        parser.parse(ParserContext::default()).unwrap(),
        vec![expected],
    );
}

/// 인라인 PK와 테이블 레벨 PK를 동시에 지정하면 에러 (#220)
#[test]
pub fn create_table_rejects_mixed_inline_and_table_level_primary_keys() {
    let text = r#"
        CREATE TABLE "test_db".t
        (
            id INTEGER PRIMARY KEY,
            PRIMARY KEY (id)
        );
    "#
    .to_owned();

    let mut parser = Parser::with_string(text).unwrap();

    let error = parser.parse(ParserContext::default()).unwrap_err();
    assert!(
        error.to_string().contains("multiple primary keys"),
        "got: {}",
        error
    );
}

/// 테이블 레벨 PK에 컬럼이 없으면 에러 (#220)
#[test]
pub fn create_table_rejects_primary_key_with_empty_column_list() {
    let text = r#"
        CREATE TABLE "test_db".t
        (
            a INTEGER,
            PRIMARY KEY ( )
        );
    "#
    .to_owned();

    let mut parser = Parser::with_string(text).unwrap();

    assert!(parser.parse(ParserContext::default()).is_err());
}

/// PRIMARY KEY 뒤 괄호가 없으면 에러 (#220)
#[test]
pub fn create_table_rejects_primary_key_without_parentheses() {
    let text = r#"
        CREATE TABLE "test_db".t
        (
            a INTEGER,
            PRIMARY KEY a
        );
    "#
    .to_owned();

    let mut parser = Parser::with_string(text).unwrap();

    assert!(parser.parse(ParserContext::default()).is_err());
}

/// 테이블 레벨 PK가 정의되지 않은 컬럼을 참조하면 에러 (#220)
#[test]
pub fn create_table_rejects_primary_key_referencing_unknown_column() {
    let text = r#"
        CREATE TABLE "test_db".t
        (
            a INTEGER,
            b INTEGER,
            PRIMARY KEY (a, nope)
        );
    "#
    .to_owned();

    let mut parser = Parser::with_string(text).unwrap();

    let error = parser.parse(ParserContext::default()).unwrap_err();
    assert!(
        error.to_string().contains("nope"),
        "error should name the unknown column, got: {}",
        error
    );
}

#[test]
pub fn create_table() {
    let text = r#"
        CREATE TABLE "test_db".person
        (
            id INTEGER PRIMARY KEY,
            name varchar(100),
            age INTEGER
        );
    "#
    .to_owned();

    let mut parser = Parser::with_string(text).unwrap();

    let expected = CreateTableQuery::builder()
        .set_table(TableName::new(
            Some("test_db".to_owned()),
            "person".to_owned(),
        ))
        .add_column(
            Column::builder()
                .set_name("id".to_owned())
                .set_data_type(DataType::Int)
                .set_primary_key(true)
                .build(),
        )
        .add_column(
            Column::builder()
                .set_name("name".to_owned())
                .set_data_type(DataType::Varchar(100))
                .build(),
        )
        .add_column(
            Column::builder()
                .set_name("age".to_owned())
                .set_data_type(DataType::Int)
                .build(),
        )
        .build();

    assert_eq!(
        parser.parse(ParserContext::default()).unwrap(),
        vec![expected],
    );
}
