use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};
use std::io::Write;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Byte {
	value: u8, bitmask: u8,
}

fn is_match_exact(slice: &[u8], bytes: &[Byte]) -> bool {
	for (&s, &b) in slice.iter().zip(bytes) {
		if s & b.bitmask != b.value & b.bitmask { return false; }
	}
	true
}

fn is_match(slice: &[u8], bytes: &[Byte]) -> bool {
	slice.windows(bytes.len()).any(|x| is_match_exact(x, bytes))
}

fn parse_byte(x: &str) -> Byte {
	let value = u8::from_str_radix(&x[..2], 16).unwrap();
	let mut bitmask = 0xFF;
	if x.as_bytes().get(2).copied() == Some(b':') {
		bitmask = u8::from_str_radix(&x[3..], 16).unwrap();
	}
	Byte { value, bitmask }
}

fn parse_bytes(x: &str) -> Vec<Byte> {
	x.split_whitespace().map(parse_byte).collect::<Vec<_>>()
}

fn main() {
	let mut args = std::env::args();
	args.next().unwrap();
	
	match &*args.next().unwrap() {
		"x" => {
			let bytes = parse_bytes(&args.next().unwrap());
			
			let blocklist = args.next().map(|x|
				if x == "-" { Vec::new() } else {
					x.split(',').map(parse_bytes).collect::<Vec<_>>()
				}
			).unwrap_or_default();

			let needlist = args.next().map(|x|
				x.split(',').map(parse_bytes).collect::<Vec<_>>()
			).unwrap_or_default();

			let gdpath = args.next()
				.unwrap_or_else(|| fs::read_to_string("./gdpath.txt").unwrap());
			let gdpath = gdpath.trim();
			let gd = fs::read(Path::new(&gdpath).join("GeometryDash.exe")).unwrap();
			let cocos = fs::read(Path::new(&gdpath).join("libcocos2d.dll")).unwrap();
			find_bytes("GeometryDash.exe", &gd, &bytes, &blocklist, &needlist);
			find_bytes("libcocos2d.dll", &cocos, &bytes, &blocklist, &needlist);
		},
		"i" => {
			let idx = usize::from_str_radix(&args.next().unwrap(), 16).unwrap();
			let amt = args.next().unwrap().parse::<usize>().unwrap();
			
			let file = args.next().unwrap();

			let gdpath = args.next()
				.unwrap_or_else(|| fs::read_to_string("./gdpath.txt").unwrap());
			let gdpath = gdpath.trim();

			let file = fs::read(Path::new(&gdpath).join(file)).unwrap();
			disasm(&file[idx..][..amt]);
		},
		_ => panic!()
	}
}

fn find_bytes(name: &str, file: &[u8], bytes: &[Byte], blocklist: &[Vec<Byte>], needlist: &[Vec<Byte>]) {
	println!("{name}:\n");
	let mut total = 0;
	let mut windows = file.windows(bytes.len());
	while let Some(idx) = windows.position(|x| is_match(x, bytes)) {
		total += idx + 1;
		let slice = &file[total - 1..][..20];
		if blocklist.iter().any(|x| is_match(slice, x)) {
			println!("({name}: blocked: matched blocklist {:x})", total - 1);
			continue;
		}
		if !needlist.iter().all(|x| is_match(slice, x)) {
			println!("({name}: blocked: didnt match needlist {:x})", total - 1);
			continue;
		}
		println!("----- {name}: {:x} -----", total - 1);
		disasm(slice);
	}
}

fn disasm(bytes: &[u8]) {
	let mut cmd = Command::new("ndisasm")
		.arg("-b32")
		.arg("-")
		.stdin(Stdio::piped())
		.spawn().unwrap();
	cmd.stdin.as_mut().unwrap().write_all(&bytes).unwrap();
	if !cmd.wait().unwrap().success() { panic!(); }
}
