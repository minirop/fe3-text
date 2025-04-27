use byteorder::LittleEndian;
use byteorder::ReadBytesExt;
use byteorder::WriteBytesExt;
use clap::Parser;
use clap::Subcommand;
use clap_num::maybe_hex;
use std::fs::File;
use std::fs::read_to_string;
use std::io::Cursor;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;

#[derive(Debug, Parser)]
struct Args {
    filename: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Compile {
        #[command(subcommand)]
        command: CompilerCommands,
    },
    Decompile {
        #[command(subcommand)]
        command: DecompilerCommands,
    },
}

#[derive(Subcommand, Debug)]
enum CompilerCommands {
    List { output: String },
}

#[derive(Subcommand, Debug)]
enum DecompilerCommands {
    Dialogue {
        #[arg(short, long, value_parser=maybe_hex::<u64>, default_value="0")]
        offset: u64,
    },
    List {
        #[arg(short, long, value_parser=maybe_hex::<u64>, default_value="0")]
        start: u64,
        #[arg(short, long, value_parser=maybe_hex::<u64>)]
        end: u64,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    match args.command {
        Commands::Compile { command } => match command {
            CompilerCommands::List { output } => compile_array_of_string(&args.filename, &output),
        },
        Commands::Decompile { command } => match command {
            DecompilerCommands::Dialogue { offset } => decompile_dialogue(&args.filename, offset),
            DecompilerCommands::List { start, end } => {
                print_array_of_strings(&args.filename, start, end)
            }
        },
    }
}

fn decompile_dialogue(filename: &str, offset: u64) -> Result<(), Box<dyn std::error::Error>> {
    let mut rom = File::open(filename)?;
    rom.seek(SeekFrom::Start(offset))?;

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
                if c == '_' {
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

fn print_array_of_strings(
    filename: &str,
    begin: u64,
    end: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut rom = File::open(filename)?;
    rom.seek(SeekFrom::Start(begin))?;
    let mut buffer = vec![0u8; (end - begin) as usize];
    rom.read(&mut buffer)?;

    let mut cursor = Cursor::new(buffer);
    while let Ok(data) = cursor.read_u16::<LittleEndian>() {
        if data == 0xFFFF {
            println!();
        } else {
            let c = CHARACTERS[0][data as usize + 1];
            let c = if c == ' ' { 'ー' } else { c };
            print!("{c}");
        }
    }

    Ok(())
}

fn compile_array_of_string(filename: &str, output: &str) -> Result<(), Box<dyn std::error::Error>> {
    let strings = read_to_string(filename)?;

    let mut output_file = File::create(output)?;

    let cs = &CHARACTERS[0];
    for string in strings.split("\n") {
        if string.len() > 0 {
            for c in string.chars() {
                let index = if c == 'ー' {
                    0xC4
                } else {
                    cs.iter().position(|&r| r == c).unwrap()
                };
                output_file.write_u16::<LittleEndian>((index - 1) as u16)?;
            }
            output_file.write_u16::<LittleEndian>(0xFFFF)?;
        }
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
        /* 40 */ 'ぶ', 'べ', 'ぼ', 'ぱ', '_', '_', '_', '_', '_', 'ぃ', '_', 'ぇ', '_', 'ゃ', 'っ', 'ゅ',
        /* 50 */ 'ょ', 'ア', 'イ', 'ウ', 'エ', 'オ', 'カ', 'キ', 'ク', 'ケ', 'コ', 'サ', 'シ', 'ス', 'セ', 'ソ',
        /* 60 */ 'タ', 'チ', 'ツ', 'テ', 'ト', 'ナ', 'ニ', 'ヌ', 'ネ', 'ノ', 'ハ', 'ヒ', 'フ', 'ヘ', 'ホ', 'マ',
        /* 70 */ 'ミ', 'ム', 'メ', 'モ', 'ヤ', 'ユ', 'ヨ', 'ラ', 'リ', 'ル', 'レ', 'ロ', 'ワ', 'ン', 'ガ', 'ギ',
        /* 80 */ 'グ', 'ゲ', 'ゴ', 'ザ', 'ジ', 'ズ', 'ゼ', 'ゾ', 'ダ', 'ヂ', 'ヅ', 'デ', 'ド', 'バ', 'ビ', 'ブ',
        /* 90 */ 'ベ', 'ボ', 'パ', 'ピ', 'プ', 'ペ', 'ポ', 'ァ', 'ィ', 'ゥ', 'ェ', 'ォ', 'ャ', 'ッ', 'ュ', 'ョ',
        /* A0 */ '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '神', '何', '会', '賊', '大', '男',
        /* B0 */ '元', '気', '出', '_', '今', '_', '帝', '_', '_', 'ー', '勇', '者', '？', '苦', '加', '争',
        /* C0 */ '・', '_', '（', '）', ' ', '！', '攻', '撃', '力', '守', '備', '魔', '竜', '石', '団', '_',
        /* D0 */ '_', '場', 'E', '必', '殺', '一', '道', '書', '杖', '系', '_', '_', '話', '兵', '士', '使',
        /* E0 */ '用', '_', '_', '法', '軍', '白', '火', '飛', '地', '暗', '黒', '進', '城', '海', '人', '騎',
        /* F0 */ '.', '王', '子', '女', '様', '父', '行', '戦', '来', '・', '母', '山', '集', '_', '剣', '国',
    ],
    [   // 12
        /* 00 */ '_', '待', '_', '_', '東', '方', '辺', '境', '_', '_', '助', '_', '_', '_', '身', '愛',
        /* 10 */ '姉', '姫', '_', '々', '_', '年', '日', '対', '_', '_', '彼', '解', '放', '島', '_', '_',
        /* 20 */ '近', '村', '旅', '悪', '_', '知', '陸', '_', '_', '建', '_', '_', '_', '占', '領', '_',
        /* 30 */ '名', '_', '_', '公', '_', '_', '前', '_', '_', '同', '盟', '_', '財', '宝', '敢', '箱',
        /* 40 */ '俺', '奴', '中', '_', '_', '下', '都', '_', '_', '無', '砦', '_', '将', '_', '_', '_',
        /* 50 */ '門', '_', '後', '_', '入', '休', '_', '_', '金', '自', '治', '好', '意', '_', '傭', '_',
        /* 60 */ '_', '持', '願', '質', '捕', '要', '_', '決', '_', '_', '見', '性', '恐', '思', '上', '悲',
        /* 70 */ '_', '分', '代', '救', '率', '_', '_', '_', '物', '多', '達', '_', '_', '家', '_', '_',
        /* 80 */ '事', '世', '三', '種', '武', '器', '_', '血', '_', '貴', '_', '_', '立', '偉', '手', '_',
        /* 90 */ '_', '向', '_', '_', '_', '美', '荒', '_', '全', '部', '隊', '取', '_', '_', '光', '殿',
        /* A0 */ '_', '止', '_', '_', '強', '_', '_', '_', '開', '族', '切', '従', '現', '_', '_', '_',
        /* B0 */ '受', '_', '_', '滅', '亡', '_', '高', '度', '_', '動', '_', '司', '祭', '失', '追', '込',
        /* C0 */ '包', '囲', '_', '突', '_', '残', '選', '命', '_', '_', '盗', '港', '_', '_', '心', '_',
        /* D0 */ '生', '足', '移', '_', '明', '望', '時', '平', '和', '支', '配', '激', '界', '_', '野', '乱',
        /* E0 */ '_', '_', '_', '_', '打', '倒', '_', '_', '死', '槍', '弓', '斧', '_', '_', '先', '_',
        /* F0 */ '兄', '_', '信', '_', '正', '義', '忠', '誠', '_', '_', '礼', '_', '_', '伝', '_', '_',
    ],
    [   // 13
        /* 00 */ '_', '_', '_', '_', '反', '_', '_', '_', '_', '_', '_', '_', '町', '隷', '天', '空',
        /* 10 */ '_', '_', '_', '連', '去', '_', '_', '_', '_', '_', '再', '_', '_', '_', '少', '_',
        /* 20 */ '_', '制', '圧', '病', '終', '_', '_', '商', '勝', '負', '_', '_', '_', '_', '_', '_',
        /* 30 */ '然', '_', '_', '_', '_', '_', '_', '装', '西', '妹', '安', '貨', '_', '_', '_', '_',
        /* 40 */ '_', '_', '_', '_', '封', '印', '_', '_', '間', '橋', '_', '脱', '_', '_', '_', '昔',
        /* 50 */ '途', '_', '_', '狂', '_', '_', '重', '栄', '発', '_', '_', '呋', '体', '令', '_', '_',
        /* 60 */ '_', '_', '接', '完', '_', '緒', '_', '_', '_', '_', '_', '_', '当', '_', '本', '_',
        /* 70 */ '少', '作', '_', '理', '_', '_', '逃', '_', '不', '_', '参', '_', '_', '_', '_', '共',
        /* 80 */ '_', '_', '感', '与', '_', '長', '_', '担', '急', '由', '_', '_', '_', '_', '_', '仲',
        /* 90 */ '変', '_', '_', '売', '買', '_', '_', '恋', '_', '_', '修', '_', '注', '_', '情', '_',
        /* A0 */ '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '有', '若', '_', '効', '_', '_',
        /* B0 */ '離', '_', '存', '_', '_', '_', '_', '_', '_', '_', '_', '抗', '_', '処', '刑', '_',
        /* C0 */ '陛', '獄', '_', '_', '_', '_', '_', '結', '_', '_', '_', '_', '_', '_', '_', '赤',
        /* D0 */ '_', '_', '_', '_', '_', '_', '派', '遣', '_', '_', '_', '_', '_', '_', '_', '_',
        /* E0 */ '_', '_', '_', '_', '果', '_', '_', '_', '_', '_', '_', '別', '_', '_', '_', '_',
        /* F0 */ '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '鋼', '_',
    ],
    [   // 14
        /* 00 */ '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_',
        /* 10 */ '_', '戻', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_',
        /* 20 */ '_', '_', '_', '_', '_', '_', '託', '娘', '_', '_', '_', '_', '_', '_', '役', '_',
        /* 30 */ '面', '安', '我', '堂', '腕', '_', '_', '業', '_', '_', '_', '_', '_', '_', '_', '_',
        /* 40 */ '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_',
        /* 50 */ '_', '_', '悔', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_',
        /* 60 */ '通', '供', '許', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_',
        /* 70 */ '_', '稼', '_', '_', '_', '_', '覇', '_', '_', '_', '付', '_', '_', '_', '_', '_',
        /* 80 */ '_', '_', '_', '_', '細', '_', '_', '川', '_', '_', '呼', '_', '_', '_', '_', '欲',
        /* 90 */ '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_',
        /* A0 */ '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_',
        /* B0 */ '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_',
        /* C0 */ '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_',
        /* D0 */ '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_',
        /* E0 */ '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_',
        /* F0 */ '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_', '_',
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
