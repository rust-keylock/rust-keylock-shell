use rust_keylock::{ Entry, Editor, UserSelection, Menu };
use std::io::prelude::*;
use std::io;
use rpassword;

/// Editor handler driven by the shell
pub struct EditorImpl;

pub fn new() -> EditorImpl {
	EditorImpl {}
}

impl Editor for EditorImpl {
	fn show_password_enter(&self) -> UserSelection {
		clear();
		let password = prompt_expect_any("Please provide your password: ", &get_secret_string_from_stdin);
		let number = prompt_expect_number("What is your favorite number?: ", &get_secret_string_from_stdin, true);
		UserSelection::ProvidedPassword(password, number)
	}

	fn show_change_password(&self) -> UserSelection {
		clear();
		let password1 = prompt_expect_any("Please provide your password: ", &get_secret_string_from_stdin);
		let password2 = prompt_expect_any("Please provide your password once again: ", &get_secret_string_from_stdin);
		if password1 != password2 {
			let _ = prompt_expect_any("The provided passwords did not match! Press any key to try again", &get_secret_string_from_stdin);
			self.show_change_password()
		}
		else {
			let number1 = prompt_expect_number("What is your favorite number?: ", &get_secret_string_from_stdin, true);
			let number2 = prompt_expect_number("Please provide your favorite number once again: ", &get_secret_string_from_stdin, true);
			if number1 != number2 {
				let _ = prompt_expect_any("The provided numbers did not match! Press any key to try again", &get_secret_string_from_stdin);
				self.show_change_password()
			}
			else {
				UserSelection::ProvidedPassword(password1, number1)
			}
		}
	}

	fn show_menu(&self, menu: &Menu, entries: &[Entry]) -> UserSelection {
		clear();
		match menu {
			&Menu::Main => show_main_menu(),
			&Menu::EntriesList => show_entries_menu(entries),
			&Menu::ShowEntry(index) => show_entry(index, entries),
			&Menu::DeleteEntry(index) => delete_entry(index),
			&Menu::NewEntry => {
				let entry = Entry::empty();
		        let new_entry = edit(&entry, &get_string_from_stdin);
				UserSelection::NewEntry(new_entry)
			},
			&Menu::EditEntry(index) => {
				let ref selected_entry = entries[index];
				let new_entry = edit(selected_entry, &get_string_from_stdin);
				UserSelection::ReplaceEntry(index, new_entry)
			},
			&Menu::ExportEntries => {
				let path_input = prompt_expect_any("Please define the path: ", &get_string_from_stdin);
				UserSelection::ExportTo(path_input)
			},
			&Menu::ImportEntries => {
				let path_input = prompt_expect_any("Please define the path: ", &get_string_from_stdin);
				let password = prompt_expect_any("Please provide the password: ", &get_secret_string_from_stdin);
				let number = prompt_expect_number("What is your favorite number?: ", &get_secret_string_from_stdin, true);
				UserSelection::ImportFrom(path_input, password, number)
			},
			other => panic!("Menu '{:?}' cannot be used with Entries. Please, consider opening a bug to the developers.", other),
		}
	}

	fn exit(&self, contents_changed: bool) -> UserSelection {
		show_exit_menu(contents_changed)
	}

	fn show_message(&self, message: &'static str) -> UserSelection {
		let expected_input = vec!("".to_string());
		let _ = prompt_expect(message, &expected_input, &get_string_from_stdin, true);
		UserSelection::Ack
	}
}

fn clear() {
	print!("{}[2J", 27 as char);
}

fn show_entries_menu(entries: &[Entry]) -> UserSelection {
	// Print the entries
	for (index, entry) in entries.iter().enumerate() {
		println!("{}. {}", index + 1, entry.name);
	}

	// Prompt for user input
	let mut expected_inputs = Vec::new();
	for i in 1..entries.len() + 1 {
		expected_inputs.push(i.to_string());
	}
	expected_inputs.push("n".to_string());
	expected_inputs.push("r".to_string());
	let input = prompt_expect("\nPlease select one of the Entries,\npress 'n' to crate a new Entry or\npress 'r' to return to the Main Menu: ", &expected_inputs, &get_string_from_stdin, true);
	// Handle user input
	match input.as_str() {
		"r" => UserSelection::GoTo(Menu::Main),
		"n" => UserSelection::GoTo(Menu::NewEntry),
		selection => {
			let index = selection.parse::<usize>().unwrap() - 1;
			UserSelection::GoTo(Menu::ShowEntry(index))
		}
	}
}

