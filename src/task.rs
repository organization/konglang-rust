use std::collections::{HashMap, VecDeque};
use std::io::{Bytes, Stdin, stdin, Read};
use crate::task::Expression::{Receive, Send, ArithmeticExpression};
use std::ops::{Index, Deref};
use std::collections::btree_map::Range;
use atoi::FromRadix10;
use crate::task::ArithmeticOperation::{Add, Multiply, Sub, Div};
use std::fmt::Error;
use dialoguer::Input;
use slice_deque::SliceDeque;
use std::net::ToSocketAddrs;
use std::thread::sleep;
use std::time::Duration;

pub struct Task {
	pub memory: HashMap<u8, isize>,
	pub code: String
}

impl Task {
	pub fn new(code: String) -> Task{
		return Self {
			memory: HashMap::new(),
			code
		}
	}

	pub fn parse(&self, position: &mut usize, closer: &Expression) -> Result<Option<isize>, String> {
		let mut integer_expected = true;
		let mut space_expected = true;
		let mut magic_expected = true;
		let mut magic = false;
		let mut closer_expected = false;
		let mut operation_expected = false;
		let mut operation: ArithmeticOperation = ArithmeticOperation::Equal;
		let mut val: isize = 0;
		while *position < self.code.len(){
			let character : char = self.code[*position..].chars().next().unwrap();

			match closer {
				ArithmeticExpression => {},
				_ => {
					if character == closer.get_end() {
						if !closer_expected {
							return Err("unexpected close in {}".to_string());
						}
						return Ok(match magic {
							true => None,
							false => Some(val)
						});
					}
				}
			}
			*position += 1;
			match character {
				' ' | '\t' | '\n' | '\r' => {
					continue;
				},
				_ => {}
			}
			if operation_expected {
				/*if character.is_digit(10) {
					continue;
				}*/
				match character {
					'+' => {
						operation = Add;
					},
					'-' => {
						operation = Sub;
					},
					'*' => {
						operation = Multiply;
					},
					'/' => {
						operation = Div;
					},
					_ => {
						match closer {
							ArithmeticExpression => {
								if !closer_expected {
									return Err("unexpected close in {}".to_string());
								}
								*position -= 1;
								return Ok(match magic {
									true => None,
									false => Some(val)
								});
							}
							_ => return Err(format!("unexpected character {} in {{}}", character))
						}
					}
				};
				space_expected = true;
				operation_expected = false;
				integer_expected = true;
				//operation.apply(&mut val, FromRadix10::from_radix_10(line[start..(position - 1)].as_ref()));
				match operation {
					Add | Sub => {
						operation.apply(&mut val, &match (&self).parse_value(position, closer) {
							Ok(value) => value,
							Err(error) => return Err(error)
						});
						operation_expected = true;
					},
					_ => {}
				}
				continue;
			}

			match character {
				'<' => {
					if !integer_expected {
						return Err("unexpected integer in {}".to_string())
					}
					let result = (&self).parse(position, &Send);
					let input : isize;
					operation.apply(&mut val, match result {
						Ok(value) => match value {
							Some(index) => {
								match self.memory.get(&(index as u8)) {
									Some(value) => value,
									None => return Err(format!("undefined index {} in {{}}", index as u8))
								}
							},
							None => {
								input = Self::get_input() as isize;
								&input
							}
						},
						Err(e) => return Err(e)
					});
					*position += 1;
					operation_expected = true;
					closer_expected = true;
					integer_expected = false;
				},
				'2' => {
					if !integer_expected {
						return Err("unexpected integer in {}".to_string())
					}
					operation.apply(&mut val, &2);
					operation_expected = true;
					closer_expected = true;
					integer_expected = false;
				}
				'#' => {
					if !magic_expected {
						return Err("unexpected # {}:{}".to_string());
					}
					integer_expected = false;
					operation_expected = false;
					closer_expected = true;
					magic = true;
				}
				_ => {
					return Err(format!("unexpected character {} in {{}}", character));
				}
			}
			magic_expected = false;
		}
		return match closer {
			ArithmeticExpression => {
				Ok(match magic {
					true => None,
					false => Some(val)
				})
			},
			_ => Err(format!("unexpected end; expected closer {} in {{}}", closer.get_end()))
		}
	}

	pub fn parse_value(&self, position: &mut usize, closer: &Expression) -> Result<isize, String> {
		return match self.parse(position, closer) {
			Ok(value) => match value {
				Some(val) => Ok(val),
				None => Err("unable to use system io here in {}".to_string())
			},
			Err(error) => Err(error)
		}
	}

