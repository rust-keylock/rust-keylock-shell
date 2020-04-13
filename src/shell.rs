// Copyright 2017 astonbitecode
// This file is part of rust-keylock password manager.
//
// rust-keylock is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// rust-keylock is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with rust-keylock.  If not, see <http://www.gnu.org/licenses/>.
use std::{io, str};
use std::io::prelude::*;
#[cfg(target_os = "windows")]
use std::process::Command;
use std::sync::Mutex;

use rpassword;
use webbrowser;

use rust_keylock::{AllConfigurations, Editor, Entry, EntryPresentationType, Menu, MessageSeverity, UserOption, UserSelection};
use rust_keylock::dropbox::DropboxConfiguration;
use rust_keylock::nextcloud::NextcloudConfiguration;

/// Editor handler driven by the shell
pub struct EditorImpl {
    previous_menu: Mutex<Option<Menu>>,
}

pub fn new() -> EditorImpl {
    EditorImpl { previous_menu: Mutex::new(None) }
}

impl EditorImpl {
    fn update_internal_state(&self, menu: &UserSelection) {
        if let UserSelection::GoTo(menu) = menu {
            self.update_menu(menu.clone())
        }
    }

    fn update_menu(&self, menu: Menu) {
        match self.previous_menu.lock() {
            Ok(mut previous_menu_mut) => {
                *previous_menu_mut = Some(menu);
            }
            Err(error) => {
                prompt_expect_any(
                    format!("Warning! Could not update the internal state. Reason: {:?}", error).as_ref(),
                    &get_string_from_stdin);
            }
        };
    }

    fn previous_menu(&self) -> Option<Menu> {
        match self.previous_menu.lock() {
            Ok(previous_menu_mut) => {
                previous_menu_mut.clone()
            }
            Err(error) => {
                prompt_expect_any(
                    format!("Warning! Could not retrieve the internal state. Reason: {:?}", error).as_ref(),
                    &get_string_from_stdin);
                Some(Menu::Main)
            }
        }
    }
}

impl Editor for EditorImpl {
    fn show_password_enter(&self) -> UserSelection {
        clear();
        let password = prompt_expect_any("Please provide your password: ", &get_secret_string_from_stdin);
        let number = prompt_expect_number("What is your favorite number?: ", &get_secret_string_from_stdin, true);
        if password.is_empty() {
            prompt_expect_any("Password cannot be empty!", &get_secret_string_from_stdin);
            self.show_password_enter()
        } else {
            UserSelection::ProvidedPassword(password, number)
        }
    }

    fn show_change_password(&self) -> UserSelection {
        clear();
        let password1 = prompt_expect_any("Please provide your password: ", &get_secret_string_from_stdin);
        let password2 = prompt_expect_any("Please provide your password once again: ", &get_secret_string_from_stdin);
        if password1 != password2 {
            let _ = prompt_expect_any("The provided passwords did not match! Press any key to try again", &get_secret_string_from_stdin);
            self.show_change_password()
        } else {
            let number1 = prompt_expect_number("What is your favorite number?: ", &get_secret_string_from_stdin, true);
            let number2 = prompt_expect_number("Please provide your favorite number once again: ", &get_secret_string_from_stdin, true);
            if number1 != number2 {
                let _ = prompt_expect_any("The provided numbers did not match! Press any key to try again", &get_secret_string_from_stdin);
                self.show_change_password()
            } else {
                UserSelection::ProvidedPassword(password1, number1)
            }
        }
    }

    fn show_menu(&self, menu: &Menu) -> UserSelection {
        clear();
        let selected = match menu {
            &Menu::Main => show_main_menu(),
            &Menu::NewEntry(ref entry_opt) => {
                let entry = entry_opt.clone().unwrap_or_else(|| Entry::empty());
                match edit(entry, &get_string_from_stdin) {
                    EditedEntry::Replace(new_entry) => UserSelection::NewEntry(new_entry),
                    EditedEntry::GeneratePassphrase(new_entry) => UserSelection::GeneratePassphrase(None, new_entry),
                    EditedEntry::Cancel => UserSelection::GoTo(Menu::EntriesList("".to_string())),
                }
            }
            &Menu::ExportEntries => {
                let path_input = prompt_expect_any("Please define the path: ", &get_string_from_stdin);
                UserSelection::ExportTo(path_input)
            }
            &Menu::ImportEntries => {
                let path_input = prompt_expect_any("Please define the path: ", &get_string_from_stdin);
                let password = prompt_expect_any("Please provide the password: ", &get_secret_string_from_stdin);
                let number = prompt_expect_number("What is your favorite number?: ", &get_secret_string_from_stdin, true);
                UserSelection::ImportFrom(path_input, password, number)
            }
            &Menu::Current => {
                UserSelection::GoTo(self.previous_menu().unwrap_or(Menu::Main))
            }
            other => panic!("Menu '{:?}' cannot be used with Entries. Please, consider opening a bug to the developers.", other),
        };
        self.update_internal_state(&selected);

        selected
    }

