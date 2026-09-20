//! `defdeviceremap`: per-device physical key remapping applied before layers.
//!
//! ```text
//! (defdeviceremap $device-id
//!   $from-key $to-key
//!   ...
//! )
//! ```
//!
//! Every event from the given `definputdevices` ID whose key is `$from-key`
//! is processed as if `$to-key` had been pressed. `$to-key` may be `none`
//! to drop the key entirely. Useful for keyboards whose physical layout
//! differs from the others (e.g. an "angle mod" on an ISO laptop keyboard)
//! so that layers can be written once for every keyboard.

use super::*;
use crate::{anyhow_expr, bail_expr};
use std::num::NonZeroU8;

/// `from` → `Some(to)` or `None` when the key is dropped.
pub type DeviceRemap = HashMap<OsCode, Option<OsCode>>;

pub fn parse_defdeviceremap(
    expr: &[SExpr],
    vars: &HashMap<String, SExpr>,
    input_devices: Option<&[(NonZeroU8, InputDeviceMatcher)]>,
) -> Result<(NonZeroU8, DeviceRemap)> {
    let mut exprs = check_first_expr(expr.iter(), "defdeviceremap")?;
    let Some(id_expr) = exprs.next() else {
        bail_expr!(
            &expr[0],
            "defdeviceremap expects a device ID followed by key pairs"
        );
    };
    let id_str = id_expr
        .atom(Some(vars))
        .ok_or_else(|| anyhow_expr!(id_expr, "device ID must be a number (1-255)"))?;
    let id_num: u8 = id_str
        .parse()
        .map_err(|_| anyhow_expr!(id_expr, "device ID must be a number (1-255)"))?;
    let id = NonZeroU8::new(id_num)
        .ok_or_else(|| anyhow_expr!(id_expr, "device ID must be nonzero (1-255)"))?;
    let known = input_devices.is_some_and(|devs| devs.iter().any(|(d, _)| *d == id));
    if !known {
        bail_expr!(
            id_expr,
            "device ID {id_num} is not defined in definputdevices"
        );
    }

    let mut remap = DeviceRemap::default();
    while let Some(from_expr) = exprs.next() {
        let Some(to_expr) = exprs.next() else {
            bail_expr!(
                from_expr,
                "defdeviceremap expects pairs: <from-key> <to-key>. Missing to-key."
            );
        };
        let from = key_expr(from_expr, vars)?;
        let to_str = to_expr
            .atom(Some(vars))
            .ok_or_else(|| anyhow_expr!(to_expr, "to-key must be a key name or none"))?;
        let to = match to_str {
            "none" | "nop" | "XX" | "✗" | "∅" | "•" => None,
            _ => Some(key_expr(to_expr, vars)?),
        };
        if remap.insert(from, to).is_some() {
            bail_expr!(from_expr, "duplicate from-key in defdeviceremap");
        }
    }
    if remap.is_empty() {
        bail_expr!(&expr[0], "defdeviceremap has no key pairs");
    }
    Ok((id, remap))
}

fn key_expr(expr: &SExpr, vars: &HashMap<String, SExpr>) -> Result<OsCode> {
    let s = expr
        .atom(Some(vars))
        .ok_or_else(|| anyhow_expr!(expr, "expected a key name"))?;
    str_to_oscode(s).ok_or_else(|| anyhow_expr!(expr, "unknown key name: {s}"))
}
