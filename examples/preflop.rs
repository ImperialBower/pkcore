use csv::Reader;
use pkcore::Shifty;
use pkcore::analysis::store::db::hup::HUPResult;
use pkcore::analysis::store::db::sqlite::Sqlable;
use pkcore::arrays::matchups::sorted_heads_up::SortedHeadsUp;
use pkcore::util::terminal::Terminal;
use rusqlite::Connection;
use std::fs::File;

/// Incremental generator of HUPResult records from distinct SortedHeadsUp combinations.
///
/// - Iterates through SortedHeadsUp::distinct() combinations
/// - Converts each SortedHeadsUp to HUPResult (which calculates wins)
/// - Uses HUPResult::exists() to check if already stored
/// - Inserts results via the Sqlable trait implementation
///
///
/// Naked
/// ```txt
/// A♠ A♥ 7♦ 7♣, 79.69% (1364608), 20.05% (343300), 0.26% (4396)
/// A♠ A♥ 6♦ 6♣, 79.66% (1363968), 20.05% (343394), 0.29% (4942)
/// A♠ A♥ 5♦ 5♣, 80.06% (1370808), 19.60% (335688), 0.34% (5808)
/// A♠ A♥ 4♦ 4♣, 80.47% (1377896), 19.15% (327870), 0.38% (6538)
/// A♠ A♥ 3♦ 3♣, 80.88% (1384984), 18.68% (319884), 0.43% (7436)
/// A♠ A♥ 2♦ 2♣, 81.30% (1392072), 18.20% (311672), 0.50% (8560)
/// A♠ A♥ A♦ A♣, 2.17% (37210), 2.17% (37210), 95.65% (1637884)
/// A♠ A♥ K♦ K♣, 81.06% (1388072), 18.55% (317694), 0.38% (6538)
/// A♠ A♥ K♠ K♣, 81.71% (1399204), 17.82% (305177), 0.46% (7923)
/// A♠ A♥ K♠ K♥, 82.36% (1410336), 17.09% (292660), 0.54% (9308)
/// A♠ A♦ K♥ Q♥, 82.13% (1406263), 17.50% (299588), 0.38% (6453)
/// A♠ A♥ Q♦ Q♣, 80.69% (1381624), 18.96% (324624), 0.35% (6056)
/// A♠ A♥ Q♠ Q♣, 81.33% (1392614), 18.24% (312251), 0.43% (7439)
/// A♠ A♥ Q♠ Q♥, 81.97% (1403604), 17.51% (299878), 0.52% (8822)
/// A♠ A♦ Q♥ J♥, 80.12% (1371916), 19.52% (334283), 0.36% (6105)
/// A♠ A♥ J♦ J♣, 80.31% (1375176), 19.36% (331554), 0.33% (5574)
/// A♠ A♥ J♠ J♣, 80.94% (1386024), 18.65% (319325), 0.41% (6955)
/// A♠ A♥ J♠ J♥, 81.58% (1396872), 17.93% (307096), 0.49% (8336)
/// A♠ A♦ J♥ T♥, 78.12% (1337569), 21.55% (368978), 0.34% (5757)
/// A♠ A♦ J♠ T♥, 82.56% (1413685), 17.08% (292487), 0.36% (6132)
/// A♠ A♦ J♥ T♠, 82.56% (1413643), 17.08% (292529), 0.36% (6132)
/// A♠ A♦ J♠ T♦, 83.22% (1424984), 16.34% (279821), 0.44% (7499)
/// A♠ A♦ J♦ T♠, 83.22% (1424984), 16.34% (279821), 0.44% (7499)
/// A♠ A♥ T♦ T♣, 79.93% (1368728), 19.77% (338484), 0.30% (5092)
/// A♠ A♥ T♠ T♣, 80.56% (1379434), 19.06% (326399), 0.38% (6471)
/// A♠ A♥ T♠ T♥, 81.19% (1390140), 18.36% (314314), 0.46% (7850)
/// A♠ A♥ 9♦ 9♣, 80.05% (1370736), 19.66% (336724), 0.28% (4844)
/// A♠ A♥ 9♠ 9♣, 80.68% (1381464), 18.96% (324610), 0.36% (6230)
/// A♠ A♥ 9♠ 9♥, 81.31% (1392192), 18.25% (312496), 0.44% (7616)
/// A♠ A♥ 8♦ 8♣, 79.68% (1364288), 20.07% (343646), 0.26% (4370)
/// A♠ A♥ 8♠ 8♣, 80.29% (1374874), 19.37% (331680), 0.34% (5750)
/// A♠ A♥ 8♠ 8♥, 80.91% (1385460), 18.67% (319714), 0.42% (7130)
/// A♠ A♥ 7♦ 7♣, 79.69% (1364608), 20.05% (343300), 0.26% (4396)
/// A♠ A♥ 7♠ 7♣, 80.31% (1375194), 19.35% (331347), 0.34% (5763)
/// A♠ A♥ 7♠ 7♥, 80.93% (1385780), 18.65% (319394), 0.42% (7130)
/// A♠ A♥ 6♦ 6♣, 79.66% (1363968), 20.05% (343394), 0.29% (4942)
/// A♠ A♥ 6♠ 6♣, 80.27% (1374538), 19.36% (331487), 0.37% (6279)
/// A♠ A♥ 6♦ 6♥, 80.27% (1374538), 19.36% (331487), 0.37% (6279)
/// A♠ A♥ 6♠ 6♥, 80.89% (1385108), 18.66% (319580), 0.44% (7616)
///
/// A♠ A♥ 5♦ 5♣, 80.06% (1370808), 19.60% (335688), 0.34% (5808)
/// A♠ A♥ 5♠ 5♣, 80.68% (1381517), 18.90% (323705), 0.41% (7082)
/// A♠ A♥ 5♠ 5♥, 81.31% (1392226), 18.20% (311722), 0.49% (8356)
/// A♠ A♥ 4♦ 4♣, 80.47% (1377896), 19.15% (327870), 0.38% (6538)
/// A♠ A♥ 4♠ 4♣, 81.10% (1388747), 18.45% (315867), 0.45% (7690)
/// A♠ A♥ 4♠ 4♥, 81.74% (1399598), 17.75% (303864), 0.52% (8842)
/// A♠ A♥ 3♦ 3♣, 80.88% (1384984), 18.68% (319884), 0.43% (7436)
/// A♠ A♥ 3♠ 3♣, 81.53% (1395977), 17.98% (307945), 0.49% (8382)
/// A♠ A♥ 3♠ 3♥, 82.17% (1406970), 17.29% (296006), 0.54% (9328)
/// A♠ A♥ 2♦ 2♣, 81.30% (1392072), 18.20% (311672), 0.50% (8560)
/// A♠ A♥ 2♠ 2♣, 81.95% (1403207), 17.51% (299910), 0.54% (9187)
/// A♠ A♥ 2♠ 2♥, 82.60% (1414342), 16.83% (288148), 0.57% (9814)
///
/// K♠ K♥ 3♦ 3♣, 80.49% (1378184), 19.08% (326652), 0.44% (7468)
/// K♠ K♥ 3♠ 3♣, 81.12% (1389038), 18.39% (314845), 0.49% (8421)
/// K♠ K♥ 3♠ 3♥, 81.75% (1399892), 17.70% (303038), 0.55% (9374)
/// K♠ K♥ 2♦ 2♣, 80.90% (1385272), 18.60% (318440), 0.50% (8592)
/// K♠ K♥ 2♠ 2♣, 81.54% (1396268), 17.92% (306810), 0.54% (9226)
/// K♠ K♥ 2♠ 2♥, 82.19% (1407264), 17.24% (295180), 0.58% (9860)
///
/// 6♠ 6♥ 5♦ 5♣, 79.73% (1365284), 18.39% (314904), 1.88% (32116)
/// 6♠ 6♥ 5♠ 5♣, 80.39% (1376436), 17.67% (302502), 1.95% (33366)
/// 6♠ 6♥ 5♠ 5♥, 81.04% (1387588), 16.94% (290100), 2.02% (34616)
///
/// 6♠ 6♥ 4♦ 4♣, 79.77% (1365852), 18.33% (313854), 1.90% (32598)
/// 6♠ 6♥ 4♠ 4♣, 80.42% (1377007), 17.61% (301564), 1.97% (33733)
/// 6♠ 6♥ 4♠ 4♥, 81.07% (1388162), 16.89% (289274), 2.04% (34868)
///
/// A♠ K♠ Q♠ J♠, 65.65% (1124180), 33.74% (577797), 0.60% (10327)
/// A♠ K♥ Q♠ J♥, 65.10% (1114667), 34.36% (588268), 0.55% (9369)
/// A♠ K♥ Q♥ J♠, 65.10% (1114667), 34.36% (588268), 0.55% (9369)
/// A♠ K♠ T♠ 9♠, 63.95% (1095005), 35.48% (607520), 0.57% (9779)
/// A♠ K♠ 8♠ 7♠, 63.25% (1083026), 36.21% (620020), 0.54% (9258)
/// A♠ K♠ J♦ J♣, 45.94% (786618), 53.68% (919198), 0.38% (6488)
/// A♠ K♠ 7♣ 6♣, 60.14% (1029832), 39.42% (674947), 0.44% (7525)
/// A♠ Q♥ 5♥ 2♠, 65.49% (1121471), 33.92% (580748), 0.59% (10085)
/// A♠ Q♥ 5♠ 2♥, 65.49% (1121471), 33.92% (580748), 0.59% (10085)
///
/// 3♣ 2♦ 3♦ 2♣, 0.71% (12216), 0.71% (12216), 98.57% (1687872)
/// ```
///
/// I'm going to add an initial step to this idea, where you input a number for the amount of
/// matchups you want to calculate. This will give me some control over the work.
///
/// What's really fun about all this is that we are progressing from simple calculations to actual
/// functional composition with the domain data. It is becoming, as I like to say, `plastic`. A
/// material that we can shape with for our own amusement and utility.
///
/// ## Insert
///
/// OK, so we've got records inserting into an actual sqlite DB. Now, we need to deal with a
/// logic flow problem in that we aren't checking if the record is already in the db BEFORE
/// we do the calculations. This is already a really heavy process. We need to save all the
/// time we can.
///
/// `cargo run --example preflop`
fn main() {
    // TODO TD: There should be an easy way to cast this into our error.
    let conn = HUPResult::open_connection().unwrap();
    HUPResult::create_table(&conn).unwrap();
    let mut rdr = reader();

    // There ought to be a clean way to do this.
    // let _shus: Vec<SortedHeadsUp> = rdr.deserialize::<SortedHeadsUp>().into_iter().collect();

    let mut shus: Vec<SortedHeadsUp> = Vec::new();
    for deserialized_shu in rdr.deserialize::<SortedHeadsUp>() {
        shus.push(deserialized_shu.unwrap())
    }

    loop {
        read_input(&conn, &mut shus);
    }
}

