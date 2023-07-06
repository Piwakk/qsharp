// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc::hir::visit::Visitor;
use qsc::hir::{Item, ItemId, LocalItemId, PackageId};
use qsc::{
    compile::{self, Error},
    PackageStore, SourceMap,
};
use qsc::{CompileUnit, Span};

/// Represents an immutable compilation state that can be used
/// to implement language service features.
pub(crate) struct Compilation {
    pub package_store: PackageStore,
    pub std_package_id: PackageId,
    pub unit: CompileUnit,
    pub errors: Vec<Error>,
}

pub(crate) fn compile_document(source_name: &str, source_contents: &str) -> Compilation {
    let mut package_store = PackageStore::new(compile::core());
    let std_package_id = package_store.insert(compile::std(&package_store));

    // Source map only contains the current document.
    let source_map = SourceMap::new([(source_name.into(), source_contents.into())], None);
    let (unit, errors) = compile::compile(&package_store, &[std_package_id], source_map);
    Compilation {
        package_store,
        std_package_id,
        unit,
        errors,
    }
}

pub(crate) fn span_contains(span: Span, offset: u32) -> bool {
    offset >= span.lo && offset < span.hi
}

pub(crate) fn map_offset(source_map: &SourceMap, source_name: &str, source_offset: u32) -> u32 {
    source_map
        .find_by_name(source_name)
        .expect("source should exist in the source map")
        .offset
        + source_offset
}

pub(crate) fn find_item<'a>(compilation: &'a Compilation, id: &ItemId) -> Option<&'a Item> {
    let mut finder_pass = FindItem {
        id: &id.item,
        item: None,
    };
    let package = if let Some(package_id) = id.package {
        &compilation
            .package_store
            .get(package_id)
            .unwrap_or_else(|| panic!("bad package id: {package_id}"))
            .package
    } else {
        &compilation.unit.package
    };
    finder_pass.visit_package(package);
    finder_pass.item
}

struct FindItem<'a, 'b> {
    pub id: &'a LocalItemId,
    pub item: Option<&'b Item>,
}

impl<'a, 'b> Visitor<'b> for FindItem<'a, 'b> {
    fn visit_item(&mut self, item: &'b Item) {
        if item.id == *self.id {
            self.item = Some(item);
        }
    }
}