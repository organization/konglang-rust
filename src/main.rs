use std::io::Read;
use std::collections::HashMap;
use crate::task::Task;
use crate::task::Expression::{Send, Receive};
use std::fs::read_to_string;
use std::env::args;

pub mod task;

fn main() {
	let args: Vec<String> = args().collect();
	if args.len() < 2 {
		println!("usage: {} FILE_PATH", args[0]);
		return;
	}
	let code = match read_to_string(args[1].as_str()) {
		Ok(code) => code,
		Err(e) => panic!(e)
	};
	let mut task = Task::new(code);
	task.memory.insert(3, 6); //메모리 1에 6 넣음
	task.memory.insert(106, 999); //메모리 4에 7넣음

	let mut position = 0;
	match task.eval(&mut position, &false) {
		Ok(_) => {},
		Err(e) => {
			let passed = task.code[..position].lines();
			let count = task.code[..position].lines().count();
			panic!(e.as_str().replace("{}", format!("{}:{}", count, passed.last().unwrap().len()).as_str()));
		}
	}
}
