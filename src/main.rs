/**
 *	Brainfuck Interpreter version 0.1
 *	
 *	This version can support all command from brainfuck.
 *	Any problem, please fork on github/nicklauri/bfi and
 *	send me pull request. Thank you.	
 *
 *	Copyright (c) 2017 by Nick Lauri.
 */

use std::io::{self, Read, Write};
use std::fs::File;
use std::env;
use std::iter;

const OP_INCR: u32 = 0;
const OP_DECR: u32 = 1;
const OP_NEXT: u32 = 2;
const OP_PREV: u32 = 3;
const OP_BGNL: u32 = 4;
const OP_ENDL: u32 = 5;
const OP_PRNT: u32 = 6;
const OP_INPT: u32 = 7;

fn bfi_u8_add(var: &mut u8, value: u32) {
	*var = (*var as u32 + value) as u8;
}

fn bfi_u8_sub(var: &mut u8, value: u32) {
	*var = (*var as i64 - value as i64) as u8;
}

fn bfi_next_cell(current_cell: u32, step: u32, max_cells: u32) -> u32 {
	if current_cell as u64 + step as u64 >= max_cells as u64 {
		return step - (max_cells - current_cell);
	}
	else {
		return current_cell + step;
	}
}

fn bfi_prev_cell(current_cell: u32, step: u32, max_cells: u32) -> u32 {
	if current_cell < step {
		return max_cells + 1 - (step - current_cell);
	}
	else {
		return current_cell - step;
	}
}

fn bfi_read_file(filename: String) -> String {
	let mut filehandle = File::open(&filename)
				.expect(format!("error: file `{}` not found!\n", filename).as_str());
	let mut raw_contents = String::new();
	match filehandle.read_to_string(&mut raw_contents) {
		Ok(_) => {},
		Err(e) => panic!(e)
	}

	raw_contents
}

fn bfi_read_input() -> String {
	let mut buffer: String = String::new();
	let stdin = io::stdin();
	let mut stdin_handle = stdin.lock();

	match stdin_handle.read_to_string(&mut buffer) {
		Ok(_) => {},
		Err(e) => panic!("bfi-preload: Can't read from stdin: {:?}", e)
	}
	buffer
}

fn bfi_compile(raw_contents: String, filename: String) -> Vec<Vec<u32>> {
	let raw_contents_len: u32 = raw_contents.len() as u32;
	let raw_contents_slice: Vec<u8> = raw_contents.into();
	let mut loop_stack = Vec::<u32>::new();
	let mut raw_contents_index: u32 = 0;
	let mut stacked_char_num: u32 = 1;
	let mut current_char: char;
	let mut previous_char: char = '\0';
	let mut compiled_code = Vec::<Vec<u32>>::new();

	while raw_contents_index <= raw_contents_len {
		current_char = if raw_contents_index < raw_contents_len
			{raw_contents_slice[raw_contents_index as usize] as char}
			else {'\0'};

		if previous_char == current_char && current_char != '[' && current_char != ']' {
			stacked_char_num += 1;
		}
		else {
			match previous_char {
				'+' => {
					compiled_code.push(vec![OP_INCR, stacked_char_num]);
				},
				'-' => {
					compiled_code.push(vec![OP_DECR, stacked_char_num]);
				},
				'<' => {
					compiled_code.push(vec![OP_PREV, stacked_char_num]);
				},
				'>' => {
					compiled_code.push(vec![OP_NEXT, stacked_char_num]);
				},
				'.' => {
					compiled_code.push(vec![OP_PRNT, stacked_char_num]);
				},
				',' => {
					compiled_code.push(vec![OP_INPT, 0]);
				},
				'[' => {
					loop_stack.push(compiled_code.len() as u32);
					compiled_code.push(vec![OP_BGNL, 0]);
					
				},
				']' => {
					match loop_stack.pop() {
						Some(ls_v) => {
							compiled_code.push(vec![OP_ENDL, ls_v]);
							compiled_code[ls_v as usize][1] = compiled_code.len() as u32 - 1;
						},
						None => {
							panic!(format!("{}:{}: incorrect close delimiter ']'", 
								filename, raw_contents_index));
						}
					}
					
				},
				_ => {}
			}
			stacked_char_num = 1;
		}
	    raw_contents_index += 1;
	    previous_char = current_char;
	}

	if !loop_stack.is_empty() {
		println!("{:?}", loop_stack);
		panic!(format!("{}:{}: missing close delimiter ']'", 
			filename, loop_stack.pop().unwrap()));
	}

	compiled_code
}

fn bfi_exectute(code: Vec<Vec<u32>>, max_pointers: u32) {
	let mut pointers: Vec<u8> = vec![0; max_pointers as usize];
	let code_size = code.len();
	let mut current_cell: usize = 0;
	let mut offset: usize = 0;
	let _stdout = io::stdout();
	let mut stdout  = _stdout.lock();
	let _stdin  = io::stdin();

	while offset < code_size {
		match code[offset][0] {
			OP_INCR => {
				bfi_u8_add(&mut pointers[current_cell], code[offset][1]);
			},
			OP_DECR => {
				bfi_u8_sub(&mut pointers[current_cell], code[offset][1]);
			},
			OP_NEXT => {
				current_cell = bfi_next_cell(current_cell as u32, code[offset][1], max_pointers) as usize;
			},
			OP_PREV => {
				current_cell = bfi_prev_cell(current_cell as u32, code[offset][1], max_pointers) as usize;
			},
			OP_BGNL => {
				if pointers[current_cell] == 0 {
					offset = code[offset][1] as usize;
				}
			},
			OP_ENDL => {
				if pointers[current_cell] == 0 {}
				else {offset = code[offset][1] as usize}
			},
			OP_PRNT => {
				stdout.write(iter::repeat(pointers[current_cell] as char)
					.take(code[offset][1] as usize)
					.collect::<String>().as_bytes()).expect("bfi-runtime: system error");
			},
			OP_INPT => {
				match stdout.flush() {
					Ok(_) => (),
					Err(e) => panic!(e)
				}

				let mut stdin = _stdin.lock();
				let mut _one_char_slice = [0];
				match stdin.read_exact(&mut _one_char_slice) {
					Ok(_) => {
						if _one_char_slice == [0] {
							println!("bfi-runtime: input is disabled while using pipe.");
						}

						pointers[current_cell] = _one_char_slice[0];
					},
					Err(e) => {
						println!("bfi-runtime: the command had comma and passed via pipe?");
						panic!("bfi-runtime: can't read from stdin: {:?}", e.to_string());
					}
				}
			},
			_ => {
				println!("bfi-runtime: invalid opcode {}@{}",
					code[offset][0], code[offset][1]);
			}
		}
		offset += 1;
	}
}

fn help() {
	println!("Br**nfuck Interpreter version 0.1");
	println!("usage: {} file.bf", env::args().nth(0).unwrap());
}

fn main() {
	const POINTER_NUMBER: u32 = 30_000;

	if let Some(filename) = env::args().nth(1) {
		bfi_exectute(bfi_compile(bfi_read_file(filename.to_owned()), filename), POINTER_NUMBER);
	}
	else if let Some(h) = env::args().nth(1) {
		if h == "-h" {
			help();
		}
		else {
			help();
		}
	}
	else {
		bfi_exectute(bfi_compile(bfi_read_input(), "<stdin>".into()), POINTER_NUMBER);
	}
}
