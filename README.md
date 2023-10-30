# freeasterix

This is a Rust implementation of the [ASTERIX][eurocontrol-asterix] protocol for serialization & deserialization.

ASTERIX is a binary data format developed and maintained by EUROCONTROL. Decoding the file format requires additional information, known as the SPEC. The spec format utilized by this library is derived from the XML provided by [Zoran Bošnjak][zoranbosnjak-specs].

## API

The library offers three main interfaces:

- `spec_parser::spec_xml::Category`: This is a struct representing a parsed XML spec. It can be instantiated using `Category::parse`.
- `obj2asterix::read_asterix`: Takes an input slice and a spec, then returns a JSON representation of the ASTERIX data.
- `obj2asterix::write_asterix`: Accepts a destination buffer, a spec, and a JSON representation. It writes the binary representation of ASTERIX data into the buffer.

## Example usage

```rust
use spec_parser::spec_xml::Category;
use obj2asterix::{read_asterix, write_asterix};
use serde_json::{Map, Value};

// 1) Read and parse the spec for a given category
let xml = std::fs::read_to_string("../specs-xml/asterix_cat062_1_18.xml")?;
let category = Category::parse(&xml)?;

// 2) Demonstration of parsing/deserialization
let input = &[0x3E, 0x00, 0x06, 0x80, 0xB0, 0xB1][..];
let mut reader = input;
let value: Map<String, Value> = read_asterix(&mut reader, &category)?;
println!("ASTERIX decoded: {:?}", value);

// 3) Demonstration of serialization
let mut writer = Vec::new();
write_asterix(&mut writer, &category, &value)?;
println!("ASTERIX encoded: {:x?}", writer);
assert_eq!(&writer, input);
```

## JSON representation

The JSON representation of ASTERIX data is structured as follows:

```json
{
    "CAT": 1, // Number of the ASTERIX category
    "records": [ // Potential multiple records in this category
        { // Start of the first record
            "010": { // UAP entries with names matching the spec
                "SAC": 0, // UAP entry fields
                "SIC": 0
            },
            "020": {
                "TYP": 0,
                "SIM": 0,
                "SSRPSR": 0,
                "ANT": 0,
                "SPI": 0,
                "RAB": 0,
                "TST": 0,
                "DS1DS2": 0,
                "ME": 0,
                "MI": 0
            },
            "030": {
                "WE": 0
            }
        }
    ]
}
```

[eurocontrol-asterix]: https://www.eurocontrol.int/asterix "All-purpose structured EUROCONTROL surveillance information exchange"
[zoranbosnjak-specs]: https://zoranbosnjak.github.io/asterix-specs/ "ASTERIX specifications"

## License

FreeAsterix is completely free for non-commercial use. For commercial projects
it however requires a license purchased.

See <https://github.com/freeasterix/freeasterix/blob/main/LICENSE> for
licensing conditions.

Contact the library vendor [Bohemia Automation
Ltd.](https://www.bohemia-automation.com/) about purchasing questions.
