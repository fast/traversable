# CHANGELOG

All notable changes to this project will be documented in this file.

## Unreleased

### Breaking changes

* Type-level `#[traverse(skip)]` is no longer supported. Use `#[traverse(skip_self)]` to skip calling the visitor for the annotated struct or enum while still traversing its children.

### New features

* Add type-level `#[traverse(skip_self)]` and `#[traverse(skip_children)]` attributes for structs and enums.
* `#[traverse(skip_children)]` skips traversing the children of the annotated struct or enum, but still calls the visitor for it, unless `#[traverse(skip_self)]` is also present.