fn show_entry(index: usize, entries: &[Entry]) -> UserSelection {
	let ref entry = entries[index];

	println!("Name: {}", entry.name);
	println!("Username: {}", entry.user);
	println!("Password: {}", entry.pass);
	println!("Description: {}", entry.desc);

	let expected_inputs = vec!("e".to_string(), "d".to_string(), "r".to_string());
	let message = r#"
Entry Menu:
	e: Edit
	d: Delete
	r: Return
	
	Selection: "#;
	let inner_input = prompt_expect(message, &expected_inputs, &get_string_from_stdin, true);
	match inner_input.as_str() {
		"e" => UserSelection::GoTo(Menu::EditEntry(index)),
		"d" => UserSelection::GoTo(Menu::DeleteEntry(index)),
		"r" => UserSelection::GoTo(Menu::EntriesList),
		other => panic!("Unexpected user selection '{:?}' in the Show Entry Menu. Please, consider opening a bug to the developers.", other),
	}
}

fn delete_entry(index: usize) -> UserSelection {
	let expected_inputs = vec!("y".to_string(), "n".to_string());
	let inner_input = prompt_expect("\nAre you sure? (y/n): ", &expected_inputs, &get_string_from_stdin, true);
	match inner_input.as_str() {
		"y" => UserSelection::DeleteEntry(index),
		"n" => UserSelection::GoTo(Menu::EntriesList),
		other => panic!("Unexpected user selection '{:?}' in the Delete Entry Menu. Please, consider opening a bug to the developers.", other),
	}
}

fn show_main_menu() -> UserSelection {
	let message = r#"
Main Menu:
	e: Show Existing Entries
	s: Save changes
	c: Change Password
	i: Import Encrypted Entries
	x: Export Entries
	q: Quit
	
	Selection: "#;

	let expected_inputs_main = vec!("e".to_string(), "s".to_string(), "q".to_string(), "c".to_string(), "i".to_string(), "x".to_string());
	let input = prompt_expect(message, &expected_inputs_main, &get_string_from_stdin, true);
	match input.as_str() {
		"e" => UserSelection::GoTo(Menu::EntriesList),
		"s" => UserSelection::GoTo(Menu::Save),
		"q" => UserSelection::GoTo(Menu::Exit),
		"c" => UserSelection::GoTo(Menu::ChangePass),
		"i" => UserSelection::GoTo(Menu::ImportEntries),
		"x" => UserSelection::GoTo(Menu::ExportEntries),
		other => panic!("Unexpected user selection '{:?}' in the Main Menu. Please, consider opening a bug to the developers.", other),
	}
}

fn prompt(message: &str) {
    print!("{}", message);
    io::stdout().flush().unwrap();
}

fn edit<T>(entry: &Entry, get_input: &T) -> Entry where T: Fn() -> String {
    prompt(format!("name ({}): ", entry.name).as_str());

	let mut line = get_input();
    let name = if line.len() == 0 {
    	entry.name.clone()
    } else {
    	line.to_string()
    };

    prompt(format!("username ({}): ", entry.user).as_str());
	line = get_input();
    let user = if line.len() == 0 {
    	entry.user.clone()
    } else {
    	line.to_string()
    };

    prompt(format!("password ({}): ", entry.pass).as_str());
    line = get_input();
    let pass = if line.len() == 0 {
    	entry.pass.clone()
    } else {
    	line.to_string()
    };

    prompt(format!("Description ({}): ", entry.desc).as_str());
    line = get_input();
    let desc = if line.len() == 0 {
    	entry.desc.clone()
    } else {
    	line
    };

    Entry::new(name, user, pass, desc)
}

fn prompt_expect_any<'a, T>(message: & str, get_input: &T) -> String where T: Fn() -> String {
	prompt(message);
	get_input()
}

fn prompt_expect_number<'a, T>(message: & str, get_input: &T, hide_input_on_error: bool) -> usize where T: Fn() -> String {
	let input = prompt_expect_any(message, get_input);
	match input.parse::<usize>() {
		Ok(num) => num,
		Err(_) => {
			let error_message = if hide_input_on_error {
				"Error: Wrong input\n".to_string()
			} else {
				format!("Error: Wrong input '{}'\n", &input)
			};
		   	prompt(error_message.as_str());
		   	prompt_expect_number(message, get_input, hide_input_on_error)
		}
	}
}

