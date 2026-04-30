use libdof::{Dof, DofIntermediate};
use wasm_bindgen::prelude::*;

fn parse_dof_inner(input: &str) -> Result<String, String> {
    let dof: Dof = serde_json::from_str(input).map_err(|e| e.to_string())?;
    serde_json::to_string(&dof).map_err(|e| e.to_string())
}

fn parse_dof_intermediate_inner(input: &str) -> Result<String, String> {
    let dof: DofIntermediate = serde_json::from_str(input).map_err(|e| e.to_string())?;
    serde_json::to_string(&dof).map_err(|e| e.to_string())
}

/// Parses and fully validates a DOF JSON string, returning normalized JSON.
///
/// All layers are expanded (shift layer generated if absent), fingering is
/// resolved to explicit finger assignments, and the board is normalized to
/// full physical key coordinates. Throws a JavaScript `Error` on failure.
#[wasm_bindgen]
pub fn parse_dof(input: &str) -> Result<String, JsError> {
    parse_dof_inner(input).map_err(|e| JsError::new(&e))
}

/// Parses a DOF JSON string without full validation.
///
/// Useful for reading partially-specified or in-progress layouts. Does not
/// expand shift layers or resolve named fingerings. Throws on JSON parse
/// errors only.
#[wasm_bindgen]
pub fn parse_dof_intermediate(input: &str) -> Result<String, JsError> {
    parse_dof_intermediate_inner(input).map_err(|e| JsError::new(&e))
}

#[cfg(test)]
mod tests {
    use super::*;

    const MINIMAL_VALID: &str = include_str!("../../../example_dofs/minimal_valid.dof");
    const MINIMAL_PARSABLE: &str =
        include_str!("../../../example_dofs/minimal_parsable.dof");
    const MAXIMAL: &str = include_str!("../../../example_dofs/maximal.dof");

    #[test]
    fn parse_minimal_valid() {
        let result = parse_dof_inner(MINIMAL_VALID).unwrap();
        let obj: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert!(obj.get("name").is_some());
        assert!(obj.get("fingering").is_some());
    }

    #[test]
    fn parse_maximal() {
        assert!(parse_dof_inner(MAXIMAL).is_ok());
    }

    #[test]
    fn round_trip() {
        let first = parse_dof_inner(MINIMAL_VALID).unwrap();
        let second = parse_dof_inner(&first).unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn parse_intermediate_accepts_minimal_parsable() {
        assert!(parse_dof_intermediate_inner(MINIMAL_PARSABLE).is_ok());
    }

    #[test]
    fn rejects_invalid_json() {
        assert!(parse_dof_inner("not json").is_err());
        assert!(parse_dof_intermediate_inner("not json").is_err());
    }

    #[test]
    fn rejects_incomplete_dof() {
        assert!(parse_dof_inner(r#"{"name":"test"}"#).is_err());
    }
}