    fn show_entries(&self, entries: Vec<Entry>, filter: String) -> UserSelection {
        clear();
        show_entries_menu(&entries, &filter)
    }

    fn show_entry(&self, entry: Entry, index: usize, presentation_type: EntryPresentationType) -> UserSelection {
        clear();
        match presentation_type {
            EntryPresentationType::View => show_entry(index, entry),
            EntryPresentationType::Delete => delete_entry(index),
            EntryPresentationType::Edit => {
                match edit(entry, &get_string_from_stdin) {
                    EditedEntry::Replace(new_entry) => UserSelection::ReplaceEntry(index, new_entry),
                    EditedEntry::GeneratePassphrase(new_entry) => UserSelection::GeneratePassphrase(Some(index), new_entry),
                    EditedEntry::Cancel => UserSelection::GoTo(Menu::EntriesList("".to_string())),
                }
            }
        }
    }

    fn show_configuration(&self, nextcloud: NextcloudConfiguration, dropbox: DropboxConfiguration) -> UserSelection {
        clear();
        edit_configuration(&nextcloud, &dropbox, &get_string_from_stdin)
    }

    fn exit(&self, contents_changed: bool) -> UserSelection {
        show_exit_menu(contents_changed)
    }

    fn show_message(&self, message: &str, options: Vec<UserOption>, severity: MessageSeverity) -> UserSelection {
        let mut whole_message = format!("[{:?}] ", severity);
        whole_message.push_str(message);
        whole_message.push_str("\n\n\tPress ");
        let expected_input_tups: Vec<(String, String)> = options.iter()
            .map(|opt| {
                if opt.short_label == "o" {
                    ("Enter".to_string(), opt.label.clone())
                } else {
                    (opt.short_label.clone(), opt.label.clone())
                }
            })
            .collect();

        for inp in expected_input_tups.iter() {
            whole_message.push('\'');
            whole_message.push_str(&inp.0);
            whole_message.push('\'');
            whole_message.push_str(" for ");
            whole_message.push_str(&inp.1);
        }

        whole_message.push_str("\n\tSelection: ");
        let expected_inputs: Vec<String> = expected_input_tups.into_iter()
            .map(|inp| {
                if inp.0 == "Enter" {
                    "\n".to_string()
                } else {
                    inp.0
                }
            })
            .collect();
        let selection_string = prompt_expect(&whole_message, &expected_inputs, &get_string_from_stdin_no_trim, true);
        let user_selection_opt = options.iter().find(|opt| {
            if selection_string == "\n" {
                &opt.short_label == "o"
            } else {
                &opt.short_label == selection_string
            }
        });

        UserSelection::UserOption(UserOption::from(user_selection_opt.unwrap()))
    }
}

