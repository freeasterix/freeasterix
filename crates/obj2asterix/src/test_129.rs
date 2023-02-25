fn make_point() -> serde_json::Value {
    serde_json::json!({
        "CAT": 129,
        "010": {
            "TYP": 0,
            "TD_Mil": 1,
            "TD_Moving": 1,
            "TD_KIND": 3,
            "RLS_Type": 7,
            "SAC": 0x69,
            "SIC": 0x69,
        },
        "040": {
            "Distance": 420.69,
            "Azimuth": 42.069,
        },
        "042": {
            "X": 420.69,
            "Y": 69.420,
        },
        "070": {
            "ToD": 42_069.33333,
        },
        "050": {
            "HeightValid": 0,
            "HeightSource": 3,
            "Height": 32.3333,
        },
        "061": {
            "RadialSpeed": 420.69,
        },
        "080": {
            "VDN": 13,
            "ODN": 1,
        },
        "140": {
            "V": 0,
            "G": 0,
            "L": 0,
            "Mode3A": 4206,
        },
        "150": {
            "OPR_ID": 42069,
        },
        "160": {
            "Callsign": "42069!!",
        },
        "100": {
            "VZ": 3,
            "DZ": 3,
            "IZ": 3,
            "PAZ_Azimuth": 42.069,
            "PAZ_Heading": 69.420,
            "KM": 1,
            "AngularSpeed": 0.423333,
        },
    })
}

fn make_trajectory() -> serde_json::Value {
    serde_json::json!({
        "CAT": 129,
        "010": {
            "TYP": 1,
            "TD_Mil": 1,
            "TD_Moving": 1,
            "TD_KIND": 3,
            "RLS_Type": 7,
            "SAC": 0x69,
            "SIC": 0x69,
        },
        "020": {
            "SK": 1,
            "RC_ID": 9999,
        },
        "030": {
            "SK": 1,
            "RC_ID": 4095,
        },
        "042": {
            "X": 420.69,
            "Y": 69.420,
        },
        /*
        "043": {
            "X": 420.69,
            "Y": 69.420,
        },
        */
        "070": {
            "ToD": 42_069.33333,
        },
        "050": {
            "HeightValid": 0,
            "HeightSource": 3,
            "Height": 32.3333,
        },
        "060": {
            "Speed": 420.69,
            "Heading": 42.069,
        },
        "061": {
            "RadialSpeed": 420.69,
        },
        "080": {
            "VDN": 13,
            "ODN": 1,
        },
        "140": {
            "V": 0,
            "G": 0,
            "L": 0,
            "Mode3A": 4206,
        },
        "150": {
            "OPR_ID": 42069,
        },
        "160": {
            "Callsign": "42069!!",
        },
        "090": {
            "MK": 2,
            "MS": 2,
            "MH": 2,
        },
        "100": {
            "VZ": 3,
            "DZ": 3,
            "IZ": 3,
            "PAZ_Azimuth": 42.069,
            "PAZ_Heading": 69.420,
            "KM": 1,
            "AngularSpeed": 0.423333,
        },
        "110": {
            "VP_RP": 7,
            "ID_RC": 7,
        },
        "120": {
            "AOKind": 27,
        },
        "130": {
            "TS": 3,
            "TT": 3,
            "OS": 15,
        },
        "132": {
            "OG": 1,
            "RC_Num": 69,
        },
        "170": {
            "AppId": "42069",
        },
        "SDI": {
            "051": {
                "FuelPercent": 99,
            },
            "052": {
                "HeightValid": 0,
                "HeightSource": 3,
                "Height": 42.069,
            },
            "053": {
                "ZoneCode": 3,
            },
            "180": {
                "V": 0,
                "G": 0,
                "L": 0,
                "Mode2MkXA": 7777,
            },
            "190": {
                "V": 0,
                "G": 0,
                "L": 0,
                "Mode1MkXA": 73,
            },
        },
    })
}

#[test]
fn test_129() -> Result<(), Box<dyn std::error::Error>> {
    let crate_dir = std::env::var("CARGO_MANIFEST_DIR")?;
    let xml_root = std::path::Path::new(&crate_dir).join("../../specs-xml");
    let spec_src = std::fs::read_to_string(xml_root.join("asterix_cat129_1_1.xml"))?;
    let spec = spec_parser::spec_xml::Category::parse(&spec_src)?;

    for data in [make_point(), make_trajectory()] {
        let data = data.as_object().ok_or("must be an object")?;
        let mut buf = Vec::new();
        crate::write_asterix(&mut buf, &spec, &data)?;
        println!("{:x?}", buf);
    }
    Ok(())
}