fn read_input(conn: &Connection, shus: &mut Vec<SortedHeadsUp>) {
    let now = std::time::Instant::now();

    let i = Terminal::receive_usize("How many runs? ");

    println!("Processing {i} hands.");

    for _ in 0..i {
        let shu = shus.pop().unwrap();
        println!("{shu}");
        process(&conn, &shu);
    }

    println!("read_input() time elapsed: {:.2?}", now.elapsed());
}

fn calc(shu: &SortedHeadsUp) -> HUPResult {
    HUPResult::from(shu)
}

/// Right now we're doing an optimization of this method. We need to be able to check if the record
/// is already in the database before we do the calculations.
///
/// And right away we see that we got the Trait's method sig wrong. Needs to be
/// `fn exists(conn: &Connection, record: &S) -> bool;`
///
/// Much better. Let's go!!!!!
///
/// Time for an overnight test.
///
/// I'm sitting here just watching it load before I go to bed. Not doing anything but enjoying the
/// fruit of a lot of hard work. It may fail, but if it does, it will be another gift. Five minutes
/// of just savoring the moment. Enjoy your wins. Take the time to appreciate it. The beast won't
/// want to. Ignore it. It's been fed. Now we feast.
///
/// ... and it works. Good night.
fn process(conn: &Connection, shu: &SortedHeadsUp) {
    if HUPResult::exists(conn, shu) {
        println!("..... already exists");
    } else {
        let hupr = calc(&shu);
        println!("..... {}", hupr);

        store(&conn, &hupr);
    }
}

