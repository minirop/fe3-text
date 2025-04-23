use byteorder::LittleEndian;
use byteorder::ReadBytesExt;
use clap::Parser;
use clap_num::maybe_hex;
use std::fs::File;
use std::io::Seek;
use std::io::SeekFrom;

#[derive(Debug, Parser)]
struct Args {
    filename: String,

    #[arg(short, long, value_parser=maybe_hex::<u64>, default_value="0")]
    offset: u64,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let mut rom = File::open(args.filename)?;
    rom.seek(SeekFrom::Start(args.offset))?;

    let mut missing = false;
    let mut page = 0;
    loop {
        let id = rom.read_u8()?;

        if id == 0 {
            let command = rom.read_u8()?;
            match command {
                0 => {
                    println!("[End]");
                    break;
                }
                0x01 => print!("\\n"),
                0x02 => println!("[ClearFrame]"),
                0x05 => println!("[Unknown05]"),
                0x07 => {
                    let zero = rom.read_u8()?;
                    assert_eq!(zero, 0);
                    let colour = rom.read_u8()?;
                    println!("[SetColor({colour:#X})]");
                }
                0x0C => {
                    let other = rom.read_u8()?;
                    println!("[Unknown0C({other})]");
                }
                0x10 => println!("[Unknown10]"),
                0x11 | 0x12 | 0x13 | 0x14 => page = command as usize - 0x11,
                0x16 => {
                    let ten = rom.read_u8()?;
                    assert_eq!(ten, 0x10);
                    let other = rom.read_u8()?;
                    // other = 0 but 1 '1'
                    println!("[Unknown16({other})]");
                }
                0x17 => {
                    let ef = rom.read_u8()?;
                    assert_eq!(ef, 0xEF);
                    let ff = rom.read_u8()?;
                    assert_eq!(ff, 0xFF);

                    println!("[Unknown17]");
                }
                0x80 => println!("[Unknown80]"),
                0x81 => println!("[Unknown81]"),
                0x84 => {
                    let portrait = rom.read_u8()?;
                    let flags = rom.read_u8()?;
                    let portrait = PORTRAITS[portrait as usize];

                    println!("[ShowPortrait({portrait}, {flags})]");
                }
                0x85 => {
                    let frame = rom.read_u8()?;

                    println!("[CloseFrame({frame})]");
                }
                0x86 => println!("[Unknown86]"),
                0x87 => println!("[Unknown87]"),
                0x88 => {
                    let unknown = rom.read_u8()?;

                    println!("[StartDialogue({unknown:#X})]");
                }
                0x89 => {
                    let song = rom.read_u8()?;
                    let volume = rom.read_u8()?;

                    println!("[PlaySong({song}, {volume})]");
                }
                0x8A => println!("[WaitForA]"),
                0x8B => println!("[Unknown8B]"),
                0x8C => println!("[Unknown8C]"),
                0x8D => {
                    let unk1 = rom.read_u8()?;
                    let unk2 = rom.read_u8()?;

                    println!("[Unknown8D({unk1}, {unk2})]");
                }
                0x8E => {
                    let unk1 = rom.read_u8()?;
                    let unk2 = rom.read_u8()?;

                    println!("[Unknown8E({unk1}, {unk2})]");
                }
                0x8F => {
                    let unk = rom.read_u8()?;
                    println!("[Unknown8F({unk:#X})]");
                }
                0x90 => println!("[Unknown90]"),
                0x91 => println!("[Unknown91]"),
                0x92 => {
                    let frame = rom.read_u8()?;

                    println!("[SwitchFrame({frame})]");
                }
                0x93 => println!("[Unknown93]"),
                0x94 => {
                    let unk = rom.read_u16::<LittleEndian>()?;
                    println!("[Unknown94({unk:#X})]");
                }
                0x95 => println!("[Unknown95]"),
                _ => panic!(
                    "Unknown command {command:#X} at index {:#X}",
                    rom.seek(SeekFrom::Current(0))?
                ),
            }
        } else {
            let mut character = id;
            while character != 0 {
                let c = CHARACTERS[page][character as usize];
                if c == '.' {
                    missing = true;
                    print!(" {:02X}/{:02X} ", page + 0x11, character);
                } else {
                    print!("{c}");
                }
                character = rom.read_u8()?;
            }

            rom.seek(SeekFrom::Current(-1))?;
        }
    }

