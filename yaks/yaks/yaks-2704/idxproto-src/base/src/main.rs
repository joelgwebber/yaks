use common::harness::{self, Backend};
use common::Doc;
use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;
struct N;
impl Backend for N {
    const NAME: &'static str = "null";
    fn create(_: &Path) -> Self { N }
    fn open(_: &Path) -> Self { N }
    fn build(&mut self, _: &[Doc], _: &HashMap<String, i64>) -> Vec<(&'static str, Duration)> { vec![] }
    fn upsert(&mut self, _: &Doc, _: i64) {}
    fn word(&self, _: &str) -> Vec<String> { vec![] }
    fn prefix(&self, _: &str) -> Vec<String> { vec![] }
    fn phrase(&self, _: &str) -> Vec<String> { vec![] }
    fn field(&self, _: &str, _: &str) -> Vec<String> { vec![] }
    fn backlinks(&self, _: &str) -> Vec<String> { vec![] }
    fn recent(&self, _: &str) -> Vec<String> { vec![] }
    fn count(&self) -> usize { 0 }
    fn all_mtimes(&self) -> Vec<(String, i64)> { vec![] }
    fn upsert_many(&mut self, _: &[(Doc, i64)]) {}
}
fn main() { harness::main::<N>(); }