fn reader() -> Reader<File> {
    let distinct_key_path = dotenvy::var("DISTINCT_KEY_PATH").unwrap();
    let file = File::open(distinct_key_path).unwrap();
    Reader::from_reader(file)
}

/// OK, it's clear that we've messed up a little bit. We need to get all the shifted results in
/// the database too, but right now `SortedHeadsUp` implements it, and `HUPResult` doesn't, but `HUPResult`
/// is the struct that stores the results. We can either hack it in, or implement `SuitShift` and
/// `Shifty` on `HUPResult`. Let's do it the right way, shall we?
///
/// OK, I'll confess that this makes me very happy:
///
/// ```txt
/// hole cards> 2
/// Processing 2 hands.
/// J♠ J♦ - J♥ J♣
/// ..... __ __ (0) __ __ (0) ties: (0)
/// >>>>> __ __ (0) __ __ (0) ties: (0) inserted!
/// A♠ 9♥ - A♦ 9♣
/// ..... __ __ (0) __ __ (0) ties: (0)
/// >>>>> __ __ (0) __ __ (0) ties: (0) already exists!
/// read_input() time elapsed: 1.69s
/// ```
///
/// UPDATE: We're still seeing some dupes when we run hup_wash. The problem has to be here.
/// Not sure if it matters right now. Thinking we need to tighten up our insert sql function.
///
/// TODO TD DEFECT: Still doing double inserts.
fn store(conn: &Connection, hup: &HUPResult) {
    for s in hup.shifts() {
        // There was a flaw in this earlier which was using
        // `match HUPResult::select(conn, &shu.get_sorted_heads_up().unwrap()) {`
        match HUPResult::select(conn, &s.get_sorted_heads_up().unwrap()) {
            None => {
                HUPResult::insert(conn, hup).unwrap();
                println!(">>>>> {s} shift inserted!");
            }
            Some(_) => {
                println!(">>>>> {s} already exists!");
            }
        }
    }
}

/// We've found the bug in our shifts, so now we need to correct our old inserts.
/// This is to make sure all our shifts are inserted for records already there. This
/// step won't be necessary when we have things fixed.
///
/// Don't think we need this any more now that we have hup_wash to straighten things out.
/// Later on we will want to run a test against the unique shu file. When done they should
/// match.
fn _insert_shifts(conn: &Connection, shu: &SortedHeadsUp) {
    let hup = HUPResult::select(conn, shu).unwrap();
    let others = hup.other_shifts();
    for hup in others {
        let newshu = hup.get_sorted_heads_up().unwrap();
        if HUPResult::exists(conn, &newshu) {
            println!(">>>>> {hup} as shift already exists!");
        } else {
            HUPResult::insert(conn, &hup).unwrap();
            println!(">>>>> {hup} shift inserted!");
        }
    }
}
