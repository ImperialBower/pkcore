use zerocopy::byteorder::{BigEndian, LittleEndian, U64};
use zerocopy::{AsBytes, FromBytes, LayoutVerified, Unaligned, U16, U32};
//         assert_eq!(sut.rank, 1);
//         assert_eq!(seven.cards(), Cards::from(sut.bc));
//         assert_eq!(five.cards(), Cards::from(sut.best));
//         assert_eq!(sut.bc, Bard(4_468_415_255_281_664));
//         assert_eq!(sut.best, Bard(4_362_862_139_015_168));
fn main() -> sled::Result<()> {
    let db = sled::open("generated/sleigh")?;

    write(&db);

    Ok(())
}

fn write(db: &sled::Db) {
    let k = Key { bcm: U64::new(1) };
    let v = Value {
        best: Default::default(),
        ckc: Default::default(),
    };
}

#[derive(FromBytes, AsBytes, Unaligned)]
#[repr(C)]
struct Key {
    bcm: U64<BigEndian>,
}

struct Value {
    best: U64<LittleEndian>,
    ckc: U64<LittleEndian>,
}
