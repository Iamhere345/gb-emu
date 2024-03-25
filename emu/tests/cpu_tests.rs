use std::{default, fs};

use emu::*;
use serde::{Deserialize, Serialize};
use serde_json::Result;

#[derive(Serialize, Deserialize, Debug)]
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
	ie: u8,
	ram: Vec<(u16, u8)>
}

#[derive(Serialize, Deserialize, Debug)]
struct Test {
	name: String,
	initial: CpuState,
	r#final: CpuState,

}

// sm83-test-data from: https://github.com/raddad772/jsmoo-json-tests
#[test]
fn sm83_test_data() {
	
	for entry_res in fs::read_dir("../tests/sm83-test-data").expect("TEST ERROR: unable to read test data") {
		
		let entry = entry_res.unwrap();

		let test_str: String = fs::read_to_string(entry.path()).unwrap();

		let test_data: Vec<Test> = serde_json::from_str(test_str.as_str()).expect(format!("Unable to parse test data in {}", entry.file_name().into_string().unwrap()).as_str());

		println!("{:?}", test_data);

	}

}