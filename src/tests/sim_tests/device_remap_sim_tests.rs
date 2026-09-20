use super::*;

const CFG: &str = "
(defcfg)
(defsrc grv z x b)
(deflayer base grv z x b)
(definputdevices 1 ((name \"laptop\")))
(defdeviceremap 1 grv z  z x  b none)
";

#[test]
fn device_remap_rewrites_keys_for_that_device_only() {
    let result = simulate(
        CFG,
        "dev:1 d:grv t:10 u:grv t:10 d:z t:10 u:z t:10 dev:0 d:grv t:10 u:grv t:10",
    )
    .to_ascii();
    assert_eq!(
        "dn:Z t:10ms up:Z t:10ms dn:X t:10ms up:X t:10ms dn:Grave t:10ms up:Grave",
        result
    );
}

#[test]
fn device_remap_none_drops_key() {
    let result = simulate(CFG, "dev:1 d:b t:10 u:b t:10 d:x t:10 u:x t:10").to_ascii();
    assert_eq!("t:20ms dn:X t:10ms up:X", result);
}
