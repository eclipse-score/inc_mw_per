/// Example for custom types usage for KVS, with serialization and deserialization.
/// - Implementing serialization/deserialization traits for custom types.
/// - Handling external and nested types.
/// - Usage with KVS.
use rust_kvs::prelude::*;
use std::net::IpAddr;
use tempfile::tempdir;

/// `Point` is used as an example of nested serializable objects.
/// Type is local and traits can be provided.
#[derive(Debug)]
struct Point {
    x: f64,
    y: f64,
}

impl KvsSerialize for Point {
    fn to_kvs(&self) -> KvsValue {
        let mut map = KvsMap::new();
        map.insert("x".to_string(), self.x.to_kvs());
        map.insert("y".to_string(), self.y.to_kvs());
        map.to_kvs()
    }
}

impl KvsDeserialize for Point {
    fn from_kvs(kvs_value: &KvsValue) -> Option<Self> {
        if let KvsValue::Object(map) = kvs_value {
            Some(Point {
                x: f64::from_kvs(map.get("x")?)?,
                y: f64::from_kvs(map.get("y")?)?,
            })
        } else {
            None
        }
    }
}

/// `IpAddr` is used as an example of external type serialization.
/// Neither `IpAddr` nor traits are local - new type pattern must be used.
struct IpAddrWrapper(pub IpAddr);

impl KvsSerialize for IpAddrWrapper {
    fn to_kvs(&self) -> KvsValue {
        KvsValue::String(self.0.to_string())
    }
}

impl KvsDeserialize for IpAddrWrapper {
    fn from_kvs(kvs_value: &KvsValue) -> Option<Self> {
        if let KvsValue::String(str) = kvs_value {
            if let Ok(ip_addr) = str.parse() {
                Some(IpAddrWrapper(ip_addr))
            } else {
                None
            }
        } else {
            None
        }
    }
}

/// Main example struct.
/// - Types defined by `KvsValue`.
/// - `u8` - additional type not defined by `KvsValue`.
/// - `nested` - nested serializable object.
/// - `ip` - external type serialized to `KvsValue`.
#[derive(Debug)]
struct Example {
    i32: i32,
    u32: u32,
    i64: i64,
    u64: u64,
    f64: f64,
    bool: bool,
    string: String,
    vec: Vec<KvsValue>,
    object: KvsMap,
    u8: u8,
    nested: Point,
    ip: IpAddr,
}

impl KvsSerialize for Example {
    fn to_kvs(&self) -> KvsValue {
        let mut map = KvsMap::new();
        // Types defined by `KvsValue`.
        map.insert("i32".to_string(), self.i32.to_kvs());
        map.insert("u32".to_string(), self.u32.to_kvs());
        map.insert("i64".to_string(), self.i64.to_kvs());
        map.insert("u64".to_string(), self.u64.to_kvs());
        map.insert("f64".to_string(), self.f64.to_kvs());
        map.insert("bool".to_string(), self.bool.to_kvs());
        map.insert("string".to_string(), self.string.to_kvs());
        map.insert("vec".to_string(), self.vec.to_kvs());
        map.insert("object".to_string(), self.object.to_kvs());
        map.insert("u8".to_string(), self.u8.to_kvs());

        // Nested serializable object.
        map.insert("nested".to_string(), self.nested.to_kvs());

        // External type serialized to `KvsValue`.
        map.insert("ip".to_string(), IpAddrWrapper(self.ip).to_kvs());

        map.to_kvs()
    }
}

impl KvsDeserialize for Example {
    fn from_kvs(kvs_value: &KvsValue) -> Option<Self> {
        if let KvsValue::Object(map) = kvs_value {
            Some(Example {
                i32: i32::from_kvs(map.get("i32")?)?,
                u32: u32::from_kvs(map.get("u32")?)?,
                i64: i64::from_kvs(map.get("i64")?)?,
                u64: u64::from_kvs(map.get("u64")?)?,
                f64: f64::from_kvs(map.get("f64")?)?,
                bool: bool::from_kvs(map.get("bool")?)?,
                string: String::from_kvs(map.get("string")?)?,
                vec: Vec::from_kvs(map.get("vec")?)?,
                object: KvsMap::from_kvs(map.get("object")?)?,
                u8: u8::from_kvs(map.get("u8")?)?,
                nested: Point::from_kvs(map.get("nested")?)?,
                ip: IpAddrWrapper::from_kvs(map.get("ip")?)?.0,
            })
        } else {
            None
        }
    }
}

fn main() -> Result<(), ErrorCode> {
    // Temporary directory.
    let dir = tempdir()?;
    let dir_string = dir.path().to_string_lossy().to_string();

    // Create initial example object.
    let object = Example {
        i32: -321,
        u32: 321,
        i64: -432,
        u64: 432,
        f64: 444.4,
        bool: true,
        string: "example".to_string(),
        vec: vec![
            KvsValue::from("one"),
            KvsValue::from("two"),
            KvsValue::from("three"),
        ],
        object: KvsMap::from([
            ("first".to_string(), KvsValue::from(-123i32)),
            ("second".to_string(), KvsValue::from(321u32)),
            (
                "third".to_string(),
                KvsValue::String("map_example".to_string()),
            ),
        ]),
        u8: 200,
        nested: Point { x: 432.1, y: 654.3 },
        ip: "127.0.0.1".parse().unwrap(),
    };

    println!("ORIGINAL OBJECT:");
    println!("{object:#?}");
    println!();

    // Create KVS instance.
    let kvs = KvsBuilder::new(InstanceId(0))
        .kvs_load(KvsLoad::Ignored)
        .defaults(KvsDefaults::Ignored)
        .dir(dir_string)
        .build()?;

    // Serialize and set object.
    let serialized_object = object.to_kvs();
    kvs.set_value("example", serialized_object.clone())?;

    println!("SERIALIZED OBJECT:");
    println!("{serialized_object:#?}");
    println!();

    // Modify and set object.
    let modified_object = if let KvsValue::Object(mut obj) = serialized_object {
        obj.insert("i32".to_string(), KvsValue::from(-54321i32));
        KvsValue::Object(obj)
    } else {
        panic!("Invalid type");
    };
    kvs.set_value("example", modified_object.clone())?;

    // Get object from KVS.
    let modified_object = kvs.get_value("example")?;

    println!("MODIFIED OBJECT:");
    println!("{modified_object:#?}");
    println!();

    // Deserialize.
    let deserialized_object = Example::from_kvs(&modified_object).unwrap();

    println!("DESERIALIZED OBJECT:");
    println!("{deserialized_object:#?}");
    println!();

    Ok(())
}
