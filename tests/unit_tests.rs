//! 单元测试入口 —— 目录结构对照 src/
//! cargo test --test unit_tests 运行所有单元测试

#[path = "unit_tests/error/mod.rs"]
mod test_error;

#[path = "unit_tests/parsers/mod.rs"]
mod test_parsers;

#[path = "unit_tests/searchers/mod.rs"]
mod test_searchers;

#[path = "unit_tests/readers/mod.rs"]
mod readers;
