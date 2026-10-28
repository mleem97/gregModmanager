# Historical manifest samples

The JSON files in this directory predate the current desktop project's
`WorkshopMetadata` serialization. They are not validated by gregModmanager and
must not be treated as a public package or upload schema.

Keep them only as historical research material. Before publishing automation,
inspect `crates/greg-core/src/models/meta.rs`, create a test
project with the target app version, and validate the generated metadata in a
disposable Steam Workshop workflow.