	pub fn eval(&mut self, position: &mut usize, use_closer: &bool) -> Result<isize, String> {
		let mut line_number = 0;
		let mut val: Option<isize> = Option::None;

		let mut arithmetic_expression_expected = true;
		let mut receive_expected = true;
		let mut closer_expected = true;
		let mut write_expected = false;
		let mut read_expected = false;
		let mut while_expected = false;

		let mut write = false;
		let mut read = false;

		let mut latest_parse_start : usize = 0;
		while *position < self.code.len() {
			let character = self.code[*position..].chars().next().unwrap();
			*position += 1;
			match character {
				' ' | '\t' | '\n' | '\r' => {
					continue;
				},
				'(' => {
					if !receive_expected {
						return Err("unexpected receive in {}".to_string());
					}
					receive_expected = false;

					let parse_result = match self.parse(position, &Receive) {
						Ok(value) => value,
						Err(e) => return Err(e)
					};
					*position += 1;
					if read {
						match parse_result {
							Some(index) => {
								self.memory.insert(index as u8, val.unwrap());
							},
							None => {
								print!("{}", val.unwrap() as u8 as char);
							}
						}
						arithmetic_expression_expected = true;
						receive_expected = true;
						closer_expected = true;
						write_expected = false;
						read_expected = false;
						while_expected = false;

						write = false;
						read = false;
					} else {
						val = parse_result;
						write_expected = true;
						read_expected = false;
						while_expected = false;
					}
				},
				'{' => {
					if !write_expected {
						return Err("unexpected write in {}".to_string());
					}
					write_expected = false;
					arithmetic_expression_expected = true;
					write = true;
				},
				'}' => {
					if !read_expected {
						return Err("unexpected read in {}".to_string());
					}
					read_expected = false;
					arithmetic_expression_expected = false;
					receive_expected = true;

					read = true;
				},
				'[' => {
					if !while_expected {
						return Err("unexpected while in {}".to_string())
					}
					while_expected = false;
					if val.unwrap() != 0 {
						let mut position_in_while: usize = *position;
						self.eval(&mut position_in_while, &true);
						let mut parse_start = latest_parse_start;
						while match self.parse_value(&mut parse_start, &ArithmeticExpression)  {
							Ok(value) => {
								value
							},
							Err(e) => return Err(e)
						} != 0 {
							position_in_while = *position;
							self.eval(&mut position_in_while, &true);
							parse_start = latest_parse_start;
						}
						*position = position_in_while;
					}
					arithmetic_expression_expected = true;
					receive_expected = true;
					closer_expected = true;
					write_expected = false;
					read_expected = false;
					while_expected = false;

					write = false;
					read = false;
				},
				']' => {
					if !closer_expected || !*use_closer{
						return Err("unexpected closer in {}".to_string())
					}
					return Ok(0);
				}
				_ => {
					if !arithmetic_expression_expected {
						return Err("unexpected expression in {}".to_string());
					}
					*position -= 1;
					arithmetic_expression_expected = false;
					latest_parse_start = *position;
					let parse_result = match self.parse_value(position, &ArithmeticExpression)  {
						Ok(value) => value,
						Err(e) => return Err(e)
					};

					if write {
						match val {
							Some(index) => {
								self.memory.insert(index as u8, parse_result);
							},
							None => {
								print!("{}", parse_result as u8 as char);
							}
						}
						arithmetic_expression_expected = true;
						receive_expected = true;
						closer_expected = true;
						write_expected = false;
						read_expected = false;
						while_expected = false;

						write = false;
						read = false;
					} else {
						val = Some(parse_result);
						arithmetic_expression_expected = false;
						receive_expected = false;
						write_expected = false;
						read_expected = true;
						while_expected = true;
					}
				}
			}

		}
		return Ok(0);
	}

	pub fn get_input() -> u8 {
		let mut input: [u8; 1] = [0];
		stdin().read(&mut input);
		if input[0].is_ascii_control() {
			return Self::get_input();
		}
		return input[0];
	}
}

pub enum Expression {
	Send,
	Receive,
	While,
	ArithmeticExpression
}

impl Expression {
	pub fn get_end(&self) -> char {
		return match self {
			&Self::Send => '>',
			&Self::Receive => ')',
			_ => panic!()
		}
	}
}

pub enum ArithmeticOperation {
	Equal,
	Add,
	Sub,
	Multiply,
	Div
}

impl ArithmeticOperation {
	pub fn apply(&self, x: &mut isize, y: &isize) {
		match self {
			&Self::Equal => *x = *y,
			&Self::Add => *x += *y,
			&Self::Sub => *x -= *y,
			&Self::Multiply => *x *= *y,
			&Self::Div => *x /= *y
		}
	}
}