    if missing {
        todo!();
    }

    Ok(())
}

#[rustfmt::skip]
const CHARACTERS: [[char; 256]; 4] = [
    [   // 11
        /* 00 */ 'X', 'あ', 'い', 'う', 'え', 'お', 'か', 'き', 'く', 'け', 'こ', 'さ', 'し', 'す', 'せ', 'そ',
        /* 10 */ 'た', 'ち', 'つ', 'て', 'と', 'な', 'に', 'ぬ', 'ね', 'の', 'は', 'ひ', 'ふ', 'へ', 'ほ', 'ま',
        /* 20 */ 'み', 'む', 'め', 'も', 'や', 'ゆ', 'よ', 'ら', 'り', 'る', 'れ', 'ろ', 'わ', 'を', 'ん', 'が',
        /* 30 */ 'ぎ', 'ぐ', 'げ', 'ご', 'ざ', 'じ', 'ず', 'ぜ', 'ぞ', 'だ', 'ぢ', 'づ', 'で', 'ど', 'ば', 'び',
        /* 40 */ 'ぶ', 'べ', 'ぼ', 'ぱ', '.', '.', '.', '.', '.', 'ぃ', '.', 'ぇ', '.', 'ゃ', 'っ', 'ゅ',
        /* 50 */ 'ょ', 'ア', 'イ', 'ウ', 'エ', 'オ', 'カ', 'キ', 'ク', 'ケ', 'コ', 'サ', 'シ', 'ス', 'セ', 'ソ',
        /* 60 */ 'タ', 'チ', 'ツ', 'テ', 'ト', 'ナ', 'ニ', 'ヌ', 'ネ', 'ノ', 'ハ', 'ヒ', 'フ', 'ヘ', 'ホ', 'マ',
        /* 70 */ 'ミ', 'ム', 'メ', 'モ', 'ヤ', 'ユ', 'ヨ', 'ラ', 'リ', 'ル', 'レ', 'ロ', 'ワ', 'ン', 'ガ', 'ギ',
        /* 80 */ 'グ', 'ゲ', 'ゴ', 'ザ', 'ジ', 'ズ', 'ゼ', 'ゾ', 'ダ', 'ヂ', 'ヅ', 'デ', 'ド', 'バ', 'ビ', 'ブ',
        /* 90 */ 'ベ', 'ボ', 'パ', 'ピ', 'プ', 'ペ', 'ポ', 'ァ', 'ィ', 'ゥ', 'ェ', 'ォ', 'ャ', 'ッ', 'ュ', 'ョ',
        /* A0 */ '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '神', '何', '会', '賊', '大', '男',
        /* B0 */ '元', '気', '出', '.', '今', '.', '帝', '.', '.', 'ー', '勇', '者', '？', '苦', '加', '争',
        /* C0 */ '.', '.', '（', '）', ' ', '！', '攻', '撃', '力', '守', '備', '魔', '竜', '石', '団', '.',
        /* D0 */ '.', '場', '.', '必', '殺', '一', '道', '書', '杖', '系', '.', '.', '話', '兵', '士', '使',
        /* E0 */ '用', '.', '.', '法', '軍', '白', '火', '飛', '地', '暗', '黒', '進', '城', '海', '人', '騎',
        /* F0 */ '.', '王', '子', '女', '様', '父', '行', '戦', '来', '・', '母', '山', '集', '.', '剣', '国',
    ],
    [   // 12
        /* 00 */ '.', '待', '.', '.', '東', '方', '辺', '境', '.', '.', '助', '.', '.', '.', '身', '愛',
        /* 10 */ '姉', '姫', '.', '々', '.', '年', '日', '対', '.', '.', '彼', '解', '放', '島', '.', '.',
        /* 20 */ '近', '村', '旅', '悪', '.', '知', '陸', '.', '.', '建', '.', '.', '.', '占', 'X', '.',
        /* 30 */ '名', '.', '.', '公', '.', '.', '前', '.', '.', '同', '盟', '.', '財', '宝', '敢', '箱',
        /* 40 */ '俺', '奴', '中', '.', '.', '下', '都', '.', '.', '無', '砦', '.', '将', '.', '.', '.',
        /* 50 */ '門', '.', '後', '.', '入', '休', '.', '.', '金', '自', '治', '好', '意', '.', '傭', '.',
        /* 60 */ '.', '持', '願', '質', '捕', '要', '.', '決', '.', '.', '見', '性', '恐', '思', '上', '悲',
        /* 70 */ '.', '分', '代', '救', '率', '.', '.', '.', '物', '多', '達', '.', '.', '家', '.', '.',
        /* 80 */ '事', '世', '三', '種', '武', '器', '.', '血', '.', '貴', '.', '.', '立', '偉', '手', '.',
        /* 90 */ '.', '向', '.', '.', '.', '美', '荒', '.', '全', '部', '隊', '取', '.', '.', '光', '殿',
        /* A0 */ '.', '止', '.', '.', '強', '.', '.', '.', '開', '族', '切', '従', '現', '.', '.', '.',
        /* B0 */ '受', '.', '.', '滅', '亡', '.', '高', '度', '.', '動', '.', '司', '祭', '失', '追', '込',
        /* C0 */ '包', '囲', '.', '突', '.', '残', '選', '命', '.', '.', '盗', '港', '.', '.', '心', '.',
        /* D0 */ '生', '足', '移', '.', '明', '望', '時', '平', '和', '支', '配', '激', '界', '.', '野', '乱',
        /* E0 */ '.', '.', '.', '.', '打', '倒', '.', '.', '死', '槍', '弓', '斧', '.', '.', '先', '.',
        /* F0 */ '兄', '.', '信', '.', '正', '義', '忠', '誠', '.', '.', '礼', '.', '.', '伝', '.', '.',
    ],
    [   // 13
        /* 00 */ '.', '.', '.', '.', '反', '.', '.', '.', '.', '.', '.', '.', '町', '隷', '天', '空',
        /* 10 */ '.', '.', '.', '連', '去', '.', '.', '.', '.', '.', '再', '.', '.', '.', '少', '.',
        /* 20 */ '.', '制', '圧', '病', '終', '.', '.', '商', '勝', '負', '.', '.', '.', '.', '.', '.',
        /* 30 */ '然', '.', '.', '.', '.', '.', '.', '装', '西', '妹', '安', '貨', '.', '.', '.', '.',
        /* 40 */ '.', '.', '.', '.', '封', '印', '.', '.', '間', '橋', '.', '脱', '.', '.', '.', '昔',
        /* 50 */ '途', '.', '.', '狂', '.', '.', '重', '栄', '発', '.', '.', '呋', '体', '令', '.', '.',
        /* 60 */ '.', '.', '接', '完', '.', '緒', '.', '.', '.', '.', '.', '.', '当', '.', '本', '.',
        /* 70 */ '少', '作', '.', '理', '.', '.', '逃', '.', '不', '.', '参', '.', '.', '.', '.', '共',
        /* 80 */ '.', '.', '感', '与', '.', '長', '.', '担', '急', '由', '.', '.', '.', '.', '.', '仲',
        /* 90 */ '変', '.', '.', '売', '買', '.', '.', '恋', '.', '.', '修', '.', '注', '.', '情', '.',
        /* A0 */ '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '有', '若', '.', '効', '.', '.',
        /* B0 */ '離', '.', '存', '.', '.', '.', '.', '.', '.', '.', '.', '抗', '.', '処', '刑', '.',
        /* C0 */ '陛', '獄', '.', '.', '.', '.', '.', '結', '.', '.', '.', '.', '.', '.', '.', '赤',
        /* D0 */ '.', '.', '.', '.', '.', '.', '派', '遣', '.', '.', '.', '.', '.', '.', '.', '.',
        /* E0 */ '.', '.', '.', '.', '果', '.', '.', '.', '.', '.', '.', '別', '.', '.', '.', '.',
        /* F0 */ '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '鋼', '.',
    ],
    [   // 14
        /* 00 */ '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.',
        /* 10 */ '.', '戻', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.',
        /* 20 */ '.', '.', '.', '.', '.', '.', '託', '娘', '.', '.', '.', '.', '.', '.', '役', '.',
        /* 30 */ '面', '安', '我', '堂', '腕', '.', '.', '業', '.', '.', '.', '.', '.', '.', '.', '.',
        /* 40 */ '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.',
        /* 50 */ '.', '.', '悔', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.',
        /* 60 */ '通', '供', '許', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.',
        /* 70 */ '.', '稼', '.', '.', '.', '.', '覇', '.', '.', '.', '付', '.', '.', '.', '.', '.',
        /* 80 */ '.', '.', '.', '.', '細', '.', '.', '川', '.', '.', '呼', '.', '.', '.', '.', '欲',
        /* 90 */ '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.',
        /* A0 */ '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.',
        /* B0 */ '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.',
        /* C0 */ '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.',
        /* D0 */ '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.',
        /* E0 */ '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.',
        /* F0 */ '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.',
    ],
];

