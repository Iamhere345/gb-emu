use emu::*;
use serde::{Deserialize, Serialize};
use serde_json::Result;

#[derive(Serialize, Deserialize)]
struct CpuState {
	pc: u16,
	sp: u16,
	a: u8,
	b: u8,
	c: u8,
	d: u8,
	e: u8,
	f: u8,
	h: u8,
	l: u8,
	ime: u8,
	ei: u8,
	ram: Vec<(u16, u8)>
}

#[derive(Serialize, Deserialize)]
struct Test {
	name: String,
	initial: CpuState,
	r#final: CpuState,

}

// sm83-test-data from: https://github.com/raddad772/jsmoo-json-tests
#[test]
fn sm83_test_data() {
	
}