//! SQLite/DuckDB view DDL over the projected tables so pm4py can
//! consume the OCEL 2.0 projection.
//!
//! NOTE on `event_id`: it is the receipt-chain ordinal (logical
//! sequence), NOT a wall clock. No `ocel:timestamp` column exists by
//! design; the DDL exposes the ordinal under `ocel:sequence`-style
//! naming and NEVER substitutes it for time.

/// The Events table DDL.
pub const EVENTS_DDL: &str = "CREATE TABLE event_table (\n  event_id BIGINT NOT NULL, -- receipt-chain ordinal; NOT a wall clock\n  activity VARCHAR NOT NULL, -- closed value list: 'receipt.append' |
                             --   'session.step' | 'session.refuse' |
                             --   'boundary.poison' (tap::ACTIVITY_*)\n  receipt_digest VARCHAR NOT NULL, -- hex BLAKE3 link hash\n  block_digest VARCHAR NOT NULL,\n  prev_hash VARCHAR NOT NULL\n);";

/// The Objects table DDL.
pub const OBJECTS_DDL: &str = "CREATE TABLE object_table (\n  object_id VARCHAR NOT NULL, -- 'file:<dev>:<ino>' or 'block:<hex8>'\n  object_type VARCHAR NOT NULL,\n  dev BIGINT, -- NULL for block-derived objects\n  ino BIGINT, -- NULL for block-derived objects\n  lease_id BIGINT NOT NULL\n);";

/// The E2O edge table DDL.
pub const E2O_DDL: &str = "CREATE TABLE event_object_table (\n  event_id BIGINT NOT NULL,\n  object_id VARCHAR NOT NULL,\n  qualifier VARCHAR NOT NULL\n);";

/// The pm4py-facing flat view (OCEL 2.0 XML/SQLite shape: events
/// joined to objects through E2O).
pub const OCEL_FLAT_VIEW_DDL: &str = "CREATE VIEW ocel_flat AS\nSELECT e.event_id, e.activity, e.receipt_digest,\n       o.object_id, o.object_type, o.lease_id, x.qualifier\nFROM event_table e\nJOIN event_object_table x ON x.event_id = e.event_id\nJOIN object_table o ON o.object_id = x.object_id;";

/// All four statements, in dependency order, newline-joined.
pub fn full_ddl() -> String {
    [
        EVENTS_DDL,
        OBJECTS_DDL,
        E2O_DDL,
        OCEL_FLAT_VIEW_DDL,
    ]
    .join("\n")
}

#[cfg(test)]
mod courts {
    use super::*;

    /// Every DDL constant is non-empty, mentions its table, and the
    /// flat view joins all three. Anti-vacuity: deleting any one
    /// statement from `full_ddl` output drops the joint count below
    /// three and the court fails.
    #[test]
    fn ddl_names_all_three_tables_and_flat_view() {
        let all = full_ddl();
        for needle in [
            "CREATE TABLE event_table",
            "CREATE TABLE object_table",
            "CREATE TABLE event_object_table",
            "CREATE VIEW ocel_flat",
            "JOIN event_object_table",
            "JOIN object_table",
        ] {
            assert!(
                all.contains(needle),
                "DDL missing required fragment: {needle}"
            );
        }
        // The ordinal-is-not-time note is load-bearing in the DDL.
        assert!(all.contains("NOT a wall clock"));
    }
}