const PORTRAITS: [&str; 256] = [
    "Marth",
    "Ceada",
    "Jagen",
    "Unknown 003",
    "Unknown 004",
    "Unknown 005",
    "Unknown 006",
    "Rickard",
    "Unknown 008",
    "Unknown 009",
    "Unknown 010",
    "Unknown 011",
    "Castor",
    "Ogma",
    "Unknown 014",
    "Julian",
    "Lena",
    "Merric",
    "Navarre",
    "Hardin",
    "Unknown 020",
    "Unknown 021",
    "Unknown 022",
    "Unknown 023",
    "Bantu",
    "Caesar",
    "Unknown 026",
    "Unknown 027",
    "Catria",
    "Maria",
    "Minerva",
    "Unknown 031",
    "Unknown 032",
    "Unknown 033",
    "Unknown 034",
    "Unknown 035",
    "Unknown 036",
    "Unknown 037",
    "Unknown 038",
    "Unknown 039",
    "Unknown 040",
    "Wendell",
    "Unknown 042",
    "Unknown 043",
    "Matthis",
    "Unknown 045",
    "Unknown 046",
    "Unknown 047",
    "Unknown 048",
    "Unknown 049",
    "Unknown 050",
    "Unknown 051",
    "Gazzak",
    "Unknown 053",
    "Unknown 054",
    "Gomer",
    "Unknown 056",
    "Unknown 057",
    "Merach",
    "Harmein",
    "Emereus",
    "Unknown 061",
    "Kannival",
    "Unknown 063",
    "Zharov",
    "Unknown 065",
    "Unknown 066",
    "Unknown 067",
    "Unknown 068",
    "Unknown 069",
    "Unknown 070",
    "Unknown 071",
    "Hyman",
    "Unknown 073",
    "Unknown 074",
    "Unknown 075",
    "Unknown 076",
    "Unknown 077",
    "Unknown 078",
    "Unknown 079",
    "Unknown 080",
    "Unknown 081",
    "Unknown 082",
    "Unknown 083",
    "Unknown 084",
    "Unknown 085",
    "Unknown 086",
    "Unknown 087",
    "Unknown 088",
    "Unknown 089",
    "Unknown 090",
    "Unknown 091",
    "Unknown 092",
    "Unknown 093",
    "Unknown 094",
    "Unknown 095",
    "Malledus",
    "Nyna",
    "King of Talys",
    "King of Aurelis",
    "Villager (Bald Dad #1)",
    "Villager (Bald Dad #2)",
    "Unknown 102",
    "Villager (old man)",
    "Villager (old woman)",
    "Villager (Uncle)",
    "Villager (auntie)",
    "Villager (male)",
    "Unknown 108",
    "Unknown 109",
    "Unknown 110",
    "Unknown 111",
    "Unknown 112",
    "Unknown 113",
    "Unknown 114",
    "Unknown 115",
    "Unknown 116",
    "Unknown 117",
    "Unknown 118",
    "Unknown 119",
    "Unknown 120",
    "Unknown 121",
    "Unknown 122",
    "Unknown 123",
    "Unknown 124",
    "Unknown 125",
    "Unknown 126",
    "Unknown 127",
    "Unknown 128",
    "Unknown 129",
    "Unknown 130",
    "Unknown 131",
    "Unknown 132",
    "Unknown 133",
    "Unknown 134",
    "Unknown 135",
    "Unknown 136",
    "Unknown 137",
    "Unknown 138",
    "Unknown 139",
    "Unknown 140",
    "Unknown 141",
    "Unknown 142",
    "Unknown 143",
    "Unknown 144",
    "Unknown 145",
    "Minerva",
    "Unknown 147",
    "Unknown 148",
    "Unknown 149",
    "Unknown 150",
    "Unknown 151",
    "Unknown 152",
    "Unknown 153",
    "Unknown 154",
    "Unknown 155",
    "Unknown 156",
    "Unknown 157",
    "Unknown 158",
    "Unknown 159",
    "Unknown 160",
    "Unknown 161",
    "Unknown 162",
    "Unknown 163",
    "Unknown 164",
    "Unknown 165",
    "Unknown 166",
    "Unknown 167",
    "Unknown 168",
    "Unknown 169",
    "Unknown 170",
    "Unknown 171",
    "Unknown 172",
    "Unknown 173",
    "Unknown 174",
    "Unknown 175",
    "Unknown 176",
    "Unknown 177",
    "Unknown 178",
    "Unknown 179",
    "Unknown 180",
    "Unknown 181",
    "Unknown 182",
    "Unknown 183",
    "Unknown 184",
    "Unknown 185",
    "Unknown 186",
    "Unknown 187",
    "Unknown 188",
    "Unknown 189",
    "Unknown 190",
    "Unknown 191",
    "Unknown 192",
    "Unknown 193",
    "Unknown 194",
    "Unknown 195",
    "Unknown 196",
    "Unknown 197",
    "Unknown 198",
    "Unknown 199",
    "Unknown 200",
    "Unknown 201",
    "Unknown 202",
    "Unknown 203",
    "Unknown 204",
    "Unknown 205",
    "Unknown 206",
    "Unknown 207",
    "Unknown 208",
    "Unknown 209",
    "Unknown 210",
    "Unknown 211",
    "Unknown 212",
    "Unknown 213",
    "Unknown 214",
    "Unknown 215",
    "Unknown 216",
    "Unknown 217",
    "Unknown 218",
    "Unknown 219",
    "Unknown 220",
    "Unknown 221",
    "Unknown 222",
    "Unknown 223",
    "Unknown 224",
    "Unknown 225",
    "Unknown 226",
    "Unknown 227",
    "Unknown 228",
    "Unknown 229",
    "Unknown 230",
    "Unknown 231",
    "Unknown 232",
    "Unknown 233",
    "Unknown 234",
    "Unknown 235",
    "Unknown 236",
    "Unknown 237",
    "Unknown 238",
    "Unknown 239",
    "Unknown 240",
    "Unknown 241",
    "Unknown 242",
    "Unknown 243",
    "Unknown 244",
    "Unknown 245",
    "Unknown 246",
    "Unknown 247",
    "Unknown 248",
    "Unknown 249",
    "Unknown 250",
    "Unknown 251",
    "Unknown 252",
    "Unknown 253",
    "Unknown 254",
    "Unknown 255",
];
