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
        /* 40 */ 'ぶ', 'べ', 'ぼ', 'ぱ', 'ぴ', '_', '_', '_', 'ぁ', 'ぃ', '_', 'ぇ', 'ぉ', 'ゃ', 'っ', 'ゅ',
        /* 50 */ 'ょ', 'ア', 'イ', 'ウ', 'エ', 'オ', 'カ', 'キ', 'ク', 'ケ', 'コ', 'サ', 'シ', 'ス', 'セ', 'ソ',
        /* 60 */ 'タ', 'チ', 'ツ', 'テ', 'ト', 'ナ', 'ニ', 'ヌ', 'ネ', 'ノ', 'ハ', 'ヒ', 'フ', 'ヘ', 'ホ', 'マ',
        /* 70 */ 'ミ', 'ム', 'メ', 'モ', 'ヤ', 'ユ', 'ヨ', 'ラ', 'リ', 'ル', 'レ', 'ロ', 'ワ', 'ン', 'ガ', 'ギ',
        /* 80 */ 'グ', 'ゲ', 'ゴ', 'ザ', 'ジ', 'ズ', 'ゼ', 'ゾ', 'ダ', 'ヂ', 'ヅ', 'デ', 'ド', 'バ', 'ビ', 'ブ',
        /* 90 */ 'ベ', 'ボ', 'パ', 'ピ', 'プ', 'ペ', 'ポ', 'ァ', 'ィ', 'ゥ', 'ェ', 'ォ', 'ャ', 'ッ', 'ュ', 'ョ',
        /* A0 */ '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '神', '何', '会', '賊', '大', '男',
        /* B0 */ '元', '気', '出', '_', '今', '皇', '帝', '聖', '_', 'ー', '勇', '者', '？', '苦', '加', '争',
        /* C0 */ '「', '」', '（', '）', ' ', '！', '攻', '撃', '力', '守', '備', '魔', '竜', '石', '団', '闘',
        /* D0 */ '技', '場', '店', '必', '殺', '一', '道', '書', '杖', '系', 'Ｍ', '星', '話', '兵', '士', '使',
        /* E0 */ '用', '運', '_', '法', '軍', '白', '火', '飛', '地', '暗', '黒', '進', '城', '海', '人', '騎',
        /* F0 */ '.', '王', '子', '女', '様', '父', '行', '戦', '来', '・', '母', '山', '集', 'ー', '剣', '国',
    ],
    [   // 12
        /* 00 */ '_', '待', '祖', '老', '東', '方', '辺', '境', '小', '援', '助', '_', '親', '討', '身', '愛',
        /* 10 */ '姉', '姫', '才', '々', '二', '年', '日', '対', '岸', '街', '彼', '解', '放', '島', '南', '北',
        /* 20 */ '近', '村', '旅', '悪', '_', '知', '陸', '草', '原', '建', '以', '他', '_', '占', '領', '_',
        /* 30 */ '名', '弟', '半', '公', '数', '_', '前', '主', '破', '同', '盟', '内', '財', '宝', '敢', '箱',
        /* 40 */ '俺', '奴', '中', '_', '_', '下', '都', '谷', '築', '無', '砦', '最', '将', '指', '揮', '唯',
        /* 50 */ '門', '始', '後', '退', '入', '休', '_', '_', '金', '自', '治', '好', '意', '的', '傭', '久',
        /* 60 */ '_', '持', '願', '質', '捕', '要', '塞', '決', '罠', '言', '見', '性', '恐', '思', '上', '悲',
        /* 70 */ '辛', '分', '代', '救', '率', '末', '千', '宮', '物', '多', '達', '目', '市', '家', '統', '合',
        /* 80 */ '事', '世', '三', '種', '武', '器', '_', '血', '_', '貴', '_', '_', '立', '偉', '手', '_',
        /* 90 */ '敗', '向', '_', '水', '_', '美', '荒', '迎', '全', '部', '隊', '取', '罪', '賢', '光', '殿',
        /* A0 */ '_', '止', '英', '雄', '強', '背', '_', '未', '開', '族', '切', '従', '現', '弱', '復', '活',
        /* B0 */ '受', '_', '誇', '滅', '亡', '古', '高', '度', '絶', '動', '塔', '司', '祭', '失', '追', '込',
        /* C0 */ '包', '囲', '所', '突', '散', '残', '選', '命', '_', '歩', '盗', '港', '＜', '＞', '心', '着',
        /* D0 */ '生', '足', '移', '文', '明', '望', '時', '平', '和', '支', '配', '激', '界', '秘', '野', '乱',
        /* E0 */ '域', '_', '_', '_', '打', '倒', '、', '_', '死', '槍', '弓', '斧', '民', '位', '先', '利',
        /* F0 */ '兄', '氷', '信', '真', '正', '義', '忠', '誠', '臣', '婚', '礼', '_', '伯', '伝', '説', '炎',
    ],
    [   // 13
        /* 00 */ '_', '紋', '章', '_', '反', '断', '_', '_', '初', '_', '消', '第', '町', '隷', '天', '空',
        /* 10 */ '_', '遠', '征', '連', '去', '嘆', '_', '_', '墓', '_', '再', '奪', '回', '_', '少', '_',
        /* 20 */ '壇', '制', '圧', '病', '終', '_', '相', '商', '勝', '負', '続', '_', '_', '_', '賞', '_',
        /* 30 */ '然', '異', '壊', '護', '熱', '識', '_', '装', '西', '妹', '安', '貨', '侵', '玉', '座', '準',
        /* 40 */ '鉄', '銀', '屋', '百', '封', '印', '盾', '電', '間', '橋', '_', '脱', '防', '壮', '冷', '昔',
        /* 50 */ '途', '官', '_', '狂', '_', '希', '重', '栄', '発', '_', '語', '呋', '体', '令', '精', '鋭',
        /* 60 */ '邪', '協', '接', '完', '政', '緒', '節', '砂', '漠', '灼', '太', '陽', '当', '党', '本', '能',
        /* 70 */ '少', '作', '幸', '理', '新', '夫', '逃', '略', '不', '永', '参', '惨', '七', '減', '成', '共',
        /* 80 */ '訪', '_', '感', '与', '挟', '長', '具', '担', '急', '由', '耳', '産', '_', '_', '超', '仲',
        /* 90 */ '変', '務', '償', '売', '買', '声', '教', '恋', '員', '服', '修', '屈', '注', '巨', '情', '馬',
        /* A0 */ '_', '森', '得', '_', '軽', '経', '験', '_', '格', '_', '有', '若', '涙', '効', '特', '殊',
        /* B0 */ '離', '在', '存', '呪', '_', '仕', '_', '興', '蛮', '神', '外', '抗', '抵', '処', '刑', '_',
        /* C0 */ '陛', '獄', '可', '個', '室', '危', '択', '結', '引', '_', '旗', '密', '_', '_', '_', '赤',
        /* D0 */ '短', '乗', '_', '_', '_', '逆', '派', '遣', '_', '踊', '迷', '_', '学', '勉', '伐', '_',
        /* E0 */ '限', '議', '_', '_', '果', '保', '害', '期', '_', '_', '岩', '別', '想', '_', '_', '_',
        /* F0 */ '_', '_', '_', '_', '_', '衛', '息', '類', '_', '実', '_', '非', '術', '_', '鋼', '継',
    ],
    [   // 14
        /* 00 */ '_', '承', '_', '_', '増', '_', '_', '_', '_', '風', '混', '_', '_', '_', '_', '定',
        /* 10 */ '_', '戻', '請', '_', '_', '任', '残', '忘', '涯', '_', '頼', '姿', '_', '遊', '_', '_',
        /* 20 */ '良', '_', '_', '師', '捨', '_', '託', '娘', '告', '_', '予', '_', '努', '焼', '役', '仮',
        /* 30 */ '面', '安', '我', '堂', '腕', '_', '_', '業', '五', '_', '_', '_', '_', '_', '応', '_',
        /* 40 */ '_', '_', '_', '_', '_', '_', '_', '報', '皆', '払', '混', '双', '口', '降', '伏', '_',
        /* 50 */ '_', '幅', '悔', '毎', '牢', '肉', '到', '_', '妃', '_', '迫', '求', '職', '魂', '紙', '_',
        /* 60 */ '通', '供', '許', '局', '送', '絡', '孫', '扉', '約', '束', '_', '万', 'ヲ', '左', '右', '峠',
        /* 70 */ '闇', '稼', '_', '台', '_', '_', '覇', '_', '怒', '威', '付', '_', '_', '_', '_', '_',
        /* 80 */ '_', '_', '_', '_', '細', '_', '_', '川', '_', '_', '呼', '起', '_', '_', '泣', '欲',
        /* 90 */ '_', '_', '_', '段', '階', '_', '_', '_', '_', '_', '_', '_', '寸', '_', '_', '_',
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
