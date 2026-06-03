# CHANGELOG

All notable changes to this project will be documented in this file.

## Unreleased

## [0.3.0] 2026-06-03

### Breaking changes

* Type-level `#[traverse(skip)]` is no longer supported. Use `#[traverse(skip_self)]` to skip calling the visitor for the annotated struct or enum while still traversing its children.

### New features

* Add type-level `#[traverse(skip_self)]` and `#[traverse(skip_children)]` attributes for structs and enums.
* `#[traverse(skip_children)]` calls the visitor for the annotated struct or enum without traversing its children.

### Documentation changes

* Document the split between type-level traversal controls and field or variant `#[traverse(skip)]`.
