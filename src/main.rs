#![allow(unused_assignments)]

use std::io::{self, Read};
use std::fs::File;
use std::env;

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

fn bfi_compile(filename: &String) -> Vec<Vec<u32>> {
	let mut filehandle = File::open(filename)
				.expect(format!("error: file `{}` not found!\n", filename).as_str());
	let mut raw_contents = String::new();
	let mut compiled_code = Vec::<Vec<u32>>::new();

	match filehandle.read_to_string(&mut raw_contents) {
		Ok(_) => {},
		Err(e) => panic!(e)
	}

	let raw_contents_len: u32 = raw_contents.len() as u32;
	let raw_contents_slice: Vec<u8> = raw_contents.into();
	let mut loop_stack = Vec::<u32>::new();
	let mut raw_contents_index: u32 = 0;
	let mut stacked_char_num: u32 = 1;
	let mut current_char: char;
	let mut previous_char: char = '\0';
	let mut offset: i32 = 0;

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
				_ => {
					offset -= 1;  // 'offset' will never increase if not bf syntax.
				}
			}
			stacked_char_num = 1;
		}

	    offset += 1;
	    raw_contents_index += 1;
	    previous_char = current_char;
	}

	if !loop_stack.is_empty() {
		panic!(format!("{}:{}: missing close delimiter ']'", 
			filename, loop_stack.pop().unwrap()));
	}

	compiled_code
}

fn bfi_exectute(code: Vec<Vec<u32>>, max_pointers: u32)  {
	let mut pointers: Vec<u8> = vec![0; max_pointers as usize];
	let code_size = code.len();
	let mut current_cell: usize = 0;
	let mut offset: usize = 0;

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
				for _ in 0..code[offset][1] {
					print!("{}", pointers[current_cell] as char)
				}
			},
			OP_INPT => {
				for character in io::stdin().bytes() {
					pointers[current_cell] = match character {
						Ok(c) => c ,
						Err(e) => 
							panic!(format!("bfi-runtime: system error {}", e.to_string()))
					};
					break;
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

fn main() {
	const MAX_POINTERS: u32 = 30_000;

	if let Some(filename) = env::args().nth(1) {
		bfi_exectute(bfi_compile(&filename), MAX_POINTERS);
	}
	else {
		println!("Br**nfuck Interpreter version 0.1");
		println!("usage: {} file.bf", env::args().nth(0).unwrap());
	}
}
