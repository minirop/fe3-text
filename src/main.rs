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
                0x87 => {
                    // end of chapter?
                    println!("[Unknown87]");
                }
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
                0x8F => {
                    let unk = rom.read_u8()?;
                    println!("[Unknown8F({unk:#X})]");
                }
                0x92 => {
                    let frame = rom.read_u8()?;

                    println!("[SwitchFrame({frame})]");
                }
                0x93 => println!("[Unknown93]"),
                0x94 => {
                    let unk1 = rom.read_u8()?;
                    let unk2 = rom.read_u8()?;
                    println!("[Unknown94({unk1:#X}, {unk2:#X})]");
                }
                _ => panic!("Unknown command {command:#X} at index {:#X}", rom.seek(SeekFrom::Current(0))?),
            }
        } else {
            let mut character = id;
            while character != 0 {
                let c = CHARACTERS[page][character as usize];
                if c == '.' {
                    print!(" {:02X}/{:02X} ", page + 0x11, character);
                } else {
                    print!("{c}");
                }
                character = rom.read_u8()?;
            }

            rom.seek(SeekFrom::Current(-1))?;
        }
    }

    Ok(())
}

#[rustfmt::skip]
const CHARACTERS: [[char; 256]; 4] = [
    [
        /* 00 */ 'X', 'あ', 'い', 'う', 'え', 'お', 'か', 'き', 'く', 'け', 'こ', 'さ', 'し', 'す', 'せ', 'そ',
        /* 10 */ 'た', 'ち', 'つ', 'て', 'と', 'な', 'に', 'ぬ', 'ね', 'の', 'は', 'ひ', 'ふ', 'へ', 'ほ', 'ま',
        /* 20 */ 'み', 'む', 'め', 'も', 'や', 'ゆ', 'よ', 'ら', 'り', 'る', 'れ', 'ろ', 'わ', 'を', 'ん', 'が',
        /* 30 */ 'ぎ', 'ぐ', 'げ', 'ご', 'ざ', 'じ', 'ず', 'ぜ', 'ぞ', 'だ', 'ぢ', 'づ', 'で', 'ど', 'ば', 'び',
        /* 40 */ 'ぶ', 'べ', 'ぼ', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', 'ゃ', 'っ', '.',
        /* 50 */ 'ょ', 'ア', 'イ', 'ウ', 'エ', 'オ', 'カ', 'キ', 'ク', 'ケ', 'コ', 'サ', 'シ', 'ス', 'セ', 'ソ',
        /* 60 */ 'タ', 'チ', 'ツ', 'テ', 'ト', 'ナ', 'ニ', 'ヌ', 'ネ', 'ノ', 'ハ', 'ヒ', 'フ', 'ヘ', 'ホ', 'マ',
        /* 70 */ 'ミ', 'ム', 'メ', 'モ', 'ヤ', 'ユ', 'ヨ', 'ラ', 'リ', 'ル', 'レ', 'ロ', '.', 'ン', 'ガ', 'ギ',
        /* 80 */ 'グ', 'ゲ', 'ゴ', 'ザ', 'ジ', 'ズ', 'ゼ', 'ゾ', 'ダ', 'ヂ', 'ヅ', 'デ', 'ド', 'バ', 'ビ', 'ブ',
        /* 90 */ 'ベ', 'ボ', 'パ', 'ピ', 'プ', 'ペ', 'ポ', '.', 'ィ', '.', '.', '.', '.', '.', '.', '.',
        /* A0 */ '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '何', '会', '賊', '大', '.',
        /* B0 */ '.', '.', '出', '.', '.', '.', '帝', '.', '.', 'ー', '勇', '者', '.', '.', '.', '.',
        /* C0 */ '.', '.', '.', '.', ' ', '！', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.',
        /* D0 */ '.', '.', '.', '.', '.', '一', '.', '.', '.', '.', '.', '.', '.', '兵', '士', '.',
        /* E0 */ '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '城', '海', '人', '騎',
        /* F0 */ '.', '王', '子', '.', '様', '父', '行', '戦', '来', '・', '.', '.', '.', '.', '.', '国',
    ],
    [
        /* 00 */ '.', '.', '.', '.', '.', '方', '.', '.', '.', '.', '助', '.', '.', '.', '.', '.',
        /* 10 */ '.', '.', '.', '々', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.',
        /* 20 */ '.', '村', '旅', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '占', 'X', '.',
        /* 30 */ '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '敢', '.',
        /* 40 */ '.', '.', '.', '.', '.', '.', '.', '.', '.', '無', '砦', '.', '.', '.', '.', '.',
        /* 50 */ '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '意', '.', '.', '.',
        /* 60 */ '.', '.', '.', '.', '.', '.', '.', '決', '.', '.', '.', '.', '.', '.', '上', '.',
        /* 70 */ '.', '.', '.', '.', '.', '.', '.', '.', '.', '多', '達', '.', '.', '.', '.', '.',
        /* 80 */ '事', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '立', '.', '.', '.',
        /* 90 */ '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '隊', '.', '.', '.', '.', '.',
        /* A0 */ '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '切', '.', '.', '.', '.', '.',
        /* B0 */ '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.',
        /* C0 */ '.', '.', '.', '突', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '心', '.',
        /* D0 */ '.', '.', '.', '.', '.', '.', '時', '.', '.', '.', '.', '.', '.', '.', '.', '.',
        /* E0 */ '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '先', '.',
        /* F0 */ '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '礼', '.', '.', '.', '.', '.',
    ],
    [
        /* 00 */ '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.',
        /* 10 */ '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.',
        /* 20 */ '.', '.', '.', '.', '.', '.', '.', '.', '.', '負', '.', '.', '.', '.', '.', '.',
        /* 30 */ '然', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.',
        /* 40 */ '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.',
        /* 50 */ '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '呋', '.', '.', '.', '.',
        /* 60 */ '.', '.', '.', '.', '.', '緒', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.',
        /* 70 */ '.', '.', '.', '理', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.',
        /* 80 */ '.', '.', '.', '.', '.', '長', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.',
        /* 90 */ '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '注', '.', '.', '.',
        /* A0 */ '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.',
        /* B0 */ '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.',
        /* C0 */ '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.',
        /* D0 */ '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.',
        /* E0 */ '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.',
        /* F0 */ '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.',
    ],
    [
        /* 00 */ '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.',
        /* 10 */ '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.',
        /* 20 */ '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '役', '.',
        /* 30 */ '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.',
        /* 40 */ '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.',
        /* 50 */ '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.',
        /* 60 */ '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.',
        /* 70 */ '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.',
        /* 80 */ '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.', '.',
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
    "Unknown 007",
    "Unknown 008",
    "Unknown 009",
    "Unknown 010",
    "Unknown 011",
    "Unknown 012",
    "Unknown 013",
    "Unknown 014",
    "Unknown 015",
    "Unknown 016",
    "Unknown 017",
    "Unknown 018",
    "Unknown 019",
    "Unknown 020",
    "Unknown 021",
    "Unknown 022",
    "Unknown 023",
    "Unknown 024",
    "Unknown 025",
    "Unknown 026",
    "Unknown 027",
    "Unknown 028",
    "Unknown 029",
    "Unknown 030",
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
    "Unknown 041",
    "Unknown 042",
    "Unknown 043",
    "Unknown 044",
    "Unknown 045",
    "Unknown 046",
    "Unknown 047",
    "Unknown 048",
    "Unknown 049",
    "Unknown 050",
    "Unknown 051",
    "Unknown 052",
    "Unknown 053",
    "Unknown 054",
    "Unknown 055",
    "Unknown 056",
    "Unknown 057",
    "Unknown 058",
    "Unknown 059",
    "Unknown 060",
    "Unknown 061",
    "Unknown 062",
    "Unknown 063",
    "Unknown 064",
    "Unknown 065",
    "Unknown 066",
    "Unknown 067",
    "Unknown 068",
    "Unknown 069",
    "Unknown 070",
    "Unknown 071",
    "Unknown 072",
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
    "Unknown 096",
    "Unknown 097",
    "King of Talys",
    "Unknown 099",
    "Unknown 100",
    "Unknown 101",
    "Unknown 102",
    "Unknown 103",
    "Unknown 104",
    "Unknown 105",
    "Unknown 106",
    "Unknown 107",
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
"Unknown 146",
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