#[cfg(target_os = "windows")]
fn clear() {
    match Command::new("cmd")
        .arg("/c")
        .arg("cls")
        .status() {
        Ok(_) => {
            // ignore
        }
        Err(error) => {
            println!("Failed to clean the command line: {:?}", error);
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn clear() {
    print!("{}[2J", 27 as char);
}

fn show_entries_menu(entries: &[Entry], filter: &str) -> UserSelection {
    if !filter.is_empty() {
        println!("Entries filtered by '{}'\n\n", filter);
    }
    // Print the entries
    for (index, entry) in entries.iter().enumerate() {
        println!("{}. {}", index + 1, entry.name);
    }

    // Prompt for user input
    let mut expected_inputs = Vec::new();
    for i in 0..entries.len() {
        expected_inputs.push((i + 1).to_string());
    }

    expected_inputs.push("n".to_string());
    expected_inputs.push("r".to_string());
    expected_inputs.push("f".to_string());

    let message = if filter.is_empty() {
        r#"
    Please select one of the Entries,
    press 'n' to crate a new Entry or
    press 'r' to return to the Main Menu.
    press 'f' to filter the presented Entries:
 "#
    } else {
        expected_inputs.push("c".to_string());
        r#"
    Please select one of the Entries,
    press 'n' to crate a new Entry,
    press 'r' to return to the Main Menu,
    press 'f' to filter the presented Entries, or
    press 'c' to clear the currently applied filter:
"#
    };
    let input = prompt_expect(message, &expected_inputs, &get_string_from_stdin, true);
    // Handle user input
    match input.as_str() {
        "r" => UserSelection::GoTo(Menu::Main),
        "n" => UserSelection::GoTo(Menu::NewEntry(None)),
        "f" => {
            let filter = prompt_expect_any("Filter by:", &get_string_from_stdin);
            UserSelection::GoTo(Menu::EntriesList(filter))
        }
        "c" => UserSelection::GoTo(Menu::EntriesList("".to_string())),
        selection => {
            let index = selection.parse::<usize>().unwrap() - 1;
            UserSelection::GoTo(Menu::ShowEntry(index))
        }
    }
}

fn show_entry(index: usize, entry: Entry) -> UserSelection {
    println!("Name: {}", entry.name);
    println!("URL: {}", entry.url);
    println!("Username: {}", entry.user);
    println!("Password: {}", entry.pass);
    println!("Description: {}", entry.desc);

    let expected_inputs = vec![
        "e".to_string(),
        "d".to_string(),
        "r".to_string(),
        "cu".to_string(),
        "cn".to_string(),
        "cp".to_string()];
    let message = r#"
Entry Menu:
	e: Edit
	d: Delete
	cu: (C)opy (U)RL
	cn: (C)opy user(N)ame
	cp:  (C)opy (P)assword
	r: Return

	Selection: "#;
    let inner_input = prompt_expect(message, &expected_inputs, &get_string_from_stdin, true);
    match inner_input.as_str() {
        "e" => UserSelection::GoTo(Menu::EditEntry(index)),
        "d" => UserSelection::GoTo(Menu::DeleteEntry(index)),
        "r" => UserSelection::GoTo(Menu::EntriesList("".to_string())),
        "cu" => UserSelection::AddToClipboard(entry.url),
        "cn" => UserSelection::AddToClipboard(entry.user),
        "cp" => UserSelection::AddToClipboard(entry.pass),
        other => {
            panic!("Unexpected user selection '{:?}' in the Show Entry Menu. Please, consider opening a bug to the developers.",
                   other)
        }
    }
}

fn delete_entry(index: usize) -> UserSelection {
    let expected_inputs = vec!["y".to_string(), "n".to_string()];
    let inner_input = prompt_expect("\nAre you sure? (y/n): ", &expected_inputs, &get_string_from_stdin, true);
    match inner_input.as_str() {
        "y" => UserSelection::DeleteEntry(index),
        "n" => UserSelection::GoTo(Menu::EntriesList("".to_string())),
        other => {
            panic!("Unexpected user selection '{:?}' in the Delete Entry Menu. Please, consider opening a bug to the developers.",
                   other)
        }
    }
}

fn show_main_menu() -> UserSelection {
    let message = r#"
Main Menu:
	e: Show (E)xisting Entries
	s: (S)ave changes
	p: Change (P)assword
	c: Edit (C)onfiguration
	i: (I)mport Encrypted Entries from the filesystem
	x: E(x)port Entries to the filesystem
	q: (Q)uit

	Selection: "#;

    let expected_inputs_main =
        vec!["e".to_string(), "s".to_string(), "q".to_string(), "c".to_string(), "i".to_string(), "x".to_string(), "p".to_string()];
    let input = prompt_expect(message, &expected_inputs_main, &get_string_from_stdin, true);
    match input.as_str() {
        "e" => UserSelection::GoTo(Menu::EntriesList("".to_string())),
        "s" => UserSelection::GoTo(Menu::Save(false)),
        "q" => UserSelection::GoTo(Menu::Exit),
        "p" => UserSelection::GoTo(Menu::ChangePass),
        "c" => UserSelection::GoTo(Menu::ShowConfiguration),
        "i" => UserSelection::GoTo(Menu::ImportEntries),
        "x" => UserSelection::GoTo(Menu::ExportEntries),
        other => panic!("Unexpected user selection '{:?}' in the Main Menu. Please, consider opening a bug to the developers.", other),
    }
}

fn prompt(message: &str) {
    print!("{}", message);
    io::stdout().flush().unwrap();
}

fn edit<T>(entry: Entry, get_input: &T) -> EditedEntry
    where T: Fn() -> String
{
    clear();
    let mut entry = entry;
    println!("Name: {}", entry.name);
    println!("URL: {}", entry.url);
    println!("Username: {}", entry.user);
    println!("Password: {}", entry.pass);
    println!("Description: {}", entry.desc);

    let expected_inputs = vec![
        "n".to_string(),
        "u".to_string(),
        "un".to_string(),
        "p".to_string(),
        "g".to_string(),
        "d".to_string(),
        "a".to_string(),
        "c".to_string()];
    let message = r#"
Entry Menu:
	n: Change (N)ame         g: (G)enerate new passphrase
	u: Change (U)RL          d: Change (D)escription
	un: Change (U)ser(N)ame  a: (A)ccept changes
	p: Change (P)assword     C: (C)ancel

	Selection: "#;
    let inner_input = prompt_expect(message, &expected_inputs, &get_string_from_stdin, true);
    match inner_input.as_str() {
        "n" => {
            prompt(format!("Changing Name ({}): ", entry.name).as_str());
            let line = get_input();
            let name = if line.is_empty() {
                entry.name.clone()
            } else {
                line.to_string()
            };
            entry.name = name;
            edit(entry, get_input)
        }
        "u" => {
            prompt(format!("Changing URL ({}): ", entry.url).as_str());
            let line = get_input();
            let url = if line.is_empty() {
                entry.url.clone()
            } else {
                line.to_string()
            };
            entry.url = url;
            edit(entry, get_input)
        }
        "un" => {
            prompt(format!("Changing Username ({}): ", entry.user).as_str());
            let line = get_input();
            let user = if line.is_empty() {
                entry.name.clone()
            } else {
                line.to_string()
            };
            entry.user = user;
            edit(entry, get_input)
        }
        "p" => {
            prompt(format!("Changing Password ({}): ", entry.pass).as_str());
            let line = get_input();
            let pass = if line.is_empty() {
                entry.pass.clone()
            } else {
                line.to_string()
            };
            entry.pass = pass;
            edit(entry, get_input)
        }
        "g" => {
            EditedEntry::GeneratePassphrase(entry)
        }
        "d" => {
            prompt(format!("Changing Description ({}): ", entry.desc).as_str());
            let line = get_input();
            let desc = if line.is_empty() {
                entry.desc.clone()
            } else {
                line.to_string()
            };
            entry.desc = desc;
            edit(entry, get_input)
        }
        "a" => {
            EditedEntry::Replace(entry)
        }
        "c" => {
            EditedEntry::Cancel
        }
        other => {
            panic!("Unexpected user selection '{:?}' in the Show Entry Menu. Please, consider opening a bug to the developers.",
                   other)
        }
    }
}

fn edit_configuration<T>(nextcloud: &NextcloudConfiguration, dropbox: &DropboxConfiguration, get_input: &T) -> UserSelection
    where T: Fn() -> String
{
    let mut ncc = NextcloudConfiguration::new(
        nextcloud.server_url.to_owned(),
        nextcloud.username.to_owned(),
        nextcloud.decrypted_password().unwrap(),
        nextcloud.use_self_signed_certificate).unwrap();
    let dbxc = DropboxConfiguration::new(dropbox.decrypted_token().unwrap()).unwrap();

    let message = r#"
n: (N)extcloud configuration
d: (D)ropbox configuration
r: (R)eturn to Main Menu

	Selection: "#;

    let expected_inputs_main = vec!["n".to_string(), "d".to_string(), "r".to_string()];
    let input = prompt_expect(message, &expected_inputs_main, &get_string_from_stdin, true);
    match input.as_str() {
        "n" => {
            prompt(format!("Username ({}): ", nextcloud.username).as_str());
            let mut line = get_input();
            let url = if line.is_empty() {
                nextcloud.server_url.clone()
            } else {
                line.to_string()
            };

            prompt(format!("Username ({}): ", nextcloud.username).as_str());
            line = get_input();
            let user = if line.is_empty() {
                nextcloud.username.clone()
            } else {
                line.to_string()
            };

            prompt(format!("password ({}): ", nextcloud.decrypted_password().unwrap()).as_str());
            line = get_input();
            let pass = if line.is_empty() {
                nextcloud.decrypted_password().unwrap()
            } else {
                line.to_string()
            };

            let y_n = if nextcloud.use_self_signed_certificate {
                "y"
            } else {
                "n"
            };
            prompt(format!("Use a self-signed certificate? (y/n) ({}): ", y_n).as_str());
            line = get_input();
            let use_self_signed = if line.is_empty() {
                nextcloud.use_self_signed_certificate
            } else {
                line == "y"
            };

            ncc = NextcloudConfiguration::new(url, user, pass, use_self_signed).unwrap();
            UserSelection::UpdateConfiguration(AllConfigurations::new(ncc, dbxc))
        }
        "d" => {
            let expected_inputs_y_n = vec!["y".to_string(), "n".to_string()];
            let dbx_url = DropboxConfiguration::dropbox_url();

            if !dropbox.is_filled() {
                let input = prompt_expect("Acquire an authentication token? (y/n):", &expected_inputs_y_n, &get_string_from_stdin, true);
                match input.as_str() {
                    "y" => {
                        match webbrowser::open(&dbx_url) {
                            Ok(_) => {
                                prompt("A URL has been opened in your browser. \n\
                            Please log in your Dropbox account and do the required actions to acquire a Dropbox authentication token.\n");
                                UserSelection::GoTo(Menu::WaitForDbxTokenCallback(dbx_url))
                            }
                            Err(_) => {
                                prompt_expect_any("Could not open the browser. Press any key to continue.", &get_string_from_stdin);
                                UserSelection::UpdateConfiguration(AllConfigurations::new(
                                    ncc,
                                    DropboxConfiguration::default()))
                            }
                        }
                    }
                    "n" => {
                        UserSelection::UpdateConfiguration(AllConfigurations::new(
                            ncc,
                            DropboxConfiguration::default()))
                    }
                    other => panic!("Unexpected user selection '{:?}' in the Configuration Menu. Please, consider opening a bug to the developers.", other),
                }
            } else {
                let input = prompt_expect("A token is acquired. Do you want to renew? (y/n):", &expected_inputs_y_n, &get_string_from_stdin, true);
                match input.as_str() {
                    "y" => {
                        match webbrowser::open(&dbx_url) {
                            Ok(_) => {
                                prompt("A URL has been opened in your browser. \n\
                            Please log in your Dropbox account and do the required actions to acquire a Dropbox authentication token.\n");
                                UserSelection::GoTo(Menu::WaitForDbxTokenCallback(dbx_url))
                            }
                            Err(_) => {
                                prompt_expect_any("Could not open the browser. Press any key to continue.", &get_string_from_stdin);
                                UserSelection::UpdateConfiguration(AllConfigurations::new(ncc, dbxc))
                            }
                        }
                    }
                    "n" => {
                        UserSelection::UpdateConfiguration(AllConfigurations::new(
                            ncc,
                            DropboxConfiguration::new(dropbox.decrypted_token().unwrap()).unwrap()))
                    }
                    other => panic!("Unexpected user selection '{:?}' in the Configuration Menu. Please, consider opening a bug to the developers.", other),
                }
            }
        }
        "r" => {
            UserSelection::GoTo(Menu::Main)
        }
        other => panic!("Unexpected user selection '{:?}' in the Edit Configuration Menu. Please, consider opening a bug to the developers.", other),
    }
}

fn prompt_expect_any<T>(message: &str, get_input: &T) -> String
    where T: Fn() -> String
{
    prompt(message);
    get_input()
}

fn prompt_expect_number<T>(message: &str, get_input: &T, hide_input_on_error: bool) -> usize
    where T: Fn() -> String
{
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

fn prompt_expect<'a, T>(message: &str, expected_inputs: &'a [String], get_input: &T, retry: bool) -> &'a String
    where T: Fn() -> String
{
    let input = prompt_expect_any(message, get_input);
    let input_str = input.as_str();
    let mut found_iter = expected_inputs.iter().filter(|inp| inp == &input_str);
    let found = found_iter.next();
    if found.is_some() {
        found.unwrap()
    } else {
        let error_message = format!("Error: Wrong input '{}'\n", input_str);

        if retry {
            prompt(error_message.as_str());
            prompt_expect(message, expected_inputs, get_input, retry)
        } else {
            panic!(error_message)
        }
    }
}

fn show_exit_menu(contents_changed: bool) -> UserSelection {
    if contents_changed {
        let expected_inputs_main = vec!["y".to_string(), "n".to_string()];
        let input = prompt_expect("WARNING!\nThere are changes that are not saved! Are you sure you want to Exit? (y/n)",
                                  &expected_inputs_main,
                                  &get_string_from_stdin,
                                  true);
        match input.as_str() {
            "n" => UserSelection::GoTo(Menu::Main),
            "y" => {
                clear();
                UserSelection::GoTo(Menu::ForceExit)
            }
            other => {
                panic!("Unexpected user selection '{:?}' in the Show Exit Menu. Please, consider opening a bug to the developers.",
                       other)
            }
        }
    } else {
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

fn get_string_from_stdin_no_trim() -> String {
    let stdin = io::stdin();
    let mut line = String::new();
    stdin.lock().read_line(&mut line).unwrap();
    line = line.replace("\r\n", "\n");
    match line.as_bytes().split_last() {
        Some((_last, rest)) => {
            if rest.is_empty() {
                "\n".to_string()
            } else {
                String::from(str::from_utf8(rest).unwrap_or(""))
            }
        }
        None => "".to_string(),
    }
}

fn get_secret_string_from_stdin() -> String {
    rpassword::prompt_password_stdout("").unwrap()
}

enum EditedEntry {
    Replace(Entry),
    GeneratePassphrase(Entry),
    Cancel,
}

#[cfg(test)]
mod test_shell {
    use rust_keylock::{Editor, Entry};

    #[test]
    fn edit_change() {
        let entry = Entry::new("name".to_string(), "url".to_string(), "user".to_string(), "pass".to_string(), "desc".to_string());
        let new_entry = super::edit(entry, &dummy_input);
        assert!(new_entry.name == dummy_input());
        assert!(new_entry.url == dummy_input());
        assert!(new_entry.user == dummy_input());
        assert!(new_entry.pass == dummy_input());
        assert!(new_entry.desc == dummy_input());
    }

    #[test]
    fn edit_leave_unchanged() {
        let entry = Entry::new("name".to_string(), "url".to_string(), "user".to_string(), "pass".to_string(), "desc".to_string());
        let new_entry = super::edit(entry, &input_with_empty_string);
        assert!(new_entry.name == "name");
        assert!(new_entry.url == "url");
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
        let expected_inputs = vec!["y".to_string(), "n".to_string()];
        let inner_input = super::prompt_expect("", &expected_inputs, &input_y, true);
        match inner_input.as_str() {
            "y" => assert!(true),
            "n" => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    fn sort_entries() {
        let mut entries = vec![Entry {
            name: "Cat".to_string(),
            url: "url1".to_string(),
            user: "user1".to_string(),
            pass: "pass1".to_string(),
            desc: "desc1".to_string(),
            encrypted: false,
        }, Entry {
            name: "Albatros".to_string(),
            url: "url2".to_string(),
            user: "user2".to_string(),
            pass: "pass2".to_string(),
            desc: "desc2".to_string(),
            encrypted: false,
        }, Entry {
            name: "Bear".to_string(),
            url: "url3".to_string(),
            user: "user3".to_string(),
            pass: "pass3".to_string(),
            desc: "desc3".to_string(),
            encrypted: false,
        }];

        let editor = super::new();
        editor.sort_entries(&mut entries);
        assert!(entries[0].name == "Albatros");
        assert!(entries[1].name == "Bear");
        assert!(entries[2].name == "Cat");
    }

    #[test]
    #[should_panic]
    fn prompt_expect_fail() {
        let expected_inputs = vec!["y".to_string(), "n".to_string()];
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