fn prompt_expect<'a, T>(message: &'static str, expected_inputs: &'a [String], get_input: &T, retry: bool) -> &'a String where T: Fn() -> String {
	let input = prompt_expect_any(message, get_input);
    let ref input_str = input.as_str();
    let mut found_iter = expected_inputs.iter().filter(|inp| {inp == &input_str});
    let found = found_iter.next();
    if found.is_some() {
    	found.unwrap()
    }
    else {
    	let error_message = format!("Error: Wrong input '{}'\n", input_str);

    	if retry {
		   	prompt(error_message.as_str());
		   	prompt_expect(message, expected_inputs, get_input, retry)
	    }
    	else {
    		panic!(error_message)
    	}
    }
}

fn show_exit_menu(contents_changed: bool) -> UserSelection {
	if contents_changed {
		let expected_inputs_main = vec!("y".to_string(), "n".to_string());
		let input = prompt_expect("WARNING!\nThere are changes that are not saved! Are you sure you want to Exit? (y/n)", &expected_inputs_main, &get_string_from_stdin, true);
		match input.as_str() {
			"n" => UserSelection::GoTo(Menu::Main),
			"y" => {
				clear();
				UserSelection::GoTo(Menu::ForceExit)
			},
			other => panic!("Unexpected user selection '{:?}' in the Show Exit Menu. Please, consider opening a bug to the developers.", other),
		}
	}
	else {
		clear();
		UserSelection::GoTo(Menu::ForceExit)
	}
}

fn get_string_from_stdin() -> String {
	let stdin = io::stdin();
    let mut line = String::new();
    stdin.lock().read_line(&mut line).unwrap();
    line.trim().to_string()
}

fn get_secret_string_from_stdin() -> String {
	rpassword::prompt_password_stdout("").unwrap()
}

#[cfg(test)]
mod test_shell {
	use rust_keylock::{Entry, Editor};

	#[test]
    fn edit_change() {
    	let entry = Entry::new("name".to_string(), "user".to_string(), "pass".to_string(), "desc".to_string());
    	let new_entry = super::edit(&entry, &dummy_input);
    	assert!(new_entry.name == dummy_input());
    	assert!(new_entry.user == dummy_input());
    	assert!(new_entry.pass == dummy_input());
    	assert!(new_entry.desc == dummy_input());
    }

	#[test]
    fn edit_leave_unchanged() {
    	let entry = Entry::new("name".to_string(), "user".to_string(), "pass".to_string(), "desc".to_string());
    	let new_entry = super::edit(&entry, &input_with_empty_string);
    	assert!(new_entry.name == "name");
    	assert!(new_entry.user == "user");
    	assert!(new_entry.pass == "pass");
    	assert!(new_entry.desc == "desc");
    }

	#[test]
    fn prompt_expect_any() {
		let inner_input = super::prompt_expect_any("", &input_y);
		assert!(inner_input == "y");
    }

    #[test]
    fn prompt_expect_number() {
		let inner_input = super::prompt_expect_number("", &number_input, true);
		assert!(inner_input == 33);
    }

	#[test]
    fn prompt_expect_success() {
    	let expected_inputs = vec!("y".to_string(), "n".to_string());
		let inner_input = super::prompt_expect("", &expected_inputs, &input_y, true);
		match inner_input.as_str() {
			"y" => assert!(true),
			"n" => assert!(true),
			_ => assert!(false),
		}
    }

		#[test]
    fn sort_entries() {
    	let mut entries = vec!(
		    Entry {
	            name: "Cat".to_string(),
	            user: "user1".to_string(),
	            pass: "pass1".to_string(),
	            desc: "desc1".to_string(),
	        },
		    Entry {
	            name: "Albatros".to_string(),
	            user: "user2".to_string(),
	            pass: "pass2".to_string(),
	            desc: "desc2".to_string(),
	        },
		    Entry {
	            name: "Bear".to_string(),
	            user: "user3".to_string(),
	            pass: "pass3".to_string(),
	            desc: "desc3".to_string(),
	        });

    	let editor = super::new();
    	editor.sort_entries(&mut entries);
    	assert!(entries[0].name == "Albatros");
    	assert!(entries[1].name == "Bear");
    	assert!(entries[2].name == "Cat");
    }

	#[test]
	#[should_panic]
	fn prompt_expect_fail() {
    	let expected_inputs = vec!("y".to_string(), "n".to_string());
		let _ = super::prompt_expect("", &expected_inputs, &input_with_empty_string, false);
    }

	fn input_with_empty_string() -> String {
    	"".to_string()
    }

    fn dummy_input() -> String {
    	"this is new".to_string()
    }

	fn input_y() -> String {
    	"y".to_string()
    }

	fn number_input() -> String {
    	"33".to_string()
    }
}