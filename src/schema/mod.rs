/// Schema system — class and field definitions extracted from
/// `libschemasystem.so` at runtime.
///
/// CS2 embeds its type information in a shared library.  This module provides
/// structures and a parser stub that can be extended to read the actual binary
/// once the schema format is fully reverse-engineered.

use std::collections::HashMap;

/// Metadata for a single field within a schema class.
#[derive(Debug, Clone)]
pub struct FieldDefinition {
    /// Field name as declared in the schema (e.g. `"m_iHealth"`).
    pub name: String,
    /// Byte offset of this field from the start of the owning object.
    pub offset: u64,
    /// Type name as recorded in the schema (e.g. `"int32"`, `"Vector"`).
    pub type_name: String,
}

/// Metadata for a single class registered in the CS2 schema system.
#[derive(Debug, Clone)]
pub struct ClassDefinition {
    /// Class name (e.g. `"C_CSPlayerPawn"`).
    pub name: String,
    /// Fields keyed by field name.
    pub fields: HashMap<String, FieldDefinition>,
}

impl ClassDefinition {
    /// Returns the byte offset of `field`, or `None` if not present.
    pub fn field_offset(&self, field: &str) -> Option<u64> {
        self.fields.get(field).map(|f| f.offset)
    }
}

/// Loaded schema: a collection of [`ClassDefinition`] objects.
#[derive(Debug, Default)]
pub struct Schema {
    classes: HashMap<String, ClassDefinition>,
}

impl Schema {
    /// Creates an empty schema.
    pub fn new() -> Self {
        Self { classes: HashMap::new() }
    }

    /// Inserts or replaces a class definition.
    pub fn insert_class(&mut self, class: ClassDefinition) {
        self.classes.insert(class.name.clone(), class);
    }

    /// Looks up a class by name.
    pub fn get_class(&self, name: &str) -> Option<&ClassDefinition> {
        self.classes.get(name)
    }

    /// Returns the byte offset of `field` inside `class`, or `None`.
    pub fn get_field_offset(&self, class: &str, field: &str) -> Option<u64> {
        self.classes.get(class)?.field_offset(field)
    }

    /// Attempts to populate the schema by parsing the `libschemasystem.so`
    /// mapped at `base_address` with the raw bytes supplied in `data`.
    ///
    /// **Current implementation**: This is a stub.  Parsing the binary schema
    /// format requires additional reverse-engineering work.  The method is
    /// provided so callers can integrate it without changing their code once a
    /// real parser is available.
    ///
    /// Returns the number of classes loaded (always 0 for the stub).
    pub fn load_from_game(&mut self, _base_address: u64, _data: &[u8]) -> usize {
        // TODO: implement ELF + CS2 schema binary format parsing.
        0
    }

    /// Loads a minimal hard-coded schema sufficient for the game reader.
    ///
    /// This is the fallback when the binary parser is unavailable.  Offsets
    /// here mirror those in [`crate::process::offsets::Interface`].
    pub fn load_defaults(&mut self) {
        macro_rules! class {
            ($name:expr, [ $( ($field:expr, $offset:expr, $ty:expr) ),* $(,)? ]) => {{
                let mut fields = HashMap::new();
                $(
                    fields.insert($field.to_owned(), FieldDefinition {
                        name: $field.to_owned(),
                        offset: $offset,
                        type_name: $ty.to_owned(),
                    });
                )*
                ClassDefinition { name: $name.to_owned(), fields }
            }};
        }

        let player_controller = class!("CCSPlayerController", [
            ("m_steamID",       0x7E0, "uint64"),
            ("m_iszPlayerName", 0x640, "char[128]"),
            ("m_hPlayerPawn",   0x7E4, "CHandle<C_CSPlayerPawn>"),
            ("m_iTeamNum",      0x3BF, "uint8"),
        ]);

        let player_pawn = class!("C_CSPlayerPawn", [
            ("m_iHealth",           0x344,  "int32"),
            ("m_ArmorValue",        0xDE4,  "int32"),
            ("m_vecAbsOrigin",      0xC8,   "Vector"),
            ("m_vecViewOffset",     0xC84,  "Vector"),
            ("m_angEyeAngles",      0x1510, "QAngle"),
            ("m_pGameSceneNode",    0x328,  "CGameSceneNode*"),
            ("m_vecVelocity",       0x3F0,  "Vector"),
            ("m_bHasDefuser",       0xDF0,  "bool"),
            ("m_bHasHelmet",        0xDF1,  "bool"),
        ]);

        let planted_c4 = class!("C_PlantedC4", [
            ("m_flC4Blow",          0xB10, "float32"),
            ("m_bBombDefused",      0xB6C, "bool"),
            ("m_flDefuseCountDown", 0xB74, "float32"),
            ("m_vecAbsOrigin",      0xC8,  "Vector"),
            ("m_hBombDefuser",      0xB64, "CHandle<CBaseEntity>"),
        ]);

        let grenade = class!("C_BaseCSGrenadeProjectile", [
            ("m_vecAbsOrigin", 0xC8, "Vector"),
        ]);

        let game_rules = class!("CGameRules", [
            ("m_bFreezePeriod", 0xA0, "bool"),
        ]);

        for def in [player_controller, player_pawn, planted_c4, grenade, game_rules] {
            self.insert_class(def);
        }
    }

    /// Returns the number of classes currently loaded.
    pub fn class_count(&self) -> usize {
        self.classes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_schema_loads() {
        let mut schema = Schema::new();
        schema.load_defaults();
        assert!(schema.class_count() > 0);
    }

    #[test]
    fn test_get_existing_class() {
        let mut schema = Schema::new();
        schema.load_defaults();
        let cls = schema.get_class("C_CSPlayerPawn");
        assert!(cls.is_some());
    }

    #[test]
    fn test_get_nonexistent_class() {
        let schema = Schema::new();
        assert!(schema.get_class("DoesNotExist").is_none());
    }

    #[test]
    fn test_field_offset_lookup() {
        let mut schema = Schema::new();
        schema.load_defaults();
        let offset = schema.get_field_offset("C_CSPlayerPawn", "m_iHealth");
        assert_eq!(offset, Some(0x344));
    }

    #[test]
    fn test_field_offset_missing_field() {
        let mut schema = Schema::new();
        schema.load_defaults();
        let offset = schema.get_field_offset("C_CSPlayerPawn", "m_nonexistent");
        assert!(offset.is_none());
    }

    #[test]
    fn test_insert_and_retrieve_class() {
        let mut schema = Schema::new();
        let mut fields = HashMap::new();
        fields.insert(
            "m_foo".to_owned(),
            FieldDefinition { name: "m_foo".to_owned(), offset: 0x10, type_name: "int32".to_owned() },
        );
        schema.insert_class(ClassDefinition { name: "CMyClass".to_owned(), fields });
        assert_eq!(schema.get_field_offset("CMyClass", "m_foo"), Some(0x10));
    }
}
