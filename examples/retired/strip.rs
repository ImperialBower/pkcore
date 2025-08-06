use pkcore::analysis::store::db::headsup_preflop_result::HUPResult;
use pkcore::analysis::store::db::sqlite::Sqlable;
use pkcore::arrays::matchups::masked::Masked;
use pkcore::arrays::matchups::masks::suit_texture::SuitTexture;
use rusqlite::Connection;

fn main() {
    let conn = Connection::open("../../generated/hups_07_31_2025.db").unwrap();
    let hupr = HUPResult::select_all(&conn);
    // let masks = Masked::parse_hups_as_vectors(&hupr);
    conn.close().unwrap();

    let conn = Connection::open("../../generated/clean_hups.db").unwrap();
    HUPResult::create_table(&conn).expect("TODO: panic message");
    for h in hupr {
        let masked = Masked::from(&h);
        match masked.texture {
            SuitTexture::Type1112a => {}
            SuitTexture::Type1112b => {}
            SuitTexture::Type1112c => {}
            SuitTexture::Type1112d => {}
            SuitTexture::Type1112e => {}
            SuitTexture::Type1223a => {}
            SuitTexture::Type1223b => {}
            SuitTexture::Type1223c => {}
            SuitTexture::Type1223d => {}
            SuitTexture::Type1212a => {}
            SuitTexture::Type1212b => {}
            _ => match HUPResult::insert(&conn, &h) {
                Ok(_) => {
                    println!("{h} inserted");
                }
                Err(e) => {
                    println!("{h} NOT inserted {:?}", e);
                }
            },
        }
    }
    conn.close().unwrap();
}
