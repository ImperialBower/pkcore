use sled::IVec;
use zerocopy::byteorder::{BigEndian, LittleEndian, U64};
use zerocopy::{AsBytes, FromBytes, Unaligned};
//         assert_eq!(sut.rank, 1);
//         assert_eq!(seven.cards(), Cards::from(sut.bc));
//         assert_eq!(five.cards(), Cards::from(sut.best));
//         assert_eq!(sut.bc, Bard(4_468_415_255_281_664));
//         assert_eq!(sut.best, Bard(4_362_862_139_015_168));
fn main() -> sled::Result<()> {
    let db = sled::open("../../generated/sleigh")?;
    let _bcm_tree = db.open_tree(b"bcm")?;

    let k = Key {
        bcm: U64::new(4_468_415_255_281_664),
    };
    let v = Value {
        best: U64::new(4_362_862_139_015_168),
        ckc: U64::new(1),
    };

    write_to_tree(&db, &k, &v)?;

    // write(&db);

    // let v = read(&db, &k).unwrap().unwrap();
    // println!("{:?}", v);
    //
    // println!("{}", db.contains_key(k.as_bytes()).unwrap());

    Ok(())
}

fn write_to_tree(tree: &sled::Tree, key: &Key, value: &Value) -> sled::Result<Option<IVec>> {
    tree.insert(key.as_bytes(), value.as_bytes())
}

fn _read(tree: &sled::Tree, key: &Key) -> sled::Result<Option<IVec>> {
    tree.get(key.as_bytes())
}

#[derive(FromBytes, AsBytes, Unaligned)]
#[repr(C)]
struct Key {
    bcm: U64<BigEndian>,
}

#[derive(FromBytes, AsBytes, Unaligned)]
#[repr(C)]
struct Value {
    best: U64<LittleEndian>,
    ckc: U64<LittleEndian>,
}
