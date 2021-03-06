// Copyright 2014-2017 The Rooster Developers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use clip;
use ffi;
use generate::{check_password_len, PasswordSpec};
use getopts;
use list;
use macros::show_error;
use password;

pub fn callback_help() {
    println!("Usage:");
    println!("    rooster regenerate -h");
    println!("    rooster regenerate <query>");
    println!();
    println!("Options:");
    println!("    -a, --alnum       Only use alpha numeric (a-z, A-Z, 0-9) in generated passwords");
    println!("    -l, --length      Set a custom length for the generated password, default is 32");
    println!("    -s, --show        Show the password instead of copying it to the clipboard");
    println!();
    println!("Examples:");
    println!("    rooster regenerate youtube");
    println!("    rooster regenerate ytb     # fuzzy-searching works too");
}

pub fn check_args(matches: &getopts::Matches) -> Result<(), i32> {
    if matches.free.len() < 2 {
        show_error("Woops, seems like the app name is missing here. For help, try:");
        show_error("    rooster regenerate -h");
        return Err(1);
    }

    Ok(())
}

pub fn callback_exec(
    matches: &getopts::Matches,
    store: &mut password::v2::PasswordStore,
) -> Result<(), i32> {
    check_args(matches)?;

    let query = &matches.free[1];

    let password = list::search_and_choose_password(
        store,
        query,
        list::WITH_NUMBERS,
        "Which password would you like to regenerate?",
    )
    .ok_or(1)?
    .clone();

    let pwspec = PasswordSpec::new(
        matches.opt_present("alnum"),
        matches
            .opt_str("length")
            .and_then(|len| check_password_len(len.parse::<usize>().ok())),
    );

    let password_as_string = match pwspec.generate_hard_password() {
        Ok(password_as_string) => password_as_string,
        Err(io_err) => {
            show_error(
                format!(
                    "Woops, I could not generate the password (reason: {:?}).",
                    io_err
                )
                .as_str(),
            );
            return Err(1);
        }
    };

    let change_result =
        store.change_password(&password.name, &|old_password: password::v2::Password| {
            password::v2::Password {
                name: old_password.name.clone(),
                username: old_password.username.clone(),
                password: password_as_string.clone(),
                created_at: old_password.created_at,
                updated_at: ffi::time(),
            }
        });

    match change_result {
        Ok(password) => {
            let show = matches.opt_present("show");
            clip::confirm_password_retrieved(show, &password);
            Ok(())
        }
        Err(err) => {
            show_error(
                format!(
                    "Woops, I couldn't save the new password (reason: {:?}).",
                    err
                )
                .as_str(),
            );
            Err(1)
        }
    }
}
